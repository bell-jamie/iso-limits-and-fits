use crate::modules::{
    cards::CardGrid, component::Component, fit::Fit, mat_data::material_list, material,
    material::Material, plot, state, theme, utils,
};
use egui::{Button, Color32, CursorIcon, RichText, Ui};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Studio {
    pub hub_id: usize,
    pub shaft_id: usize,
    pub state: state::State,
    pub material_library: Vec<Material>,
    pub hub_library: Vec<Component>,
    pub shaft_library: Vec<Component>,
}

impl Default for Studio {
    fn default() -> Self {
        Self {
            hub_id: 0,
            shaft_id: 0,
            state: state::State::default(),
            material_library: material_list().into_iter().collect(),
            hub_library: vec![Component::default_hub()],
            shaft_library: vec![Component::default_shaft()],
        }
    }
}

impl Studio {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install(&cc.egui_ctx);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    fn show_status_bar(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(8, 4))
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

                        ui.label(env!("CARGO_PKG_VERSION"))
                            .on_hover_cursor(egui::CursorIcon::Help)
                            .on_hover_text(format_changelog(CHANGELOG_ENTRIES));
                        ui.label(" v");

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
        ui.heading("Library");
        ui.separator();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Hubs section
                utils::accordion(ui, "hubs_accordion", "Hubs", false, |ui| {
                    ui.label("Content of section 1");
                });
                ui.label(egui::RichText::new("Bolts").strong());
                ui.add_space(4.0);
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
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(ui, |ui| {
                card_grid.render_cards(self, ui);
            });
    }

    fn show_menu_bar(&mut self, ui: &mut Ui) {
        egui::MenuBar::new().ui(ui, |ui| {
            egui::widgets::global_theme_preference_switch(ui);
            self.state.zoom.show(ui);

            // ui.separator();

            // ui.toggle_value(&mut self.state.advanced, "Advanced");
            if ui
                .add(Button::selectable(self.state.advanced, "Advanced").frame_when_inactive(true))
                .clicked()
            {
                self.state.advanced = !self.state.advanced;
            }

            // ui.toggle_value(&mut self.state.thermal, "Thermal");
            // ui.toggle_value(&mut self.state.interference, "Inteference");

            // ui.button("Stress").on_hover_text("Add me");

            if ui
                .add(Button::new("Reset").frame_when_inactive(true))
                .clicked()
            {
                self.hub_id = 0;
                self.shaft_id = 0;
                self.state = state::State::default();
                self.material_library = material_list().into_iter().collect();
                self.hub_library = vec![Component::default_hub()];
                self.shaft_library = vec![Component::default_shaft()];
            }

            if self.state.debug {
                ui.separator();

                ui.label(RichText::new("DEBUG").strong().color(Color32::RED))
                    .on_hover_cursor(CursorIcon::default());

                ui.toggle_value(&mut self.state.force_valid, "Force Valid");

                // if ui.add(Button::new("Random")).clicked() {
                //     self.state.sync_size = false;
                //     self.hole = Feature::random(true, self.state.force_valid);
                //     self.shaft = Feature::random(false, self.state.force_valid);
                //     self.fit = Fit::new(&self.hole, &self.shaft);
                // }
            }
        });
    }

    pub fn get_hub_name(&self) -> Option<&str> {
        self.hub_library
            .get(self.state.hub_id)
            .map(|hub| hub.name.as_str())
    }

    pub fn get_hub_name_mut(&mut self) -> Option<&mut String> {
        if let Some(hub) = self.get_hub_mut() {
            Some(&mut hub.name)
        } else {
            None
        }
    }

    pub fn get_shaft_name(&self) -> Option<&str> {
        self.shaft_library
            .get(self.state.shaft_id)
            .map(|shaft| shaft.name.as_str())
    }

    pub fn get_shaft_name_mut(&mut self) -> Option<&mut String> {
        if let Some(shaft) = self.get_shaft_mut() {
            Some(&mut shaft.name)
        } else {
            None
        }
    }

    pub fn get_hub(&self) -> Option<&Component> {
        self.hub_library.get(self.hub_id)
    }

    pub fn get_shaft(&self) -> Option<&Component> {
        self.shaft_library.get(self.shaft_id)
    }

    pub fn get_hub_mut(&mut self) -> Option<&mut Component> {
        self.hub_library.get_mut(self.hub_id)
    }

    pub fn get_shaft_mut(&mut self) -> Option<&mut Component> {
        self.shaft_library.get_mut(self.shaft_id)
    }

    pub fn get_material(&self, id: usize) -> Option<&Material> {
        self.material_library.get(id)
    }

    pub fn get_material_mut(&mut self, id: usize) -> Option<&mut Material> {
        self.material_library.get_mut(id)
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

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            self.show_status_bar(ui);
        });

        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            self.show_menu_bar(ui);
        });

        if self.state.show_library_panel {
            egui::SidePanel::left("library_panel")
                .default_width(250.0)
                .resizable(true)
                .show(ctx, |ui| {
                    self.show_library_panel(ui);
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_central_content(ui);

            // ui.add_space(5.0);

            // ui.horizontal(|ui| {
            //     ui.heading(RichText::new("[PFS]").strong());
            //     ui.heading("|");
            //     ui.heading("Precision Fit Studio");
            // });

            // // Maybe the material feature button shouldn't be part of the enum and instead should be a toggle?
            // // This would mean that it could keep displaying the info
            // // Orrr maybe just all the information gets spat out at the end in a spreadsheet style thing

            // ui.add_space(10.0);

            // if self.state.advanced {
            //     ui.horizontal(|ui| {
            //         self.hub.show(ui, &mut self.state, &mut self.materials);

            //         ui.add_space(10.0);

            //         self.shaft.show(ui, &mut self.state, &mut self.materials);
            //     });

            //     ui.add_space(10.0);

            //     plot::side_by_side(ui, &self.state, &self.hub, &self.shaft);

            //     ui.add_space(10.0);

            //     material::temperature_input(ui, &mut self.state, &mut self.hub, &mut self.shaft);

            //     ui.add_space(10.0);

            //     material::temperature_output(ui, &mut self.state, &self.hub, &self.shaft);
            // } else {
            //     // Simple mode
            //     self.hub.show(ui, &mut self.state, &mut self.materials);

            //     ui.add_space(10.0);

            //     self.shaft.show(ui, &mut self.state, &mut self.materials);

            //     ui.add_space(10.0);

            //     let fit = Fit::new(&self.hub, &self.shaft);
            //     fit.show(ui, &self.state);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

struct ChangelogEntry {
    version: &'static str,
    notes: &'static [&'static str],
}

const CHANGELOG_ENTRIES: &[ChangelogEntry] = &[
    ChangelogEntry {
        version: "0.5.0",
        notes: &[
            "Full ISO limits and fits tables enabled.",
            "Debug mode added — click alpha.",
        ],
    },
    ChangelogEntry {
        version: "0.5.1",
        notes: &["Minor UI change for fits."],
    },
    ChangelogEntry {
        version: "0.5.2",
        notes: &[
            "Fixed manual limits not working.",
            "Tooltips added.",
            "Header bar tweaked.",
        ],
    },
    ChangelogEntry {
        version: "0.6.0",
        notes: &[
            "Thermal fit analysis added.",
            "General UI tweaks and new symbols.",
        ],
    },
    ChangelogEntry {
        version: "0.6.1",
        notes: &["Added zoom feature."],
    },
    ChangelogEntry {
        version: "0.6.2",
        notes: &[
            "Added temperature sync.",
            "Added separate temperature output.",
            "UI tweaks.",
        ],
    },
    ChangelogEntry {
        version: "0.6.3",
        notes: &["Soroush quickfix to lookup table."],
    },
    ChangelogEntry {
        version: "0.6.4",
        notes: &["Corrected logic for P to ZC deviation deltas."],
    },
    ChangelogEntry {
        version: "0.7.0",
        notes: &[
            "Simple mode and advanced mode.",
            "Interference stresses",
            "Zoom feature tweaked.",
        ],
    },
];

fn format_changelog(entries: &[ChangelogEntry]) -> String {
    let mut changelog = String::from("Version Notes\n\n");

    for entry in entries {
        changelog.push_str(&format!("{}\n", entry.version));
        for note in entry.notes {
            changelog.push_str(&format!("- {}\n", note));
        }
        changelog.push('\n');
    }

    // Pop off the last two newlines
    changelog.pop();
    changelog.pop();
    changelog
}
