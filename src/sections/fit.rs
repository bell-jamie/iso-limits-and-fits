use egui::{Grid, RichText, Ui};

use super::{
    feature::Feature,
    utils::{decimals, State},
};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Fit {
    pub kind: String,
    pub class: String,
    pub mmc: f64,
    pub lmc: f64,
    pub mid: f64,
    pub hole: Feature,
    pub shaft: Feature,
}

impl Fit {
    pub fn new(hole: &Feature, shaft: &Feature) -> Self {
        let mmc = hole.lower_limit(false) - shaft.upper_limit(false);
        let lmc = hole.upper_limit(false) - shaft.lower_limit(false);
        let mid = (mmc + lmc) / 2.0;

        let kind = if mmc >= 0.0 {
            "Clearance".to_owned()
        } else if lmc <= 0.0 {
            "Interference".to_owned()
        } else {
            "Transition".to_owned()
        };

        let class = if mid >= 0.0 {
            "Clearance".to_owned()
        } else {
            "Interference".to_owned()
        };

        Self {
            kind,
            class,
            mmc,
            lmc,
            mid,
            hole: hole.clone(),
            shaft: shaft.clone(),
        }
    }

    pub fn default() -> Self {
        Self {
            kind: "Clearance".to_owned(),
            class: "clearance".to_owned(),
            mmc: 24.0,
            lmc: 0.0,
            mid: 12.0,
            hole: Feature::default_hole(),
            shaft: Feature::default_shaft(),
        }
    }

    pub fn show(&self, ui: &mut egui::Ui, state: &State) {
        let (units, scale) = if self.mmc.abs() < 1.0 && self.lmc.abs() < 1.0 {
            ("µm", 1_000.0)
        } else {
            ("mm", 1.0)
        };

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                self.fit_title_ui(ui);

                ui.add_space(5.0);

                egui::Frame::group(ui.style())
                    .inner_margin(10.0)
                    .rounding(10.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            self.fit_output_ui(ui, units, scale, false);
                        });
                    });
            });

            if state.thermal {
                ui.add_space(30.0);

                ui.vertical(|ui| {
                    ui.label(RichText::new("At Temperature").strong().size(15.0));

                    ui.add_space(5.0);

                    egui::Frame::group(ui.style())
                        .inner_margin(10.0)
                        .rounding(10.0)
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                self.fit_output_ui(ui, units, scale, true);
                            });
                        });
                });
            }
        });
    }

    fn fit_title_ui(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("{} Fit", self.kind,))
                    .strong()
                    .size(15.0),
            );

            ui.vertical(|ui| {
                ui.add_space(1.0);

                if self.hole.standard && self.shaft.standard {
                    let fit_text = if self.hole.size == self.shaft.size {
                        format!(
                            "{} {}{} / {}{}",
                            self.hole.size,
                            self.hole.iso.deviation,
                            self.hole.iso.grade,
                            self.shaft.iso.deviation,
                            self.shaft.iso.grade,
                        )
                    } else {
                        format!(
                            "{} {}{} / {} {}{}",
                            self.hole.size,
                            self.hole.iso.deviation,
                            self.hole.iso.grade,
                            self.shaft.size,
                            self.shaft.iso.deviation,
                            self.shaft.iso.grade,
                        )
                    };

                    if ui.button(fit_text.clone()).on_hover_text("Copy").clicked() {
                        ui.output_mut(|o| o.copied_text = fit_text);
                    }
                }
            });
        });
    }

    fn fit_output_ui(&self, ui: &mut Ui, units: &str, scale: f64, thermal: bool) {
        let id = if thermal { "thermal_fit" } else { "fit" };

        let condition = |mc: f64| {
            if mc.is_sign_positive() {
                "clearance"
            } else {
                "interference"
            }
        };

        let mmc = self.hole.lower_limit(thermal) - self.shaft.upper_limit(thermal);
        let lmc = self.hole.upper_limit(thermal) - self.shaft.lower_limit(thermal);
        let mid = (mmc + lmc) / 2.0;

        let mmc_type = condition(mmc);
        let lmc_type = condition(lmc);
        let mid_type = condition(mid);

        Grid::new(id)
            .striped(false)
            .min_col_width(10.0)
            .show(ui, |ui| {
                ui.label("🌑")
                    .on_hover_cursor(egui::CursorIcon::Default)
                    .on_hover_text("Max material condition");
                ui.label(format!("{:.} {units}", decimals(scale * mmc.abs(), 3)));
                ui.label(mmc_type);
                ui.end_row();

                ui.label("🌓")
                    .on_hover_cursor(egui::CursorIcon::Default)
                    .on_hover_text("Mid limits");
                ui.label(format!("{:.} {units}", decimals(scale * mid.abs(), 3)));
                ui.label(mid_type);
                ui.end_row();

                ui.label("🌕")
                    .on_hover_cursor(egui::CursorIcon::Default)
                    .on_hover_text("Min material condition");
                ui.label(format!("{:.} {units}", decimals(scale * lmc.abs(), 3)));
                ui.label(lmc_type);
                ui.end_row();
            });
    }

    // fn thermal_output_ui(&self, ui: &mut Ui, units: &str, scale: f64) {
    //     let mmc = self.hole.lower_limit(true) - self.shaft.upper_limit(true);
    //     let lmc = self.hole.upper_limit(true) - self.shaft.lower_limit(true);

    //     Grid::new("fit_thermal")
    //         .striped(false)
    //         .min_col_width(10.0)
    //         .show(ui, |ui| {
    //             ui.label(format!("{:.} {units}", decimals(scale * mmc.abs(), -1)));
    //             ui.end_row();

    //             ui.label(format!(
    //                 "{:.} {units}",
    //                 decimals(scale * self.mid.abs(), -1)
    //             ));
    //             ui.end_row();

    //             ui.label(format!("{:.} {units}", decimals(scale * lmc.abs(), -1)));
    //             ui.end_row();
    //         });
    // }
}
