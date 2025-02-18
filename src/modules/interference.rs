// pub struct Interference {
//     pub hole: Feature,
//     pub shaft: Feature,

// }

pub fn interference_pressure(
    hole_inner: f64,
    hole_outer: f64,
    hole_youngs: f64,
    hole_poissons: f64,
    shaft_inner: f64,
    shaft_outer: f64,
    shaft_youngs: f64,
    shaft_poissons: f64,
) -> Option<f64> {
    if hole_outer <= hole_inner || shaft_outer <= shaft_inner || shaft_outer <= hole_inner {
        return None; // Avoid division by zero or invalid geometry
    }

    let delta = shaft_outer - hole_inner;

    let term_hole = (hole_inner / hole_youngs)
        * ((hole_outer.powi(2) + hole_inner.powi(2)) / (hole_outer.powi(2) - hole_inner.powi(2))
            + hole_poissons);

    let term_shaft = (hole_inner / shaft_youngs)
        * ((shaft_outer.powi(2) + shaft_inner.powi(2))
            / (shaft_outer.powi(2) - shaft_inner.powi(2))
            - shaft_poissons);

    Some(delta / (term_hole + term_shaft))
}

pub fn thick_wall_stress(
    pressure_inner: f64,
    pressure_outer: f64,
    radius_inner: f64,
    radius_outer: f64,
    radius: f64,
) -> (f64, f64, f64) {
    let ri_sq = radius_inner.powi(2);
    let ro_sq = radius_outer.powi(2);
    let r_sq = radius.powi(2);
    let denominator = ro_sq - ri_sq;

    let a = (pressure_inner * ri_sq - pressure_outer * ro_sq) / denominator;
    let b = (pressure_inner - pressure_outer) * ro_sq * ri_sq / denominator;

    let hoop_stress = a + b / r_sq;
    let radial_stress = a - b / r_sq;
    let axial_stress = pressure_inner * ri_sq / denominator;

    (hoop_stress, radial_stress, axial_stress)
}

pub fn full_run(
    hole_inner: f64,
    hole_outer: f64,
    hole_youngs: f64,
    hole_poissons: f64,
    shaft_inner: f64,
    shaft_outer: f64,
    shaft_youngs: f64,
    shaft_poissons: f64,
) -> (f64, f64) {
    let displacement =
        |radius, youngs, hoop, poissons, radial| (radius / youngs) * (hoop - poissons * radial);
    let pressure = interference_pressure(
        hole_inner,
        hole_outer,
        hole_youngs,
        hole_poissons,
        shaft_inner,
        shaft_outer,
        shaft_youngs,
        shaft_poissons,
    )
    .unwrap();
    let (hole_hoop, hole_radial, _) = thick_wall_stress(
        pressure,
        0.0,
        hole_inner / 2.0,
        hole_outer / 2.0,
        hole_inner / 2.0,
    );
    let (shaft_hoop, shaft_radial, _) = thick_wall_stress(
        0.0,
        pressure,
        shaft_inner / 2.0,
        shaft_outer / 2.0,
        shaft_outer / 2.0,
    );
    let hole_displacement = displacement(
        hole_inner / 2.0,
        hole_youngs,
        hole_hoop,
        hole_poissons,
        hole_radial,
    );
    let shaft_displacement = displacement(
        shaft_outer / 2.0,
        shaft_youngs,
        shaft_hoop,
        shaft_poissons,
        shaft_radial,
    );
    let hole_outer_displacement = displacement(
        hole_outer / 2.0,
        hole_youngs,
        hole_hoop,
        hole_poissons,
        hole_radial,
    );
    let shaft_inner_displacement = displacement(
        shaft_inner / 2.0,
        shaft_youngs,
        shaft_hoop,
        shaft_poissons,
        shaft_radial,
    );

    println!("Pressure {pressure}");

    println!("Hole hoop {hole_hoop}");
    println!("Hole radial {hole_radial}");
    println!("Shaft hoop {shaft_hoop}");
    println!("Shaft radial {shaft_radial}");

    println!("Hole outer {hole_outer_displacement}");
    println!("Shaft inner {shaft_inner_displacement}");

    (hole_displacement, shaft_displacement)
}

#[cfg(test)]
mod tests {
    use super::{full_run, *};

    #[test]
    fn test_contact_pressure() {
        // https://www.tribology-abc.com/calculators/e3_8.htm
        // https://courses.washington.edu/me354a/Thick%20Walled%20Cylinders.pdf
        // https://calcdevice.com/interference-fit-connection-id116.html
        let tolerance = 1e0;
        let ans = 56.07726200542971;
        let pressure =
            interference_pressure(9.990, 15.0, 100_000.0, 0.33, 5.0, 10.010, 210_000.0, 0.33)
                .unwrap();

        println!(
            "Calculated pressure / correct pressure = {} / {}",
            pressure, ans
        );
        assert!((ans - pressure).abs() < tolerance);
    }

    #[test]
    fn test_thick_wall_stress() {
        // https://www.mydatabook.org/solid-mechanics/stress-for-thick-walled-cylinders-and-spheres-using-lames-equations/
        // https://www.roymech.co.uk/Useful_Tables/Mechanics/Cylinders.html
        let tolerance = 1e0;
        let hoop_inner = -105238095.23809522;
        let hoop_outer = -65238095.23809524;
        let radial_inner = -10000000.000000007;
        let radial_outer = -50000000.0;
        let axial = 1904761.904761905;

        let (calc_hoop_inner, calc_radial_inner, calc_axial) =
            thick_wall_stress(10_000_000.0, 50_000_000.0, 100.0, 250.0, 100.0);
        let (calc_hoop_outer, calc_radial_outer, _) =
            thick_wall_stress(10_000_000.0, 50_000_000.0, 100.0, 250.0, 250.0);

        println!("Hoop stress (inner) = {} / {}", calc_hoop_inner, hoop_inner);
        println!("Hoop stress (outer) = {} / {}", calc_hoop_outer, hoop_outer);
        println!(
            "Radial stress (inner) = {} / {}",
            calc_radial_inner, radial_inner
        );
        println!(
            "Radial stress (outer) = {} / {}",
            calc_radial_outer, radial_outer
        );
        println!("Axial stress = {} / {}", calc_axial, axial);

        assert!((hoop_inner - calc_hoop_inner).abs() < tolerance);
        assert!((hoop_outer - calc_hoop_outer).abs() < tolerance);
        assert!((radial_inner - calc_radial_inner).abs() < tolerance);
        assert!((radial_outer - calc_radial_outer).abs() < tolerance);
        assert!((axial - calc_axial).abs() < tolerance);
    }

    #[test]
    fn test_full_run() {
        // https://www.engineersedge.com/calculators/machine-design/press-fit/press-fit-calculator.htm
        let tolerance = 1e-5;
        let hole = 8.22e-3;
        let shaft = -1.78e-3;

        let (hole_displacement, shaft_displacement) =
            full_run(9.990, 15.0, 100_000.0, 0.33, 5.0, 10.010, 210_000.0, 0.33);

        println!("Hole displacement: {hole_displacement} / {hole}");
        println!("Shaft displacement: {shaft_displacement} / {shaft}");

        assert!((hole - hole_displacement).abs() < tolerance);
        assert!((shaft - shaft_displacement).abs() < tolerance);
    }
}
