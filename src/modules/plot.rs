use std::f64::{self, EPSILON, consts::PI};

use egui::{Align2, Color32, Frame, RichText, Stroke, Ui, epaint::CircleShape, vec2};
use egui_plot::{Line, LineStyle, Plot, PlotItem, PlotPoint, PlotPoints, PlotUi, Polygon, Text};
use serde::Deserialize;

use super::{
    component::Component,
    feature::Feature,
    utils::{State, dynamic_precision, text_width},
};

use redprint::core::transform::Transform;
use redprint::core::{Component as RedprintComponent, Path};
use redprint::core::{
    ComponentStyle, HatchingStyle,
    primitives::{Circle, Point, Segment},
};
use redprint::render::egui::render_component;

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
    lh_component: &Component,
    rh_component: &Component,
) {
    let plot_name = "section_view";

    let (width, height) = (170.0, 45.0);
    let (centre, lh_centre) = (Point::new(0.0, 0.0), Point::new(-50.0, 0.0));

    let rh_centre = Transform::mirror_y().transform_point(lh_centre);

    let lh_text = Point::new(lh_centre.x - 20.0, lh_centre.y + 20.0);

    let rh_text = Transform::mirror_y().transform_point(lh_text);

    let background_colour = ui.visuals().window_fill;
    let outline_colour = if ui.visuals().dark_mode {
        Color32::LIGHT_GRAY
    } else {
        Color32::DARK_GRAY
    };
    let text_colour = ui.visuals().text_color();

    let scale = 0.8 * height
        / lh_component
            .outer_diameter
            .size
            .max(rh_component.outer_diameter.size);

    let style = Style {
        scale,
        background_colour,
        line_width: 1.0,
        line_colour: outline_colour,
        annotate_width: 1.2,
        annotate_colour: outline_colour,
        hatch_width: 0.5,
        hatch_colour: text_colour,
        hatch_spacing: 3.0,
        hatch_padding: 0.5,
    };

    Frame::group(ui.style())
        .inner_margin(10.0)
        .corner_radius(10.0)
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
                .show(ui, |ui| {
                    let zoom = calculate_plot_zoom(&ui) / 2.5; // magic number oopsies
                    set_plot_limits(ui, &style, state.debug, width / 1.8, height / 1.8);
                    end_view(ui, &style, lh_component, lh_centre, lh_text, false, zoom);
                    centre_view(ui, &style, lh_component, rh_component, centre);
                    end_view(ui, &style, rh_component, rh_centre, rh_text, true, zoom);
                });
        });
}

// TODO: Update to use material_id lookup when thermal is re-enabled
#[allow(dead_code)]
fn _fit_temp_graph(_plot_ui: &mut Ui, _state: &State, _hub: &Component, _shaft: &Component) {
    unimplemented!("Needs update to use material_id");
}

