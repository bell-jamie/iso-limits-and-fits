use egui::{ComboBox, DragValue, Grid, RichText, SelectableLabel, Ui};
use rand::Rng;

use super::{
    material::Material,
    tolerance::{GradesDeviations, Iso, Tolerance},
    utils::{check_width, decimals, State},
};

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Feature {
    pub hole: bool,
    pub standard: bool,
    pub primary: bool,
    pub enabled: bool,
    pub size: f64,
    pub iso: Iso,
    pub tolerance: Tolerance,
}

impl Feature {
    pub fn default_hole() -> Self {
        Feature {
            hole: true,
            standard: true,
            primary: true,
            enabled: true,
            size: 10.0,
            iso: Iso::new("H", "7"),
            tolerance: Tolerance::new(0.015, 0.0),
        }
    }

    pub fn default_shaft() -> Self {
        Feature {
            hole: false,
            standard: true,
            primary: true,
            enabled: true,
            size: 10.0,
            iso: Iso::new("h", "6"),
            tolerance: Tolerance::new(0.0, -0.009),
        }
    }

    pub fn default_inner() -> Self {
        Feature {
            hole: true,
            standard: true,
            primary: false,
            enabled: false,
            size: 5.0,
            iso: Iso::new("H", "12"),
            tolerance: Tolerance::new(0.120, 0.0),
        }
    }

    pub fn default_outer() -> Self {
        Feature {
            hole: false,
            standard: true,
            primary: false,
            enabled: false,
            size: 15.0,
            iso: Iso::new("h", "12"),
            tolerance: Tolerance::new(0.0, -0.180),
        }
    }

    pub fn random(hole: bool, valid: bool) -> Self {
        let mut rng = rand::thread_rng();

        loop {
            let size = rng.gen_range(0..3_150) as f64;
            let grades = GradesDeviations::default().it_numbers;
            let grade = &grades[rng.gen_range(0..grades.len())];
            let deviations = if hole {
                GradesDeviations::default().hole_letters
            } else {
                GradesDeviations::default().shaft_letters
            };
            let deviation = &deviations[rng.gen_range(0..deviations.len())];
            let iso = Iso::new(deviation, grade);

            if valid && iso.convert(size).is_none() {
                continue;
            }

            let tolerance = match iso.convert(size) {
                Some(tolerance) => tolerance,
                None => Tolerance::new(0.0, 0.0),
            };

            return Feature {
                hole,
                standard: true,
                primary: true,
                enabled: true,
                size,
                iso,
                tolerance,
            };
        }
    }

    pub fn upper_limit(&self, mat: Option<&Material>) -> f64 {
        if let Some(material) = mat {
            self.temp(self.size + self.tolerance.upper, material)
        } else {
            self.size + self.tolerance.upper
        }
    }

    pub fn middle_limit(&self, mat: Option<&Material>) -> f64 {
        (self.upper_limit(mat) + self.lower_limit(mat)) / 2.0
    }

    pub fn lower_limit(&self, mat: Option<&Material>) -> f64 {
        if let Some(material) = mat {
            self.temp(self.size + self.tolerance.lower, material)
        } else {
            self.size + self.tolerance.lower
        }
    }

