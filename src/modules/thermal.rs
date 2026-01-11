use crate::Studio;
use crate::modules::{
    material::Material,
    utils::{at_temp, dynamic_precision},
};
use egui::{Color32, Ui};
use egui_plot::{Line, LineStyle, Plot, PlotBounds, PlotPoint, PlotPoints, Points, Polygon};
use redprint::core::primitives::{Point as RedPoint, Segment};
use redprint::core::{Component as RedComponent, ConstraintSpec};
use redprint::render::egui::View;

fn default_true() -> bool {
    true
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Thermal {
    pub enabled: bool,
    pub lower: f64,
    pub upper: f64,
    pub hub_series: Vec<f64>,
    pub shaft_series: Vec<f64>,
    // Display toggles for intersection points
    pub show_limit_intersections: bool,
    pub show_mid_limit_intersections: bool,
    pub show_at_temp_intersections: bool,
    // Component colours (with gamma pre-applied for plot fill)
    pub hub_colour: Color32,
    pub shaft_colour: Color32,
    pub colour_gamma: f32,
    #[serde(skip)]
    pub output_view: View,
    #[serde(skip, default = "default_true")]
    pub fit_plot_needs_reset: bool,
}

impl Thermal {
    pub fn default() -> Self {
        Self {
            enabled: false,
            upper: 120.0,
            lower: 20.0,
            hub_series: vec![20.0, 120.0],
            shaft_series: vec![20.0, 120.0],
            show_limit_intersections: true,
            show_mid_limit_intersections: false,
            show_at_temp_intersections: false,
            hub_colour: Color32::RED.gamma_multiply(0.2),
            shaft_colour: Color32::BLUE.gamma_multiply(0.2),
            colour_gamma: 0.2,
            output_view: View::new(),
            fit_plot_needs_reset: true,
        }
    }
}

pub fn split_temp_input(app: &mut Studio, ui: &mut Ui) {
    // Rebuild the view if the number of temperature points changed
    // We use a single component with one path containing all named points
    if app.thermal.output_view.len() != 1
        || app
            .thermal
            .output_view
            .get(0)
            .map(|c| c.source_points().len() != app.thermal.hub_series.len())
            .unwrap_or(true)
    {
        app.thermal.output_view = View::new();

        // Build a single component with a path containing all temperature points
        let mut builder = RedComponent::builder("hub_temp_curve")
            .cull(false)
            .add_path();

        // Add named points and constrain each to its fixed X position
        for (i, &temp) in app.thermal.hub_series.iter().enumerate() {
            let point_name = format!("p{}", i);
            builder = builder
                .named_point(&point_name, RedPoint::new(i as f64, temp))
                .constrain(
                    format!("fix_x_{}", i),
                    &[&point_name],
                    ConstraintSpec::FixedX { x: i as f64 },
                );
        }

        app.thermal.output_view.add(builder.build());
    }

    let plot = Plot::new("temp_input")
        .height(100.0)
        .allow_drag(!app.thermal.output_view.is_dragging())
        .show_grid(true);

    let plot_response = plot.show(ui, |plot_ui| {
        // Render the single component (line + draggable points)
        app.thermal.output_view.render_all_in(plot_ui);
    });

    let result = app.thermal.output_view.show_interaction(ui, &plot_response);

    if result.changed {
        // Update hub_temp_series with the new point positions
        if let Some(comp) = app.thermal.output_view.get(0) {
            for (i, temp) in app.thermal.hub_series.iter_mut().enumerate() {
                if let Some(point) = comp.get_point(&format!("p{}", i)) {
                    *temp = point.y;
                }
            }
        }
        ui.ctx().request_repaint();
    }
}

pub fn fit_temp_plot(app: &mut Studio, ui: &mut Ui) {
    let (t0, t1) = (-273.0, 1000.0);
    let line_colour = ui.visuals().text_color();
    let (Some(hub), Some(shaft)) = (app.library.get_hub(), app.library.get_shaft()) else {
        return;
    };

    let mut shaded_buffer = Vec::new();
    let mut line_buffer = Vec::new();
    let mut segments: Vec<Segment> = Vec::new();

    let (x0, x1) = (app.thermal.lower, app.thermal.upper);
    let (mut y0, mut y1) = (f64::MAX, f64::MIN);

    let formatter = |name: &str, point: &PlotPoint| {
        // At some point I'd like to have the name show up in the hover
        // I'd need to name the lines, however I can't get it to show up currently
        // Iterate through the name in the main plot loop [name_1, name_2]
        let position = point.to_pos2();
        let temp = position.x;
        let size = position.y;
        let temp_dp = dynamic_precision(temp as f64, 2);
        let size_dp = dynamic_precision(size as f64, 4);
        if name.is_empty() {
            format!("{size:.size_dp$} mm\n{temp:.temp_dp$}°C")
        } else {
            format!("{name}\n{size:.size_dp$} mm\n{temp:.temp_dp$}°C")
        }
    };

    let legend = {
        let mut legend = egui_plot::Legend::default();
        legend.position = egui_plot::Corner::LeftTop;
        legend
    };

    let hub_colour = app.thermal.hub_colour;
    let shaft_colour = app.thermal.shaft_colour;

    for (component, feature, fill_colour) in &[
        (hub, &hub.inner_diameter, hub_colour),
        (shaft, &shaft.outer_diameter, shaft_colour),
    ] {
        let cte = app
            .library
            .get_material(component.material_id)
            .unwrap_or(&Material::default())
            .cte;
        let (feature_upper_t0, feature_middle_t0, feature_lower_t0) = (
            PlotPoint::new(t0, at_temp(feature.upper_limit(), t0, cte)),
            PlotPoint::new(t0, at_temp(feature.middle_limit(), t0, cte)),
            PlotPoint::new(t0, at_temp(feature.lower_limit(), t0, cte)),
        );
        let (feature_upper_t1, feature_middle_t1, feature_lower_t1) = (
            PlotPoint::new(t1, at_temp(feature.upper_limit(), t1, cte)),
            PlotPoint::new(t1, at_temp(feature.middle_limit(), t1, cte)),
            PlotPoint::new(t1, at_temp(feature.lower_limit(), t1, cte)),
        );
        let outer_points = vec![
            feature_lower_t0,
            feature_upper_t0,
            feature_upper_t1,
            feature_lower_t1,
        ];
        shaded_buffer.push(
            Polygon::new(&component.name, PlotPoints::Owned(outer_points))
                .fill_color(*fill_colour)
                .stroke(egui::Stroke {
                    width: 1.5,
                    color: line_colour,
                }),
        );
        line_buffer.push(
            Line::new(
                "",
                PlotPoints::Owned(vec![feature_middle_t0, feature_middle_t1]),
            )
            .color(line_colour)
            .style(LineStyle::dashed_dense()),
        );

        // Collect segments for intersection points (upper, middle, lower)
        segments.push(Segment::new(
            RedPoint::new(t0, at_temp(feature.upper_limit(), t0, cte)),
            RedPoint::new(t1, at_temp(feature.upper_limit(), t1, cte)),
        ));
        segments.push(Segment::new(
            RedPoint::new(t0, at_temp(feature.middle_limit(), t0, cte)),
            RedPoint::new(t1, at_temp(feature.middle_limit(), t1, cte)),
        ));
        segments.push(Segment::new(
            RedPoint::new(t0, at_temp(feature.lower_limit(), t0, cte)),
            RedPoint::new(t1, at_temp(feature.lower_limit(), t1, cte)),
        ));

        y0 = y0.min(at_temp(feature.lower_limit(), x0, cte));
        y1 = y1.max(at_temp(feature.upper_limit(), x1, cte));
    }

    let y_lim = 10.0 * hub.outer_diameter.size;
    line_buffer.push(
        Line::new(
            "",
            PlotPoints::Owned(vec![PlotPoint::new(x0, 0.0), PlotPoint::new(x0, y_lim)]),
        )
        .color(line_colour)
        .style(LineStyle::dashed_loose()),
    );
    line_buffer.push(
        Line::new(
            "",
            PlotPoints::Owned(vec![PlotPoint::new(x1, 0.0), PlotPoint::new(x1, y_lim)]),
        )
        .color(line_colour)
        .style(LineStyle::dashed_loose()),
    );

    // Find intersections between hub and shaft segments based on toggles
    // Segments are stored as: [hub_upper, hub_middle, hub_lower, shaft_upper, shaft_middle, shaft_lower]
    // Limit lines = upper and lower (indices 0, 2 for hub; 3, 5 for shaft)
    // Mid limit lines = middle (indices 1 for hub; 4 for shaft)
    // At temp lines are the vertical dashed lines at x0 and x1
    let mut intersection_points: Vec<PlotPoint> = Vec::new();

    // Limit line intersections (upper/lower vs upper/lower)
    if app.thermal.show_limit_intersections {
        for hub_idx in [0, 2] {
            for shaft_idx in [3, 5] {
                if let Some(pt) = segments[hub_idx].intersect(&segments[shaft_idx]) {
                    intersection_points.push(PlotPoint::new(pt.x, pt.y));
                }
            }
        }
    }

    // Mid limit line intersections (middle vs middle, and middle vs limits)
    if app.thermal.show_mid_limit_intersections {
        // Middle vs middle
        if let Some(pt) = segments[1].intersect(&segments[4]) {
            intersection_points.push(PlotPoint::new(pt.x, pt.y));
        }
        // Middle vs limits
        for hub_idx in [1] {
            for shaft_idx in [3, 5] {
                if let Some(pt) = segments[hub_idx].intersect(&segments[shaft_idx]) {
                    intersection_points.push(PlotPoint::new(pt.x, pt.y));
                }
            }
        }
        for hub_idx in [0, 2] {
            for shaft_idx in [4] {
                if let Some(pt) = segments[hub_idx].intersect(&segments[shaft_idx]) {
                    intersection_points.push(PlotPoint::new(pt.x, pt.y));
                }
            }
        }
    }

    // At temp intersections (where vertical lines at x0/x1 cross the feature lines)
    if app.thermal.show_at_temp_intersections {
        for seg in &segments {
            // Intersection at x0
            let y_at_x0 =
                seg.p1.y + (seg.p2.y - seg.p1.y) * (x0 - seg.p1.x) / (seg.p2.x - seg.p1.x);
            intersection_points.push(PlotPoint::new(x0, y_at_x0));
            // Intersection at x1
            let y_at_x1 =
                seg.p1.y + (seg.p2.y - seg.p1.y) * (x1 - seg.p1.x) / (seg.p2.x - seg.p1.x);
            intersection_points.push(PlotPoint::new(x1, y_at_x1));
        }
    }

    // Calculate the default bounds for reset
    let padding = 0.1 * (x1 - x0);
    let default_bounds = PlotBounds::from_min_max([x0 - padding, y0], [x1 + padding, y1]);
    let needs_reset = app.thermal.fit_plot_needs_reset;

    let response = Plot::new("fit_temp_plot")
        .label_formatter(formatter)
        // .legend(legend)
        .show_grid(false)
        .show_background(false)
        .show(ui, |plot_ui| {
            // Only set bounds on first render or explicit reset
            if needs_reset {
                plot_ui.set_plot_bounds(default_bounds);
            }

            shaded_buffer.into_iter().for_each(|p| plot_ui.polygon(p));
            line_buffer.into_iter().for_each(|l| plot_ui.line(l));

            // Plot intersection points for cursor snapping
            if !intersection_points.is_empty() {
                plot_ui.points(
                    Points::new("Intersection", PlotPoints::Owned(intersection_points))
                        .shape(egui_plot::MarkerShape::Asterisk)
                        .radius(5.0)
                        .color(line_colour),
                );
            }
        });

    // Clear the reset flag after applying
    if needs_reset {
        app.thermal.fit_plot_needs_reset = false;
    }

    // Reset bounds on double-click
    if response.response.double_clicked() {
        app.thermal.fit_plot_needs_reset = true;
    }
}
