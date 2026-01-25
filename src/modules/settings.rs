#[derive(serde::Deserialize, serde::Serialize)]
pub struct Settings {
    trailing_zeros: bool,
}

impl Settings {
    pub fn default() -> Self {
        Self {
            trailing_zeros: false,
        }
    }
}
