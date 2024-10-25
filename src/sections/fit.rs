use crate::sections::{feature::Feature, utils::decimal_places};

pub struct Fit {
    pub kind: String,
    pub class: String,
    pub upper: f64,
    pub lower: f64,
    pub target: f64,
}

impl Fit {
    pub fn new(hole: &Feature, shaft: &Feature) -> Self {
        let mmc = hole.size.lower - shaft.size.upper;
        let lmc = hole.size.upper - shaft.size.lower;

        let upper = mmc.max(lmc);
        let lower = mmc.min(lmc);
        let target = mmc - (mmc - lmc) / 2.0;

        let kind = if mmc >= 0.0 {
            "Clearance".to_owned()
        } else if lmc <= 0.0 {
            "Interference".to_owned()
        } else {
            "Transition".to_owned()
        };

        let class = if target >= 0.0 {
            "clearance".to_owned()
        } else {
            "interference".to_owned()
        };

        Self {
            kind,
            class,
            upper,
            lower,
            target,
        }
    }

    pub fn default() -> Self {
        Self {
            kind: "Clearance".to_owned(),
            class: "clearance".to_owned(),
            upper: 24.0,
            lower: 0.0,
            target: 12.0,
        }
    }

    pub fn show(&self, ui: &mut egui::Ui, id: &str) {
        ui.label(
            egui::RichText::new(format!("{} Fit", self.kind))
                .strong()
                .underline(),
        );
        ui.add_space(5.0);

        egui::Grid::new(id).striped(false).show(ui, |ui| {
            ui.label(format!(
                "{}:",
                if self.kind == "Transition" {
                    "Clearance"
                } else {
                    "Maximum"
                }
            ));
            ui.label(format!("{:.} µm", decimal_places(1000.0 * self.upper, -1)));
            ui.end_row();

            if self.kind == "Transition" {
                ui.label(format!("{}:", "Interference"));
                ui.label(format!("{:.} µm", -decimal_places(1000.0 * self.lower, -1)));
            } else {
                ui.label(format!("{}:", "Minimum"));
                ui.label(format!("{:.} µm", decimal_places(1000.0 * self.lower, -1)));
            }
            ui.end_row();

            ui.label("Mid-limits:");
            ui.label(format!(
                "{:.} µm {}",
                decimal_places(1000.0 * self.target, -1),
                if self.class == "Transition" {
                    &self.class
                } else {
                    ""
                }
            ));
            ui.end_row();
        });
    }
}
