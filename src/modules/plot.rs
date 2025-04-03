use std::f64::{consts::PI, EPSILON};

use egui::{vec2, Align2, Color32, Frame, RichText, Stroke, Ui};
use egui_plot::{Line, Plot, PlotPoint, PlotPoints, PlotUi, Polygon, Text};

use super::{
    component::Component,
    feature::Feature,
    geometry::{Path, Point, Rectangle, Segment},
    utils::text_width,
};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Style {
    scale: f64,
    background_colour: Color32,
    line_width: f32,
    line_colour: Color32,
    annotate_width: f32,
    annotate_colour: Color32,
    hatch_width: f32,
    hatch_colour: Color32,
    hatch_spacing: f64,
    hatch_padding: f64,
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
    let plot_name = "bruh";
    let (width, height) = (170.0, 60.0);
    let (centre, left_centre, right_centre) = (
        Point::new(0.0, 0.0),
        Point::new(-50.0, 0.0),
        Point::new(50.0, 0.0),
    );
    let background_colour = ui.visuals().window_fill;
    let outline_colour = if ui.visuals().dark_mode {
        Color32::LIGHT_GRAY
    } else {
        Color32::DARK_GRAY
    };

    let scale = {
        let mut denominator = 1.0f64;

        for component in [left_component, right_component] {
            if component.outer_diameter.enabled {
                denominator = denominator.max(component.outer_diameter.size);
            } else {
                denominator = denominator.max(component.inner_diameter.size);
            }
        }

        0.6 * height / denominator
    };

    // let scale = 1.0;

    let style = Style {
        scale,
        background_colour,
        line_width: 1.0,
        line_colour: outline_colour,
        annotate_width: 1.5,
        annotate_colour: outline_colour,
        hatch_width: 0.5,
        hatch_colour: outline_colour,
        hatch_spacing: 2.5,
        hatch_padding: 0.5,
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
                    set_plot_limits(ui, &style, true, width / 2.0, height / 2.0);

                    end_view(ui, &style, left_component, false, &left_centre);

                    centre_view(ui, &style, left_component, right_component, &centre);

                    end_view(ui, &style, right_component, true, &right_centre);
                });
        });
}

pub fn set_plot_limits(ui: &mut PlotUi, style: &Style, visible: bool, x: f64, y: f64) {
    if visible {
        let (dx, dy) = (x / 10.0, y / 10.0);
        let mut marker = Path {
            points: vec![
                Point::new(-x + dx, y),
                Point::new(-x, y),
                Point::new(-x, y - dy),
            ],
        };
        for _ in 0..2 {
            for _ in 0..2 {
                ui.line(marker.to_line().stroke(Stroke {
                    width: 1.0,
                    color: Color32::RED,
                }));

                marker.mirror_in_y();
            }

            marker.mirror_in_x();
        }
    } else {
        ui.polygon(
            Rectangle::new([-x, y], [x, -y])
                .to_poly()
                .stroke(Stroke {
                    width: 1.0,
                    color: style.background_colour,
                })
                .fill_color(Color32::TRANSPARENT),
        );
    }
}

pub fn end_view(
    ui: &mut PlotUi,
    style: &Style,
    component: &Component,
    right: bool,
    centre: &Point,
) {
    let mut centre_size = 0.0f64;
    let line = Stroke {
        width: style.line_width,
        color: style.line_colour,
    };

    if component.outer_diameter.enabled {
        // Outer circle
        plot_circle(
            ui,
            line,
            Color32::TRANSPARENT,
            centre,
            style.scale * component.outer_diameter.size,
        );

        // Outer diameter dimension
        diameter_limits(
            ui,
            style,
            centre,
            &component.outer_diameter,
            if right { 60.0 } else { 180.0 - 60.0 },
            5.0,
        );

        centre_size = centre_size.max(component.outer_diameter.size);
    }

    if component.inner_diameter.enabled {
        // Inner circle
        plot_circle(
            ui,
            line,
            Color32::TRANSPARENT,
            centre,
            style.scale * component.inner_diameter.size,
        );

        // Inner diameter dimension
        diameter_limits(
            ui,
            style,
            centre,
            &component.inner_diameter,
            if right { -60.0 } else { 60.0 - 180.0 },
            5.0,
        );

        centre_size = centre_size.max(component.inner_diameter.size);
    }

    // Centre mark
    plot_centre_mark(ui, line, centre, style.scale * centre_size, 0.0);
}