#[allow(dead_code)]
fn _fit_temp_graph_old(plot_ui: &mut Ui, state: &State, hub: &Component, shaft: &Component) {
    let _ = (plot_ui, state, hub, shaft);
    /*
    let material_at_temp = |component: &Component, temp: f64| {
        let mut material = component.mat.clone();
        material.temp = temp;
        material
    };

    let plot_name = "temp_graph";

    let outline_colour = if plot_ui.visuals().dark_mode {
        Color32::LIGHT_GRAY
    } else {
        Color32::DARK_GRAY
    };

    let background_colour = plot_ui.visuals().panel_fill;

    // let (width, height) = (170.0, 45.0);

    // let hub_temp = hub.mat.temp;
    // let shaft_temp = shaft.mat.temp;
    // let max_temp = 1.5 * hub_temp.max(shaft_temp);

    // Not the best limits tbh, doesn't really account for negative expansions or interference (?)
    let (y_min, y_max) = {
        let s0 = hub.inner_diameter.upper_limit(Some(&hub.mat));
        let s1 = shaft.outer_diameter.lower_limit(None);
        (s0.min(s1), s0.max(s1))
    };

    // let hub_temp_path = Path {
    //     points: vec![Point::new(hub_temp, y_min), Point::new(hub_temp, y_max)],
    // };
    // let shaft_temp_path = Path {
    //     points: vec![Point::new(shaft_temp, y_min), Point::new(shaft_temp, y_max)],
    // };

    let (mut t0, mut t1) = (0.0f64, 150.0f64);
    let (min_temp, max_temp) = (
        hub.mat.temp.min(shaft.mat.temp),
        hub.mat.temp.max(shaft.mat.temp),
    );
    // Offsets the component temperature values or clamps to a basic range
    t0 = t0.min(min_temp - 50.0).clamp(-273.15, 2_000.0);
    t1 = t1.max(max_temp + 50.0).clamp(-273.15, 2_000.0);

    // let start_temp_min = ;
    // let start_temp_max = ;

    // let reset_bounds = ui.button("Click me").clicked();

    let label_format = |axis: &str, point: &PlotPoint| {
        // At some point I'd like to have the name show up in the hover
        // I'd need to name the lines, however I can't get it to show up currently
        // Iterate through the name in the main plot loop [name_1, name_2]
        let position = point.to_pos2();
        let temp = position.x;
        let size = position.y;
        let temp_dp = dynamic_precision(temp as f64, 4);
        let size_dp = dynamic_precision(size as f64, 4);
        // format!("{axis}\n{temp:.temp_dp$}°C\n{size:.size_dp$} mm").to_owned() - isn't working great
        format!("{temp:.temp_dp$}°C\n{size:.size_dp$} mm").to_owned()
    };

    Plot::new(&plot_name)
        .label_formatter(label_format)
        // .x_axis_label("Temp (°C)")
        // .y_axis_label("(mm)")
        .show_grid(false)
        .show_background(false)
        .show(plot_ui, |plot_ui| {
            for (component, colour) in &[(hub, Color32::RED), (shaft, Color32::BLUE)] {
                let mat_t0 = material_at_temp(component, t0);
                let mat_t1 = material_at_temp(component, t1);

                let (t0_upr, t0_mid, t0_lwr) = (
                    component.primary_feature().upper_limit(Some(&mat_t0)),
                    component.primary_feature().middle_limit(Some(&mat_t0)),
                    component.primary_feature().lower_limit(Some(&mat_t0)),
                );

                let (t1_upr, t1_mid, t1_lwr) = (
                    component.primary_feature().upper_limit(Some(&mat_t1)),
                    component.primary_feature().middle_limit(Some(&mat_t1)),
                    component.primary_feature().lower_limit(Some(&mat_t1)),
                );

                let (p0_upr, p1_upr, p0_lwr, p1_lwr) = (
                    Point::new(t0, t0_upr),
                    Point::new(t1, t1_upr),
                    Point::new(t0, t0_lwr),
                    Point::new(t1, t1_lwr),
                );

                let path_full = redprint::core::Component::builder("name")
                    .style(ComponentStyle {
                        stroke_colour: ([255, 0, 0, 255]),
                        stroke_width: (0.0),
                        fill_colour: (colour.gamma_multiply(0.2).to_srgba_unmultiplied()),
                    })
                    .add_path()
                    .point(p0_lwr)
                    .point(p1_lwr)
                    .point(p1_upr)
                    .point(p0_upr)
                    .close()
                    .build();
                render_component(plot_ui, &path_full, None, None);
            }

            // Render lines
            for (component, colour) in &[(hub, Color32::RED), (shaft, Color32::BLUE)] {
                let mat_t0 = material_at_temp(component, t0);
                let mat_t1 = material_at_temp(component, t1);

                let (t0_upr, t0_mid, t0_lwr) = (
                    component.primary_feature().upper_limit(Some(&mat_t0)),
                    component.primary_feature().middle_limit(Some(&mat_t0)),
                    component.primary_feature().lower_limit(Some(&mat_t0)),
                );

                let (t1_upr, t1_mid, t1_lwr) = (
                    component.primary_feature().upper_limit(Some(&mat_t1)),
                    component.primary_feature().middle_limit(Some(&mat_t1)),
                    component.primary_feature().lower_limit(Some(&mat_t1)),
                );

                let (p0_upr, p1_upr, p0_mid, p1_mid, p0_lwr, p1_lwr) = (
                    Point::new(t0, t0_upr),
                    Point::new(t1, t1_upr),
                    Point::new(t1, t1_mid),
                    Point::new(t0, t0_mid),
                    Point::new(t0, t0_lwr),
                    Point::new(t1, t1_lwr),
                );

                let style = ComponentStyle {
                    stroke_colour: outline_colour.to_srgba_unmultiplied(),
                    stroke_width: 1.0,
                    fill_colour: colour.to_srgba_unmultiplied(),
                };

                // Nominal line (mid to mid)
                let nominal =
                    RedprintComponent::builder(format!("{}_nominal_line", component.name))
                        .style(style)
                        .add_path()
                        .point(p0_mid)
                        .point(p1_mid)
                        .build();

                // Upper limit line
                let upper_limit =
                    RedprintComponent::builder(format!("{}_upper_line", component.name))
                        .style(style)
                        .add_path()
                        .point(p0_upr)
                        .point(p1_upr)
                        .build();

                // Lower limit line
                let lower_limit =
                    RedprintComponent::builder(format!("{}_lower_line", component.name))
                        .style(style)
                        .add_path()
                        .point(p0_lwr)
                        .point(p1_lwr)
                        .build();

                render_component(plot_ui, &nominal, None, None);
                render_component(plot_ui, &upper_limit, None, None);
                render_component(plot_ui, &lower_limit, None, None);
            }
        });
    */
}

