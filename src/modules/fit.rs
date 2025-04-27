use egui::{Grid, RichText, Ui};

use super::{
    component::Component,
    utils::{decimals, State},
};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Fit {
    pub kind: String,
    pub class: String,
    pub mmc: f64,
    pub lmc: f64,
    pub mid: f64,
    pub female: Component,
    pub male: Component,
}

impl Fit {
    pub fn new(female: &Component, male: &Component) -> Self {
        let mmc = female.inner_diameter.lower_limit(None) - male.outer_diameter.upper_limit(None);
        let lmc = female.inner_diameter.upper_limit(None) - male.outer_diameter.lower_limit(None);
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
            female: female.clone(),
            male: male.clone(),
        }
    }

    // pub fn default() -> Self {
    //     Self {
    //         kind: "Clearance".to_owned(),
    //         class: "clearance".to_owned(),
    //         mmc: 24.0,
    //         lmc: 0.0,
    //         mid: 12.0,
    //         hole: Feature::default_hole(),
    //         shaft: Feature::default_shaft(),
    //     }
    // }

    pub fn show(&self, ui: &mut egui::Ui, state: &State) {
        let (units, scale) = if self.mmc.abs() < 1.0 && self.lmc.abs() < 1.0 {
            ("µm", 1_000.0)
        } else {
            ("mm", 1.0)
        };

        ui.horizontal(|ui| {
            egui::Frame::group(ui.style())
                .inner_margin(10.0)
                .rounding(10.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        self.fit_title_ui(ui);

                        ui.add_space(5.0);

                        self.fit_output_ui(ui, units, scale, false);
                    });
                });

            if state.thermal {
                ui.add_space(30.0);

                egui::Frame::group(ui.style())
                    .inner_margin(10.0)
                    .rounding(10.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("At Temperature").strong().size(15.0));

                            ui.add_space(5.0);

                            self.fit_output_ui(ui, units, scale, true);
                        });
                    });
            }
        });
    }

    /// Insert gaussian distribution for statistical tolerancing
    /// Show how the (nominal) fit varies with temperature, two straight lines on
    /// a graph, highlighting the material intersection temperature
    pub fn show_advanced(&mut self, ui: &mut egui::Ui, state: &State) {
        let (units, scale) = if self.mmc.abs() < 1.0 && self.lmc.abs() < 1.0 {
            ("µm", 1_000.0)
        } else {
            ("mm", 1.0)
        };

        ui.horizontal(|ui| {
            egui::Frame::group(ui.style())
                .inner_margin(10.0)
                .rounding(10.0)
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        self.fit_title_ui(ui);

                        ui.add_space(5.0);

                        self.fit_output_ui(ui, units, scale, false);
                    });
                });

            if state.thermal {
                ui.add_space(30.0);

                egui::Frame::group(ui.style())
                    .inner_margin(10.0)
                    .rounding(10.0)
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("At Temperature").strong().size(15.0));

                            ui.add_space(5.0);

                            self.fit_output_ui(ui, units, scale, true);
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

                if self.female.inner_diameter.standard && self.male.outer_diameter.standard {
                    let fit_text =
                        if self.female.inner_diameter.size == self.male.outer_diameter.size {
                            format!(
                                "{} {}{} / {}{}",
                                self.female.inner_diameter.size,
                                self.female.inner_diameter.iso.deviation,
                                self.female.inner_diameter.iso.grade,
                                self.male.outer_diameter.iso.deviation,
                                self.male.outer_diameter.iso.grade,
                            )
                        } else {
                            format!(
                                "{} {}{} / {} {}{}",
                                self.female.inner_diameter.size,
                                self.female.inner_diameter.iso.deviation,
                                self.female.inner_diameter.iso.grade,
                                self.male.outer_diameter.size,
                                self.male.outer_diameter.iso.deviation,
                                self.male.outer_diameter.iso.grade,
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

        let (female_mat, male_mat) = if thermal {
            (Some(&self.female.mat), Some(&self.male.mat))
        } else {
            (None, None)
        };

        let condition = |mc: f64| {
            if mc.is_sign_positive() {
                "clearance"
            } else {
                "interference"
            }
        };

        let mmc = self.female.inner_diameter.lower_limit(female_mat)
            - self.male.outer_diameter.upper_limit(male_mat);
        let lmc = self.female.inner_diameter.upper_limit(female_mat)
            - self.male.outer_diameter.lower_limit(male_mat);
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
                ui.label(format!("{:.} {units}", decimals(scale * mmc.abs(), 1)));
                ui.label(mmc_type);
                ui.end_row();

                ui.label("🌓")
                    .on_hover_cursor(egui::CursorIcon::Default)
                    .on_hover_text("Mid limits");
                ui.label(format!("{:.} {units}", decimals(scale * mid.abs(), 1)));
                ui.label(mid_type);
                ui.end_row();

                ui.label("🌕")
                    .on_hover_cursor(egui::CursorIcon::Default)
                    .on_hover_text("Min material condition");
                ui.label(format!("{:.} {units}", decimals(scale * lmc.abs(), 1)));
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
