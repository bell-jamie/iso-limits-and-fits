use crate::modules::{
    component::Component, fit::Fit, mat_data::material_list, material, material::Material, plot,
    utils::State,
};
use egui::{Button, Color32, CursorIcon, RichText};
use std::collections::BTreeSet;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct LimitsFitsApp {
    hub: Component,
    shaft: Component,
    state: State,
    materials: BTreeSet<Material>,
}

impl Default for LimitsFitsApp {
    fn default() -> Self {
        Self {
            hub: Component::default_hub(),
            shaft: Component::default_shaft(),
            state: State::default(),
            materials: material_list(),
        }
    }
}

impl LimitsFitsApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for LimitsFitsApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                egui::widgets::global_theme_preference_switch(ui);
                self.state.zoom.show(ui, ctx);

                ui.separator();

                ui.toggle_value(&mut self.state.advanced, "Advanced");
                // ui.toggle_value(&mut self.state.thermal, "Thermal");
                // ui.toggle_value(&mut self.state.interference, "Inteference");

                // ui.button("Stress").on_hover_text("Add me");

                if ui.add(Button::new("Reset")).clicked() {
                    self.hub = Component::default_hub();
                    self.shaft = Component::default_shaft();
                    self.state = State::default();
                    self.materials = material_list();
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
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("ISO Limits and Fits Tool");

            // Maybe the material feature button shouldn't be part of the enum and instead should be a toggle?
            // This would mean that it could keep displaying the info
            // Orrr maybe just all the information gets spat out at the end in a spreadsheet style thing

            ui.add_space(10.0);

            if self.state.advanced {
                ui.horizontal(|ui| {
                    self.hub.show(ui, &mut self.state, &mut self.materials);

                    ui.add_space(10.0);

                    self.shaft.show(ui, &mut self.state, &mut self.materials);
                });

                ui.add_space(10.0);

                material::temperature_input(ui, &mut self.state, &mut self.hub, &mut self.shaft);

                ui.add_space(10.0);

                plot::side_by_side(ui, &self.state, &self.hub, &self.shaft);

                ui.add_space(10.0);

                material::temperature_output(ui, &mut self.state, &self.hub, &self.shaft);

                ui.add_space(10.0);

                let mut fit = Fit::new(&self.hub, &self.shaft);

                fit.show_advanced(ui, &self.state);
            } else {
                // Simple mode
                self.hub.show(ui, &mut self.state, &mut self.materials);

                ui.add_space(10.0);

                self.shaft.show(ui, &mut self.state, &mut self.materials);

                ui.add_space(10.0);

                let fit = Fit::new(&self.hub, &self.shaft);
                fit.show(ui, &self.state);
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                signature(self, ui);
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

fn signature(app: &mut LimitsFitsApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let release_colour = Color32::from_rgb(0, 169, 0);
        // let version_colour = Color32::from_rgb(169, 0, 0);

        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(". Created by ");
        ui.hyperlink_to("James Bell", "https://www.linkedin.com/in/bell-jamie/");
        ui.label(".");
        ui.label(" v");
        ui.label(env!("CARGO_PKG_VERSION"))
            .on_hover_cursor(egui::CursorIcon::Help)
            .on_hover_text(format_changelog(CHANGELOG_ENTRIES));
        if ui.colored_label(release_colour, " alpha")
            .on_hover_cursor(egui::CursorIcon::Help)
            .on_hover_text("This is an alpha release, bugs are to be expected — check your work (like Soroush does).\nClick to enable debug mode.")
            .clicked() { app.state.debug = !app.state.debug; }
        if app.state.debug {
            ui.add_space(5.0);
            if ui.ctx().has_requested_repaint() {
                ui.colored_label(Color32::RED, "Repainting...");
            }
        }
    });
}
