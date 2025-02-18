use egui::{vec2, Align2, Color32, Frame, RichText, Stroke, Ui};
use egui_plot::{Line, Plot, PlotPoint, PlotPoints, PlotUi, Polygon, Text};

use super::{
    component::Component,
    feature::Feature,
    utils::{dynamic_precision, text_width},
};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct PlotStyle {
    scale: f64,
    line_width: f32,
    line_colour: Color32,
    fill_colour: Color32,
}

impl PlotStyle {
    pub fn default() -> Self {
        PlotStyle {
            scale: 1.0,
            line_width: 1.0,
            line_colour: Color32::DARK_GRAY,
            fill_colour: Color32::TRANSPARENT,
        }
    }
}

// Must be even
const RESOLUTION: usize = 1_000;

pub fn side_by_side(ui: &mut Ui, left_component: &Component, right_component: &Component) {
    let outline_colour = if ui.visuals().dark_mode {
        Color32::LIGHT_GRAY
    } else {
        Color32::DARK_GRAY
    };

    let centre = [0.0; 2];
    let scale = 1.0;
    let plot_name = "bruh";

    Frame::group(ui.style())
        .inner_margin(10.0)
        .rounding(10.0)
        .show(ui, |ui| {
            ui.set_min_size(vec2(510.0, 200.0));
            ui.set_max_size(vec2(510.0, 200.0));

            Plot::new(&plot_name)
                .data_aspect(1.0)
                .include_x(20.0)
                .show_axes(false)
                .show_grid(false)
                .show_background(false)
                .show_x(false)
                .show_y(false)
                .allow_drag(false)
                .allow_zoom(false)
                .allow_scroll(false)
                .allow_boxed_zoom(false)
                .show(ui, |ui| {
                    let line_style = PlotStyle {
                        scale: 1.0,
                        line_width: 1.0,
                        line_colour: outline_colour,
                        fill_colour: Color32::TRANSPARENT,
                    };
                    let annotate_style = PlotStyle {
                        scale: 1.0,
                        line_width: 1.5,
                        line_colour: outline_colour,
                        fill_colour: Color32::TRANSPARENT,
                    };

                    end_view(
                        ui,
                        left_component,
                        [-15.0, 0.0],
                        &line_style,
                        &annotate_style,
                    );
                    end_view(
                        ui,
                        right_component,
                        [15.0, 0.0],
                        &line_style,
                        &annotate_style,
                    );
                });
        });
}

pub fn end_view(
    ui: &mut PlotUi,
    component: &Component,
    centre: [f64; 2],
    line_style: &PlotStyle,
    annotate_style: &PlotStyle,
) {
    // Outer circle
    plot_circle(ui, line_style, centre, component.outer_diameter.size);

    // Inner circle
    plot_circle(ui, line_style, centre, component.inner_diameter.size);

    // Centre mark
    plot_centre_mark(
        ui,
        line_style,
        centre,
        component
            .inner_diameter
            .size
            .max(component.outer_diameter.size),
        0.0,
    );

    // Outer diameter dimension
    plot_diameter_limits(
        ui,
        annotate_style,
        centre,
        &component.outer_diameter,
        110.0,
        5.0,
    );

    // Inner diameter dimension
    plot_diameter_limits(
        ui,
        annotate_style,
        centre,
        &component.inner_diameter,
        70.0,
        5.0,
    );
}

pub fn plot_circle(ui: &mut PlotUi, style: &PlotStyle, centre: [f64; 2], diameter: f64) {
    let radius = diameter / 2.0;
    let mut points = Vec::with_capacity(RESOLUTION);

    for i in 0..RESOLUTION {
        let theta = 2.0 * std::f64::consts::PI * (i as f64 / RESOLUTION as f64);
        let x = centre[0] + radius * theta.cos();
        let y = centre[1] + radius * theta.sin();
        points.push([x, y]);
    }

    ui.polygon(
        Polygon::new(PlotPoints::from(points))
            .fill_color(style.fill_colour)
            .stroke(Stroke {
                width: style.line_width,
                color: style.line_colour,
            }),
    );
}

pub fn plot_centre_mark(
    ui: &mut PlotUi,
    style: &PlotStyle,
    centre: [f64; 2],
    size: f64,
    angle: f64,
) {
    let distances = vec![0.05, 0.1, 0.6];
    let mut points = vec![
        [-distances[2], 0.0],
        [-distances[1], 0.0],
        [-distances[0], 0.0],
        [distances[0], 0.0],
        [distances[1], 0.0],
        [distances[2], 0.0],
    ];

    rotate_points(&mut points, centre, angle);
    translate_points(&mut points, centre);
    scale_points(&mut points, centre, size);

    for _ in 0..2 {
        for pair in points.chunks(2) {
            ui.line(
                Line::new(PlotPoints::from_iter(pair.iter().map(|&[x, y]| [x, y])))
                    .width(style.line_width)
                    .color(style.line_colour),
            );
        }

        rotate_points(&mut points, centre, 90.0);
    }
}

