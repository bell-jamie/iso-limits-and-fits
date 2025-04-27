use super::material::Material;

/// https://www.londonbronze.co.uk/Docs/Phosphor%20Bronze%20PB104.pdf
pub fn pb104() -> Material {
    Material {
        name: "Phosphor Bronze — PB104".to_owned(),
        temp: 20.0,
        cte: 17.0,
        poissons: 0.34,
        youngs: 105_000.0,
        ys: 360.0,
        uts: 500.0,
    }
}