pub fn centre_view(
    ui: &mut PlotUi,
    style: &Style,
    left_component: &Component,
    right_component: &Component,
    centre: &Point,
) {
    // Aspect ratio will be 1:1 for length to height
    let right = if left_component.outer_diameter.enabled {
        0.5 * style.scale * left_component.outer_diameter.size
    } else {
        0.5 * style.scale * left_component.inner_diameter.size
    };
    let left = -right;

    if left_component.outer_diameter.enabled {
        let upper = style.scale * left_component.outer_diameter.size / 2.0;
        let lower = style.scale * left_component.inner_diameter.size / 2.0;

        let mut p1 = Point::new(left, upper);
        let mut p2 = Point::new(right, lower);

        hatched_section(ui, style, 45.0, &p1, &p2); // upper rect

        p1.mirror_in_x();
        p2.mirror_in_x();

        hatched_section(ui, style, 45.0, &p1, &p2);
        // lower rect
    }

    if right_component.inner_diameter.enabled {
        let upper = style.scale * right_component.outer_diameter.size / 2.0;
        let lower = style.scale * right_component.inner_diameter.size / 2.0;

        let mut p1 = Point::new(left, upper);
        let mut p2 = Point::new(right, lower);

        hatched_section(ui, style, -45.0, &p1, &p2); // upper rect

        p1.mirror_in_x();
        p2.mirror_in_x();

        hatched_section(ui, style, -45.0, &p1, &p2);
    // lower rect
    } else {
        let upper = style.scale * right_component.outer_diameter.size / 2.0;
        let lower = -upper;

        let p1 = Point::new(left, upper);
        let p2 = Point::new(right, lower);

        hatched_section(ui, style, -45.0, &p1, &p2);
    }

    // TODO
    // Add white underlay to centreline for separation
    let line = Stroke {
        width: style.line_width,
        color: style.line_colour,
    };
    plot_centreline(ui, line, centre, right, 0.0);
}

pub fn hatched_section(ui: &mut PlotUi, style: &Style, angle: f64, p1: &Point, p2: &Point) {
    // Create section outline and bounding box for the hatching
    let section = Rectangle::from(p1, p2);
    let mut hatching = section.clone();
    hatching.offset(-style.hatch_padding); // Add padding to hatching

    // ui.polygon(hatching.to_poly()); // Uncomment to show hatching outline

    // Drawing section outline
    ui.polygon(
        section
            .to_poly()
            .fill_color(Color32::TRANSPARENT)
            .stroke(Stroke {
                width: style.line_width,
                color: style.line_colour,
            }),
    );

    // HATCHING AT 90º CURRENTLY BROKEN
    // HATCHING MOVES WITH SIZE CHANGE... CAUSE IS FROM_CENTRE METHOD

    let construction_length = section.width().max(section.height());
    let mut hatch = Segment::from_centre(&section.centre(), construction_length, angle);
    hatch.offset_vector(construction_length, angle + 90.0);

    while let None = hatching.intersections(&hatch) {
        hatch.offset_vector(style.hatch_spacing, angle - 90.0);
    }

    while let Some(ints) = hatching.intersections(&hatch) {
        let p1 = ints[0];
        let p2 = ints[1];

        let points = PlotPoints::new(vec![p1.to_arr(), p2.to_arr()]);

        ui.line(Line::new(points).stroke(Stroke {
            width: style.hatch_width,
            color: style.hatch_colour,
        }));

        hatch.offset_vector(style.hatch_spacing, angle - 90.0);
    }
}

pub fn diameter_limits(
    ui: &mut PlotUi,
    style: &Style,
    centre: &Point,
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
    let mut arrow_line = Path {
        points: vec![
            Point::new(style.scale * feature.size / 2.0, 0.0),
            Point::new(style.scale * feature.size / 2.0 + length * style.scale, 0.0),
        ],
    };
    arrow_line.translate(centre.x, centre.y);
    arrow_line.rotate(centre, angle);

    let (tip, knee) = (arrow_line.points[0], arrow_line.points[1]); // Define the tip and knee of the arrow.
    let is_hand = angle.abs() <= 90.0; // Determine if we are using a "hand" (right-hand) orientation.
    let x_offset = if is_hand { 1.5 } else { -1.5 }; // x_offset is positive for right-hand, negative for left-hand.

    let text_width = |text: &str| text_width(ui.ctx(), text).x as f64 / 13.0; // Closure to compute the text width (normalized by 13.0) for a given string.
    let width = text_width(&upper_text).max(text_width(&lower_text)); // Compute the maximum width needed between the two texts.

    let x = knee.x + x_offset * style.scale; // Calculate the base x position relative to the knee point.
    let diameter_x = if is_hand {
        x + style.scale // For right-hand, the diameter is drawn to the right.
    } else {
        x - (2.0 + 1.25 * width) * style.scale - 5.0 // BODGED! For left-hand, account for the extra text width and padding.
    };

    // Define the positions for the arrow leader end, diameter line,
    // and text anchor points (upper and lower).
    let (end, diameter, upper_pos, lower_pos) = (
        Point::new(x, knee.y),
        Point::new(diameter_x, knee.y),
        Point::new(diameter_x + style.scale, knee.y + style.scale),
        Point::new(diameter_x + style.scale, knee.y - style.scale),
    );

    let line = Stroke {
        width: style.annotate_width,
        color: style.annotate_colour,
    };

    // Plot the arrow leader and the diameter symbol.
    plot_arrow_leader(ui, line, &tip, &knee, &end);
    plot_diameter_symbol(ui, line, &diameter);

    // Closure to draw text at a given position with the specified text.
    let mut draw_text = |pos: Point, text| {
        ui.text(
            Text::new(pos.to_pp(), RichText::new(text).strong().size(13.0))
                .anchor(Align2::LEFT_CENTER),
        );
    };

    // Render the upper and lower limit texts.
    draw_text(upper_pos, upper_text);
    draw_text(lower_pos, lower_text);
}

