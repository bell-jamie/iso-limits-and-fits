use super::{tolerance::Tolerance, utils::decimals};

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Size {
    pub basic: f64,
    pub upper: f64,
    pub lower: f64,
}

impl Size {
    pub fn new(basic: f64, tolerance: &Tolerance) -> Self {
        let upper = basic + tolerance.upper;
        let lower = basic + tolerance.lower;

        Self {
            basic,
            upper,
            lower,
        }
    }

    pub fn mid(&self) -> f64 {
        (self.upper + self.lower) / 2.0
    }
}
