use egui::{Color32, CornerRadius, FontDefinitions, Stroke, Style};

/// Fit zone colors for interference/clearance display
#[derive(Clone, Copy)]
pub struct FitZoneColors {
    pub clearance: [u8; 4],    // RGBA
    pub interference: [u8; 4], // RGBA
}

impl FitZoneColors {
    pub fn light() -> Self {
        Self {
            clearance: [0, 80, 220, 120],     // Saturated blue with alpha
            interference: [220, 20, 20, 120], // Saturated red with alpha
        }
    }

    pub fn dark() -> Self {
        Self {
            clearance: [40, 120, 255, 120], // Saturated bright blue for dark mode
            interference: [255, 40, 40, 120], // Saturated bright red for dark mode
        }
    }

    pub fn from_dark_mode(dark_mode: bool) -> Self {
        if dark_mode {
            Self::dark()
        } else {
            Self::light()
        }
    }
}

/// Apply all application-specific theming.
/// Call exactly once at startup.
pub fn install(ctx: &egui::Context) {
    // Fonts
    // ctx.set_fonts(fonts());

    // Font styling
    apply_font_styling(ctx);

    // Apply shared styling to both themes
    ctx.style_mut_of(egui::Theme::Light, apply_shared_styling);
    ctx.style_mut_of(egui::Theme::Dark, apply_shared_styling);

    // Apply theme-specific colors
    ctx.style_mut_of(egui::Theme::Light, apply_light_colors);
    ctx.style_mut_of(egui::Theme::Dark, apply_dark_colors);
}

// fn fonts() -> FontDefinitions {
//     let mut fonts = FontDefinitions::default();
//     fonts.families.insert(
//         egui::TextStyle::Small,
//         (egui::FontFamily::Proportional, 5.0),
//     );
//     fonts
// }

fn apply_shared_styling(style: &mut Style) {
    let rounding = CornerRadius::same(10);
    let rounding_small = CornerRadius::same(5);

    style.visuals.widgets.noninteractive.corner_radius = rounding; // frames
    style.visuals.widgets.inactive.corner_radius = rounding_small;
    style.visuals.widgets.hovered.corner_radius = rounding_small; // hovered outline rad
    style.visuals.widgets.active.corner_radius = rounding_small;
    style.visuals.widgets.open.corner_radius = rounding_small; // open combo boxes
    // style.visuals.window_corner_radius = rounding_small; // unknown
    style.visuals.menu_corner_radius = rounding_small; // combo box lists

    // Ensure buttons always show their frame (background/stroke)
    style.visuals.button_frame = true;

    // Expand widget bounds slightly so stroke is visible
    style.visuals.widgets.inactive.expansion = 0.0;
    style.visuals.widgets.hovered.expansion = 1.0;
    style.visuals.widgets.active.expansion = 1.0;
    style.visuals.widgets.open.expansion = 0.0;

    // Window top bar highlighting
    style.visuals.window_highlight_topmost = false;

    // Trailing colour on sliders
    style.visuals.slider_trailing_fill = true;

    // style.visuals.text_edit_bg_color = Some(Color32::from_rgb(0x3a, 0x3a, 0x3a));
}

fn apply_light_colors(style: &mut Style) {
    // --bg-dark: #ffffff, --bg-card: #ffffff, --border-color: #e5e7eb, --accent-primary: #2563eb
    let border = Color32::from_rgb(0xd1, 0xd5, 0xdb); // darker border for visibility
    let background_colour = Color32::from_rgb(0xff, 0xff, 0xff);
    let widget_inactive = Color32::from_rgb(0xf3, 0xf4, 0xf6);
    let accent_colour = Color32::from_rgb(0x25, 0x63, 0xeb);
    let accent_stroke = Stroke {
        width: 1.0,
        color: accent_colour,
    };
    apply_colors(
        style,
        border,
        background_colour,
        widget_inactive,
        accent_colour,
        accent_stroke,
    );
}

