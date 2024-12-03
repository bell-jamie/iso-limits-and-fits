use super::{size::Size, tolerance::Tolerance, utils::decimals};

#[derive(Clone, serde::Deserialize, serde::Serialize)]
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

    pub fn show(&self, ui: &mut egui::Ui, id: &str, name: &str) {
        let mut units = "µm";
        let mut scale = 1_000.0;

        if self.tolerance.upper.abs() >= 1.0 || self.tolerance.lower.abs() >= 1.0 {
            units = "mm";
            scale = 1.0;
        }

        ui.label(egui::RichText::new(name).strong().underline());
        ui.add_space(5.0);

        egui::Grid::new(id).striped(false).show(ui, |ui| {
            ui.label("Maximum:");
            ui.label(format!(
                "{:.} mm ({} {units})",
                decimals(self.size.upper, -1),
                decimals(scale * self.tolerance.upper, -1)
            ));
            ui.end_row();

            ui.label("Minimum:");
            ui.label(format!(
                "{:.} mm ({} {units})",
                decimals(self.size.lower, -1),
                decimals(scale * self.tolerance.lower, -1)
            ));
            ui.end_row();

            ui.label("Mid-limits:");
            ui.label(format!(
                "{:.} mm ± {:.} {units}",
                decimals(self.size.mid(), -1),
                decimals(scale * self.tolerance.mid(), -1)
            ));
            ui.end_row();
        });
    }
}
