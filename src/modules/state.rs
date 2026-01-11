use egui::Ui;
use redprint::render::egui::View;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct State {
    pub advanced: bool,
    pub debug: bool,
    pub force_valid: bool,
    pub sync_size: bool,
    pub synced_size: f64,
    pub interference: bool,
    pub scale: Scale,
    pub show_library_panel: bool,
    pub show_egui_settings: bool,
}

impl State {
    pub fn default() -> Self {
        State {
            advanced: false,
            debug: false,
            force_valid: false,
            sync_size: true,
            synced_size: 10.0,
            interference: false,
            scale: Scale::default(),
            show_library_panel: false,
            show_egui_settings: false,
        }
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Scale {
    pub expand: bool,
    pub value: f32,
}

impl Scale {
    pub fn default() -> Self {
        Scale {
            expand: false,
            value: 1.5,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let (min, max) = (0.5, 3.0);
        if ui
            .horizontal(|ui| {
                ui.toggle_value(&mut self.expand, "🔍")
                    .on_hover_text("Ui scale");

                if self.expand {
                    let slider =
                        ui.add(egui::Slider::new(&mut self.value, min..=max).show_value(false));
                    if !slider.is_pointer_button_down_on() {
                        ui.ctx().set_zoom_factor(self.value);
                    }
                    ui.label(format!("{:.1}x", self.value));
                }
            })
            .response
            .clicked_elsewhere()
        {
            self.expand = false;
        }
    }
}
