use egui::Ui;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct State {
    pub advanced: bool,
    pub debug: bool,
    pub force_valid: bool,
    pub sync_size: bool,
    pub synced_size: f64,
    pub sync_temp: bool,
    pub synced_temp: f64,
    pub thermal: bool,
    pub interference: bool,
    pub zoom: Zoom,
    pub hub_id: usize,
    pub shaft_id: usize,
    pub show_library_panel: bool,
}

impl State {
    pub fn default() -> Self {
        State {
            advanced: false,
            debug: false,
            force_valid: false,
            sync_size: true,
            synced_size: 10.0,
            sync_temp: true,
            synced_temp: 20.0,
            thermal: false,
            interference: false,
            zoom: Zoom::default(),
            hub_id: 0,
            shaft_id: 0,
            show_library_panel: false,
        }
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Zoom {
    pub expand: bool,
    pub scale: f32,
}

impl Zoom {
    pub fn default() -> Self {
        Zoom {
            expand: false,
            scale: 1.7,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.toggle_value(&mut self.expand, "🔍")
            .on_hover_text("Zoom");

        let (min_zoom, max_zoom) = (0.5, 3.0);

        // Handles the ui zoom slider
        if self.expand {
            ui.label(format!("{:.1}x", self.scale));

            if ui
                .add(egui::Slider::new(&mut self.scale, min_zoom..=max_zoom).show_value(false))
                .is_pointer_button_down_on()
            {
                return;
            }
        }

        self.scale = self.scale.clamp(min_zoom, max_zoom);
        ui.ctx().set_zoom_factor(self.scale);
    }
}
