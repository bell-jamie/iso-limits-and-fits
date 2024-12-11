#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Material {
    pub cte: f64,
}

impl Material {
    pub fn default() -> Self {
        Material { cte: 0.000_01 }
    }
}
