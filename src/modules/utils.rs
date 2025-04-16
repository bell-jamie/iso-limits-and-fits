use egui::{emath, Context, RichText, Ui, Vec2};

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
            scale: 1.0,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, ctx: &Context) {
        ui.toggle_value(&mut self.expand, "🔍")
            .on_hover_text("Zoom");

        if self.expand {
            ui.label(format!("{:.1}x", self.scale));

            if !ui
                .add(egui::Slider::new(&mut self.scale, 0.5..=3.0).show_value(false))
                .is_pointer_button_down_on()
            {
                ctx.set_zoom_factor(self.scale);
            }
        } else {
            ctx.set_zoom_factor(self.scale);
        }
    }
}

pub fn decimals(num: f64, decimals: i32) -> f64 {
    // Negative decimals inherit the default decimal places value
    let power = if decimals >= 0 { decimals } else { 4 };
    let factor = 10f64.powi(power);
    (num * factor).round() / factor
}

pub fn dynamic_precision(value: f64, max_decimals: isize) -> usize {
    // Handle NaN, infinite, and zero values
    if value.is_nan() || value.is_infinite() {
        return 0; //
    }

    // Handle zero
    let magnitude = if value <= 1.0 && value >= -1.0 {
        0 // Default magnitude for zero
    } else {
        value.abs().log10().floor() as isize
    };

    // Calculate precision dynamically
    (max_decimals - magnitude).max(0) as usize
}

pub fn text_width(ctx: &Context, text: &str) -> Vec2 {
    // Returns the x and y size of the text
    ctx.fonts(|f| {
        f.layout_no_wrap(
            text.to_string(),
            egui::FontId::default(),
            egui::Color32::WHITE,
        )
    })
    .size()
}

/// This function is framerate dependant...
pub fn lerp_untimed(current: f64, target: f64, rate: f64, tol: f64) -> f64 {
    if (current - target).abs() > tol {
        current + rate * (target - current)
    } else {
        target
    }
}

// pub fn check_width(ui: &mut Ui) {
//     let width = ui.min_rect().width();
//     ui.label(RichText::new(format!("{width} pixels")).strong())
//         .on_hover_text(format!("{width}"));
// }

// pub fn linspace(a: f64, b: f64, n: usize) -> Vec<f64> {
//     (0..n)
//         .map(|i| {
//             let t = i as f64 / (n as f64 - 1.0);
//             a + t * (b - a)
//         })
//         .collect()
// }
