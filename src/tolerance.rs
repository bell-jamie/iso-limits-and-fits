#![warn(clippy::all)]

pub struct Core {
    pub basic_size: f64,
    pub hole_deviation: String,
    pub hole_grade: String,
    pub shaft_deviation: String,
    pub shaft_grade: String,
}

pub struct Tolerance {
    pub upper: f64,
    pub lower: f64,
    pub mid_limits: f64,
}

impl Tolerance {
    fn new(tolerances: (f64, f64)) -> Self {
        let (upper, lower) = tolerances;
        let mid_limits = upper - (upper + lower) / 2.0;

        Self {
            upper,
            lower,
            mid_limits,
        }
    }
}

pub struct Size {
    pub basic: f64,
    pub upper: f64,
    pub lower: f64,
    pub mid_limits: f64,
}

impl Size {
    fn new(basic: f64, tolerance: &Tolerance) -> Self {
        let upper = basic + tolerance.upper;
        let lower = basic + tolerance.lower;
        let mid_limits = (upper + lower) / 2.0;

        Self {
            basic,
            upper,
            lower,
            mid_limits,
        }
    }
}

pub struct Fit {
    pub class: String,
    pub upper: f64,
    pub lower: f64,
    pub mid_class: String,
    pub mid_limits: f64,
}

impl Fit {
    fn new(hole: &Feature, shaft: &Feature) -> Self {
        let mmc = hole.tolerance.lower - shaft.tolerance.upper;
        let lmc = hole.tolerance.upper - shaft.tolerance.lower;
        let mid_limits = mmc - (mmc - lmc) / 2.0;

        let upper = mmc.max(lmc);
        let lower = mmc.min(lmc);

        let class = if mmc >= 0.0 {
            "Clearance".to_owned()
        } else if lmc <= 0.0 {
            "Interference".to_owned()
        } else {
            "Transition".to_owned()
        };

        let mid_class = if mid_limits >= 0.0 {
            "clearance".to_owned()
        } else {
            "interference".to_owned()
        };

        Self {
            class,
            upper,
            lower,
            mid_class,
            mid_limits,
        }
    }
}

pub struct Feature {
    pub size: Size,
    pub tolerance: Tolerance,
}

impl Feature {
    fn new(tolerance: Tolerance, size: Size) -> Self {
        Self { size, tolerance }
    }
}

pub struct Result {
    pub fit: Fit,
    pub hole: Feature,
    pub shaft: Feature,
}

pub const GRADE_MAP: &[&str; 20] = &[
    "01", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15",
    "16", "17", "18",
];

