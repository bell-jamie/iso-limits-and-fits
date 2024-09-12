use crate::tolerance;
use tolerance::{calculate_fit, FitResult};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
// if we add new fields, give them default values when deserializing old state

pub struct LimitsFitsApp {
    // Example stuff:
    #[serde(skip)]
    basic_size: f64,
    hole_deviation: String,
    hole_grade: String,
    shaft_deviation: String,
    shaft_grade: String,
    it_numbers: Vec<String>,
    hole_position_letters: Vec<String>,
    shaft_position_letters: Vec<String>,
    #[serde(skip)]
    result: Option<FitResult>,
    // #[serde(skip)] // This how you opt-out of serialization of a field
}

impl Default for LimitsFitsApp {
    fn default() -> Self {
        Self {
            basic_size: 10.0,
            hole_deviation: "H".to_owned(),
            hole_grade: "7".to_owned(),
            shaft_deviation: "h".to_owned(),
            shaft_grade: "6".to_owned(),
            it_numbers: vec![
                "01", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                "15", "16", "17", "18",
            ]
            .iter()
            .map(|it| it.to_string())
            .collect(),
            hole_position_letters: vec![
                "A", "B", "C", "CD", "D", "E", "EF", "F", "FG", "G", "H", "JS", "J", "K", "M", "N",
                "P", "R", "S", "T", "U", "V", "X", "Y", "Z", "ZA", "ZB", "ZC",
            ]
            .iter()
            .map(|deviation| deviation.to_string())
            .collect(),
            shaft_position_letters: vec![
                "a", "b", "c", "cd", "d", "e", "ef", "f", "fg", "g", "h", "js", "j", "k", "m", "n",
                "p", "r", "s", "t", "u", "v", "x", "y", "z", "za", "zb", "zc",
            ]
            .iter()
            .map(|deviation| deviation.to_string())
            .collect(),
            result: None,
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

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("ISO Limits and Fits Tool");

            ui.label(egui::RichText::new("Input").strong().underline());

            // ui.horizontal(|ui| {
            //     ui.label("Basic Size (mm):");
            //     ui.text_edit_singleline(&mut self.basic_size);
            // });

            ui.horizontal(|ui| {
                ui.label("Basic Size (mm):");
                ui.add(egui::DragValue::new(&mut self.basic_size).speed(0.1));
            });

            ui.horizontal(|ui| {
                ui.label("Hole Tolerance:");
                egui::ComboBox::from_id_source("hole-fundamental_deviation")
                    .selected_text(&self.hole_deviation)
                    .show_ui(ui, |ui| {
                        for letter in &self.hole_position_letters {
                            ui.selectable_value(&mut self.hole_deviation, letter.clone(), letter);
                        }
                    });
                egui::ComboBox::from_id_source("hole-tolerance-grade")
                    .selected_text(&self.hole_grade)
                    .show_ui(ui, |ui| {
                        for grade in &self.it_numbers {
                            ui.selectable_value(&mut self.hole_grade, grade.clone(), grade);
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Shaft Tolerance:");
                egui::ComboBox::from_id_source("shaft-fundamental-deviation")
                    .selected_text(&self.shaft_deviation)
                    .show_ui(ui, |ui| {
                        for letter in &self.shaft_position_letters {
                            ui.selectable_value(&mut self.shaft_deviation, letter.clone(), letter);
                        }
                    });
                egui::ComboBox::from_id_source("shaft-tolerance-grade")
                    .selected_text(&self.shaft_grade)
                    .show_ui(ui, |ui| {
                        for grade in &self.it_numbers {
                            ui.selectable_value(&mut self.shaft_grade, grade.clone(), grade);
                        }
                    });
            });

            self.result = calculate_fit(
                self.basic_size,
                &self.hole_deviation,
                &self.hole_grade,
                &self.shaft_deviation,
                &self.shaft_grade,
            );

            // if ui.button("Calculate").clicked() {
            //     self.result = calculate_fit(
            //         self.basic_size,
            //         &self.hole_deviation,
            //         &self.hole_grade,
            //         &self.shaft_deviation,
            //         &self.shaft_grade,
            //     );
            // }

            ui.separator();

            ui.label(egui::RichText::new("Results").strong().underline());

            if let Some(result) = &self.result {
                ui.label(format!(
                    "Fit: {:.3} µm {}",
                    result.fit_midlimits, result.fit_type
                ));
                ui.horizontal(|ui| {
                    ui.label("Hole size:");
                    ui.vertical(|ui| {
                        ui.label(format!("{:.3}", result.hole_lmc));
                        ui.label(format!("{:.3}", result.hole_mmc));
                    })
                });
                ui.horizontal(|ui| {
                    ui.label("Shaft size:");
                    ui.vertical(|ui| {
                        ui.label(format!("{:.3}", result.shaft_mmc));
                        ui.label(format!("{:.3}", result.shaft_lmc));
                    })
                });
            } else {
                ui.label(format!("No Results"));
            }

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/bell-jamie/iso-limits-and-fits",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
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
    });
}
