use egui::RichText;
use rand::Rng;

use super::{
    tolerance::{GradesDeviations, Iso, Tolerance},
    utils::decimals,
};
use std::ops::RangeInclusive;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Feature {
    pub hole: bool,
    pub standard: bool,
    pub size: f64,
    pub iso: Iso,
    pub tolerance: Tolerance,
}

impl Feature {
    // pub fn new(tolerance: &Tolerance, size: &Size) -> Self {
    //     Self {
    //         size: size.clone(),
    //         tolerance: tolerance.clone(),
    //     }
    // }

    // pub fn create(tolerance: &Tolerance, basic_size: f64) -> Self {
    //     Self {
    //         size: Size::new(basic_size, tolerance),
    //         tolerance: tolerance.clone(),
    //     }
    // }

    pub fn default_hole() -> Self {
        Feature {
            hole: true,
            standard: true,
            size: 10.0,
            iso: Iso::new("H", "7"),
            tolerance: Tolerance::new(0.015, 0.0),
        }
    }

    pub fn default_shaft() -> Self {
        Feature {
            hole: false,
            standard: true,
            size: 10.0,
            iso: Iso::new("h", "6"),
            tolerance: Tolerance::new(0.0, -0.009),
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
                size,
                iso,
                tolerance,
            };
        }
    }

    // pub fn from_iso() -> Self {}

    // pub fn from_tol(hole: bool, size: f64, upper: f64, lower: f64) -> Self {
    //     Feature {
    //         hole,
    //         standard: false,
    //         size: size,
    //         isofit: IsoFit::new() {
    //             deviation:
    //         }
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

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let dropdowns = GradesDeviations::default();
        let combo_width = 50.0;
        let col_width = 50.0;
        let id = if self.hole { "hole" } else { "shaft" };

        ui.horizontal(|ui| {
            ui.label(RichText::new(if self.hole { "Hole:  " } else { "Shaft: " }).strong());
            ui.toggle_value(&mut self.standard, "ISO");
            ui.add(
                egui::DragValue::new(&mut self.size)
                    .speed(0.1)
                    .range(RangeInclusive::new(0.0, 3_150.0)),
            );

            if self.standard {
                egui::ComboBox::from_id_salt(format!("{}_deviation", id))
                    .width(combo_width)
                    .selected_text(&self.iso.deviation)
                    .show_ui(ui, |ui| {
                        for letter in if self.hole {
                            &dropdowns.hole_letters
                        } else {
                            &dropdowns.shaft_letters
                        } {
                            ui.selectable_value(&mut self.iso.deviation, letter.clone(), letter);
                        }
                    });
                egui::ComboBox::from_id_salt(format!("{}_grade", id))
                    .width(combo_width)
                    .selected_text(&self.iso.grade)
                    .show_ui(ui, |ui| {
                        for grade in &dropdowns.it_numbers {
                            ui.selectable_value(&mut self.iso.grade, grade.clone(), grade);
                        }
                    });
                ui.end_row();
            } else {
                egui::Grid::new([id, "non_standard"].concat())
                    .striped(false)
                    .min_col_width(col_width)
                    .show(ui, |ui| {
                        ui.add_sized(
                            [50.0, 18.0],
                            egui::DragValue::new(&mut self.tolerance.upper)
                                .speed(0.001)
                                .range(RangeInclusive::new(self.tolerance.lower, 3_000.0))
                                .min_decimals(3),
                        );
                        ui.add_sized(
                            [50.0, 18.0],
                            egui::DragValue::new(&mut self.tolerance.lower)
                                .speed(0.001)
                                .range(RangeInclusive::new(-self.size, self.tolerance.upper))
                                .min_decimals(3),
                        );
                    });
            }
        });

        // Set tolerance values to selected ISO value (if possible)

        if let Some(mut tolerance) = self.iso.convert(self.size) {
            tolerance.round(-1);
            self.tolerance = tolerance;

            let (units, scale) =
                if self.tolerance.upper.abs() < 1.0 && self.tolerance.lower.abs() < 1.0 {
                    ("µm", 1_000.0)
                } else {
                    ("mm", 1.0)
                };

            ui.add_space(5.0);

            egui::Grid::new(id).striped(false).show(ui, |ui| {
                ui.label("Upper:");
                ui.label(format!("{:.} mm", decimals(self.upper_limit(), -1)));
                ui.label(format!(
                    "{}{} {units}",
                    if self.tolerance.upper.is_sign_positive() {
                        "+"
                    } else {
                        ""
                    },
                    decimals(scale * self.tolerance.upper, -1)
                ));
                ui.end_row();

                ui.label("Lower:");
                ui.label(format!("{:.} mm", decimals(self.lower_limit(), -1)));
                ui.label(format!(
                    "{}{} {units}",
                    if self.tolerance.lower.is_sign_positive() {
                        "+"
                    } else {
                        ""
                    },
                    decimals(scale * self.tolerance.lower, -1)
                ));
                ui.end_row();

                ui.label("Middle:");
                ui.label(format!("{:.} mm", decimals(self.middle_limit(), -1)));
                ui.label(format!(
                    "±{:.} {units}",
                    decimals(scale * self.tolerance.mid(), -1)
                ));
                ui.end_row();
            });
        } else {
            ui.colored_label(
                egui::Color32::RED,
                "Invalid fundamental deviation",
            )
            .on_hover_cursor(egui::CursorIcon::Help)
            .on_hover_text("This combination of size, deviation and tolerance grade does not exist within the ISO limits and fits system. Please refer to the ISO preferred fits.");
        }
    }
}
