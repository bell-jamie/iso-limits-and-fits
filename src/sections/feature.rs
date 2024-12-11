use egui::{ComboBox, DragValue, Grid, RichText};
use rand::Rng;

use super::{
    material::Material,
    tolerance::{GradesDeviations, Iso, Tolerance},
    utils::{decimals, State},
};

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Feature {
    pub hole: bool,
    pub standard: bool,
    pub size: f64,
    pub iso: Iso,
    pub tolerance: Tolerance,
    pub mat: Material,
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
            mat: Material::default(),
        }
    }

    pub fn default_shaft() -> Self {
        Feature {
            hole: false,
            standard: true,
            size: 10.0,
            iso: Iso::new("h", "6"),
            tolerance: Tolerance::new(0.0, -0.009),
            mat: Material::default(),
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
                mat: Material::default(),
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

    pub fn show(&mut self, ui: &mut egui::Ui, state: &State) {
        let dropdowns = GradesDeviations::default();
        let width = 50.0;
        let sized = [width, 18.0];
        let id = if self.hole { "hole" } else { "shaft" };

        ui.horizontal(|ui| {
            ui.label(RichText::new(if self.hole { "Hole:  " } else { "Shaft: " }).strong());
            ui.toggle_value(&mut self.standard, "ISO")
                .on_hover_text("Toggle ISO limits");
            ui.add(
                DragValue::new(&mut self.size)
                    .speed(0.1)
                    .range(0.0..=3_150.0),
            )
            .on_hover_text("Size");

            if self.standard {
                ComboBox::from_id_salt(format!("{}_deviation", id))
                    .width(width)
                    .selected_text(&self.iso.deviation)
                    .show_ui(ui, |ui| {
                        for letter in if self.hole {
                            &dropdowns.hole_letters
                        } else {
                            &dropdowns.shaft_letters
                        } {
                            ui.selectable_value(&mut self.iso.deviation, letter.clone(), letter);
                        }
                    })
                    .response
                    .on_hover_text("Deviation");

                ComboBox::from_id_salt(format!("{}_grade", id))
                    .width(width)
                    .selected_text(&self.iso.grade)
                    .show_ui(ui, |ui| {
                        for grade in &dropdowns.it_numbers {
                            ui.selectable_value(&mut self.iso.grade, grade.clone(), grade);
                        }
                    })
                    .response
                    .on_hover_text("Grade");
                ui.end_row();
            } else {
                Grid::new([id, "non_standard"].concat())
                    .striped(false)
                    .min_col_width(width)
                    .show(ui, |ui| {
                        ui.add_sized(
                            sized,
                            DragValue::new(&mut self.tolerance.upper)
                                .speed(0.001)
                                .range(self.tolerance.lower..=f64::MAX)
                                .min_decimals(3),
                        )
                        .on_hover_text("Upper limit");
                        ui.add_sized(
                            sized,
                            DragValue::new(&mut self.tolerance.lower)
                                .speed(0.001)
                                .range(-self.size..=self.tolerance.upper)
                                .min_decimals(3),
                        )
                        .on_hover_text("Lower limit");
                    });
            }

            if state.thermal {
                ui.add_sized(
                    sized,
                    DragValue::new(&mut self.mat.cte)
                        .speed(0.001)
                        .range(0.0..=f64::MAX)
                        .min_decimals(3),
                )
                .on_hover_text("Thermal expansion coefficient");
            }
        });

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

        Grid::new(id).striped(false).show(ui, |ui| {
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
    }
}
