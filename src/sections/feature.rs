use egui::RichText;

use super::{
    size::Size,
    tolerance::{GradesDeviations, IsoFit, Tolerance},
    utils::decimals,
};
use std::ops::RangeInclusive;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Feature {
    pub hole: bool,
    pub standard: bool,
    pub size: f64,
    pub isofit: IsoFit,
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
            isofit: IsoFit::new("H", "7"),
            tolerance: Tolerance::new(0.015, 0.0),
        }
    }

    pub fn default_shaft() -> Self {
        Feature {
            hole: false,
            standard: true,
            size: 10.0,
            isofit: IsoFit::new("h", "6"),
            tolerance: Tolerance::new(0.0, -0.009),
        }
    }

    // pub fn from_iso() -> Self {

    // }

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

    pub fn show(&mut self, ui: &mut egui::Ui, id: &str) {
        let dropdowns = GradesDeviations::default();
        let combo_width = 50.0;
        let col_width = 50.0;

        ui.horizontal(|ui| {
            if self.hole {
                ui.label(RichText::new("Hole:  ").strong());
            } else {
                ui.label(RichText::new("Shaft: ").strong());
            }

            ui.toggle_value(&mut self.standard, "ISO");

            ui.add(
                egui::DragValue::new(&mut self.size)
                    .speed(0.1)
                    .range(RangeInclusive::new(0.0, 3_150.0)),
            );

            if self.standard {
                // Set tolerance values to selected ISO value

                if let Some(mut tolerance) = self.isofit.convert(self.size) {
                    tolerance.round(3);
                    self.tolerance = tolerance;
                }

                egui::ComboBox::from_id_salt([id, "deviation"].concat())
                    .width(combo_width)
                    .selected_text(&self.isofit.deviation)
                    .show_ui(ui, |ui| {
                        if self.hole {
                            for letter in &dropdowns.hole_position_letters {
                                ui.selectable_value(
                                    &mut self.isofit.deviation,
                                    letter.clone(),
                                    letter,
                                );
                            }
                        } else {
                            for letter in &dropdowns.shaft_position_letters {
                                ui.selectable_value(
                                    &mut self.isofit.deviation,
                                    letter.clone(),
                                    letter,
                                );
                            }
                        }
                    });
                egui::ComboBox::from_id_salt([id, "grade"].concat())
                    .width(combo_width)
                    .selected_text(&self.isofit.grade)
                    .show_ui(ui, |ui| {
                        for grade in &dropdowns.it_numbers {
                            ui.selectable_value(&mut self.isofit.grade, grade.clone(), grade);
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

        let mut units = "µm";
        let mut scale = 1_000.0;

        if self.tolerance.upper.abs() >= 1.0 || self.tolerance.lower.abs() >= 1.0 {
            units = "mm";
            scale = 1.0;
        }

        ui.add_space(5.0);

        egui::Grid::new(id).striped(false).show(ui, |ui| {
            ui.label("Maximum:");
            ui.label(format!(
                "{:.} mm ({} {units})",
                decimals(self.upper_limit(), -1),
                decimals(scale * self.tolerance.upper, -1)
            ));
            ui.end_row();

            ui.label("Minimum:");
            ui.label(format!(
                "{:.} mm ({} {units})",
                decimals(self.lower_limit(), -1),
                decimals(scale * self.tolerance.lower, -1)
            ));
            ui.end_row();

            ui.label("Mid-limits:");
            ui.label(format!(
                "{:.} mm ± {:.} {units}",
                decimals(self.middle_limit(), -1),
                decimals(scale * self.tolerance.mid(), -1)
            ));
            ui.end_row();
        });
    }
}
