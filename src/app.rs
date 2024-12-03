use std::ops::RangeInclusive;

use crate::sections::{fit::Fit, input::Input, visual_fit::VisualFit};
use egui::Color32;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
// if we add new fields, give them default values when deserializing old state

pub struct LimitsFitsApp {
    hole_input: Input,
    shaft_input: Input,
    fit: Fit,
    #[serde(skip)]
    test_visual: VisualFit,
}

impl Default for LimitsFitsApp {
    fn default() -> Self {
        Self {
            hole_input: Input::default_hole(),
            shaft_input: Input::default_shaft(),
            fit: Fit::default(),
            test_visual: VisualFit::default(),
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
                ui.toggle_value(&mut self.test_visual.display, "Visual");

                if ui.add(egui::Button::new("Reset")).clicked() {
                    self.hole_input = Input::default_hole();
                    self.shaft_input = Input::default_shaft();
                    self.fit = Fit::default();
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("ISO Limits and Fits Tool");

            // ----------------------------------------------------------------------------

            ui.label(egui::RichText::new("Inputs").strong().underline());
            ui.add_space(5.0);

            let hole_option = self.hole_input.show(ui, true, "hole_input");
            let shaft_option = self.shaft_input.show(ui, false, "shaft_input");
            // if ui.toggle_value(&mut self.standard, "Lock size").clicked() sync button?

            ui.separator();

            if let (Some(hole), Some(shaft)) = (hole_option, shaft_option) {
                self.fit = Fit::new(&hole, &shaft);
                self.fit.show(ui, "fit_results");
                hole.show(ui, "hole_size", "Hole");
                shaft.show(ui, "shaft_size", "Shaft");
            }

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
        let version_colour = Color32::from_rgb(0, 169, 0);
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
            .on_hover_text(
                "Tolerances currently not working past JS/js. Visualisation system currently WIP.",
            );
        // ui.colored_label(version_colour, " v");
        // ui.colored_label(version_colour, env!("CARGO_PKG_VERSION"));
        // ui.colored_label(version_colour, " alpha")
        //     .on_hover_ui(|ui| {
        //         ui.add(egui::Image::new(egui::include_image!(
        //             "../assets/nervous.png"
        //         )));
        //     });
        ui.colored_label(version_colour, " alpha")
            .on_hover_cursor(egui::CursorIcon::Help)
            .on_hover_text("This is an alpha release, bugs are to be expected — check your work.");
    });
}