fn arrow_head(line: Stroke, centre: &Point, angle: f64) -> Polygon {
    let mut head = Path {
        points: vec![
            Point::new(0.0, 0.0),
            Point::new(0.8, -0.3),
            Point::new(0.8, 0.3),
        ],
    };
    head.translate(centre.x, centre.y);
    head.scale(centre, 2.0);
    head.rotate(centre, angle);
    head.to_poly().fill_color(line.color).stroke(line)
}

pub fn plot_circle(ui: &mut PlotUi, line: Stroke, fill: Color32, centre: &Point, diameter: f64) {
    let radius = diameter / 2.0;
    let mut points = Vec::with_capacity(RESOLUTION);

    for i in 0..RESOLUTION {
        let theta = 2.0 * std::f64::consts::PI * (i as f64 / RESOLUTION as f64);
        let x = centre.x + radius * theta.cos();
        let y = centre.y + radius * theta.sin();
        points.push([x, y]);
    }

    ui.polygon(
        Polygon::new(PlotPoints::from(points))
            .fill_color(fill)
            .stroke(line),
    );
}

pub fn plot_centre_mark(ui: &mut PlotUi, line: Stroke, centre: &Point, size: f64, angle: f64) {
    let distances = vec![0.05, 0.1, 0.6];
    let mut cross_bar = Path {
        points: vec![
            Point::new(-distances[2], 0.0),
            Point::new(-distances[1], 0.0),
            Point::new(-distances[0], 0.0),
            Point::new(distances[0], 0.0),
            Point::new(distances[1], 0.0),
            Point::new(distances[2], 0.0),
        ],
    };

    cross_bar.rotate(centre, angle);
    cross_bar.translate(centre.x, centre.y);
    cross_bar.scale(centre, size);

    for _ in 0..2 {
        for pair in cross_bar.points.chunks(2) {
            ui.line(
                Line::new(PlotPoints::from_iter(pair.iter().map(|&p| [p.x, p.y]))).stroke(line),
            );
        }

        cross_bar.rotate(centre, 90.0);
    }
}

pub fn plot_centreline(ui: &mut PlotUi, line: Stroke, centre: &Point, size: f64, angle: f64) {
    let mut coords = vec![0.0, 0.05, 0.15, 0.55, 0.65, 0.75, 0.85, 1.25];

    for coord in coords.iter_mut() {
        *coord *= size;
    }

    for _ in 0..2 {
        for pair in coords.chunks(2) {
            ui.line(Line::new(PlotPoints::from(vec![[pair[0], 0.0], [pair[1], 0.0]])).stroke(line));
        }

        for coord in coords.iter_mut() {
            *coord = -*coord;
        }
    }
}

pub fn plot_diameter_symbol(ui: &mut PlotUi, line: Stroke, centre: &Point) {
    let diameter = 2.5;
    let bar_length = 4.5;
    let mut bar = Path {
        points: vec![
            Point::new(0.0, -0.5 * bar_length),
            Point::new(0.0, 0.5 * bar_length),
        ],
    };
    bar.translate(centre.x, centre.y);
    bar.rotate(centre, -30.0);
    plot_circle(ui, line, Color32::TRANSPARENT, centre, diameter);
    ui.line(bar.to_line().stroke(line));
}

pub fn plot_arrow_leader(ui: &mut PlotUi, line: Stroke, tip: &Point, knee: &Point, end: &Point) {
    let angle = (knee.y - tip.y).atan2(knee.x - tip.x) * (180.0 / std::f64::consts::PI);
    ui.line(
        Line::new(PlotPoints::new(vec![
            tip.to_arr(),
            knee.to_arr(),
            end.to_arr(),
        ]))
        .stroke(line),
    );
    ui.polygon(arrow_head(line, tip, angle));
}