pub fn plot_diameter_symbol(ui: &mut PlotUi, style: &PlotStyle, centre: [f64; 2]) {
    let overhang = 1.2;
    let mut bar = vec![
        [0.0, -style.scale / overhang],
        [0.0, style.scale / overhang],
    ];
    translate_points(&mut bar, centre);
    rotate_points(&mut bar, centre, -30.0);

    plot_circle(ui, style, centre, 1.0);
    ui.line(
        Line::new(PlotPoints::new(bar))
            .width(style.line_width)
            .color(style.line_colour),
    );
}

pub fn plot_arrow_leader(
    ui: &mut PlotUi,
    style: &PlotStyle,
    tip: [f64; 2],
    knee: [f64; 2],
    end: [f64; 2],
) {
    let angle = (knee[1] - tip[1]).atan2(knee[0] - tip[0]) * (180.0 / std::f64::consts::PI);
    ui.line(
        Line::new(PlotPoints::new(vec![tip, knee, end]))
            .width(style.line_width)
            .color(style.line_colour),
    );
    ui.polygon(arrow_head(style, tip, angle));
}

pub fn plot_diameter_limits(
    ui: &mut PlotUi,
    style: &PlotStyle,
    centre: [f64; 2],
    feature: &Feature,
    angle: f64,
    length: f64,
) {
    // Format the upper and lower limit text strings.
    let (upper_text, lower_text) = (
        format!("{:.3}", feature.upper_limit(None)),
        format!("{:.3}", feature.lower_limit(None)),
    );

    // Create points for the arrow line.
    let mut points = vec![
        [feature.size / 2.0, 0.0],
        [feature.size / 2.0 + length, 0.0],
    ];
    rotate_points(&mut points, [0.0; 2], angle);
    translate_points(&mut points, centre);

    let (tip, knee) = (points[0], points[1]); // Define the tip and knee of the arrow.
    let is_hand = angle.abs() <= 90.0; // Determine if we are using a "hand" (right-hand) orientation.
    let x_offset = if is_hand { 1.5 } else { -1.5 }; // x_offset is positive for right-hand, negative for left-hand.

    let text_width = |text: &str| text_width(ui.ctx(), text).x as f64 / 13.0; // Closure to compute the text width (normalized by 13.0) for a given string.
    let width = text_width(&upper_text).max(text_width(&lower_text)); // Compute the maximum width needed between the two texts.

    let x = knee[0] + x_offset * style.scale; // Calculate the base x position relative to the knee point.
    let diameter_x = if is_hand {
        x + style.scale // For right-hand, the diameter is drawn to the right.
    } else {
        x - 2.0 * style.scale - 1.25 * width // For left-hand, account for the extra text width and padding.
    };

    // Define the positions for the arrow leader end, diameter line,
    // and text anchor points (upper and lower).
    let (end, diameter, upper_pos, lower_pos) = (
        [x, knee[1]],
        [diameter_x, knee[1]],
        [diameter_x + style.scale, knee[1] + style.scale],
        [diameter_x + style.scale, knee[1] - style.scale],
    );

    // Plot the arrow leader and the diameter symbol.
    plot_arrow_leader(ui, style, tip, knee, end);
    plot_diameter_symbol(ui, style, diameter);

    // Closure to draw text at a given position with the specified text.
    let mut draw_text = |pos, text| {
        ui.text(
            Text::new(
                PlotPoint::from(pos),
                RichText::new(text).strong().size(13.0 * style.scale as f32),
            )
            .anchor(Align2::LEFT_CENTER),
        );
    };

    // Render the upper and lower limit texts.
    draw_text(upper_pos, upper_text);
    draw_text(lower_pos, lower_text);
}

fn rotate_points(points: &mut [[f64; 2]], centre: [f64; 2], angle: f64) {
    let radians = std::f64::consts::PI / 180.0;
    let rotation = [(angle * radians).cos(), (angle * radians).sin()];

    for point in points.iter_mut() {
        let (x, y) = (point[0], point[1]);
        point[0] = centre[0] + (x - centre[0]) * rotation[0] - (y - centre[1]) * rotation[1];
        point[1] = centre[1] + (x - centre[0]) * rotation[1] + (y - centre[1]) * rotation[0];
    }
}

fn scale_points(points: &mut [[f64; 2]], centre: [f64; 2], scale: f64) {
    for point in points.iter_mut() {
        point[0] = ((point[0] - centre[0]) * scale) + centre[0];
        point[1] = ((point[1] - centre[1]) * scale) + centre[1];
    }
}

fn translate_points(points: &mut [[f64; 2]], target: [f64; 2]) {
    for point in points.iter_mut() {
        point[0] += target[0];
        point[1] += target[1];
    }
}

fn arrow_head(style: &PlotStyle, centre: [f64; 2], angle: f64) -> Polygon {
    let mut points = vec![[0.0, 0.0], [0.8, -0.3], [0.8, 0.3], [0.0, 0.0]];
    translate_points(&mut points, centre);
    scale_points(&mut points, centre, style.scale);
    rotate_points(&mut points, centre, angle);

    Polygon::new(PlotPoints::from(points))
        .fill_color(style.line_colour)
        .stroke(Stroke {
            width: style.line_width,
            color: style.line_colour,
        })
}
