use egui::{Color32, CornerRadius, Stroke, Style};

/// Apply all application-specific theming.
/// Call exactly once at startup.
pub fn install(ctx: &egui::Context) {
    // Apply shared styling to both themes
    ctx.style_mut_of(egui::Theme::Light, apply_shared_styling);
    ctx.style_mut_of(egui::Theme::Dark, apply_shared_styling);

    // Apply theme-specific colors
    ctx.style_mut_of(egui::Theme::Light, apply_light_colors);
    ctx.style_mut_of(egui::Theme::Dark, apply_dark_colors);
}

fn apply_shared_styling(style: &mut Style) {
    let rounding = CornerRadius::same(10);
    let rounding_small = CornerRadius::same(5);

    style.visuals.widgets.noninteractive.corner_radius = rounding; // frames
    style.visuals.widgets.inactive.corner_radius = rounding_small;
    style.visuals.widgets.hovered.corner_radius = rounding_small;
    style.visuals.widgets.active.corner_radius = rounding_small;
    style.visuals.widgets.open.corner_radius = rounding_small;
    style.visuals.window_corner_radius = rounding_small;
    style.visuals.menu_corner_radius = rounding_small; // combo boxes

    // Ensure buttons always show their frame (background/stroke)
    style.visuals.button_frame = true;

    // Expand widget bounds slightly so stroke is visible
    style.visuals.widgets.inactive.expansion = 0.0;
    style.visuals.widgets.hovered.expansion = 1.0;
    style.visuals.widgets.active.expansion = 1.0;
    style.visuals.widgets.open.expansion = 0.0;
}

fn apply_light_colors(style: &mut Style) {
    // --bg-dark: #ffffff, --bg-card: #ffffff, --border-color: #e5e7eb, --accent-primary: #2563eb
    let border = Color32::from_rgb(0xd1, 0xd5, 0xdb); // darker border for visibility
    let bg = Color32::from_rgb(0xff, 0xff, 0xff);

    style.visuals.panel_fill = bg;
    style.visuals.window_fill = bg;

    style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(0xe5, 0xe7, 0xeb);
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border);

    // Interactive widgets: subtle fill
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(0xf3, 0xf4, 0xf6);
    style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(0xf3, 0xf4, 0xf6);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(0xe5, 0xe7, 0xeb);
    style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(0xe5, 0xe7, 0xeb);
    style.visuals.widgets.active.bg_fill = Color32::from_rgb(0xd1, 0xd5, 0xdb);
    style.visuals.widgets.active.weak_bg_fill = Color32::from_rgb(0xd1, 0xd5, 0xdb);
    style.visuals.widgets.open.bg_fill = Color32::from_rgb(0xf3, 0xf4, 0xf6);
    style.visuals.widgets.open.weak_bg_fill = Color32::from_rgb(0xf3, 0xf4, 0xf6);

    style.visuals.selection.bg_fill = Color32::from_rgb(0x25, 0x63, 0xeb);
    style.visuals.selection.stroke.color = Color32::WHITE;
}

fn apply_dark_colors(style: &mut Style) {
    // --bg-dark: #1a1a1a, --bg-card: #2a2a2a, --border-color: #3a3a3a, --accent-primary: #4a9eff
    let border = Color32::from_rgb(0x3a, 0x3a, 0x3a);
    let background_colour = Color32::from_rgb(0x1a, 0x1a, 0x1a);
    // let background_colour = Color32::from_rgb(0x2a, 0x2a, 0x2a);
    let accent_colour = Color32::from_rgb(0x4a, 0x9e, 0xff);
    let accent_stroke = Stroke {
        width: 1.0,
        color: Color32::from_rgb(0x4a, 0x9e, 0xff),
    };

    // Sets the general background colour and ensure that textboxes are seamless
    style.visuals.panel_fill = background_colour;
    style.visuals.extreme_bg_color = background_colour;

    // style.visuals.window_fill = background_colour;
    // style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(0x3a, 0x3a, 0x3a);

    // Interactive widgets: subtle fill
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(0x3a, 0x3a, 0x3a);
    style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(0x3a, 0x3a, 0x3a);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(0x4a, 0x4a, 0x4a);

    // Makes the outlines of buttons highlight in the accent colour when hovered and interacted with
    style.visuals.widgets.hovered.bg_stroke = accent_stroke;
    style.visuals.widgets.active.bg_stroke = accent_stroke;

    // style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x4a, 0x4a, 0x4a);
    // style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x4a, 0x9e, 0xff);
    // style.visuals.widgets.active.bg_fill = Color32::from_rgb(0x5a, 0x5a, 0x5a);
    // style.visuals.widgets.active.weak_bg_fill = Color32::from_rgb(0x5a, 0x5a, 0x5a);
    // style.visuals.widgets.open.bg_fill = Color32::from_rgb(0x3a, 0x3a, 0x3a);
    // style.visuals.widgets.open.weak_bg_fill = Color32::from_rgb(0x3a, 0x3a, 0x3a);

    style.visuals.selection.bg_fill = accent_colour; // highlight colour
}