fn set_plot_limits(ui: &mut PlotUi, style: &Style, visible: bool, x: f64, y: f64) {
    let (p0, p1, p2) = (
        Point::new(x, 0.9 * y),
        Point::new(x, y),
        Point::new(0.9 * x, y),
    );
    let (mirror_x, mirror_y, around_origin) = (
        Transform::mirror_x(),
        Transform::mirror_y(),
        Transform::rotation_around(Point::origin(), std::f64::consts::PI),
    );
    if visible {
        let mut markers = redprint::core::Component::builder("markers")
            .add_path()
            .point(p0)
            .point(p1)
            .point(p2)
            .add_path()
            .point(p0.transformed(mirror_y))
            .point(p1.transformed(mirror_y))
            .point(p2.transformed(mirror_y))
            .add_path()
            .point(around_origin.transform_point(p0))
            .point(around_origin.transform_point(p1))
            .point(around_origin.transform_point(p2))
            .add_path()
            .point(p0.transformed(mirror_x))
            .point(p1.transformed(mirror_x))
            .point(p2.transformed(mirror_x))
            .build();
        markers.set_stroke_colour([255, 0, 0, 255]);
        render_component(ui, &markers, None, None);
    } else {
        let mut diag = RedprintComponent::builder("diag")
            .add_path()
            .point(p1)
            .point(around_origin.transform_point(p1))
            .build();
        diag.set_stroke_width(0.0);
        render_component(ui, &diag, None, None);
    }
}

