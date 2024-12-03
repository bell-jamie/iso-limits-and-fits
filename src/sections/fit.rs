use super::{feature::Feature, tolerance::Tolerance, utils::decimals};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Fit {
    pub kind: String,
    pub class: String,
    pub upper: f64,
    pub lower: f64,
    pub target: f64,
    pub hole: Feature,
    pub shaft: Feature,
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
            "Clearance".to_owned()
        } else {
            "Interference".to_owned()
        };

        Self {
            kind,
            class,
            upper,
            lower,
            target,
            hole: hole.clone(),
            shaft: shaft.clone(),
        }
    }

    pub fn default() -> Self {
        Self {
            kind: "Clearance".to_owned(),
            class: "clearance".to_owned(),
            upper: 24.0,
            lower: 0.0,
            target: 12.0,
            hole: Feature::create(&Tolerance::new(0.015, 0.0), 10.0),
            shaft: Feature::create(&Tolerance::new(0.0, -0.09), 10.0),
        }
    }

    pub fn show(&self, ui: &mut egui::Ui, id: &str) {
        let mut units = "µm";
        let mut scale = 1_000.0;

        if self.upper.abs() >= 1.0 || self.lower.abs() >= 1.0 {
            units = "mm";
            scale = 1.0;
        }

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
            ui.label(format!("{:.} {units}", decimals(scale * self.upper, -1),));
            ui.end_row();

            if self.kind == "Transition" {
                ui.label(format!("{}:", "Interference"));
                ui.label(format!("{:.} {units}", -decimals(scale * self.lower, -1)));
            } else {
                ui.label(format!("{}:", "Minimum"));
                ui.label(format!("{:.} {units}", decimals(scale * self.lower, -1)));
            }
            ui.end_row();

            ui.label("Mid-limits:");
            ui.label(format!(
                "{:.} {units} {}",
                decimals(scale * self.target, -1),
                if self.kind == "Transition" {
                    format!("({})", self.class)
                } else {
                    "".to_string()
                }
            ));
            ui.end_row();
        });
    }
}