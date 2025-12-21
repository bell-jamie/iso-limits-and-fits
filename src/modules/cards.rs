use crate::LimitsFitsApp;
use crate::modules::component::Component;
use crate::modules::component::Focus;
use crate::modules::utils::decimals;
use egui::{Align, Frame, Grid, Layout, Ui};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CardGrid {
    gap: f32,
    card_width: f32,
}

impl Default for CardGrid {
    fn default() -> Self {
        Self {
            gap: 10.0,
            card_width: 250.0,
        }
    }
}

impl CardGrid {
    fn hub_input(&self, app: &mut LimitsFitsApp, ui: &mut Ui) {
        let name = app.get_hub_name().unwrap_or("Hub").to_string();
        let advanced = app.state.advanced;

        // Extract data we need before mutable borrow
        let (focus, material_id, compliment) = {
            let Some(hub) = app.get_hub_mut() else {
                return;
            };

            // If not in advanced mode, force focus to primary feature
            if !advanced {
                hub.focus = if hub.inner_diameter.primary {
                    Focus::Inner
                } else {
                    Focus::Outer
                };
            }

            let focus = hub.focus.clone();
            let material_id = hub.material_id;
            let compliment = match focus {
                Focus::Inner => hub.outer_diameter.clone(),
                Focus::Outer => hub.inner_diameter.clone(),
                Focus::Material => hub.inner_diameter.clone(), // not used
            };
            (focus, material_id, compliment)
        };

        Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            // Title bar
            ui.horizontal(|ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if let Some(hub) = app.get_hub_mut() {
                        component_title_bar(ui, hub, advanced);
                    }
                    if let Some(name) = app.get_hub_name_mut() {
                        // TODO this will need an "else" clause
                        ui.text_edit_singleline(name);
                    }
                });
            });
            ui.separator();

            // Content based on focus
            let hub_id = app.hub_id;
            match focus {
                Focus::Inner => {
                    if let Some(hub) = app.hub_library.get_mut(hub_id) {
                        hub.inner_diameter
                            .show(ui, &mut app.state, &name, &compliment);
                    }
                }
                Focus::Outer => {
                    if let Some(hub) = app.hub_library.get_mut(hub_id) {
                        hub.outer_diameter
                            .show(ui, &mut app.state, &name, &compliment);
                    }
                }
                Focus::Material => {
                    if let Some(mat) = app.material_library.get_mut(material_id) {
                        mat.input(ui, &mut Default::default(), &name);
                    }
                }
            }
        });
    }

    fn shaft_input(&self, app: &mut LimitsFitsApp, ui: &mut Ui) {
        let name = app.get_shaft_name().unwrap_or("Shaft").to_string();
        let advanced = app.state.advanced;

        // Extract data we need before mutable borrow
        let (focus, material_id, compliment) = {
            let Some(shaft) = app.get_shaft_mut() else {
                return;
            };

            // If not in advanced mode, force focus to primary feature
            if !advanced {
                shaft.focus = if shaft.inner_diameter.primary {
                    Focus::Inner
                } else {
                    Focus::Outer
                };
            }

            let focus = shaft.focus.clone();
            let material_id = shaft.material_id;
            let compliment = match focus {
                Focus::Inner => shaft.outer_diameter.clone(),
                Focus::Outer => shaft.inner_diameter.clone(),
                Focus::Material => shaft.inner_diameter.clone(), // not used
            };
            (focus, material_id, compliment)
        };

        Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            // Title bar
            ui.horizontal(|ui| {
                ui.label(&name);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if let Some(shaft) = app.get_shaft_mut() {
                        component_title_bar(ui, shaft, advanced);
                    }
                });
            });
            ui.separator();

            // Content based on focus
            let shaft_id = app.shaft_id;
            match focus {
                Focus::Inner => {
                    if let Some(shaft) = app.shaft_library.get_mut(shaft_id) {
                        shaft
                            .inner_diameter
                            .show(ui, &mut app.state, &name, &compliment);
                    }
                }
                Focus::Outer => {
                    if let Some(shaft) = app.shaft_library.get_mut(shaft_id) {
                        shaft
                            .outer_diameter
                            .show(ui, &mut app.state, &name, &compliment);
                    }
                }
                Focus::Material => {
                    if let Some(mat) = app.material_library.get_mut(material_id) {
                        mat.input(ui, &mut Default::default(), &name);
                    }
                }
            }
        });
    }

    fn fit_output(&self, app: &mut LimitsFitsApp, ui: &mut Ui) {
        let (hub, shaft) = match (app.get_hub(), app.get_shaft()) {
            (Some(h), Some(s)) => (h, s),
            _ => return,
        };

        // Calculate fit values
        let mmc = hub.inner_diameter.lower_limit(None) - shaft.outer_diameter.upper_limit(None);
        let lmc = hub.inner_diameter.upper_limit(None) - shaft.outer_diameter.lower_limit(None);
        let mid = (mmc + lmc) / 2.0;

        // Determine fit type
        let fit_kind = if mmc >= 0.0 {
            "Clearance"
        } else if lmc <= 0.0 {
            "Interference"
        } else {
            "Transition"
        };

        // Determine units based on magnitude
        let (units, scale) = if mmc.abs() < 1.0 && lmc.abs() < 1.0 {
            ("µm", 1_000.0)
        } else {
            ("mm", 1.0)
        };

        let condition = |mc: f64| {
            if mc.is_sign_positive() {
                "clearance"
            } else {
                "interference"
            }
        };

        let mmc_type = condition(mmc);
        let lmc_type = condition(lmc);
        let mid_type = condition(mid);

        // Build fit title string for copy button
        let fit_text = if hub.inner_diameter.standard && shaft.outer_diameter.standard {
            if hub.inner_diameter.size == shaft.outer_diameter.size {
                Some(format!(
                    "{} {}{} / {}{}",
                    hub.inner_diameter.size,
                    hub.inner_diameter.iso.deviation,
                    hub.inner_diameter.iso.grade,
                    shaft.outer_diameter.iso.deviation,
                    shaft.outer_diameter.iso.grade,
                ))
            } else {
                Some(format!(
                    "{} {}{} / {} {}{}",
                    hub.inner_diameter.size,
                    hub.inner_diameter.iso.deviation,
                    hub.inner_diameter.iso.grade,
                    shaft.outer_diameter.size,
                    shaft.outer_diameter.iso.deviation,
                    shaft.outer_diameter.iso.grade,
                ))
            }
        } else {
            None
        };

        Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            // Title bar
            ui.horizontal(|ui| {
                ui.label(format!("{} Fit", fit_kind));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    // Copy button for fit designation (reversed order due to right-to-left)
                    if let Some(text) = &fit_text {
                        if ui.button(text).on_hover_text("Copy").clicked() {
                            ui.ctx().copy_text(text.clone());
                        }
                    }
                });
            });
            ui.separator();

            // Content
            Grid::new("fit_output")
                .striped(false)
                .min_col_width(10.0)
                .spacing([15.0, 4.0])
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
        });
    }

    pub fn render_cards(&self, app: &mut LimitsFitsApp, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(self.card_width);
                self.hub_input(app, ui);
                ui.add_space(self.gap);
                self.shaft_input(app, ui);
                ui.add_space(self.gap);
                self.fit_output(app, ui);
            });
            // self.visual // you were working here!
        });
    }
}

/// Renders focus buttons for a component title bar.
/// Called within a right-to-left layout, so buttons are added in reverse order.
fn component_title_bar(ui: &mut Ui, component: &mut Component, advanced: bool) {
    // Focus buttons in reverse order (right-to-left layout)
    for (focus_val, label) in [
        (Focus::Material, "MATL"),
        (Focus::Outer, "OD"),
        (Focus::Inner, "ID"),
    ] {
        let selected = component.focus == focus_val;
        let button = egui::Button::new(label)
            .selected(selected)
            .frame(true)
            .frame_when_inactive(true);

        if ui.add_enabled(advanced, button).clicked() {
            component.focus = focus_val;
        }
    }
}