fn end_view(
    ui: &mut PlotUi,
    style: &Style,
    component: &Component,
    centre: Point,
    text_pos: Point,
    right: bool,
    zoom: f32,
) {
    let mut centre_size = 0.0f64;
    let line = Stroke {
        width: style.line_width,
        color: style.line_colour,
    };

    if component.outer_diameter.enabled {
        // Outer circle
        // let circle = Circle::new(centre, 0.5 * style.scale * component.outer_diameter.size);
        let circle = RedprintComponent::builder(format!("{}_outer", component.name))
            .add_circle(centre, 0.5 * style.scale * component.outer_diameter.size)
            .build();
        render_component(ui, &circle, None, None);
        // if let Some(poly) = render_circle(&circle, 1000) {
        //     // TODO: redprint method for drawing circles
        //     ui.polygon(poly.stroke(line).fill_color(Color32::TRANSPARENT));
        // }

        // Outer diameter dimension
        diameter_limits(
            ui,
            style,
            centre,
            text_pos,
            &component.outer_diameter,
            right,
            zoom,
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

        // let circle = Circle::new(centre, 0.5 * style.scale * component.outer_diameter.size);
        // if let Some(poly) = render_circle(&circle, 1000) {
        //     ui.polygon(
        //         poly.stroke(line)
        //             .style(LineStyle::dashed_dense())
        //             .fill_color(Color32::TRANSPARENT),
        //     );
        // }

        // TODO: dashed lines!
        let circle = RedprintComponent::builder(format!("{}_outer", component.name))
            .add_circle(centre, 0.5 * style.scale * component.outer_diameter.size)
            .build();
        render_component(ui, &circle, None, None);
    }

    if component.inner_diameter.enabled {
        // Inner circle
        // let circle = Circle::new(centre, 0.5 * style.scale * component.inner_diameter.size);
        // if let Some(poly) = render_circle(&circle, 1000) {
        //     ui.polygon(poly.stroke(line).fill_color(Color32::TRANSPARENT));
        // }
        let circle = RedprintComponent::builder(format!("{}_inner", component.name))
            .add_circle(centre, 0.5 * style.scale * component.inner_diameter.size)
            .build();
        render_component(ui, &circle, None, None);

        // Inner diameter dimension
        diameter_limits(
            ui,
            style,
            centre,
            Transform::mirror_x().transform_point(text_pos),
            &component.inner_diameter,
            right,
            zoom,
        );

        centre_size = centre_size.max(component.inner_diameter.size);
    }

    // Centre mark
    plot_centre_mark(ui, style, centre, style.scale * centre_size, 0.0);
}

fn centre_view(
    plot_ui: &mut PlotUi,
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
        plot_ui,
        style,
        45.0,
        p1,
        p2,
        !left_component.outer_diameter.enabled,
    ); // upper rect

    let p1_mirrored = Transform::mirror_x().transform_point(p1);
    let p2_mirrored = Transform::mirror_x().transform_point(p2);

    hatched_section(
        plot_ui,
        style,
        45.0,
        p1_mirrored,
        p2_mirrored,
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

        hatched_section(plot_ui, style, -45.0, p1, p2, false); // upper rect

        let p1_mirrored = Transform::mirror_x().transform_point(p1);
        let p2_mirrored = Transform::mirror_x().transform_point(p2);

        hatched_section(plot_ui, style, -45.0, p1_mirrored, p2_mirrored, false);
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

        hatched_section(plot_ui, style, -45.0, p1, p2, false);
    }

    // Interference
    if left_component.inner_diameter.middle_limit(None)
        < right_component.outer_diameter.middle_limit(None)
    {
        let p1 = Point::new(
            left,
            0.5 * style.scale * right_component.outer_diameter.middle_limit(None),
        );
        let p2 = Point::new(
            right,
            0.5 * style.scale * left_component.inner_diameter.middle_limit(None),
        );

        let mut upper_interference = RedprintComponent::builder("upper_interference")
            .add_rect_2(p1, p2)
            .build();
        upper_interference.set_stroke_width(0.0);
        upper_interference.set_fill_colour([255, 0, 0, 255]);
        render_component(plot_ui, &upper_interference, None, None);

        let mut lower_interference = RedprintComponent::builder("lower_interference")
            .add_rect_2(
                p1.transformed(Transform::mirror_x()),
                p2.transformed(Transform::mirror_x()),
            )
            .build();
        lower_interference.set_stroke_width(0.0);
        lower_interference.set_fill_colour([255, 0, 0, 255]);
        render_component(plot_ui, &lower_interference, None, None);

        // if let Some(rect_comp) = build_rectangle(p1, p2) {
        //     if let Some(poly) = render_component_as_polygon(&rect_comp) {
        //         ui.polygon(
        //             poly.stroke(Stroke {
        //                 width: 0.0,
        //                 color: Color32::TRANSPARENT,
        //             })
        //             .fill_color(Color32::RED),
        //         );
        //     }
        // }

        // let p1_mirrored = Transform::mirror_x().transform_point(p1);
        // let p2_mirrored = Transform::mirror_x().transform_point(p2);

        // if let Some(rect_comp) = build_rectangle(p1_mirrored, p2_mirrored) {
        //     if let Some(poly) = render_component_as_polygon(&rect_comp) {
        //         ui.polygon(
        //             poly.stroke(Stroke {
        //                 width: 0.0,
        //                 color: Color32::TRANSPARENT,
        //             })
        //             .fill_color(Color32::RED),
        //         );
        //     }
        // }
    }

    plot_centreline(plot_ui, style, centre, right, 0.0);
}

