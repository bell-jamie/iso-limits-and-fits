use std::ops::RangeInclusive;

use crate::sections::{
    feature::Feature,
    tolerance::{GradesDeviations, IsoFit, Tolerance},
};
use egui::Ui;

pub struct Input {
    pub size: f64,
    pub tolerance: Tolerance,
    pub isofit: IsoFit,
    pub standard: bool,
}

impl Input {
    pub fn default_hole() -> Self {
        Input {
            size: 10.0,
            tolerance: Tolerance {
                upper: 0.015,
                lower: 0.0,
            },
            isofit: IsoFit {
                deviation: "H".to_owned(),
                grade: "7".to_owned(),
            },
            standard: true,
        }
    }

    pub fn default_shaft() -> Self {
        Input {
            size: 10.0,
            tolerance: Tolerance {
                upper: 0.0,
                lower: -0.009,
            },
            isofit: IsoFit {
                deviation: "h".to_owned(),
                grade: "6".to_owned(),
            },
            standard: true,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, hole: bool, id: &str) -> Option<Feature> {
        let dropdowns = GradesDeviations::default();

        ui.horizontal(|ui| {
            ui.toggle_value(&mut self.standard, "ISO");

            if hole {
                ui.label("Hole :");
            } else {
                ui.label("Shaft: ");
            }

            ui.add(
                egui::DragValue::new(&mut self.size)
                    .speed(0.1)
                    .range(RangeInclusive::new(0.0, 3_000.0)),
            );

            if self.standard {
                // Set tolerance values to selected ISO value

                if let Some(mut tolerance) = self.isofit.convert(self.size) {
                    tolerance.round(3);
                    self.tolerance = tolerance;
                }

                egui::ComboBox::from_id_salt([id, "deviation"].concat())
                    .selected_text(&self.isofit.deviation)
                    .show_ui(ui, |ui| {
                        if hole {
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
                    .selected_text(&self.isofit.grade)
                    .show_ui(ui, |ui| {
                        for grade in &dropdowns.it_numbers {
                            ui.selectable_value(&mut self.isofit.grade, grade.clone(), grade);
                        }
                    });
                ui.end_row();
            } else {
                egui::Grid::new([id, "non_standard"])
                    .striped(false)
                    .min_col_width(1.0)
                    .show(ui, |ui| {
                        ui.label("Upper");
                        ui.add(
                            egui::DragValue::new(&mut self.tolerance.upper)
                                .speed(0.001)
                                .min_decimals(3),
                        );
                        ui.end_row();
                        ui.label("Lower");
                        ui.add(
                            egui::DragValue::new(&mut self.tolerance.lower)
                                .speed(0.001)
                                .min_decimals(3),
                        );
                        ui.end_row();
                    });
            }
        });

        if self.standard {
            if let Some(tolerance) = self.isofit.convert(self.size) {
                Some(Feature::create(&tolerance, self.size))
            } else {
                None
            }
        } else {
            Some(Feature::create(&self.tolerance, self.size))
        }
    }
}
