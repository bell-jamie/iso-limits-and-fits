use crate::sections::{size::Size, tolerance::Tolerance};

pub struct Feature {
    pub size: Size,
    pub tolerance: Tolerance,
}

impl Feature {
    pub fn new(tolerance: &Tolerance, size: &Size) -> Self {
        Self {
            size: size.clone(),
            tolerance: tolerance.clone(),
        }
    }

    pub fn create(tolerance: &Tolerance, basic_size: f64) -> Self {
        Self {
            size: Size::new(basic_size, tolerance),
            tolerance: tolerance.clone(),
        }
    }
}
