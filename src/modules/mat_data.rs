use super::material::Material;
use std::collections::BTreeSet;

pub fn material_list() -> BTreeSet<Material> {
    vec![
        // https://www.londonbronze.co.uk/Docs/Phosphor%20Bronze%20PB104.pdf
        Material {
            name: "PB104".to_owned(),
            cte: 17.0,
            poissons: 0.34,
            youngs: 105.0,
            ys: 360.0,
            uts: 500.0,
        },
        // https://www.ensingerplastics.com/en/shapes/peek-tecapeek-pvx-black
        Material {
            name: "TECAPEEK PVX Black".to_owned(),
            cte: 30.0,      // this eventually needs to be temp dependant... equation?
            poissons: 0.37, // approx 0.37
            youngs: 5.5,
            ys: 84.0,
            uts: 84.0,
        },
        // https://asm.matweb.com/search/specificmaterial.asp?bassnum=mtp641
        Material {
            name: "Titanium 6Al-4V".to_owned(),
            cte: 8.6, // 9.2 > 250º, 9.7 > 500º
            poissons: 0.342,
            youngs: 113.8,
            ys: 880.0,
            uts: 950.0,
        },
        // https://www.matweb.com/search/datasheet.aspx?MatGUID=fd1b43a97a8a44129b32b9de0d7d6c1a
        Material {
            name: "4340 Steel - Annealed".to_owned(),
            cte: 12.3, // check datasheet, lots of values
            poissons: 0.30,
            youngs: 129.0,
            ys: 470.0,
            uts: 745.0,
        },
    ]
    .into_iter()
    .collect::<_>()
}