pub const DEVIATION_MAP: &[&str; 28] = &[
    "a", "b", "c", "cd", "d", "e", "ef", "f", "fg", "g", "h", "js", "j", "k", "m", "n", "p", "r",
    "s", "t", "u", "v", "x", "y", "z", "za", "zb", "zc",
];
// Nominal size in millimetres
// Tolerances as 1/10th of a micrometre
pub const STANDARD_TOLERANCE_GRADES: &[[i32; 21]; 21] = &[
    [
        3, 3, 5, 8, 12, 20, 30, 40, 60, 100, 140, 250, 400, 600, 1_000, 1_400, 2_500, 4_000, 6_000,
        10_000, 14_000,
    ],
    [
        6, 4, 6, 10, 15, 25, 40, 50, 80, 120, 180, 300, 480, 750, 1_200, 1_800, 3_000, 4_800,
        7_500, 12_000, 18_000,
    ],
    [
        10, 4, 6, 10, 15, 25, 40, 60, 90, 150, 220, 360, 580, 900, 1_500, 2_200, 3_600, 5_800,
        9_000, 15_000, 22_000,
    ],
    [
        18, 5, 8, 12, 20, 30, 50, 80, 110, 180, 270, 430, 700, 1_100, 1_800, 2_700, 4_300, 7_000,
        11_000, 18_000, 27_000,
    ],
    [
        30, 6, 10, 15, 25, 40, 60, 90, 130, 210, 330, 520, 840, 1_300, 2_100, 3_300, 5_200, 8_400,
        13_000, 21_000, 33_000,
    ],
    [
        50, 6, 10, 15, 25, 40, 70, 110, 160, 250, 390, 620, 1_000, 1_600, 2_500, 3_900, 6_200,
        10_000, 16_000, 25_000, 39_000,
    ],
    [
        80, 8, 12, 20, 30, 50, 80, 130, 190, 300, 460, 740, 1_200, 1_900, 3_000, 4_600, 7_400,
        12_000, 19_000, 30_000, 46_000,
    ],
    [
        120, 10, 15, 25, 40, 60, 100, 150, 220, 350, 540, 870, 1_400, 2_200, 3_500, 5_400, 8_700,
        14_000, 22_000, 35_000, 54_000,
    ],
    [
        180, 12, 20, 35, 50, 80, 120, 180, 250, 400, 630, 1_000, 1_600, 2_500, 4_000, 6_300,
        10_000, 16_000, 25_000, 40_000, 63_000,
    ],
    [
        250, 20, 30, 45, 70, 100, 140, 200, 290, 460, 720, 1_150, 1_850, 2_900, 4_600, 7_200,
        11_500, 18_500, 29_000, 46_000, 72_000,
    ],
    [
        315, 25, 40, 60, 80, 120, 160, 230, 320, 520, 810, 1_300, 2_100, 3_200, 5_200, 8_100,
        13_000, 21_000, 32_000, 52_000, 81_000,
    ],
    [
        400, 30, 50, 70, 90, 130, 180, 250, 360, 570, 890, 1_400, 2_300, 3_600, 5_700, 8_900,
        14_000, 23_000, 36_000, 57_000, 89_000,
    ],
    [
        500, 40, 60, 80, 100, 150, 200, 270, 400, 630, 970, 1_550, 2_500, 4_000, 6_300, 9_700,
        15_500, 25_000, 40_000, 63_000, 97_000,
    ],
    [
        630, -1, -1, 90, 110, 160, 220, 320, 440, 700, 1_100, 1_750, 2_800, 4_400, 7_0000, 11_000,
        17_500, 28_000, 44_000, 70_000, 110_000,
    ],
    [
        800, -1, -1, 100, 130, 180, 250, 360, 500, 800, 1_250, 2_000, 3_200, 5_000, 8_000, 12_500,
        20_000, 32_000, 50_000, 80_000, 125_000,
    ],
    [
        1_000, -1, -1, 110, 150, 210, 280, 400, 560, 900, 1_400, 2_300, 3_600, 5_600, 9_000,
        14_000, 23_000, 36_000, 56_000, 90_000, 140_000,
    ],
    [
        1_250, -1, -1, 130, 180, 240, 330, 470, 660, 1_050, 1_650, 2_600, 4_200, 6_600, 10_500,
        16_500, 26_000, 42_000, 66_000, 105_000, 165_000,
    ],
    [
        1_600, -1, -1, 150, 210, 290, 390, 550, 780, 1_250, 1_950, 3_100, 5_000, 7_800, 12_500,
        19_500, 31_000, 50_000, 78_000, 125_000, 195_000,
    ],
    [
        2_000, -1, -1, 180, 250, 350, 460, 650, 920, 1_500, 2_300, 3_700, 6_000, 9_200, 15_000,
        23_000, 37_000, 60_000, 92_000, 150_000, 230_000,
    ],
    [
        2_500, -1, -1, 220, 300, 410, 550, 780, 1_100, 1_750, 2_800, 4_400, 7_000, 11_000, 17_500,
        28_000, 44_000, 70_000, 110_000, 175_000, 280_000,
    ],
    [
        3_150, -1, -1, 260, 360, 500, 680, 960, 1_350, 2_100, 3_300, 5_400, 8_600, 13_500, 21_000,
        33_000, 54_000, 86_000, 135_000, 210_000, 330_000,
    ],
];

