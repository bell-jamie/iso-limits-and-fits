use crate::Studio;
use crate::modules::component::Component;
use crate::modules::component::Focus;
use crate::modules::feature::Feature;
use crate::modules::utils::decimals;
use egui::{Align, Frame, Grid, Layout, Ui};

/// Wrapper type for component drag payload
#[derive(Clone, Copy)]
pub struct ComponentDrag(pub usize);

/// Wrapper type for material drag payload
#[derive(Clone, Copy)]
pub struct MaterialDrag(pub usize);

/// Identifies whether a card is for hub or shaft
#[derive(Clone, Copy)]
enum CardType {
    Hub,
    Shaft,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CardGrid {
    gap: f32,
    card_width: f32,
}

impl Default for CardGrid {
    fn default() -> Self {
        Self {
            gap: 7.0,
            card_width: 250.0,
        }
    }
}

impl CardGrid {
    fn component_input(&self, app: &mut Studio, ui: &mut Ui, card_type: CardType) {
        let (name, component_id) = match card_type {
            CardType::Hub => (app.get_hub_name().unwrap_or("Hub").to_string(), app.hub_id),
            CardType::Shaft => (
                app.get_shaft_name().unwrap_or("Shaft").to_string(),
                app.shaft_id,
            ),
        };
        let advanced = app.state.advanced;
        let card_id = match card_type {
            CardType::Hub => "hub_card",
            CardType::Shaft => "shaft_card",
        };

        // Extract data we need before mutable borrow
        let (focus, material_id, compliment) = {
            let component = match card_type {
                CardType::Hub => app.get_hub_mut(),
                CardType::Shaft => app.get_shaft_mut(),
            };
            let Some(component) = component else {
                return;
            };

            // Handle auto-scale (sync is handled in feature.rs when size changes)
            component.handle_auto_scale(ui);

            // If not in advanced mode, force focus to primary feature
            // Hub's primary feature is inner diameter, shaft's is outer diameter
            if !advanced {
                component.focus = match card_type {
                    CardType::Hub => Focus::Inner,
                    CardType::Shaft => Focus::Outer,
                };
            }

            let focus = component.focus.clone();
            let material_id = component.material_id;
            let compliment = match focus {
                Focus::Inner => component.outer_diameter.clone(),
                Focus::Outer => component.inner_diameter.clone(),
                Focus::Material => component.inner_diameter.clone(), // not used
            };
            (focus, material_id, compliment)
        };

        // Check if we're dragging a component or material over this card
        let is_being_dragged = egui::DragAndDrop::has_payload_of_type::<ComponentDrag>(ui.ctx())
            || egui::DragAndDrop::has_payload_of_type::<MaterialDrag>(ui.ctx());
        let frame = Frame::group(ui.style());

        let frame_response = frame.show(ui, |ui| {
            ui.push_id(card_id, |ui| {
                ui.set_width(ui.available_width());
                // Title bar
                ui.horizontal(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let component = match card_type {
                            CardType::Hub => app.get_hub_mut(),
                            CardType::Shaft => app.get_shaft_mut(),
                        };
                        if let Some(component) = component {
                            component_input_title_bar(ui, component, advanced);
                        }
                        let name_mut = match card_type {
                            CardType::Hub => app.get_hub_name_mut(),
                            CardType::Shaft => app.get_shaft_name_mut(),
                        };
                        if let Some(name) = name_mut {
                            ui.text_edit_singleline(name);
                        }
                    });
                });
                ui.separator();

                // Content based on focus
                // is_primary: hub's inner is primary, shaft's outer is primary
                let is_hub = matches!(card_type, CardType::Hub);
                match focus {
                    Focus::Inner => {
                        let is_primary = is_hub; // inner is primary for hub
                        if let Some(component) = app.component_library.get_mut(component_id) {
                            // Primary features are always enabled
                            if is_primary {
                                component.inner_diameter.enabled = true;
                            }
                            component.inner_diameter.show(
                                ui,
                                &mut app.state,
                                &name,
                                &compliment,
                                is_primary,
                            );
                        }
                    }
                    Focus::Outer => {
                        let is_primary = !is_hub; // outer is primary for shaft
                        if let Some(component) = app.component_library.get_mut(component_id) {
                            // Primary features are always enabled
                            if is_primary {
                                component.outer_diameter.enabled = true;
                            }
                            component.outer_diameter.show(
                                ui,
                                &mut app.state,
                                &name,
                                &compliment,
                                is_primary,
                            );
                        }
                    }
                    Focus::Material => {
                        if let Some(mat) = app.material_library.get_mut(material_id) {
                            mat.input(ui, &mut Default::default(), &name);
                        }
                    }
                }
            });
        });

        // Highlight when dragging over
        if is_being_dragged {
            let rect = frame_response.response.rect;
            if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
                if rect.contains(pointer_pos) {
                    let stroke = egui::Stroke::new(1.5, ui.visuals().selection.bg_fill);
                    ui.painter().rect_stroke(
                        rect,
                        frame.corner_radius,
                        stroke,
                        egui::StrokeKind::Outside,
                    );
                }
            }
        }

        // Check payload type first, then consume only the matching one
        // dnd_release_payload consumes the payload, so we must check type before calling it
        let is_component_drag = egui::DragAndDrop::has_payload_of_type::<ComponentDrag>(ui.ctx());
        let is_material_drag = egui::DragAndDrop::has_payload_of_type::<MaterialDrag>(ui.ctx());

        if is_component_drag {
            if let Some(payload) = frame_response
                .response
                .dnd_release_payload::<ComponentDrag>()
            {
                // Prevent the same component from being used as both hub and shaft
                match card_type {
                    CardType::Hub => {
                        if payload.0 != app.shaft_id {
                            app.hub_id = payload.0;
                        }
                    }
                    CardType::Shaft => {
                        if payload.0 != app.hub_id {
                            app.shaft_id = payload.0;
                        }
                    }
                }
            }
        } else if is_material_drag {
            if let Some(payload) = frame_response
                .response
                .dnd_release_payload::<MaterialDrag>()
            {
                match card_type {
                    CardType::Hub => {
                        if let Some(hub) = app.get_hub_mut() {
                            hub.material_id = payload.0;
                        }
                    }
                    CardType::Shaft => {
                        if let Some(shaft) = app.get_shaft_mut() {
                            shaft.material_id = payload.0;
                        }
                    }
                }
            }
        }
    }

    fn fit_output(&self, app: &mut Studio, ui: &mut Ui) {
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

    fn fit_display(&self, app: &mut Studio, ui: &mut Ui) {
        Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(ui.available_height());

            crate::modules::plot::fit_display(app, ui);

            // let box_1 = egui_plot::BoxPlot::new(
            //     "hub_box",
            //     vec![egui_plot::BoxElem::new(
            //         1.0,
            //         egui_plot::BoxSpread::new(0.1, 0.2, 0.3, 0.4, 0.5),
            //     )],
            // );

            // egui_plot::Plot::new("test")
            //     .show_axes(false)
            //     .show_background(false)
            //     .show_grid(false)
            //     .show(ui, |plot_ui| {
            //         plot_ui.box_plot(box_1);
            //     });
        });
    }

    pub fn render_cards(&self, app: &mut Studio, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(self.card_width);
                self.component_input(app, ui, CardType::Hub);
                ui.add_space(self.gap);
                self.component_input(app, ui, CardType::Shaft);

                // Handle size sync after inputs but before fit calculation
                let state = app.state.clone();
                if let Some(hub) = app.get_hub_mut() {
                    hub.handle_sync(state.clone(), ui, true); // is_hub = true
                }
                if let Some(shaft) = app.get_shaft_mut() {
                    shaft.handle_sync(state, ui, false); // is_hub = false
                }

                ui.add_space(self.gap);
                self.fit_output(app, ui);
            });
            ui.vertical(|ui| {
                ui.set_width(self.card_width / 2.0);
                self.fit_display(app, ui);
            })
            // self.visual // you were working here!
        });
    }
}

/// Renders focus buttons for a component title bar.
/// Called within a right-to-left layout, so buttons are added in reverse order.
fn component_input_title_bar(ui: &mut Ui, component: &mut Component, advanced: bool) {
    // Focus buttons in reverse order (right-to-left layout)
    for (focus_val, label) in [
        (Focus::Material, "MAT"),
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
