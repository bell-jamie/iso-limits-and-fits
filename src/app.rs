/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
// if we add new fields, give them default values when deserializing old state
pub struct LimitsFitsApp {
    // Example stuff:
    #[serde(skip)]
    hub_nominal_size: String,
    hub_tolerance_band: String,
    hub_tolerance_bands: Vec<String>,
    hub_tolerance_grade: String,
    hub_tolerance_grades: Vec<String>,
    shaft_nominal_size: String,
    shaft_tolerance_band: String,
    shaft_tolerance_bands: Vec<String>,
    shaft_tolerance_grade: String,
    shaft_tolerance_grades: Vec<String>,
    // #[serde(skip)] // This how you opt-out of serialization of a field
}

impl Default for LimitsFitsApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            hub_nominal_size: "10.0".to_owned(),
            hub_tolerance_band: "H".to_owned(),
            hub_tolerance_bands: vec![
                "A", "B", "C", "CD", "D", "E", "EF", "F", "FG", "G", "H", "JS", "J", "K", "M", "N",
                "P", "R", "S", "T", "U", "V", "X", "Y", "Z", "ZA", "ZB", "ZC",
            ]
            .iter()
            .map(|class| class.to_string())
            .collect(),
            hub_tolerance_grade: "7".to_owned(),
            hub_tolerance_grades: vec![
                "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16",
            ]
            .iter()
            .map(|grade| grade.to_string())
            .collect(),
            shaft_nominal_size: "10.0".to_owned(),
            shaft_tolerance_band: "h".to_owned(),
            shaft_tolerance_bands: vec![
                "a", "b", "c", "cd", "d", "e", "ef", "f", "fg", "g", "h", "js", "j", "k", "m", "n",
                "p", "r", "s", "t", "u", "v", "x", "y", "z", "za", "zb", "zc",
            ]
            .iter()
            .map(|band| band.to_string())
            .collect(),
            shaft_tolerance_grade: "6".to_owned(),
            shaft_tolerance_grades: vec![
                "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16",
            ]
            .iter()
            .map(|grade| grade.to_string())
            .collect(),
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

            ui.label(egui::RichText::new("Hub").strong().underline());

            ui.horizontal(|ui| {
                ui.label("Nominal Size:");
                ui.text_edit_singleline(&mut self.hub_nominal_size);
            });

            ui.horizontal(|ui| {
                ui.label("Tolerance Class:");
                egui::ComboBox::from_id_source("hub-tolerance-band")
                    .selected_text(&self.hub_tolerance_band)
                    .show_ui(ui, |ui| {
                        for band in &self.hub_tolerance_bands {
                            ui.selectable_value(&mut self.hub_tolerance_band, band.clone(), band);
                        }
                    });
                egui::ComboBox::from_id_source("hub-tolerance-grade")
                    .selected_text(&self.hub_tolerance_grade)
                    .show_ui(ui, |ui| {
                        for grade in &self.hub_tolerance_grades {
                            ui.selectable_value(
                                &mut self.hub_tolerance_grade,
                                grade.clone(),
                                grade,
                            );
                        }
                    });
            });

            ui.label(egui::RichText::new("Shaft").strong().underline());

            ui.horizontal(|ui| {
                ui.label("Nominal Size:");
                ui.text_edit_singleline(&mut self.shaft_nominal_size);
            });

            ui.horizontal(|ui| {
                ui.label("Tolerance Class:");
                egui::ComboBox::from_id_source("shaft-tolerance-band")
                    .selected_text(&self.shaft_tolerance_band)
                    .show_ui(ui, |ui| {
                        for band in &self.shaft_tolerance_bands {
                            ui.selectable_value(&mut self.shaft_tolerance_band, band.clone(), band);
                        }
                    });
                egui::ComboBox::from_id_source("shaft-tolerance-grade")
                    .selected_text(&self.shaft_tolerance_grade)
                    .show_ui(ui, |ui| {
                        for grade in &self.shaft_tolerance_grades {
                            ui.selectable_value(
                                &mut self.shaft_tolerance_grade,
                                grade.clone(),
                                grade,
                            );
                        }
                    });
            });

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
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
        ui.label(".");
    });
}