fn apply_dark_colors(style: &mut Style) {
    // --bg-dark: #1a1a1a, --bg-card: #2a2a2a, --border-color: #3a3a3a, --accent-primary: #4a9eff
    let border = Color32::from_rgb(0x3a, 0x3a, 0x3a);
    let background_colour = Color32::from_rgb(0x1a, 0x1a, 0x1a);
    let widget_inactive = Color32::from_rgb(0x3a, 0x3a, 0x3a);
    let accent_colour = Color32::from_rgb(0x4a, 0x9e, 0xff);
    let accent_stroke = Stroke {
        width: 1.0,
        color: Color32::from_rgb(0x4a, 0x9e, 0xff),
    };
    apply_colors(
        style,
        border,
        background_colour,
        widget_inactive,
        accent_colour,
        accent_stroke,
    );
}

fn apply_colors(
    style: &mut Style,
    _border: Color32,
    background_colour: Color32,
    widget_inactive: Color32,
    accent_colour: Color32,
    accent_stroke: Stroke,
) {
    // Sets the general background colour and ensure that textboxes are seamless
    style.visuals.panel_fill = background_colour;
    // style.visuals.extreme_bg_color = background_colour; // don't want to affect scrollbars as well
    style.visuals.text_edit_bg_color = Some(background_colour); // this way we only affect textedit
    style.visuals.window_fill = background_colour;
    // style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(0x3a, 0x3a, 0x3a);

    // Interactive widgets: subtle fill
    style.visuals.widgets.inactive.bg_fill = widget_inactive;
    style.visuals.widgets.inactive.weak_bg_fill = widget_inactive;
    // style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(0x4a, 0x4a, 0x4a);

    // Makes the outlines of buttons highlight in the accent colour when hovered and interacted with
    style.visuals.widgets.hovered.bg_stroke = accent_stroke;
    style.visuals.widgets.active.bg_stroke = accent_stroke;

    // style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x4a, 0x4a, 0x4a);
    // style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(0x4a, 0x9e, 0xff);
    // style.visuals.widgets.active.bg_fill = Color32::from_rgb(0x5a, 0x5a, 0x5a);
    // style.visuals.widgets.active.weak_bg_fill = Color32::from_rgb(0x5a, 0x5a, 0x5a);
    // style.visuals.widgets.open.bg_fill = Color32::from_rgb(0x3a, 0x3a, 0x3a);
    // style.visuals.widgets.open.weak_bg_fill = Color32::from_rgb(0x3a, 0x3a, 0x3a);

    style.visuals.selection.bg_fill = accent_colour;
    style.visuals.selection.stroke.color = Color32::WHITE;
}

pub fn _fonts() -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();

    // Register custom font
    // fonts.font_data.insert(
    //     "MyFont".to_owned(),
    //     egui::FontData::from_static(include_bytes!("fonts/my_font.ttf")),
    // );

    // Make it the first choice for proportional text
    // fonts
    //     .families
    //     .entry(egui::FontFamily::Proportional)
    //     .or_default()
    //     .insert(0, "MyFont".to_owned());

    // Optional: use it for monospace as well
    // fonts
    //     .families
    //     .entry(egui::FontFamily::Monospace)
    //     .or_default()
    //     .insert(0, "MyFont".to_owned());

    fonts
}

pub fn apply_font_styling(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.text_styles = [
        (
            egui::TextStyle::Heading,
            egui::FontId::new(20.0, egui::FontFamily::Proportional), // 18.0
        ),
        (
            egui::TextStyle::Name("SubHeading".into()),
            egui::FontId::new(18.0, egui::FontFamily::Proportional), // don't know how this works
        ),
        (
            egui::TextStyle::Body,
            egui::FontId::new(15.0, egui::FontFamily::Proportional), // 13.0
        ),
        (
            egui::TextStyle::Button,
            egui::FontId::new(15.0, egui::FontFamily::Proportional), // 13.0
        ),
        (
            egui::TextStyle::Small,
            egui::FontId::new(12.0, egui::FontFamily::Proportional), // 9.0
        ),
        (
            egui::TextStyle::Monospace,
            egui::FontId::new(15.0, egui::FontFamily::Monospace), // 13.0
        ),
    ]
    .into();

    ctx.set_style(style);
}
