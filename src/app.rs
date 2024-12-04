use crate::sections::{feature::Feature, fit::Fit, visual_fit::VisualFit};
use egui::Color32;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
// if we add new fields, give them default values when deserializing old state

pub struct LimitsFitsApp {
    hole: Feature,
    shaft: Feature,
    fit: Fit,
    #[serde(skip)]
    test_visual: VisualFit,
    sync_size: bool,
}

impl Default for LimitsFitsApp {
    fn default() -> Self {
        Self {
            hole: Feature::default_hole(),
            shaft: Feature::default_shaft(),
            fit: Fit::default(),
            test_visual: VisualFit::default(),
            sync_size: true,
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
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);

                // Add sync button and inital sync
                if ui.toggle_value(&mut self.sync_size, "Sync").clicked() {
                    self.shaft.size = self.hole.size;
                }

                // ui.toggle_value(&mut self.test_visual.display, "Visual");

                if ui.add(egui::Button::new("Reset")).clicked() {
                    self.hole = Feature::default_hole();
                    self.shaft = Feature::default_shaft();
                    self.fit = Fit::default();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("ISO Limits and Fits Tool");

            // ----------------------------------------------------------------------------

            let (hole_size_last, shaft_size_last) = (self.hole.size, self.shaft.size);

            ui.add_space(10.0);

            self.hole.show(ui, "hole_feature");

            ui.add_space(10.0);

            self.shaft.show(ui, "shaft_feature");

            ui.add_space(10.0);

            // Size sync button
            if self.sync_size {
                if self.hole.size != hole_size_last {
                    self.shaft.size = self.hole.size;
                } else if self.shaft.size != shaft_size_last {
                    self.hole.size = self.shaft.size;
                }
            }

            // ui.separator();

            self.fit = Fit::new(&self.hole, &self.shaft);
            self.fit.show(ui, "fit_results");

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });

        if self.test_visual.display {
            egui::SidePanel::right("right_panel").show(ctx, |ui| {
                self.test_visual.show(ui, &self.fit, "test_visual");
            });
        }
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let changelog = String::from("Version Notes\n\n- UI restructure\n- Size sync button added\n- Visuals temporarily removed again\n- Dropdowns temporarily limited to JS/js\n");
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
            .on_hover_text(changelog);
        ui.colored_label(release_colour, " alpha")
            .on_hover_cursor(egui::CursorIcon::Help)
            .on_hover_text("This is an alpha release, bugs are to be expected — check your work.");
    });
}
