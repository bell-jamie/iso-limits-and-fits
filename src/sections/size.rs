use super::{tolerance::Tolerance, utils::decimal_places};

#[derive(Clone)]
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

    pub fn show(&self, ui: &mut egui::Ui, id: &str, name: &str) {
        ui.label(egui::RichText::new(name).strong().underline());
        ui.add_space(5.0);

        egui::Grid::new(id).striped(false).show(ui, |ui| {
            ui.label("Maximum:");
            ui.label(format!(
                "{:.} mm ({} µm)",
                decimal_places(self.upper, -1),
                decimal_places(1000.0 * (self.upper - self.basic), -1)
            ));
            ui.end_row();

            ui.label("Minimum:");
            ui.label(format!(
                "{:.} mm ({} µm)",
                decimal_places(self.lower, -1),
                decimal_places(1000.0 * (self.lower - self.basic), -1)
            ));
            ui.end_row();

            ui.label("Mid-limits:");
            ui.label(format!(
                "{:.} mm ± {:.} µm",
                decimal_places(self.mid(), -1),
                decimal_places(1000.0 * (self.upper - self.mid()), -1)
            ));
            ui.end_row();
        });
    }
}
