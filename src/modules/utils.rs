use egui::{Context, RichText, Ui, Vec2};

/// Trait to extend floats with a `sanitise` method
pub trait SanitiseFloat: Sized {
    /// Rounds the number to the specified number of decimal places.
    fn sanitise(self, decimals: usize) -> Self;
}

impl SanitiseFloat for f64 {
    fn sanitise(self, decimals: usize) -> Self {
        let factor = 10f64.powi(decimals as i32);
        (self * factor).round() / factor
    }
}

impl SanitiseFloat for f32 {
    fn sanitise(self, decimals: usize) -> Self {
        let factor = 10f32.powi(decimals as i32);
        (self * factor).round() / factor
    }
}

/// Formats a floating-point value to the specified number of significant figures.
///
/// - Rounds to nearest (IEEE-754 `round` semantics)
/// - Preserves sign
/// - Returns `"0"` for zero
/// - Returns `"NaN"` / `"inf"` / `"-inf"` unchanged
pub fn fix_sf<T: Into<f64>>(value: T, sig_figs: isize) -> String {
    let value = value.into();

    if !value.is_finite() {
        return value.to_string();
    }

    if value == 0.0 {
        return "0".to_string();
    }

    let sig_figs = sig_figs.max(1);
    let magnitude = value.abs().log10().floor() as isize;
    let decimals = sig_figs - 1 - magnitude;

    let factor = 10f64.powi(decimals as i32);
    let rounded = (value * factor).round() / factor;

    if decimals > 0 {
        format!("{:.*}", decimals as usize, rounded)
    } else {
        rounded.trunc().to_string()
    }
}

/// Rounds `num` to the specified number of decimal places and returns it as a string.
pub fn fix_dp<T: Into<f64>>(num: T, dp: usize) -> String {
    format!("{:.*}", dp, num.into())
}

/// Rounds `num` to the specified number of decimal places and returns a string.
/// Trailing zeros are trimmed, but at least one decimal place is preserved.
pub fn lim_dp<T: Into<f64>>(num: T, dp: usize) -> String {
    let num = num.into();
    let mut s = format!("{:.*}", dp, num);

    if s.contains('.') {
        // Remove trailing zeros
        while s.ends_with('0') {
            s.pop();
        }
        // Ensure at least one decimal place remains
        if s.ends_with('.') {
            s.push('0');
        }
    }

    s
}

/// Returns the number of decimal places needed to truncate a value
/// to a roughly constant number of significant figures.
///
/// Precision decreases as the value’s order of magnitude increases.
/// Returns `0` for NaN or infinite values.
pub fn decimals_for_sig_figs(value: f64, max_sig_figs: isize) -> usize {
    if value.is_nan() || value.is_infinite() {
        return 0;
    }

    let magnitude = if value >= -1.0 && value <= 1.0 {
        0
    } else {
        value.abs().log10().floor() as isize
    };

    (max_sig_figs - magnitude).max(0) as usize
}

/// Truncates `value` to the specified number of significant figures.
///
/// Uses `decimals_for_sig_figs` to determine the required decimal places.
/// NaN and infinite values are returned unchanged.
pub fn truncate_to_sig_figs(value: f64, max_sig_figs: isize) -> f64 {
    if value.is_nan() || value.is_infinite() {
        return value;
    }

    let decimals = decimals_for_sig_figs(value, max_sig_figs);
    let factor = 10f64.powi(decimals as i32);

    (value * factor).trunc() / factor
}

/// Returns the number of decimal places required to fully display `value` precisely.
pub fn display_dp(value: f64) -> usize {
    if value == 0.0 || value.is_nan() || value.is_infinite() {
        return 0;
    }

    let s = format!("{}", value.abs());

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

    ui.add_space(ui.style().spacing.item_spacing.y);
    if open { Some(add_contents(ui)) } else { None }
}

/// Returns the dimension adjusted to a given temperature.
///
/// * `size` — Nominal dimension at 20 °C
/// * `temp` — Actual temperature in °C
/// * `cte` — Coefficient of thermal expansion in µm/(m·°C)
pub fn at_temp(size: f64, temp: f64, cte: f64) -> f64 {
    let delta_t = temp - 20.0; // 20°C as per ISO 1:2016
    size * (1.0 + cte * delta_t * 0.000_001)
}

/// Truncates a string to a maximum length, adding an ellipsis if truncated.
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}…", &s[..max_len.saturating_sub(1)])
    } else {
        s.to_string()
    }
}

/// Truncates a string to fit within a given pixel width, adding an ellipsis if truncated.
pub fn truncate_string_to_width(ctx: &Context, s: &str, max_width: f32) -> String {
    let font_id = egui::TextStyle::Body.resolve(&ctx.style());
    let full_width = ctx
        .fonts_mut(|f| f.layout_no_wrap(s.to_string(), font_id.clone(), egui::Color32::WHITE))
        .size()
        .x;

    if full_width <= max_width {
        return s.to_string();
    }

    // Binary search for the right length
    let ellipsis = "…";
    let ellipsis_width = ctx
        .fonts_mut(|f| {
            f.layout_no_wrap(ellipsis.to_string(), font_id.clone(), egui::Color32::WHITE)
        })
        .size()
        .x;

    let target_width = max_width - ellipsis_width - ctx.style().spacing.item_spacing.x;
    if target_width <= 0.0 {
        return ellipsis.to_string();
    }

    // Find longest substring that fits
    let chars: Vec<char> = s.chars().collect();
    let mut lo = 0;
    let mut hi = chars.len();

    while lo < hi {
        let mid = (lo + hi + 1) / 2;
        let substr: String = chars[..mid].iter().collect();
        let width = ctx
            .fonts_mut(|f| f.layout_no_wrap(substr, font_id.clone(), egui::Color32::WHITE))
            .size()
            .x;

        if width <= target_width {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }

    if lo == 0 {
        ellipsis.to_string()
    } else {
        let substr: String = chars[..lo].iter().collect();
        format!("{}{}", substr, ellipsis)
    }
}

/// Format a floating-point temperature as "{value} ºC"
pub fn format_temp(t: f64) -> String {
    format!("{} ºC", truncate_to_sig_figs(t, 3))
}

/// Parse a temperature string like "-12.3 ºC" into an f64
pub fn parse_temp(s: &str) -> Option<f64> {
    let filtered = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect::<String>();
    filtered.parse::<f64>().ok()
}
