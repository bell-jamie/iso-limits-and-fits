use egui::{vec2, Align2, Color32, Frame, RichText, Stroke, Ui};
use egui_plot::{Line, Plot, PlotPoint, PlotPoints, PlotUi, Polygon, Text};

use super::{
    component::Component,
    feature::Feature,
    geometry::Point,
    geometry::{Rectangle, Segment},
    utils::text_width,
};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct PlotStyle {
    scale: f64,
    width: f32,
    colour: Color32,
    fill: Color32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct HatchStyle {
    angle: f64,
    spacing: f64,
    padding: f64,
    width: f32,
    colour: Color32,
}

// impl PlotStyle {
//     pub fn default() -> Self {
//         PlotStyle {
//             scale: 1.0,
//             line_width: 1.0,
//             line_colour: Color32::DARK_GRAY,
//             fill_colour: Color32::TRANSPARENT,
//         }
//     }
// }

// Must be even
const RESOLUTION: usize = 1_000;

pub fn side_by_side(ui: &mut Ui, left_component: &Component, right_component: &Component) {
    let outline_colour = if ui.visuals().dark_mode {
        Color32::LIGHT_GRAY
    } else {
        Color32::DARK_GRAY
    };

    // let centre = [0.0; 2];
    // let scale = 1.0;
    let plot_name = "bruh";

    // let scale = 10.0 / left_component.outer_diameter.upper_limit(None).max(left_component.)

    let scale = {
        let mut divisor = 1.0f64;

        for component in [left_component, right_component] {
            if component.outer_diameter.enabled {
                divisor = divisor.max(component.outer_diameter.size);
            } else {
                divisor = divisor.max(component.inner_diameter.size);
            }
        }

        divisor / 15.0
    };

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
                        scale,
                        width: 1.0,
                        colour: outline_colour,
                        fill: Color32::TRANSPARENT,
                    };
                    let annotate_style = PlotStyle {
                        scale,
                        width: 1.5,
                        colour: outline_colour,
                        fill: Color32::TRANSPARENT,
                    };
                    let hatch_style = HatchStyle {
                        angle: 45.0,
                        spacing: 1.8,
                        padding: 0.25,
                        width: 0.8,
                        colour: outline_colour,
                    };

                    end_view(
                        ui,
                        left_component,
                        false,
                        [1.5 * -15.0 * scale, 0.0],
                        &line_style,
                        &annotate_style,
                    );

                    centre_view(
                        ui,
                        1.0,
                        left_component,
                        right_component,
                        [0.0, 0.0],
                        &line_style,
                        &annotate_style,
                        &hatch_style,
                    );

                    end_view(
                        ui,
                        right_component,
                        true,
                        [1.5 * 15.0 * scale, 0.0],
                        &line_style,
                        &annotate_style,
                    );
                });
        });
}

pub fn end_view(
    ui: &mut PlotUi,
    component: &Component,
    right: bool,
    centre: [f64; 2],
    line_style: &PlotStyle,
    annotate_style: &PlotStyle,
) {
    let mut centre_size = 0.0f64;

    if component.outer_diameter.enabled {
        // Outer circle
        plot_circle(ui, line_style, centre, component.outer_diameter.size);

        // Outer diameter dimension
        plot_diameter_limits(
            ui,
            annotate_style,
            centre,
            &component.outer_diameter,
            if right { 60.0 } else { 180.0 - 60.0 },
            5.0,
        );

        centre_size = centre_size.max(component.outer_diameter.size);
    }

    if component.inner_diameter.enabled {
        // Inner circle
        plot_circle(ui, line_style, centre, component.inner_diameter.size);

        // Inner diameter dimension
        plot_diameter_limits(
            ui,
            annotate_style,
            centre,
            &component.inner_diameter,
            if right { -60.0 } else { 60.0 - 180.0 },
            5.0,
        );

        centre_size = centre_size.max(component.inner_diameter.size);
    }

    // Centre mark
    plot_centre_mark(ui, line_style, centre, centre_size, 0.0);
}

