use super::{
    lookup::{
        DELTA, DEVIATIONS_A_G, DEVIATIONS_K_ZC, DEVIATION_MAP, GRADE_MAP, LOWER_J,
        STANDARD_TOLERANCE_GRADES, UPPER_J,
    },
    utils::decimals,
};

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Tolerance {
    pub upper: f64,
    pub lower: f64,
}

impl Tolerance {
    pub fn new(upper: f64, lower: f64) -> Self {
        Self { upper, lower }
    }

    pub fn mid(&self) -> f64 {
        self.upper - (self.upper + self.lower) / 2.0
    }

    pub fn round(&mut self, n: i32) {
        self.upper = decimals(self.upper, n);
        self.lower = decimals(self.lower, n);
    }
}

pub struct GradesDeviations {
    pub it_numbers: Vec<String>,
    pub hole_letters: Vec<String>,
    pub shaft_letters: Vec<String>,
}

impl GradesDeviations {
    pub fn default() -> Self {
        let it_numbers = GRADE_MAP.iter().map(|it| it.to_string()).collect();

        let hole_letters = DEVIATION_MAP
            .iter()
            .map(|deviation| deviation.to_string().to_uppercase())
            .collect::<Vec<_>>();

        let shaft_letters = DEVIATION_MAP
            .iter()
            .map(|deviation| deviation.to_string())
            .collect::<Vec<_>>();

        Self {
            it_numbers,
            hole_letters,
            shaft_letters,
        }
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Iso {
    pub deviation: String,
    pub grade: String,
}

impl Iso {
    pub fn new(d: &str, g: &str) -> Self {
        Self {
            deviation: d.to_owned(),
            grade: g.to_owned(),
        }
    }

    pub fn convert(&self, size: f64) -> Option<Tolerance> {
        let hole = self.deviation.chars().next().unwrap().is_uppercase();

        // Calculate integer size for lookup
        let int_size = size.ceil() as i32;

        // Lookup table indices
        let idx_grade = GRADE_MAP.iter().position(|&g| g == &self.grade)? + 1; // +1 to ignore column
        let idx_dev = DEVIATION_MAP
            .iter()
            .position(|&d| d.eq_ignore_ascii_case(&self.deviation))?
            + 1;
        let idx_tol = STANDARD_TOLERANCE_GRADES
            .iter()
            .position(|&s| s[0] >= int_size)?;

        // International tolerance grade value converted to nanometres
        let tolerance = *STANDARD_TOLERANCE_GRADES[idx_tol].get(idx_grade)? * 100;
        if tolerance == -100 {
            return None;
        }

        if hole {
            Self::lookup_hole(int_size, tolerance, idx_dev, idx_grade)
        } else {
            Self::lookup_shaft(int_size, tolerance, idx_dev, idx_grade)
        }
    }

    fn lookup_hole(size: i32, tol: i32, idx_dev: usize, idx_grade: usize) -> Option<Tolerance> {
        // Helper function to convert nanometre integers to metre floats
        let flt = |d: i32| d as f64 / 1_000_000.0;

        // Helper function to retrieve lookup value, filtering -1, micrometre -> nanometre
        let rtv = |d: i32| if d != -1 { Some(d * 1000) } else { None };

        if (0..11).contains(&idx_dev) {
            // A to G
            let idx_size = DEVIATIONS_A_G.iter().position(|&s| s[0] >= size)?;
            let dev = rtv(*DEVIATIONS_A_G[idx_size].get(idx_dev)?)?;
            // let dev = Self::get_dev(size, idx_dev, DEVIATIONS_A_G)?
            if (idx_dev == 1 || idx_dev == 2) && size == 1 {
                // This covers deviations A and B for sizes <= 1
                None
            } else {
                Some(Tolerance::new(flt(dev + tol), flt(dev)))
            }
        } else if idx_dev == 11 {
            // H
            Some(Tolerance::new(flt(tol), 0.0))
        } else if idx_dev == 12 {
            // JS
            Some(Tolerance::new(flt(tol / 2), -flt(tol / 2)))
        } else if idx_dev == 13 && (8..11).contains(&idx_grade) {
            // J
            let idx_size = UPPER_J.iter().position(|&s| s[0] >= size)?;
            let dev = rtv(*UPPER_J[idx_size].get(idx_grade - 7)?)?;
            Some(Tolerance::new(flt(dev), flt(dev - tol)))
        } else if idx_dev == 14 {
            // K
            let idx_size = DEVIATIONS_K_ZC.iter().position(|&s| s[0] >= size)?;
            let dev =
                -rtv(*DEVIATIONS_K_ZC[idx_size].get(idx_dev - 13)?)? + Self::delta(size, idx_grade);
            if idx_grade > 10 && size > 3 {
                None
            } else {
                Some(Tolerance::new(flt(dev), flt(dev - tol)))
            }
        } else if idx_dev == 15 {
            // M
            let idx_size = DEVIATIONS_K_ZC.iter().position(|&s| s[0] >= size)?;
            let mut dev =
                -rtv(*DEVIATIONS_K_ZC[idx_size].get(idx_dev - 13)?)? + Self::delta(size, idx_grade);
            if idx_grade == 8 && size > 250 && size <= 315 {
                dev += 2_000; // M6 special case
            }
            Some(Tolerance::new(flt(dev), flt(dev - tol)))
        } else if idx_dev == 16 {
            // N
            let idx_size = DEVIATIONS_K_ZC.iter().position(|&s| s[0] >= size)?;
            let dev =
                -rtv(*DEVIATIONS_K_ZC[idx_size].get(idx_dev - 13)?)? + Self::delta(size, idx_grade);
            if (idx_grade > 10 && size > 500) || (idx_grade > 10 && size <= 1) {
                None
            } else {
                Some(Tolerance::new(flt(dev), flt(dev - tol)))
            }
        } else if (17..30).contains(&idx_dev) && idx_grade > 9 {
            // P to ZC - This needs to be checked, doesn't exist below IT7?
            let idx_size = DEVIATIONS_K_ZC.iter().position(|&s| s[0] >= size)?;
            let dev = -rtv(*DEVIATIONS_K_ZC[idx_size].get(idx_dev - 13)?)?
                + if idx_grade > 9 {
                    Self::delta(size, idx_grade) // Above IT7 delta is added
                } else {
                    0
                };
            Some(Tolerance::new(flt(dev), flt(dev - tol)))
        } else {
            None
        }
    }

    fn lookup_shaft(size: i32, tol: i32, idx_dev: usize, idx_grade: usize) -> Option<Tolerance> {
        // Helper function to convert nanometre integers to metre floats
        let flt = |d: i32| d as f64 / 1_000_000.0;

        // Helper function to retrieve lookup value, filtering -1, micrometre -> nanometre
        let rtv = |d: i32| if d != -1 { Some(d * 1000) } else { None };

        if (0..11).contains(&idx_dev) {
            // a to g
            let idx_size = DEVIATIONS_A_G.iter().position(|&s| s[0] >= size)?;
            let dev = -rtv(*DEVIATIONS_A_G[idx_size].get(idx_dev)?)?;
            if dev == -1 || ((idx_dev == 1 || idx_dev == 2) && size == 1) {
                // This covers deviations a and b for sizes <= 1
                None
            } else {
                Some(Tolerance::new(flt(dev), flt(dev - tol)))
            }
        } else if idx_dev == 11 {
            // h
            Some(Tolerance::new(0.0, -flt(tol)))
        } else if idx_dev == 12 {
            // js
            Some(Tolerance::new(flt(tol / 2), -flt(tol / 2)))
        } else if idx_dev == 13 && idx_grade > 6 && idx_grade < 11 {
            // j
            let idx_size = LOWER_J.iter().position(|&s| s[0] >= size)?;
            let dev = -rtv(*LOWER_J[idx_size].get(idx_grade.max(8) - 7)?)?;
            Some(Tolerance::new(flt(dev + tol), flt(dev)))
        } else if idx_dev == 14 {
            // k
            let idx_size = DEVIATIONS_K_ZC.iter().position(|&s| s[0] >= size)?;
            let dev = if idx_grade > 5 && idx_grade < 10 {
                rtv(*DEVIATIONS_K_ZC[idx_size].get(idx_dev - 13)?)?
            } else {
                0
            };
            Some(Tolerance::new(flt(dev + tol), flt(dev)))
        } else if (15..28).contains(&idx_dev) {
            // m to zc
            let idx_size = DEVIATIONS_K_ZC.iter().position(|&s| s[0] >= size)?;
            let dev = rtv(*DEVIATIONS_K_ZC[idx_size].get(idx_dev - 13)?)?;
            Some(Tolerance::new(flt(dev + tol), flt(dev)))
        } else {
            None
        }
    }

    fn delta(size: i32, grade: usize) -> i32 {
        if size > 500 || grade < 4 || grade > 9 {
            0
        } else {
            let idx = DELTA.iter().position(|&s| s[0] >= size).unwrap();
            100 * DELTA[idx][grade - 4]
        }
    }

    // fn get_dev(size: i32, idx_dev: usize, dev_map: &[[i32]]) -> Option<i32> {
    //     dev_map.iter().position(|&s| s[0] >= size).and_then(|idx_size| {
    //         let dev = *dev_map[idx_size].get(idx_dev)?;
    //         if dev != -1 { Some(1000 * dev) } else { None }
    //     })
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grade_tolerance() {
        let test_vec = vec![
            (
                Iso::new("H", "7").convert(10.0),
                Some(Tolerance::new(0.015, 0.000)),
            ),
            (
                Iso::new("js", "4").convert(5.4),
                Some(Tolerance::new(0.002, -0.002)),
            ),
            (
                Iso::new("H", "7").convert(52.8),
                Some(Tolerance::new(0.030, 0.000)),
            ),
            (
                Iso::new("g", "6").convert(52.8),
                Some(Tolerance::new(-0.010, -0.029)),
            ),
            // (
            //     Iso::new("K", "6").convert(10.0),
            //     Some(Tolerance::new(0.002, -0.007)),
            // ),
            // (
            //     Iso::new("K", "3").convert(50.0),
            //     Some(Tolerance::new(-0.0005, -0.0045)),
            // ),
            // (
            //     Iso::new("T", "3").convert(53.0),
            //     Some(Tolerance::new(-0.066, -0.071)),
            // ),
        ];

        for test in test_vec.iter() {
            if let (Some(iso), Some(bilateral)) = test {
                assert_eq!(decimals(iso.upper, 4), decimals(bilateral.upper, 4));
                assert_eq!(decimals(iso.lower, 4), decimals(bilateral.lower, 4));
            }
        }
    }

    #[test]
    fn test_delta_fn() {
        assert_eq!(Iso::delta(165, 7), 100 * DELTA[8][3]);
        assert_eq!(Iso::delta(19, 5), 100 * DELTA[4][1]);
        assert_eq!(Iso::delta(333, 8), 100 * DELTA[11][4]);
        assert_eq!(Iso::delta(38, 5), 100 * DELTA[5][1]);
    }
}