// Nominal size in millimetres
// Deviation in micrometres
pub const LOWER_DEVIATIONS_A_JS: &[[i32; 11]; 30] = &[
    [3, 270, 140, 60, 34, 20, 14, 10, 6, 4, 2],
    [6, 270, 140, 70, 46, 30, 20, 14, 10, 6, 4],
    [10, 280, 150, 80, 56, 40, 25, 18, 13, 8, 5],
    // 14 --
    [18, 290, 150, 95, 70, 50, 32, 23, 16, 10, 6],
    // 24 --
    [30, 300, 160, 110, 85, 65, 40, 28, 20, 12, 7],
    [40, 310, 170, 120, 100, 80, 50, 35, 25, 15, 9],
    [50, 320, 180, 130, 100, 80, 50, 35, 25, 15, 9],
    [65, 340, 190, 140, -1, 100, 60, -1, 30, -1, 10],
    [80, 360, 200, 150, -1, 100, 60, -1, 30, -1, 10],
    [100, 380, 220, 170, -1, 120, 72, -1, 36, -1, 12],
    [120, 410, 240, 180, -1, 120, 72, -1, 36, -1, 12],
    [140, 460, 260, 200, -1, 145, 85, -1, 43, -1, 14],
    [160, 520, 280, 210, -1, 145, 85, -1, 43, -1, 14],
    [200, 660, 340, 240, -1, 170, 100, -1, 50, -1, 15],
    [225, 740, 380, 260, -1, 170, 100, -1, 50, -1, 15],
    [250, 820, 420, 280, -1, 170, 100, -1, 50, -1, 15],
    [280, 920, 480, 300, -1, 190, 110, -1, 56, -1, 17],
    [315, 1_050, 540, 330, -1, 190, 110, -1, 56, -1, 17],
    [355, 1_200, 600, 360, -1, 210, 125, -1, 62, -1, 18],
    [400, 1_350, 680, 400, -1, 210, 125, -1, 62, -1, 18],
    [450, 1_500, 760, 440, -1, 230, 135, -1, 68, -1, 20],
    [500, 1_650, 840, 480, -1, 230, 135, -1, 68, -1, 20],
    // 560 --
    [630, -1, -1, -1, -1, 260, 145, -1, 76, -1, 22],
    // 710
    [800, -1, -1, -1, -1, 290, 160, -1, 80, -1, 24],
    // 900
    [1_000, -1, -1, -1, -1, 320, 170, -1, 86, -1, 26],
    // 1_120
    [1_250, -1, -1, -1, -1, 350, 195, -1, 98, -1, 28],
    // 1_400
    [1_600, -1, -1, -1, -1, 390, 220, -1, 110, -1, 30],
    // 1_800
    [2_000, -1, -1, -1, -1, 430, 240, -1, 120, -1, 32],
    // 2_240
    [2_500, -1, -1, -1, -1, 480, 260, -1, 130, -1, 34],
    // 2_800
    [3_150, -1, -1, -1, -1, 520, 290, -1, 145, -1, 38],
];

pub fn n_round(num: f64, decimals: i32) -> f64 {
    // Negative decimals inherit the default decimal places value
    let power = if decimals >= 0 { decimals } else { 6 };
    let factor = 10f64.powi(power);
    (num * factor).round() / factor
}

pub fn get_tolerance(size: f64, deviation_str: &str, grade_str: &str) -> Option<(f64, f64)> {
    let size_rounded = size.ceil() as i32;
    let grade_id = GRADE_MAP.iter().position(|&g| g == grade_str)? + 1; // +1 to ignore size column
    let deviation_id = DEVIATION_MAP
        .iter()
        .position(|&d| d.eq_ignore_ascii_case(deviation_str))?
        + 1; // +1 to ignore size column
    let tolerance_row = STANDARD_TOLERANCE_GRADES
        .iter()
        .position(|&r| r[0] >= size_rounded)?;
    let deviation_row = LOWER_DEVIATIONS_A_JS
        .iter()
        .position(|&r| r[0] >= size_rounded)?;

    let tolerance = *STANDARD_TOLERANCE_GRADES[tolerance_row].get(grade_id)? as f64 / 10_000.0;
    let deviation = if deviation_str.to_uppercase() == "H" {
        0.0
    } else if deviation_str.to_uppercase() == "JS" {
        -tolerance / 2.0
    } else {
        *LOWER_DEVIATIONS_A_JS[deviation_row].get(deviation_id)? as f64 / 1_000.0
    };

    if deviation_str.chars().next().unwrap().is_uppercase() {
        Some((deviation + tolerance, deviation))
    } else {
        Some((-deviation, -deviation - tolerance))
    }
}

pub fn calculate_fit(core: &Core) -> Option<Result> {
    let hole_tolerance = Tolerance::new(get_tolerance(
        core.basic_size,
        &core.hole_deviation,
        &core.hole_grade,
    )?);
    let hole_size = Size::new(core.basic_size, &hole_tolerance);
    let hole = Feature::new(hole_tolerance, hole_size);

    let shaft_tolerance = Tolerance::new(get_tolerance(
        core.basic_size,
        &core.shaft_deviation,
        &core.shaft_grade,
    )?);
    let shaft_size = Size::new(core.basic_size, &shaft_tolerance);
    let shaft = Feature::new(shaft_tolerance, shaft_size);

    let fit = Fit::new(&hole, &shaft);

    Some(Result { fit, hole, shaft })
}
