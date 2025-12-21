use egui::{Context, RichText, Ui, Vec2, emath};

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

        // Handles the scroll and keyboard inputs - disabled for now
        // ctx.input(|i| {
        //     if i.modifiers.command {
        //         if i.raw_scroll_delta.y != 0.0 {
        //             self.scale += i.raw_scroll_delta.y * 1e-3;
        //         }

        //         if i.key_pressed(egui::Key::Plus) {
        //             self.scale += 0.1;
        //         } else if i.key_pressed(egui::Key::Minus) {
        //             self.scale -= 0.1;
        //         }
        //     }
        // });

        self.scale = self.scale.clamp(min_zoom, max_zoom);
        ui.ctx().set_zoom_factor(self.scale);
    }
}

pub fn decimals(num: f64, decimals: i32) -> f64 {
    // Negative decimals inherit the default decimal places value
    let power = if decimals >= 0 { decimals } else { 4 };
    let factor = 10f64.powi(power);
    (num * factor).round() / factor
}

/// Computes dynamic level of precision based on value magnitude
/// Used to keep a constant number of figures
pub fn dynamic_precision(value: f64, max_decimals: isize) -> usize {
    if value.is_nan() || value.is_infinite() {
        return 0; //
    }

    let magnitude = if value <= 1.0 && value >= -1.0 {
        0 // Default magnitude for zero
    } else {
        value.abs().log10().floor() as isize
    };

    (max_decimals - magnitude).max(0) as usize
}

/// Returns the number of decimal places required to fully display `value` precisely.
/// Takes 'decimals' argument to limit required precision
pub fn req_precision(value: f64, decimals: isize) -> usize {
    if value == 0.0 || value.is_nan() || value.is_infinite() {
        return 0;
    }

    // Format with limited precision
    let limit = if decimals >= 0 { decimals as usize } else { 4 };
    let s = format!("{:.limit$}", value.abs());

    // Find decimal point
    if let Some(pos) = s.find('.') {
        // Trim trailing zeros after decimal point
        let decimals = &s[pos + 1..];
        decimals.trim_end_matches('0').len()
    } else {
        0
    }
}

pub fn text_width(ctx: &Context, text: &str, size: f32) -> Vec2 {
    // Returns the x and y size of the text
    let font_id = egui::FontId {
        size,
        ..Default::default()
    };
    ctx.fonts_mut(|f| f.layout_no_wrap(text.to_string(), font_id.clone(), egui::Color32::WHITE))
        .size()
}

/// This function is framerate dependant.
pub fn lerp_untimed(current: f64, target: f64, rate: f64, tol: f64) -> Option<f64> {
    if (current - target).abs() > tol {
        Some(current + rate * (target - current))
    } else {
        None
    }
}

pub fn check_width(ui: &mut Ui) {
    let width = ui.min_rect().width();
    ui.label(RichText::new(format!("{width} pixels")).strong())
        .on_hover_text(format!("{width}"));
}

// pub fn linspace(a: f64, b: f64, n: usize) -> Vec<f64> {
//     (0..n)
//         .map(|i| {
//             let t = i as f64 / (n as f64 - 1.0);
//             a + t * (b - a)
//         })
//         .collect()
// }