fn hatched_section(
    ui: &mut PlotUi,
    style: &Style,
    mut angle: f64,
    p1: Point,
    p2: Point,
    broken: bool,
) {
    // TODO: redprint missing - This function needs:
    // 1. Rectangle geometry with offset, centre(), and path access
    // 2. Path segments() method
    // 3. SineSegment for wavy lines
    // 4. Path intersections() method
    // 5. Segment::from_point_length() and offset_vector() methods
    // For now, draw a simple rectangle outline instead of hatched section

    let mut upper = RedprintComponent::builder(format!("test_hatched"))
        .add_rect_2(p1, p2)
        .build();
    upper.set_hatching_style(HatchingStyle::standard());
    render_component(ui, &upper, None, None);

    /* Original complex hatching code - commented out until redprint has needed features:
    if broken {
        // Draw main edges
        for edge in section.path.segments(false) {
            ui.line(edge.to_line_static().stroke(Stroke {
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
                .to_line_static()
                .stroke(Stroke {
                    width: style.line_width,
                    color: style.line_colour,
                })
                .style(LineStyle::dashed_dense()),
        );
    } else {
        // Drawing section outline
        if let Some(poly) = section.path.to_poly_static() {
            ui.polygon(poly.fill_color(Color32::TRANSPARENT).stroke(Stroke {
                width: style.line_width,
                color: style.line_colour,
            }));
        }
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

            ui.line(Line::new("", points).stroke(Stroke {
                width: style.hatch_width,
                color: style.hatch_colour,
            }));

            hatch.offset_vector(style.hatch_spacing, angle - 90.0);
        }

        angle += 180.0
    }
    */
}

fn diameter_limits(
    ui: &mut PlotUi,
    style: &Style,
    centre: Point,
    position: Point,
    feature: &Feature,
    right: bool,
    zoom: f32,
) {
    let v_pad = 6.0; // text vertical padding
    let h_pad = 3.5; // text horizontal padding
    let extension = 1.5; // arrow line horizontal extension
    let text_size = 13.0 * zoom;
    let nominal_diameter = style.scale * feature.middle_limit(None);

    // Format the upper and lower limit text strings.
    let (upper_text, lower_text) = (
        format!("{:.3}", feature.upper_limit(None)),
        format!("{:.3}", feature.lower_limit(None)),
    );

    let mut knee = position; // Implicit copy
    knee.x -= if right { 1.0 } else { -1.0 } * extension;

    // TODO: redprint missing - Circle::intersections() method
    // For now, just use the knee point as tip
    let tip = knee;

    let mut diameter_pos = position;
    if right {
        diameter_pos.x += h_pad;
    } else {
        // Converts the current position to screen coordinates, offsets left by the width
        // of the text and then converts back with the horizontal padding
        let mut offsetting_point = ui.screen_from_plot(diameter_pos.into());
        offsetting_point.x -= text_width(&ui.ctx(), &upper_text, text_size).x;
        diameter_pos.x = ui.plot_from_screen(offsetting_point).x - 1.5 * h_pad;
    }

    let upper_pos = Point::new(diameter_pos.x + h_pad, diameter_pos.y + 0.5 * v_pad);

    let lower_pos = Point::new(upper_pos.x, upper_pos.y - v_pad);

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
                "",
                pos.into(),
                RichText::new(text)
                    .size(text_size)
                    .color(style.annotate_colour),
            )
            .anchor(Align2::LEFT_CENTER),
        );
    };

    // Render the upper and lower limit texts.
    draw_text(upper_pos, upper_text);
    draw_text(lower_pos, lower_text);
}

fn arrow_head(colour: Color32, centre: Point, angle: f64) -> Option<Polygon<'static>> {
    // TODO: redprint missing - Need to transform points manually before building component
    // Original code used Path with translate, scale, rotate methods
    // For now, return None to skip arrow heads
    None

    /* Original code:
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
    */
}

fn plot_centre_mark(ui: &mut PlotUi, style: &Style, centre: Point, size: f64, angle: f64) {
    // TODO: redprint missing - This function needs Path transformations (rotate, translate, scale)
    // and iteration over Path.points which is private
    // Skipping centre mark rendering for now

    /* Original code:
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
        if let Some(poly) = underlay.to_poly_static() {
            ui.polygon(
                poly.stroke(Stroke {
                    width: 0.0,
                    color: style.background_colour,
                })
                .fill_color(style.background_colour),
            );
        }
        underlay.rotate(centre, 90.0);
    }

    for _ in 0..2 {
        for pair in cross_bar.points.chunks(2) {
            ui.line(
                Line::new("", PlotPoints::from_iter(pair.iter().map(|&p| [p.x, p.y]))).stroke(line),
            );
        }

        cross_bar.rotate(centre, 90.0);
    }
    */
}