pub fn centre_view(
    ui: &mut PlotUi,
    scale: f64,
    left_component: &Component,
    right_component: &Component,
    centre: [f64; 2],
    line_style: &PlotStyle,
    annotate_style: &PlotStyle,
    hatch_style: &HatchStyle,
) {
    // Aspect ratio will be 1:1 for length to height
    let right = if left_component.outer_diameter.enabled {
        0.5 * left_component.outer_diameter.size
    } else {
        0.5 * left_component.inner_diameter.size
    };
    let left = -right;

    if right_component.inner_diameter.enabled {
        let upper = scale * right_component.outer_diameter.size / 2.0;
        let lower = scale * right_component.inner_diameter.size / 2.0;

        let mut p1 = Point::new(left, upper);
        let mut p2 = Point::new(right, lower);

        plot_hatched_section(ui, scale, line_style, hatch_style, &p1, &p2); // upper rect

        p1.mirror_in_x();
        p2.mirror_in_x();

        plot_hatched_section(ui, scale, line_style, hatch_style, &p1, &p2); // lower rect
    } else {
        let upper = scale * right_component.outer_diameter.size / 2.0;
        let lower = -upper;

        let p1 = Point::new(left, upper);
        let p2 = Point::new(right, lower);

        plot_hatched_section(ui, scale, line_style, hatch_style, &p1, &p2);
    }

    let mut hatch_style = hatch_style.clone();
    hatch_style.angle = -hatch_style.angle;

    if left_component.outer_diameter.enabled {
        let upper = scale * left_component.outer_diameter.size / 2.0;
        let lower = scale * left_component.inner_diameter.size / 2.0;

        let mut p1 = Point::new(left, upper);
        let mut p2 = Point::new(right, lower);

        plot_hatched_section(ui, scale, line_style, &hatch_style, &p1, &p2); // upper rect

        p1.mirror_in_x();
        p2.mirror_in_x();

        plot_hatched_section(ui, scale, line_style, &hatch_style, &p1, &p2); // lower rect
    }

    plot_centreline(ui, line_style, centre, right, 0.0);
}

pub fn plot_hatched_section(
    ui: &mut PlotUi,
    scale: f64,
    line_style: &PlotStyle,
    hatch_style: &HatchStyle,
    p1: &Point,
    p2: &Point,
) {
    let centroid = Segment::from(p1, p2).midpoint().to_arr();
    let pad = hatch_style.padding;
    let mut outline_points = Rectangle::from(p1, p2).to_vec();

    scale_points(&mut outline_points, centroid, scale);

    let section = Polygon::new(PlotPoints::from(outline_points))
        .fill_color(Color32::TRANSPARENT)
        .stroke(Stroke {
            width: line_style.width,
            color: line_style.colour,
        });

    ui.polygon(section);

    let p3 = Point::new(p1.x + pad, p1.y - pad);
    let p4 = Point::new(p1.x - pad, p1.y + pad);

    let mut hatch_points = Rectangle::from(&p3, &p4).to_vec();

    scale_points(&mut hatch_points, centroid, scale);

    let hatch_bounding_box = Polygon::new(PlotPoints::from(hatch_points))
        .fill_color(Color32::RED)
        .stroke(Stroke {
            width: line_style.width,
            color: Color32::TRANSPARENT,
        });

    // ui.polygon(hatch_bounding_box);

    // This is the custom geometry driven section
    let mut anchor = p3.clone();
    anchor.x += 0.01; // fiddle to stop FP errors
    let mut hatch = Segment::from_point_angle(&anchor, hatch_style.angle.to_radians());
    let hatch_box = Rectangle::from(p1, p2);

    // Going to assume that the hatching angle is between 0 & 90
    while let Some(ints) = hatch_box.intersections(&hatch) {
        let points = vec![ints[0].to_arr(), ints[1].to_arr()];

        ui.line(Line::new(PlotPoints::new(points)).stroke(Stroke {
            width: line_style.width,
            color: line_style.colour,
        }));

        hatch.p1.x += hatch_style.spacing;
        hatch.p2.x += hatch_style.spacing;
    }
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
            .fill_color(style.fill)
            .stroke(Stroke {
                width: style.width,
                color: style.colour,
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
                    .width(style.width)
                    .color(style.colour),
            );
        }

        rotate_points(&mut points, centre, 90.0);
    }
}

pub fn plot_centreline(
    ui: &mut PlotUi,
    style: &PlotStyle,
    centre: [f64; 2],
    size: f64,
    angle: f64,
) {
    let mut coords = vec![0.0, 0.05, 0.15, 0.55, 0.65, 0.75, 0.85, 1.25];

    for coord in coords.iter_mut() {
        *coord *= size;
    }

    for _ in 0..2 {
        for pair in coords.chunks(2) {
            ui.line(
                Line::new(PlotPoints::from(vec![[pair[0], 0.0], [pair[1], 0.0]]))
                    .width(style.width)
                    .color(style.colour),
            );
        }

        for coord in coords.iter_mut() {
            *coord = -*coord;
        }
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

    plot_circle(ui, style, centre, style.scale);
    ui.line(
        Line::new(PlotPoints::new(bar))
            .width(style.width)
            .color(style.colour),
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
            .width(style.width)
            .color(style.colour),
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
        [feature.size / 2.0 + length * style.scale, 0.0],
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
        x - (2.0 + 1.25 * width) * style.scale // For left-hand, account for the extra text width and padding.
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
                RichText::new(text).strong().size(13.0),
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
        .fill_color(style.colour)
        .stroke(Stroke {
            width: style.width,
            color: style.colour,
        })
}