    fn temp(&self, size: f64, mat: &Material) -> f64 {
        let delta_temp = mat.temp - 20.0;
        size * (1.0 + mat.cte * 0.000_001 * delta_temp)
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        state: &mut State,
        mat: &mut Material,
        id: &str,
        compliment: &Feature,
    ) {
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                self.feature_input_ui(ui, id, state);
                if self.enabled {
                    self.feature_output_ui(ui, id, None);
                }
            });

            // if state.thermal {
            //     egui::Frame::group(ui.style())
            //         .inner_margin(10.0)
            //         .rounding(10.0)
            //         .show(ui, |ui| {
            //             ui.vertical(|ui| {
            //                 self.thermal_input_ui(ui, state, mat);
            //                 self.feature_output_ui(ui, &(id.to_owned() + "_thermal"), Some(mat));
            //                 // self.thermal_output_ui(ui, id);
            //             });
            //         });
            // }
        });
    }

    fn feature_input_ui(&mut self, ui: &mut Ui, id: &str, state: &mut State) {
        let dropdowns = GradesDeviations::default();

        ui.horizontal(|ui| {
            if ui
                .add_sized([35.0, 18.0], SelectableLabel::new(self.standard, "ISO"))
                .on_hover_text("Toggle ISO limits")
                .clicked()
            {
                self.standard = !self.standard;
            }

            if self.primary {
                ui.toggle_value(&mut state.sync_size, "🔃")
                    .on_hover_text("Sync");
            } else {
                ui.toggle_value(&mut self.enabled, "✔")
                    .on_hover_text("Enable dimension");
            }

            let size_drag = ui.add_enabled(self.enabled, |ui: &mut Ui| {
                ui.add_sized(
                    [50.0, 18.0],
                    DragValue::new(&mut self.size)
                        // .custom_formatter(|s, _| format!("{s} mm"))
                        // .custom_parser(|s| {
                        //     let to_parse = s
                        //         .chars()
                        //         .filter(|c| c.is_ascii_digit() || c == &'.')
                        //         .collect::<String>();
                        //     to_parse.parse::<f64>().ok()
                        // })
                        .speed(0.1)
                        .range(0.0..=3_150.0),
                )
                .on_hover_text("Size")
            });

            if self.primary && size_drag.changed() {
                state.synced_size = self.size;
            }

            if self.standard {
                ui.add_enabled(self.enabled, |ui: &mut Ui| {
                    ComboBox::from_id_salt(format!("{}_deviation", id))
                        .width(50.0)
                        .selected_text(&self.iso.deviation)
                        .show_ui(ui, |ui| {
                            for letter in if self.hole {
                                &dropdowns.hole_letters
                            } else {
                                &dropdowns.shaft_letters
                            } {
                                ui.selectable_value(
                                    &mut self.iso.deviation,
                                    letter.clone(),
                                    letter,
                                );
                            }
                        })
                        .response
                        .on_hover_text("Deviation")
                });

                ui.add_enabled(self.enabled, |ui: &mut Ui| {
                    ComboBox::from_id_salt(format!("{}_grade", id))
                        .width(50.0)
                        .selected_text(&self.iso.grade)
                        .show_ui(ui, |ui| {
                            for grade in &dropdowns.it_numbers {
                                ui.selectable_value(&mut self.iso.grade, grade.clone(), grade);
                            }
                        })
                        .response
                        .on_hover_text("Grade")
                });
                ui.end_row();
            } else {
                ui.add_enabled(self.enabled, |ui: &mut Ui| {
                    ui.add_sized(
                        [50.0, 18.0],
                        DragValue::new(&mut self.tolerance.lower)
                            .speed(0.001)
                            .range(-self.size..=self.tolerance.upper)
                            .min_decimals(3),
                    )
                    .on_hover_text("Lower limit")
                });
                ui.add_enabled(self.enabled, |ui: &mut Ui| {
                    ui.add_sized(
                        [50.0, 18.0],
                        DragValue::new(&mut self.tolerance.upper)
                            .speed(0.001)
                            .range(self.tolerance.lower..=f64::MAX)
                            .min_decimals(3),
                    )
                    .on_hover_text("Upper limit")
                });
            }

            // check_width(ui);
        });
    }

    fn feature_output_ui(&mut self, ui: &mut Ui, id: &str, mat: Option<&Material>) {
        if !self.standard {
        } else if let Some(mut tolerance) = self.iso.convert(self.size) {
            tolerance.round(-1);
            self.tolerance = tolerance;
        } else {
            ui.colored_label(
                egui::Color32::RED,
                "Invalid fundamental deviation",
            )
            .on_hover_cursor(egui::CursorIcon::Help)
            .on_hover_text("This combination of size, deviation and tolerance grade does not exist within the ISO limits and fits system. Please refer to the ISO preferred fits.");
            return;
        }

        let (units, scale) = if self.tolerance.upper.abs() < 1.0 && self.tolerance.lower.abs() < 1.0
        {
            ("µm", 1_000.0)
        } else {
            ("mm", 1.0)
        };

        ui.add_space(5.0);

        Grid::new(id)
            .striped(false)
            .min_col_width(10.0)
            .show(ui, |ui| {
                ui.label("⬆")
                    .on_hover_cursor(egui::CursorIcon::Default)
                    .on_hover_text("Upper limit");
                ui.label(format!("{:.}", decimals(self.upper_limit(mat), -1)));
                ui.label("mm");
                if mat.is_none() {
                    ui.label(format!(
                        "{}{}",
                        if self.tolerance.upper.is_sign_positive() {
                            "+"
                        } else {
                            ""
                        },
                        decimals(scale * self.tolerance.upper, -1)
                    ));
                    ui.label(format!("{units}"));
                }
                ui.end_row();

                // ui.label("⬌");
                ui.label("⬍")
                    .on_hover_cursor(egui::CursorIcon::Default)
                    .on_hover_text("Mid-limits");
                ui.label(format!("{:.}", decimals(self.middle_limit(mat), -1)));
                ui.label("mm");
                if mat.is_none() {
                    ui.label(format!("±{:.}", decimals(scale * self.tolerance.mid(), -1)));
                    ui.label(format!("{units}"));
                }
                ui.end_row();

                ui.label("⬇")
                    .on_hover_cursor(egui::CursorIcon::Default)
                    .on_hover_text("Lower limit");
                ui.label(format!("{:.}", decimals(self.lower_limit(mat), -1)));
                ui.label("mm");
                if mat.is_none() {
                    ui.label(format!(
                        "{}{}",
                        if self.tolerance.lower.is_sign_positive() {
                            "+"
                        } else {
                            ""
                        },
                        decimals(scale * self.tolerance.lower, -1)
                    ));
                    ui.label(format!("{units}"));
                }
                ui.end_row();
            });
    }

    fn thermal_input_ui(&mut self, ui: &mut Ui, state: &mut State, mat: &mut Material) {
        ui.horizontal(|ui| {
            if state.sync_temp {
                mat.temp = state.synced_temp;
            }

            ui.toggle_value(&mut state.sync_temp, "🔃")
                .on_hover_text("Sync");

            let temp_drag = ui
                .add_sized(
                    [45.0, 18.0],
                    egui::DragValue::new(&mut mat.temp)
                        .custom_formatter(|t, _| format!("{t} ºC"))
                        .custom_parser(|t| {
                            let to_parse = t
                                .chars()
                                .filter(|c| c.is_ascii_digit() || c == &'.' || c == &'-')
                                .collect::<String>();
                            to_parse.parse::<f64>().ok()
                        })
                        .speed(1.0)
                        .range(-273.15..=10_000.0)
                        .min_decimals(1),
                )
                .on_hover_text("Temperature");

            if temp_drag.changed() {
                state.synced_temp = mat.temp;
            }

            ui.add_sized(
                [60.0, 18.0],
                DragValue::new(&mut mat.cte)
                    .custom_formatter(|e, _| format!("{e:.1} ¹/k")) // /ºC")) ¹/k
                    .custom_parser(|t| {
                        let parsed = t
                            .chars()
                            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                            .collect::<String>();
                        parsed.parse::<f64>().ok()
                    })
                    .speed(0.1)
                    .range(0.0..=f64::MAX)
                    .min_decimals(1),
            )
            .on_hover_text("Thermal expansion coefficient");

            if self.hole {
                if ui
                    .add_sized([40.0, 18.0], egui::Button::new("Oven"))
                    .on_hover_text("Set to 170 ºC")
                    .clicked()
                {
                    mat.temp = 170.0;
                }
            } else {
                if ui
                    .add_sized([40.0, 18.0], egui::Button::new("LN"))
                    .on_hover_text("Set to -196 ºC")
                    .clicked()
                {
                    mat.temp = -196.0;
                }
            }
        });
    }

    // fn thermal_output_ui(&mut self, ui: &mut Ui, id: &str) {
    //     ui.add_space(5.0);
    //     Grid::new(&(id.to_owned() + "_thermal"))
    //         .striped(false)
    //         .show(ui, |ui| {
    //             ui.label(format!("{}", decimals(self.upper_limit(true), 4)));
    //             ui.label("mm");
    //             ui.end_row();

    //             ui.label(format!("{}", decimals(self.middle_limit(true), 4)));
    //             ui.label("mm");
    //             ui.end_row();

    //             ui.label(format!("{}", decimals(self.lower_limit(true), 4)));
    //             ui.label("mm");
    //             ui.end_row();
    //         });
    // }
}
