use std::f64::{consts::PI, EPSILON};

use egui::{epaint::CircleShape, vec2, Align2, Color32, Frame, RichText, Stroke, Ui};
use egui_plot::{Line, LineStyle, Plot, PlotItem, PlotPoint, PlotPoints, PlotUi, Polygon, Text};
use serde::Deserialize;

use super::{
    component::Component,
    feature::Feature,
    geometry::{Circle, Path, Point, Rectangle, Segment, SineSegment},
    utils::{text_width, State},
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

pub fn side_by_side(
    ui: &mut Ui,
    state: &State,
    left_component: &Component,
    right_component: &Component,
) {
    let plot_name = "section_view";

    let (width, height) = (170.0, 45.0);
    let (centre, left_centre) = (Point::new(0.0, 0.0), Point::new(-50.0, 0.0));

    let mut right_centre = left_centre;
    right_centre.mirror_in_y();

    let mut left_text = left_centre;
    left_text.offset(-20.0, 20.0);

    let mut right_text = left_text;
    right_text.mirror_in_y();

    let background_colour = ui.visuals().window_fill;
    let outline_colour = if ui.visuals().dark_mode {
        Color32::LIGHT_GRAY
    } else {
        Color32::DARK_GRAY
    };
    let text_colour = ui.visuals().text_color();

    let scale = 0.8 * height
        / left_component
            .outer_diameter
            .size
            .max(right_component.outer_diameter.size);

    let style = Style {
        scale,
        background_colour,
        line_width: 1.0,
        line_colour: outline_colour,
        annotate_width: 1.2,
        annotate_colour: outline_colour,
        hatch_width: 0.5,
        hatch_colour: text_colour,
        hatch_spacing: 2.5,
        hatch_padding: 0.5,
    };

    Frame::group(ui.style())
        .inner_margin(10.0)
        .rounding(10.0)
        .show(ui, |ui| {
            // ui.set_min_size(vec2(514.0, 175.0));
            ui.set_max_size(vec2(514.0, 160.0));

            Plot::new(&plot_name)
                .data_aspect(1.0)
                .include_x(20.0)
                .show_axes(false)
                .show_grid(false)
                .show_background(false)
                .show_x(false)
                .show_y(false)
                // .allow_drag(false)
                // .allow_zoom(false)
                // .allow_scroll(false)
                // .allow_boxed_zoom(false)
                .show(ui, |ui| {
                    set_plot_limits(ui, &style, state.debug, width / 1.8, height / 1.8);

                    end_view(ui, &style, left_component, left_centre, left_text, false);

                    centre_view(ui, &style, left_component, right_component, centre);

                    end_view(ui, &style, right_component, right_centre, right_text, true);

                    // let p1 = Point::new(-50.0, 20.0);
                    // let p2 = Point::new(30.0, -10.0);
                    // let p3 = Point::new(5.0, -5.0);

                    // let test = SineSegment::new(p1, p2, 10.0, 2.0);
                    // let seg = Segment::new(p1, p2);
                    // ui.line(test.to_path().to_line());
                    // ui.line(seg.to_line());

                    // let intersections = test.to_path().fast_intersections(seg, false);

                    // for int in intersections.into_iter().flatten() {
                    //     ui.polygon(Circle::new(int, 2.0).to_poly());
                    // }
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
    centre: Point,
    text_pos: Point,
    right: bool,
) {
    let mut centre_size = 0.0f64;
    let line = Stroke {
        width: style.line_width,
        color: style.line_colour,
    };

    if component.outer_diameter.enabled {
        // Outer circle
        ui.polygon(
            Circle::new(centre, 0.5 * style.scale * component.outer_diameter.size)
                .to_poly()
                .stroke(line)
                .fill_color(Color32::TRANSPARENT),
        );

        // Outer diameter dimension
        diameter_limits(
            ui,
            style,
            centre,
            text_pos,
            &component.outer_diameter,
            right,
        );

        centre_size = centre_size.max(component.outer_diameter.size);
    } else if !component.outer_diameter.primary {
        // Create boundary
        // let boundary_size = component.outer_diameter.size; //style.scale * component.inner_diameter.size;
        // let (mut p1, mut p2) = (centre, centre);
        // p1.x -= boundary_size;
        // p2.x += boundary_size;
        // p1.rotate(centre, 45.0);
        // p2.rotate(centre, 45.0);
        // ui.polygon(
        //     Rectangle::from_2(p1, p2)
        //         .to_poly()
        //         .stroke(line)
        //         .style(LineStyle::dashed_dense()),
        // );

        ui.polygon(
            Circle::new(centre, 0.5 * style.scale * component.outer_diameter.size)
                .to_poly()
                .stroke(line)
                .style(LineStyle::dashed_dense())
                .fill_color(Color32::TRANSPARENT),
        );
    }

    if component.inner_diameter.enabled {
        // Inner circle
        ui.polygon(
            Circle::new(centre, 0.5 * style.scale * component.inner_diameter.size)
                .to_poly()
                .stroke(line)
                .fill_color(Color32::TRANSPARENT),
        );

        // Inner diameter dimension
        diameter_limits(
            ui,
            style,
            centre,
            {
                let mut point = text_pos.clone();
                point.mirror_in_x();
                point
            },
            &component.inner_diameter,
            right,
        );

        centre_size = centre_size.max(component.inner_diameter.size);
    }

    // Centre mark
    plot_centre_mark(ui, style, centre, style.scale * centre_size, 0.0);
}

pub fn centre_view(
    ui: &mut PlotUi,
    style: &Style,
    left_component: &Component,
    right_component: &Component,
    centre: Point,
) {
    // Aspect ratio will be 1:1 for length to height
    // let right = if left_component.outer_diameter.enabled {
    //     0.5 * style.scale * left_component.outer_diameter.size
    // } else {
    //     0.5 * style.scale * left_component.inner_diameter.size
    // };
    let right = 0.5 * style.scale * left_component.outer_diameter.size;
    let left = -right;

    let mut p1 = Point::new(
        left,
        style.scale * left_component.outer_diameter.middle_limit(None) / 2.0,
    );
    let mut p2 = Point::new(
        right,
        style.scale * left_component.inner_diameter.middle_limit(None) / 2.0,
    );

    hatched_section(
        ui,
        style,
        45.0,
        p1,
        p2,
        !left_component.outer_diameter.enabled,
    ); // upper rect

    p1.mirror_in_x();
    p2.mirror_in_x();

    hatched_section(
        ui,
        style,
        45.0,
        p1,
        p2,
        !left_component.outer_diameter.enabled,
    ); // lower rect

    if right_component.inner_diameter.enabled {
        let mut p1 = Point::new(
            left,
            style.scale * right_component.outer_diameter.middle_limit(None) / 2.0,
        );
        let mut p2 = Point::new(
            right,
            style.scale * right_component.inner_diameter.middle_limit(None) / 2.0,
        );

        hatched_section(ui, style, -45.0, p1, p2, false); // upper rect

        p1.mirror_in_x();
        p2.mirror_in_x();

        hatched_section(ui, style, -45.0, p1, p2, false);
    // lower rect
    } else {
        let p1 = Point::new(
            left,
            style.scale * right_component.outer_diameter.middle_limit(None) / 2.0,
        );
        let p2 = Point::new(
            right,
            -style.scale * right_component.outer_diameter.middle_limit(None) / 2.0,
        );

        hatched_section(ui, style, -45.0, p1, p2, false);
    }

    // Interference
    if left_component.inner_diameter.middle_limit(None)
        < right_component.outer_diameter.middle_limit(None)
    {
        let mut p1 = Point::new(
            left,
            0.5 * style.scale * right_component.outer_diameter.middle_limit(None),
        );
        let mut p2 = Point::new(
            right,
            0.5 * style.scale * left_component.inner_diameter.middle_limit(None),
        );

        ui.polygon(
            Rectangle::from_2(p1, p2)
                .to_poly()
                .stroke(Stroke {
                    width: 0.0,
                    color: Color32::TRANSPARENT,
                })
                .fill_color(Color32::RED),
        );

        p1.mirror_in_x();
        p2.mirror_in_x();

        ui.polygon(
            Rectangle::from_2(p1, p2)
                .to_poly()
                .stroke(Stroke {
                    width: 0.0,
                    color: Color32::TRANSPARENT,
                })
                .fill_color(Color32::RED),
        );
    }

    plot_centreline(ui, style, centre, right, 0.0);
}

pub fn hatched_section(
    ui: &mut PlotUi,
    style: &Style,
    mut angle: f64,
    p1: Point,
    p2: Point,
    broken: bool,
) {
    // Create section outline and bounding box for the hatching
    let mut section = Rectangle::from_2(p1, p2);
    let mut hatching = section.clone();
    hatching.offset(-style.hatch_padding); // Add padding to hatching

    // ui.polygon(hatching.to_poly()); // Uncomment to show hatching outline

    if broken {
        // Draw main edges
        for edge in section.path.segments(false) {
            ui.line(edge.to_line().stroke(Stroke {
                width: style.line_width,
                color: style.line_colour,
            }));
        }

        let section_sine = SineSegment {
            s: section.path.segments(true)[3],
            a: 1.5,
            n: 1.0,
        };
        let hatching_sine = SineSegment {
            s: hatching.path.segments(true)[3],
            a: 1.5,
            n: 1.0,
        };

        section.path.insert(4, section_sine.to_path());
        hatching.path.insert(4, hatching_sine.to_path());

        // Draw sine edge
        ui.line(
            section_sine
                .to_path()
                .to_line()
                .stroke(Stroke {
                    width: style.line_width,
                    color: style.line_colour,
                })
                .style(LineStyle::dashed_dense()),
        );
    } else {
        // Drawing section outline
        ui.polygon(
            section
                .path
                .to_poly()
                .fill_color(Color32::TRANSPARENT)
                .stroke(Stroke {
                    width: style.line_width,
                    color: style.line_colour,
                }),
        );
    }

    // HATCHING MOVES WITH SIZE CHANGE... CAUSE IS FROM_CENTRE METHOD

    for _ in 0..2 {
        let mut hatch = Segment::from_point_length(section.centre(), 10.0, angle);

        loop {
            let intersections = hatching.path.intersections(hatch, true);
            let [p1, p2, ..] = intersections.as_slice() else {
                break; // Moves on if there aren't two intersection points
            };
            let points = PlotPoints::new(vec![p1.to_array(), p2.to_array()]);

            ui.line(Line::new(points).stroke(Stroke {
                width: style.hatch_width,
                color: style.hatch_colour,
            }));

            hatch.offset_vector(style.hatch_spacing, angle - 90.0);
        }

        angle += 180.0
    }
}

pub fn diameter_limits(
    ui: &mut PlotUi,
    style: &Style,
    centre: Point,
    position: Point,
    feature: &Feature,
    right: bool,
) {
    let v_pad = 5.0;
    let h_pad = 3.5;
    let extension = 1.5;
    let nominal_diameter = style.scale * feature.middle_limit(None);

    // Format the upper and lower limit text strings.
    let (upper_text, lower_text) = (
        format!("{:.3}", feature.upper_limit(None)),
        format!("{:.3}", feature.lower_limit(None)),
    );

    let mut knee = position; // Implicit copy
    knee.x -= if right { 1.0 } else { -1.0 } * extension;

    let leader_spine = Segment::new(knee, centre);
    let feature_circle = Circle::new(centre, 0.5 * nominal_diameter);
    let tip = feature_circle
        .intersections(&leader_spine)
        .unwrap_or(vec![centre])[0];

    let mut diameter_pos = position;
    if right {
        diameter_pos.x += h_pad;
    } else {
        // Converts the current position to screen coordinates, offsets left by the width
        // of the text and then converts back with the horizontal padding
        let mut offsetting_point = ui.screen_from_plot(diameter_pos.to_plotpoint());
        offsetting_point.x -= text_width(&ui.ctx(), &upper_text).x;
        diameter_pos.x = ui.plot_from_screen(offsetting_point).x - h_pad;
    }

    let mut upper_pos = diameter_pos;
    upper_pos.offset(h_pad, 0.5 * v_pad);

    let mut lower_pos = upper_pos;
    lower_pos.y -= v_pad;

    let line = Stroke {
        width: style.annotate_width,
        color: style.annotate_colour,
    };

    // Plot the arrow leader and the diameter symbol.
    plot_arrow_leader(ui, line, tip, knee, position);
    plot_diameter_symbol(ui, line, diameter_pos);

    // Closure to draw text at a given position with the specified text.
    let mut draw_text = |pos: Point, text| {
        ui.text(
            Text::new(
                pos.to_plotpoint(),
                RichText::new(text).size(13.0).color(style.annotate_colour),
            )
            .anchor(Align2::LEFT_CENTER),
        );
    };

    // Render the upper and lower limit texts.
    draw_text(upper_pos, upper_text);
    draw_text(lower_pos, lower_text);
}

fn arrow_head(colour: Color32, centre: Point, angle: f64) -> Polygon {
    let mut head = Path {
        points: vec![
            Point::new(0.0, 0.0),
            Point::new(0.8, -0.3),
            Point::new(0.8, 0.3),
        ],
    };
    head.translate(centre.x, centre.y);
    head.scale(centre, 3.0);
    head.rotate(centre, angle);
    head.to_poly().fill_color(colour).stroke(Stroke {
        width: 0.1,
        color: colour,
    })
}

pub fn plot_centre_mark(ui: &mut PlotUi, style: &Style, centre: Point, size: f64, angle: f64) {
    let line = Stroke {
        width: style.line_width,
        color: style.line_colour,
    };
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

    let (ux, uy) = (0.6, style.hatch_padding);
    let mut underlay = Path {
        points: vec![
            Point::new(-ux * size, -uy),
            Point::new(-ux * size, uy),
            Point::new(ux * size, uy),
            Point::new(ux * size, -uy),
        ],
    };

    underlay.rotate(centre, angle);
    underlay.translate(centre.x, centre.y);

    for _ in 0..2 {
        ui.polygon(
            underlay
                .to_poly()
                .stroke(Stroke {
                    width: 0.0,
                    color: style.background_colour,
                })
                .fill_color(style.background_colour),
        );
        underlay.rotate(centre, 90.0);
    }

    for _ in 0..2 {
        for pair in cross_bar.points.chunks(2) {
            ui.line(
                Line::new(PlotPoints::from_iter(pair.iter().map(|&p| [p.x, p.y]))).stroke(line),
            );
        }

        cross_bar.rotate(centre, 90.0);
    }
}

pub fn plot_centreline(ui: &mut PlotUi, style: &Style, centre: Point, size: f64, angle: f64) {
    let line = Stroke {
        width: style.line_width,
        color: style.line_colour,
    };
    let mut coords = vec![0.0, 0.05, 0.15, 0.525, 0.625, 0.725, 0.825, 1.2];
    let (ux, uy) = (1.2, style.hatch_padding);
    let underlay = Path {
        points: vec![
            Point::new(-ux * size, -uy),
            Point::new(-ux * size, uy),
            Point::new(ux * size, uy),
            Point::new(ux * size, -uy),
        ],
    };
    ui.polygon(
        underlay
            .to_poly()
            .stroke(Stroke {
                width: 0.0,
                color: style.background_colour,
            })
            .fill_color(style.background_colour),
    );

    for coord in coords.iter_mut() {
        *coord *= size;
    }

    for _ in 0..2 {
        for chunk in coords.chunks_exact(2) {
            ui.line(
                Line::new(PlotPoints::from(vec![[chunk[0], 0.0], [chunk[1], 0.0]])).stroke(line),
            );
        }

        for coord in coords.iter_mut() {
            *coord = -*coord;
        }
    }
}

pub fn plot_diameter_symbol(ui: &mut PlotUi, line: Stroke, centre: Point) {
    let diameter = 3.0;
    let bar_length = 5.5;
    let mut bar = Path {
        points: vec![
            Point::new(0.0, -0.5 * bar_length),
            Point::new(0.0, 0.5 * bar_length),
        ],
    };
    bar.translate(centre.x, centre.y);
    bar.rotate(centre, -30.0);
    ui.polygon(
        Circle::new(centre, diameter / 2.0)
            .to_poly()
            .stroke(line)
            .fill_color(Color32::TRANSPARENT),
    );
    ui.line(bar.to_line().stroke(line));
}

pub fn plot_arrow_leader(ui: &mut PlotUi, line: Stroke, mut tip: Point, knee: Point, end: Point) {
    let angle = (knee.y - tip.y).atan2(knee.x - tip.x) * (180.0 / std::f64::consts::PI);
    ui.polygon(arrow_head(line.color, tip, angle));
    tip.vector_offset(angle, 0.5); // This is to stop the square line blunting the arrow tip
    ui.line(
        Line::new(PlotPoints::new(vec![
            tip.to_array(),
            knee.to_array(),
            end.to_array(),
        ]))
        .stroke(line),
    );
}
