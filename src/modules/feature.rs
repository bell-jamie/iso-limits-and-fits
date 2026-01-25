use crate::modules::{
    state::State,
    tolerance::{GradesDeviations, Iso, Tolerance},
    utils::SanitiseFloat,
};
use egui::{Button, ComboBox, DragValue, Grid, Ui};

#[derive(Clone, Default, serde::Deserialize, serde::Serialize)]
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

    pub fn upper_limit(&self) -> f64 {
        self.size + self.tolerance.upper
    }

    pub fn middle_limit(&self) -> f64 {
        (self.upper_limit() + self.lower_limit()) / 2.0
    }

    pub fn lower_limit(&self) -> f64 {
        self.size + self.tolerance.lower
    }
}

/// Renders the feature input and output UI.
///
/// This is a free function (not a method) to avoid borrow conflicts when
/// the feature is stored inside Studio and we need to pass both.
///
/// # Arguments
/// * `feature` - The feature to render
/// * `state` - Mutable reference to the app state (for sync behavior)
/// * `ui` - The egui UI context
/// * `id` - Unique identifier for this feature's UI elements
/// * `compliment` - The complementary feature (for size range constraints)
/// * `is_primary` - Whether this is the primary feature (affects sync behavior)
pub fn show_feature(
    feature: &mut Feature,
    state: &mut State,
    ui: &mut Ui,
    id: &str,
    compliment: &Feature,
    is_primary: bool,
) {
    ui.add_space(5.0);

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            feature_input_ui(feature, state, ui, id, compliment, is_primary);
            if feature.enabled {
                feature_output_ui(feature, ui, id);
            }
        });
    });
}

fn feature_input_ui(
    feature: &mut Feature,
    state: &mut State,
    ui: &mut Ui,
    id: &str,
    compliment: &Feature,
    is_primary: bool,
) {
    let dropdowns = GradesDeviations::default();
    let size_range = if compliment.enabled {
        if feature.hole {
            // prevent zero wall thickness
            0.0..=compliment.lower_limit() - feature.tolerance.upper
        } else {
            compliment.upper_limit() - feature.tolerance.lower..=3_150.0
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
                Button::selectable(feature.standard, "ISO").frame_when_inactive(true),
            )
            .on_hover_text("Toggle ISO limits")
            .clicked()
        {
            feature.standard = !feature.standard;
        }

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
                .add(Button::selectable(feature.enabled, "✔").frame_when_inactive(true))
                .on_hover_text("Enable dimension")
                .clicked()
            {
                feature.enabled = !feature.enabled;
            }
        }

        let size_drag = ui.add_enabled(feature.enabled, |ui: &mut Ui| {
            ui.add_sized(
                [flex_width, 18.0],
                DragValue::new(&mut feature.size)
                    .speed(0.1)
                    .range(size_range),
            )
            .on_hover_text("Size")
        });

        if is_primary && size_drag.changed() {
            state.synced_size = feature.size;
        }

        if feature.standard {
            ui.add_enabled(feature.enabled, |ui: &mut Ui| {
                ComboBox::from_id_salt(format!("{}_deviation", id))
                    .width(flex_width)
                    .selected_text(&feature.iso.deviation)
                    .show_ui(ui, |ui| {
                        for letter in if feature.hole {
                            &dropdowns.hole_letters
                        } else {
                            &dropdowns.shaft_letters
                        } {
                            ui.selectable_value(&mut feature.iso.deviation, letter.clone(), letter);
                        }
                    })
                    .response
                    .on_hover_text("Deviation")
            });

            ui.add_enabled(feature.enabled, |ui: &mut Ui| {
                ComboBox::from_id_salt(format!("{}_grade", id))
                    .width(flex_width)
                    .selected_text(&feature.iso.grade)
                    .show_ui(ui, |ui| {
                        for grade in &dropdowns.it_numbers {
                            ui.selectable_value(&mut feature.iso.grade, grade.clone(), grade);
                        }
                    })
                    .response
                    .on_hover_text("Grade")
            });
            ui.end_row();
        } else {
            ui.add_enabled(feature.enabled, |ui: &mut Ui| {
                ui.add_sized(
                    [flex_width, 18.0],
                    DragValue::new(&mut feature.tolerance.lower)
                        .speed(0.001)
                        .range(-feature.size..=feature.tolerance.upper)
                        .min_decimals(3),
                )
                .on_hover_text("Lower limit")
            });
            ui.add_enabled(feature.enabled, |ui: &mut Ui| {
                ui.add_sized(
                    [flex_width, 18.0],
                    DragValue::new(&mut feature.tolerance.upper)
                        .speed(0.001)
                        .range(feature.tolerance.lower..=f64::MAX)
                        .min_decimals(3),
                )
                .on_hover_text("Upper limit")
            });
        }
    });
}

fn feature_output_ui(feature: &mut Feature, ui: &mut Ui, id: &str) {
    if !feature.standard {
    } else if let Some(tolerance) = feature.iso.convert(feature.size) {
        feature.tolerance = tolerance;
    } else {
        ui.colored_label(egui::Color32::RED, "Invalid fundamental deviation")
            .on_hover_cursor(egui::CursorIcon::Help)
            .on_hover_text("This combination of size, deviation and tolerance grade does not exist within the ISO limits and fits system. Please refer to the ISO preferred fits.");
        return;
    }

    let (units, scale) =
        if feature.tolerance.upper.abs() < 1.0 && feature.tolerance.lower.abs() < 1.0 {
            ("µm", 1_000.0)
        } else {
            ("mm", 1.0)
        };

    ui.add_space(5.0);

    let (upper, middle, lower) = (
        feature.upper_limit().sanitise(6),
        feature.middle_limit().sanitise(6),
        feature.lower_limit().sanitise(6),
    );

    Grid::new(id)
        .striped(false)
        .min_col_width(10.0)
        .spacing([20.0, 4.0])
        .show(ui, |ui| {
            ui.label("⬆")
                .on_hover_cursor(egui::CursorIcon::Default)
                .on_hover_text("Upper limit");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("mm");
                ui.label(format!("{}", upper));
            });
            ui.label(if feature.tolerance.upper.is_sign_positive() {
                "+"
            } else {
                "-"
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(units);
                ui.label(format!("{}", scale * feature.tolerance.upper.abs()));
            });
            ui.end_row();

            ui.label("⬍")
                .on_hover_cursor(egui::CursorIcon::Default)
                .on_hover_text("Mid-limits");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("mm");
                ui.label(format!("{}", middle));
            });
            ui.label("±");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(units);
                ui.label(format!("{}", scale * feature.tolerance.mid()));
            });
            ui.end_row();

            ui.label("⬇")
                .on_hover_cursor(egui::CursorIcon::Default)
                .on_hover_text("Lower limit");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("mm");
                ui.label(format!("{}", lower));
            });
            ui.label(if feature.tolerance.lower.is_sign_positive() {
                "+"
            } else {
                "-"
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(units);
                ui.label(format!("{}", scale * feature.tolerance.lower.abs()));
            });
            ui.end_row();
        });
}
