use crate::modules::{
    cards::CardGrid, component::Component, mat_data::material_list, material, material::Material,
    material::show_material_editor, plot, theme, utils,
};
use crate::modules::{library::Library, settings::Settings, state::State, thermal::Thermal};
use egui::{Button, Color32, CursorIcon, RichText, Ui};

#[derive(serde::Deserialize, serde::Serialize)]
// #[serde(default)]
pub struct Studio {
    pub settings: Settings,
    pub state: State,
    pub thermal: Thermal,
    pub library: Library,
}

impl Default for Studio {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            state: State::default(),
            thermal: Thermal::default(),
            library: Library::default(),
        }
    }
}

impl Studio {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install(&cc.egui_ctx);

        let app: Self = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };

        cc.egui_ctx.set_zoom_factor(app.state.scale.value);
        app
    }

    fn show_menu_bar(&mut self, ui: &mut Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            if ui.button("🖹").on_hover_text("Library").clicked() {
                self.state.show_library_panel = !self.state.show_library_panel;
            };
            self.state.scale.show(ui);

            egui::widgets::global_theme_preference_switch(ui);

            ui.separator();

            // ui.toggle_value(&mut self.state.advanced, "Advanced");
            // if ui
            //     .add(Button::selectable(self.state.advanced, "Advanced").frame_when_inactive(true))
            //     .clicked()
            // {
            //     self.state.advanced = !self.state.advanced;
            // }

            ui.toggle_value(&mut self.thermal.enabled, "Thermal");
            // if ui.add(Button::selectable(self.state.thermal, "Thermal")).clicked() {
            //     self.state.thermal = !self.state.thermal;
            // }

            // ui.toggle_value(&mut self.state.thermal, "Thermal");
            // ui.toggle_value(&mut self.state.interference, "Inteference");

            // ui.button("Stress").on_hover_text("Add me");

            if ui
                .add(Button::new("Reset").frame_when_inactive(true))
                .clicked()
            {
                ui.ctx()
                    .data_mut(|d| d.insert_temp(egui::Id::new("pending_reset"), true));
            }

            if self.state.debug {
                ui.separator();
                ui.label(RichText::new("DEBUG").strong().color(Color32::RED))
                    .on_hover_cursor(CursorIcon::default());
                if ui
                    .add(
                        Button::selectable(self.state.advanced, "Advanced")
                            .frame_when_inactive(true),
                    )
                    .clicked()
                {
                    self.state.advanced = !self.state.advanced;
                }
                ui.toggle_value(&mut self.state.force_valid, "Force Valid");

                if ui.button("egui Settings").clicked() {
                    self.state.show_egui_settings = true;
                }

                egui::Window::new("Settings")
                    .id(egui::Id::new("egui_settings"))
                    .open(&mut self.state.show_egui_settings)
                    .collapsible(false)
                    .resizable(true)
                    .show(ui.ctx(), |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let ctx = ui.ctx().clone();
                            ctx.settings_ui(ui);
                        });
                    });

                // if ui.add(Button::new("Random")).clicked() {
                //     self.state.sync_size = false;
                //     self.hole = Feature::random(true, self.state.force_valid);
                //     self.shaft = Feature::random(false, self.state.force_valid);
                //     self.fit = Fit::new(&self.hole, &self.shaft);
                // }
            }
        });
    }

    fn show_status_bar(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let release_colour = Color32::from_rgb(0, 169, 0);

                    // Left section - signature
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.label("Created by ");
                    ui.hyperlink_to("James Bell", "https://www.linkedin.com/in/bell-jamie/");
                    ui.label(".");

                    // Right section - version info
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;

                        if ui.colored_label(release_colour, " alpha")
                            .on_hover_cursor(egui::CursorIcon::Help)
                            .on_hover_text("This is an alpha release, bugs are to be expected — check your work (like Soroush does).\nClick to enable debug mode.")
                            .clicked() { self.state.debug = !self.state.debug; }

                        ui.add_space(5.0);

                        if ui.add(Button::new(format!("v{}", env!("CARGO_PKG_VERSION"))).frame(false))
                            .on_hover_text("View changelog")
                            .clicked()
                        {
                            ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("show_changelog"), true));
                        }

                        if self.state.debug {
                            ui.add_space(5.0);
                            if ui.ctx().has_requested_repaint() {
                                ui.colored_label(Color32::RED, "Repainting...");
                            }
                        }
                    });
                });
            });
    }

    fn show_library_panel(&mut self, ui: &mut egui::Ui) {
        ui.add_space(5.0);
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Library").strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(Button::new("↺").frame(false).small())
                    .on_hover_text("Reset library")
                    .clicked()
                {
                    ui.ctx()
                        .data_mut(|d| d.insert_temp(egui::Id::new("pending_library_reset"), true));
                }
            });
        });
        ui.separator();

        let Self { state, library, .. } = self;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                library.render(state, ui);
            });
    }

    fn show_central_content(&mut self, ui: &mut Ui) {
        let card_grid = CardGrid::default();

        ui.horizontal(|ui| {
            ui.heading(RichText::new("[PFS]").strong());
            // ui.heading("|");
            ui.heading(RichText::new("Precision Fit Studio"));
        });

        ui.add_space(ui.style().spacing.window_margin.bottomf());

        egui::ScrollArea::both()
            .auto_shrink([false, false])
            // .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                card_grid.render_cards(self, ui);
            });
    }
}

impl eframe::App for Studio {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        crate::modules::shortcuts::inputs(ctx, self);

        crate::modules::modal::delete_component(ctx, self);
        crate::modules::modal::delete_material(ctx, self);
        crate::modules::modal::reset_confirm(ctx, self);
        crate::modules::modal::library_reset_confirm(ctx, self);
        crate::modules::modal::thermal_colours(ctx, self);
        crate::modules::modal::changelog(ctx);

        // Material editor window
        if self.state.show_material_editor {
            show_material_editor(self, ctx);
        }

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.show_status_bar(ui);
        });

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.show_menu_bar(ui);
        });

        if self.state.show_library_panel {
            egui::SidePanel::left("library_panel")
                .default_width(250.0)
                .min_width(150.0)
                .resizable(true)
                .show(ctx, |ui| {
                    self.show_library_panel(ui);
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_central_content(ui);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