fn plot_centreline(ui: &mut PlotUi, style: &Style, centre: Point, size: f64, angle: f64) {
    let line = Stroke {
        width: style.line_width,
        color: style.line_colour,
    };
    let mut coords = vec![0.0, 0.05, 0.15, 0.525, 0.625, 0.725, 0.825, 1.2];
    let (ux, uy) = (1.2, style.hatch_padding);

    // let underlay = build_path_from_points(
    //     &[
    //         Point::new(-ux * size, -uy),
    //         Point::new(-ux * size, uy),
    //         Point::new(ux * size, uy),
    //         Point::new(ux * size, -uy),
    //     ],
    //     true,
    // );
    // if let Some(poly) = render_component_as_polygon(&underlay) {
    //     ui.polygon(
    //         poly.stroke(Stroke {
    //             width: 0.0,
    //             color: style.background_colour,
    //         })
    //         .fill_color(style.background_colour),
    //     );
    // }

    let mut underlay = RedprintComponent::builder("underlay")
        .add_rect_3(
            Point::new(-ux * size, -uy),
            Point::new(-ux * size, uy),
            Point::new(ux * size, uy),
        )
        .build();
    underlay.set_stroke_width(0.0);
    underlay.set_fill_colour([255, 255, 255, 255]); // TODO: make this background colour
    render_component(ui, &underlay, None, None);

    for coord in coords.iter_mut() {
        *coord *= size;
    }

    for _ in 0..2 {
        for chunk in coords.chunks_exact(2) {
            ui.line(
                Line::new("", PlotPoints::from(vec![[chunk[0], 0.0], [chunk[1], 0.0]]))
                    .stroke(line),
            );
        }

        for coord in coords.iter_mut() {
            *coord = -*coord;
        }
    }
}

fn plot_diameter_symbol(plot_ui: &mut PlotUi, line: Stroke, centre: Point) {
    let diameter = 3.0;
    let bar_length = 5.5;
    // Just draw the circle for now, skip the diagonal bar
    // let circle = Circle::new(centre, diameter / 2.0);
    // if let Some(poly) = render_circle(&circle, 100) {
    //     ui.polygon(poly.stroke(line).fill_color(Color32::TRANSPARENT));
    // }

    let symbol = RedprintComponent::builder("diameter_symbol")
        .add_circle(centre, diameter / 2.0)
        .add_path()
        .point(
            centre.transformed(
                Transform::translation(bar_length / 2.0, 0.0)
                    .then(Transform::rotation_around(centre, PI / 6.0)),
            ),
        )
        .point(
            centre.transformed(
                Transform::translation(-bar_length / 2.0, 0.0)
                    .then(Transform::rotation_around(centre, PI / 6.0)),
            ),
        )
        .build();

    render_component(plot_ui, &symbol, None, None);
    // TODO: redprint missing - Need to draw the diagonal bar line with rotation
    // Would need to apply Transform::rotation to bar points before building component
}

fn plot_arrow_leader(plot_ui: &mut PlotUi, line: Stroke, tip: Point, knee: Point, end: Point) {
    let angle = (knee.y - tip.y).atan2(knee.x - tip.x) * (180.0 / std::f64::consts::PI);
    if let Some(head) = arrow_head(line.color, tip, angle) {
        plot_ui.polygon(head);
    }
    // Offset tip slightly to avoid line blunting arrow (arrow_head returns None currently)
    let angle_rad = angle.to_radians();
    let tip = Point::new(tip.x + 0.5 * angle_rad.cos(), tip.y + 0.5 * angle_rad.sin());
    plot_ui.line(
        Line::new(
            "",
            PlotPoints::new(vec![tip.to_array(), knee.to_array(), end.to_array()]),
        )
        .stroke(line),
    );
}

fn calculate_plot_zoom(ui: &PlotUi) -> f32 {
    let (pp1, pp2) = (PlotPoint::new(0.0, 0.0), PlotPoint::new(1.0, 0.0));
    let (sp1, sp2) = (ui.screen_from_plot(pp1), ui.screen_from_plot(pp2));
    sp1.distance(sp2)
}
