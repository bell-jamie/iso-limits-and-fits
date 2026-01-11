use egui::{Button, ComboBox, DragValue, Grid, RichText, Ui};

use super::{
    material::Material,
    state::State,
    tolerance::{GradesDeviations, Iso, Tolerance},
    utils::{decimals, req_precision},
};

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Feature {
    pub hole: bool,
    pub standard: bool,
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
            enabled: false,
            size: 10.0,
            iso: Iso::new("H", "7"),
            tolerance: Tolerance::new(0.015, 0.0),
        }
    }

    pub fn default_shaft() -> Self {
        Feature {
            hole: false,
            standard: true,
            enabled: false,
            size: 10.0,
            iso: Iso::new("h", "6"),
            tolerance: Tolerance::new(0.0, -0.009),
        }
    }

    pub fn default_inner() -> Self {
        Feature {
            hole: true,
            standard: true,
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
            enabled: false,
            size: 15.0,
            iso: Iso::new("h", "12"),
            tolerance: Tolerance::new(0.0, -0.180),
        }
    }

    // pub fn random(hole: bool, valid: bool) -> Self {
    //     let mut rng = rand::thread_rng();

    //     loop {
    //         let size = rng.gen_range(0..3_150) as f64;
    //         let grades = GradesDeviations::default().it_numbers;
    //         let grade = &grades[rng.gen_range(0..grades.len())];
    //         let deviations = if hole {
    //             GradesDeviations::default().hole_letters
    //         } else {
    //             GradesDeviations::default().shaft_letters
    //         };
    //         let deviation = &deviations[rng.gen_range(0..deviations.len())];
    //         let iso = Iso::new(deviation, grade);

    //         if valid && iso.convert(size).is_none() {
    //             continue;
    //         }

    //         let tolerance = match iso.convert(size) {
    //             Some(tolerance) => tolerance,
    //             None => Tolerance::new(0.0, 0.0),
    //         };

    //         return Feature {
    //             hole,
    //             standard: true,
    //             primary: true,
    //             enabled: true,
    //             size,
    //             iso,
    //             tolerance,
    //         };
    //     }
    // }

    pub fn upper_limit(&self) -> f64 {
        self.size + self.tolerance.upper
    }

    pub fn middle_limit(&self) -> f64 {
        (self.upper_limit() + self.lower_limit()) / 2.0
    }

    pub fn lower_limit(&self) -> f64 {
        self.size + self.tolerance.lower
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        state: &mut State,
        id: &str,
        compliment: &Feature,
        is_primary: bool,
    ) {
        ui.add_space(5.0);

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                self.feature_input_ui(ui, id, state, compliment, is_primary);
                if self.enabled {
                    self.feature_output_ui(ui, id, None);
                }
            });
        });
    }

    fn feature_input_ui(
        &mut self,
        ui: &mut Ui,
        id: &str,
        state: &mut State,
        compliment: &Feature,
        is_primary: bool,
    ) {
        let dropdowns = GradesDeviations::default();
        let size_range = if compliment.enabled {
            if self.hole {
                // prevent zero wall thickness
                0.0..=compliment.lower_limit() - self.tolerance.upper
            } else {
                compliment.upper_limit() - self.tolerance.lower..=3_150.0
            }
        } else {
            0.0..=3_150.0
        };

        // Calculate available width for expanding elements
        let available = ui.available_width();
        let spacing = ui.spacing().item_spacing.x;
        // Fixed elements: ISO button (35), sync/enable button (~24), spacing between 5 elements
        let fixed_width = 35.0 + 24.0 + spacing * 4.0;
        // Remaining width split between size input and two combo boxes (or tolerance inputs)
        let flex_width = ((available - fixed_width) / 3.0).max(50.0);

        ui.horizontal(|ui| {
            if ui
                .add_sized(
                    [35.0, 18.0],
                    Button::selectable(self.standard, "ISO").frame_when_inactive(true),
                )
                .on_hover_text("Toggle ISO limits")
                .clicked()
            {
                self.standard = !self.standard;
            }

            // if self.primary {
            //     ui.toggle_value(&mut state.sync_size, "🔃")
            //         .on_hover_text("Sync");
            // } else {
            //     ui.toggle_value(&mut self.enabled, "✔")
            //         .on_hover_text("Enable dimension");
            // }

            if is_primary {
                if ui
                    .add(Button::selectable(state.sync_size, "🔃").frame_when_inactive(true))
                    .on_hover_text("Sync")
                    .clicked()
                {
                    state.sync_size = !state.sync_size;
                }
            } else {
                if ui
                    .add(Button::selectable(self.enabled, "✔").frame_when_inactive(true))
                    .on_hover_text("Enable dimension")
                    .clicked()
                {
                    self.enabled = !self.enabled;
                }
            }

            let size_drag = ui.add_enabled(self.enabled, |ui: &mut Ui| {
                ui.add_sized(
                    [flex_width, 18.0],
                    DragValue::new(&mut self.size).speed(0.1).range(size_range),
                )
                .on_hover_text("Size")
            });

            if is_primary && size_drag.changed() {
                state.synced_size = self.size;
            }

            if self.standard {
                ui.add_enabled(self.enabled, |ui: &mut Ui| {
                    ComboBox::from_id_salt(format!("{}_deviation", id))
                        .width(flex_width)
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
                        .width(flex_width)
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
                        [flex_width, 18.0],
                        DragValue::new(&mut self.tolerance.lower)
                            .speed(0.001)
                            .range(-self.size..=self.tolerance.upper)
                            .min_decimals(3),
                    )
                    .on_hover_text("Lower limit")
                });
                ui.add_enabled(self.enabled, |ui: &mut Ui| {
                    ui.add_sized(
                        [flex_width, 18.0],
                        DragValue::new(&mut self.tolerance.upper)
                            .speed(0.001)
                            .range(self.tolerance.lower..=f64::MAX)
                            .min_decimals(3),
                    )
                    .on_hover_text("Upper limit")
                });
            }
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

        let upper_trunc = decimals(self.upper_limit(), -1);
        let middle_trunc = decimals(self.middle_limit(), -1);
        let lower_trunc = decimals(self.lower_limit(), -1);

        let precision = (req_precision(upper_trunc, -1))
            .max(req_precision(middle_trunc, -1))
            .max(req_precision(lower_trunc, -1));

        Grid::new(id)
            .striped(false)
            .min_col_width(10.0)
            .spacing([15.0, 4.0])
            .show(ui, |ui| {
                // ui.label(RichText::new("⬆").color(ui.visuals().selection.bg_fill))
                ui.label("⬆")
                    .on_hover_cursor(egui::CursorIcon::Default)
                    .on_hover_text("Upper limit");
                ui.label(format!("{:.precision$} mm", upper_trunc));

                // if mat.is_none() {
                ui.label(format!(
                    "{}",
                    if self.tolerance.upper.is_sign_positive() {
                        "+"
                    } else {
                        "-"
                    }
                ));
                ui.label(format!(
                    "{} {units}",
                    decimals(scale * self.tolerance.upper.abs(), -1)
                ));
                // ui.label(format!(
                //     "{}{}",
                //     if self.tolerance.upper.is_sign_positive() {
                //         "+"
                //     } else {
                //         ""
                //     },
                //     decimals(scale * self.tolerance.upper, -1)
                // ));
                // ui.label(format!("{units}"));
                // }
                ui.end_row();

                // ui.label("⬌");
                ui.label("⬍")
                    .on_hover_cursor(egui::CursorIcon::Default)
                    .on_hover_text("Mid-limits");
                ui.label(format!("{:.precision$} mm", middle_trunc));
                // ui.label("mm");
                ui.label("±");
                ui.label(format!(
                    "{} {units}",
                    decimals(scale * self.tolerance.mid(), -1)
                ));
                // if mat.is_none() {
                //     ui.label(format!("±{:.}", decimals(scale * self.tolerance.mid(), -1)));
                //     ui.label(format!("{units}"));
                // }
                ui.end_row();

                ui.label("⬇")
                    .on_hover_cursor(egui::CursorIcon::Default)
                    .on_hover_text("Lower limit");
                ui.label(format!("{:.precision$} mm", lower_trunc));
                // ui.label("mm");
                ui.label(format!(
                    "{}",
                    if self.tolerance.lower.is_sign_positive() {
                        "+"
                    } else {
                        "-"
                    }
                ));
                ui.label(format!(
                    "{} {units}",
                    decimals(scale * self.tolerance.lower.abs(), -1)
                ));
                // if mat.is_none() {
                // ui.label(format!(
                //     "{}{}",
                //     if self.tolerance.lower.is_sign_positive() {
                //         "+"
                //     } else {
                //         ""
                //     },
                //     decimals(scale * self.tolerance.lower, -1)
                // ));
                // ui.label(format!("{units}"));
                // }
                ui.end_row();
            });
    }
}
