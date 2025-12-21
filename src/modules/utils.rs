use egui::{Context, RichText, Ui, Vec2};

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

/// Custom accordion widget with title on left and toggle icon on right
/// Collapsed: Title            -
/// Extended:  Title            v
pub fn accordion<R>(
    ui: &mut Ui,
    id: impl std::hash::Hash,
    title: impl Into<String>,
    default_open: bool,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    let id = ui.make_persistent_id(id);
    let mut open = ui
        .ctx()
        .data_mut(|d| *d.get_persisted_mut_or(id, default_open));

    let title = title.into();
    let icon = if open { "⏷" } else { "⏴" };

    let available_width = ui.available_width();
    let height = ui.spacing().interact_size.y;

    ui.allocate_ui_with_layout(
        egui::vec2(available_width, height),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.label(RichText::new(&title).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(egui::Button::new(icon).frame(false)).clicked() {
                    open = !open;
                    ui.ctx().data_mut(|d| d.insert_persisted(id, open));
                }
            });
        },
    );

    if open { Some(add_contents(ui)) } else { None }
}
