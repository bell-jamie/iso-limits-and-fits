use crate::Studio;
use crate::modules::component::Focus;
use crate::modules::feature::show_feature;
use crate::modules::material::Material;
use crate::modules::utils::{at_temp, fix_dp, format_temp, parse_temp, truncate_string};
use egui::{Align, Frame, Grid, Layout, Ui};
use egui::{DragValue, RichText};

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
    /// Renders a component input card (hub or shaft).
    ///
    /// Follows the extract-then-mutate pattern to handle borrow checker constraints
    /// when Feature::show needs both &mut self and &mut Studio.
    fn component_input(&self, app: &mut Studio, ui: &mut Ui, card_type: CardType) {
        // EXTRACT: Gather all data needed from app
        let (name, component_id, advanced, card_id, is_hub) = {
            let (name, id) = match card_type {
                CardType::Hub => (
                    truncate_string(app.library.get_hub_name().unwrap_or("Hub"), 10),
                    app.library.hub_id,
                ),
                CardType::Shaft => (
                    truncate_string(app.library.get_shaft_name().unwrap_or("Shaft"), 10),
                    app.library.shaft_id,
                ),
            };
            let card_id = match card_type {
                CardType::Hub => "hub_card",
                CardType::Shaft => "shaft_card",
            };
            (
                name,
                id,
                app.state.advanced,
                card_id,
                matches!(card_type, CardType::Hub),
            )
        };

        // EXTRACT: Get component data and handle auto-scale
        let (focus, compliment) = {
            let component = match card_type {
                CardType::Hub => app.library.get_hub_mut(),
                CardType::Shaft => app.library.get_shaft_mut(),
            };
            let Some(component) = component else {
                return;
            };

            // Handle auto-scale (sync is handled in feature.rs when size changes)
            component.handle_auto_scale(ui);

            // If not in advanced mode, force focus to primary feature
            if !advanced {
                component.focus = match card_type {
                    CardType::Hub => Focus::Inner,
                    CardType::Shaft => Focus::Outer,
                };
            }

            let focus = component.focus.clone();
            let compliment = match focus {
                Focus::Inner => component.outer_diameter.clone(),
                Focus::Outer => component.inner_diameter.clone(),
            };
            (focus, compliment)
        };

        // Check drag state before rendering
        let is_being_dragged = egui::DragAndDrop::has_payload_of_type::<ComponentDrag>(ui.ctx())
            || egui::DragAndDrop::has_payload_of_type::<MaterialDrag>(ui.ctx());
        let frame = Frame::group(ui.style());

        // RENDER: Main card frame
        let frame_response = frame.show(ui, |ui| {
            ui.push_id(card_id, |ui| {
                ui.set_width(ui.available_width());

                // Title bar with focus buttons and name input
                self.component_input_title_bar(app, ui, card_type, advanced);
                ui.separator();

                // Content based on focus
                self.component_input_content(
                    app,
                    ui,
                    &focus,
                    component_id,
                    &name,
                    &compliment,
                    is_hub,
                );
            });
        });

        // Highlight when dragging over
        if is_being_dragged {
            self.component_input_drag_highlight(ui, &frame_response.response, &frame);
        }

        // Handle drag-and-drop payload release
        self.component_input_dnd_release(app, ui, card_type, &frame_response.response);
    }

    /// Renders the component input title bar with focus buttons and name input.
    fn component_input_title_bar(
        &self,
        app: &mut Studio,
        ui: &mut Ui,
        card_type: CardType,
        advanced: bool,
    ) {
        ui.horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // Focus buttons (OD, ID)
                if let Some(component) = match card_type {
                    CardType::Hub => app.library.get_hub_mut(),
                    CardType::Shaft => app.library.get_shaft_mut(),
                } {
                    for (focus_val, label) in [(Focus::Outer, "OD"), (Focus::Inner, "ID")] {
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

                // Name input
                if let Some(name) = match card_type {
                    CardType::Hub => app.library.get_hub_name_mut(),
                    CardType::Shaft => app.library.get_shaft_name_mut(),
                } {
                    ui.text_edit_singleline(name);
                }
            });
        });
    }

    /// Renders the component input card content based on the current focus.
    fn component_input_content(
        &self,
        app: &mut Studio,
        ui: &mut Ui,
        focus: &Focus,
        component_id: usize,
        name: &str,
        compliment: &crate::modules::feature::Feature,
        is_hub: bool,
    ) {
        let is_primary = match focus {
            Focus::Inner => is_hub,
            Focus::Outer => !is_hub,
        };

        // Enable primary features
        if is_primary {
            match focus {
                Focus::Inner => {
                    app.library.components[component_id].inner_diameter.enabled = true;
                }
                Focus::Outer => {
                    app.library.components[component_id].outer_diameter.enabled = true;
                }
            }
        }

        // Use free function with split borrows to avoid borrow conflicts
        let Studio { state, library, .. } = app;
        let feature = match focus {
            Focus::Inner => &mut library.components[component_id].inner_diameter,
            Focus::Outer => &mut library.components[component_id].outer_diameter,
        };
        show_feature(feature, state, ui, name, compliment, is_primary);
    }

    /// Renders the drag highlight when hovering over a component input card.
    fn component_input_drag_highlight(
        &self,
        ui: &mut Ui,
        response: &egui::Response,
        frame: &Frame,
    ) {
        let rect = response.rect;
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

    /// Handles drag-and-drop payload release for component input cards.
    fn component_input_dnd_release(
        &self,
        app: &mut Studio,
        ui: &mut Ui,
        card_type: CardType,
        response: &egui::Response,
    ) {
        let is_component_drag = egui::DragAndDrop::has_payload_of_type::<ComponentDrag>(ui.ctx());
        let is_material_drag = egui::DragAndDrop::has_payload_of_type::<MaterialDrag>(ui.ctx());

        if is_component_drag {
            if let Some(payload) = response.dnd_release_payload::<ComponentDrag>() {
                match card_type {
                    CardType::Hub => {
                        if payload.0 != app.library.shaft_id {
                            app.library.hub_id = payload.0;
                        }
                    }
                    CardType::Shaft => {
                        if payload.0 != app.library.hub_id {
                            app.library.shaft_id = payload.0;
                        }
                    }
                }
            }
        } else if is_material_drag {
            if let Some(payload) = response.dnd_release_payload::<MaterialDrag>() {
                match card_type {
                    CardType::Hub => {
                        if let Some(hub) = app.library.get_hub_mut() {
                            hub.material_id = payload.0;
                        }
                    }
                    CardType::Shaft => {
                        if let Some(shaft) = app.library.get_shaft_mut() {
                            shaft.material_id = payload.0;
                        }
                    }
                }
            }
        }
    }

    fn fit_output(&self, app: &mut Studio, ui: &mut Ui) {
        let (hub, shaft) = match (app.library.get_hub(), app.library.get_shaft()) {
            (Some(h), Some(s)) => (h, s),
            _ => return,
        };

        // Calculate fit values
        let mmc = hub.inner_diameter.lower_limit() - shaft.outer_diameter.upper_limit();
        let lmc = hub.inner_diameter.upper_limit() - shaft.outer_diameter.lower_limit();
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
                    ui.label(format!("{:.} {units}", fix_dp(scale * mmc.abs(), 1)));
                    ui.label(mmc_type);
                    ui.end_row();

                    ui.label("🌓")
                        .on_hover_cursor(egui::CursorIcon::Default)
                        .on_hover_text("Mid limits");
                    ui.label(format!("{:.} {units}", fix_dp(scale * mid.abs(), 1)));
                    ui.label(mid_type);
                    ui.end_row();

                    ui.label("🌕")
                        .on_hover_cursor(egui::CursorIcon::Default)
                        .on_hover_text("Least material condition");
                    ui.label(format!("{:.} {units}", fix_dp(scale * lmc.abs(), 1)));
                    ui.label(lmc_type);
                    ui.end_row();
                });
        });
    }

    fn fit_display(&self, app: &mut Studio, ui: &mut Ui) {
        Frame::group(ui.style()).show(ui, |ui| {
            // ui.set_width(ui.available_width());
            // ui.set_height(ui.available_height());
            crate::modules::plot::fit_display(app, ui);
        });
    }

    fn temp_display(&self, app: &mut Studio, ui: &mut Ui) {
        // Get component names for colour pickers (truncated to 12 chars)
        let hub_name = app
            .library
            .get_hub()
            .map(|h| truncate_string(&h.name, 12))
            .unwrap_or_else(|| "Hub".to_string());
        let shaft_name = app
            .library
            .get_shaft()
            .map(|s| truncate_string(&s.name, 12))
            .unwrap_or_else(|| "Shaft".to_string());

        Frame::group(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Thermal"));
                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    if ui.button("➖").clicked() {
                        app.thermal.enabled = false;
                    };
                });
            });
            ui.separator();

            // Top row: Settings in horizontal layout
            ui.horizontal(|ui| {
                // Temperature inputs
                ui.vertical(|ui| {
                    ui.label(RichText::new("Temperature").strong());
                    Grid::new("thermal_temps").num_columns(2).show(ui, |ui| {
                        ui.label("Lower");
                        ui.add(
                            DragValue::new(&mut app.thermal.lower)
                                .range(-273.15..=app.thermal.upper)
                                .custom_formatter(|t, _| format_temp(t))
                                .custom_parser(|s| parse_temp(s))
                                .min_decimals(1)
                                .speed(1),
                        );
                        ui.end_row();

                        ui.label("Upper");
                        ui.add(
                            DragValue::new(&mut app.thermal.upper)
                                .range(app.thermal.lower..=10_000.0)
                                .custom_formatter(|t, _| format_temp(t))
                                .custom_parser(|s| parse_temp(s))
                                .min_decimals(1)
                                .speed(1),
                        );
                        ui.end_row();
                    });
                });

                ui.separator();

                // Component colour pickers
                ui.vertical(|ui| {
                    ui.label(RichText::new("Colours").strong());
                    ui.horizontal(|ui| {
                        ui.color_edit_button_srgba(&mut app.thermal.hub_colour);
                        ui.label(&hub_name);
                        if ui
                            .small_button("↺")
                            .on_hover_text("Reset to default")
                            .clicked()
                        {
                            app.thermal.hub_colour =
                                egui::Color32::RED.gamma_multiply(app.thermal.colour_gamma);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.color_edit_button_srgba(&mut app.thermal.shaft_colour);
                        ui.label(&shaft_name);
                        if ui
                            .small_button("↺")
                            .on_hover_text("Reset to default")
                            .clicked()
                        {
                            app.thermal.shaft_colour =
                                egui::Color32::BLUE.gamma_multiply(app.thermal.colour_gamma);
                        }
                    });
                });

                ui.separator();

                // Intersection toggles
                ui.vertical(|ui| {
                    ui.label(RichText::new("Component Intersections").strong());
                    ui.checkbox(
                        &mut app.thermal.show_component_limit_intersections,
                        "Limits",
                    );
                    ui.checkbox(
                        &mut app.thermal.show_component_mid_intersections,
                        "Mid-limits",
                    );
                });
                ui.vertical(|ui| {
                    ui.label(RichText::new("Temperature Intersections").strong());
                    ui.checkbox(&mut app.thermal.show_temp_limit_intersections, "Limits");
                    ui.checkbox(&mut app.thermal.show_temp_mid_intersections, "Mid-limits");
                });
            });

            ui.separator();

            // Thermal fit outputs side by side
            if let (Some(hub), Some(shaft)) = (app.library.get_hub(), app.library.get_shaft()) {
                let hub_cte = app
                    .library
                    .get_material(hub.material_id)
                    .unwrap_or(&Material::default())
                    .cte;
                let shaft_cte = app
                    .library
                    .get_material(shaft.material_id)
                    .unwrap_or(&Material::default())
                    .cte;

                ui.horizontal(|ui| {
                    for (label, temp) in
                        [("Lower", app.thermal.lower), ("Upper", app.thermal.upper)]
                    {
                        ui.vertical(|ui| {
                            // Calculate dimensions at temperature
                            let hub_lower =
                                at_temp(hub.inner_diameter.lower_limit(), temp, hub_cte);
                            let hub_upper =
                                at_temp(hub.inner_diameter.upper_limit(), temp, hub_cte);
                            let shaft_lower =
                                at_temp(shaft.outer_diameter.lower_limit(), temp, shaft_cte);
                            let shaft_upper =
                                at_temp(shaft.outer_diameter.upper_limit(), temp, shaft_cte);

                            // Calculate fit values at temperature
                            let mmc = hub_lower - shaft_upper;
                            let lmc = hub_upper - shaft_lower;
                            let mid = (mmc + lmc) / 2.0;

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

                            ui.label(RichText::new(format!("{label} ({temp:.0}°C)")).strong());

                            Grid::new(format!("thermal_fit_{label}"))
                                .striped(false)
                                .min_col_width(10.0)
                                .spacing([15.0, 4.0])
                                .show(ui, |ui| {
                                    ui.label("🌑")
                                        .on_hover_cursor(egui::CursorIcon::Default)
                                        .on_hover_text("Max material condition");
                                    ui.label(format!("{:.} {units}", fix_dp(scale * mmc.abs(), 1)));
                                    ui.label(condition(mmc));
                                    ui.end_row();

                                    ui.label("🌓")
                                        .on_hover_cursor(egui::CursorIcon::Default)
                                        .on_hover_text("Mid limits");
                                    ui.label(format!("{:.} {units}", fix_dp(scale * mid.abs(), 1)));
                                    ui.label(condition(mid));
                                    ui.end_row();

                                    ui.label("🌕")
                                        .on_hover_cursor(egui::CursorIcon::Default)
                                        .on_hover_text("Least material condition");
                                    ui.label(format!("{:.} {units}", fix_dp(scale * lmc.abs(), 1)));
                                    ui.label(condition(lmc));
                                    ui.end_row();
                                });
                        });

                        ui.separator();
                    }
                });
            }

            ui.separator();
            crate::modules::thermal::fit_temp_plot(app, ui);
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
                let sync_size = app.state.sync_size;
                let synced_size = app.state.synced_size;
                if let Some(hub) = app.library.get_hub_mut() {
                    if sync_size {
                        hub.inner_diameter.size = synced_size;
                    }
                }
                if let Some(shaft) = app.library.get_shaft_mut() {
                    if sync_size {
                        shaft.outer_diameter.size = synced_size;
                    }
                }

                ui.add_space(self.gap);
                self.fit_output(app, ui);
            });
            ui.vertical(|ui| {
                ui.set_width(self.card_width / 2.0);
                self.fit_display(app, ui);
            });
            if app.thermal.enabled {
                ui.vertical(|ui| {
                    ui.set_width(self.card_width * 2.0);
                    self.temp_display(app, ui);
                });
            }
        });
    }
}
