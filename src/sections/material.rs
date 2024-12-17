#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Material {
    pub cte: f64,
    pub temp: f64,
    pub youngs: f64,
    pub uts: f64,
}

impl Material {
    pub fn default() -> Self {
        Material {
            cte: 12.0,
            temp: 20.0,
            youngs: 200_000.0, // MPa
            uts: 500.0,        // MPa
        }
    }
}
