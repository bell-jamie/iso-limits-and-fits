use crate::Studio;
use crate::modules::{
    material::Material,
    utils::{at_temp, decimals_for_sig_figs, fix_dp},
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
    pub temp: f64,
    pub hub_series: Vec<f64>,
    pub shaft_series: Vec<f64>,
    // Display toggle for intersection points
    pub show_intersections: bool,
    // Component colours (with gamma pre-applied for plot fill)
    pub hub_colour: Color32,
    pub shaft_colour: Color32,
    pub colour_gamma: f32,
    #[serde(skip)]
    pub output_view: View,
    #[serde(skip, default = "default_true")]
    pub fit_plot_needs_reset: bool,
    // Tracking fields for detecting input data changes (to trigger plot bounds reset)
    #[serde(skip)]
    last_hub_id: Option<usize>,
    #[serde(skip)]
    last_shaft_id: Option<usize>,
    #[serde(skip)]
    last_hub_upper: Option<f64>,
    #[serde(skip)]
    last_hub_lower: Option<f64>,
    #[serde(skip)]
    last_shaft_upper: Option<f64>,
    #[serde(skip)]
    last_shaft_lower: Option<f64>,
    #[serde(skip)]
    last_hub_cte: Option<f64>,
    #[serde(skip)]
    last_shaft_cte: Option<f64>,
}

impl Thermal {
    pub fn default() -> Self {
        Self {
            enabled: false,
            temp: 120.0,
            hub_series: vec![20.0, 120.0],
            shaft_series: vec![20.0, 120.0],
            show_intersections: true,
            hub_colour: Color32::RED.gamma_multiply(0.2),
            shaft_colour: Color32::BLUE.gamma_multiply(0.2),
            colour_gamma: 0.2,
            output_view: View::new(),
            fit_plot_needs_reset: true,
            last_hub_id: None,
            last_shaft_id: None,
            last_hub_upper: None,
            last_hub_lower: None,
            last_shaft_upper: None,
            last_shaft_lower: None,
            last_hub_cte: None,
            last_shaft_cte: None,
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
    let line_colour = ui.visuals().text_color();
    let (Some(hub), Some(shaft)) = (app.library.get_hub(), app.library.get_shaft()) else {
        return;
    };

    let (x0, x1) = (20.0, app.thermal.temp);
    let (t0, t1) = (-273.0, 2000.0);

    // Get current values for change detection
    let current_hub_id = app.library.hub_id;
    let current_shaft_id = app.library.shaft_id;
    let current_hub_upper = hub.inner_diameter.upper_limit();
    let current_hub_lower = hub.inner_diameter.lower_limit();
    let current_shaft_upper = shaft.outer_diameter.upper_limit();
    let current_shaft_lower = shaft.outer_diameter.lower_limit();
    let current_hub_cte = app
        .library
        .get_material(hub.material_id)
        .unwrap_or(&Material::default())
        .cte;
    let current_shaft_cte = app
        .library
        .get_material(shaft.material_id)
        .unwrap_or(&Material::default())
        .cte;

    // Check if any input data changed (excluding temperature bounds which are the vertical lines)
    let data_changed = app.thermal.last_hub_id != Some(current_hub_id)
        || app.thermal.last_shaft_id != Some(current_shaft_id)
        || app.thermal.last_hub_upper != Some(current_hub_upper)
        || app.thermal.last_hub_lower != Some(current_hub_lower)
        || app.thermal.last_shaft_upper != Some(current_shaft_upper)
        || app.thermal.last_shaft_lower != Some(current_shaft_lower)
        || app.thermal.last_hub_cte != Some(current_hub_cte)
        || app.thermal.last_shaft_cte != Some(current_shaft_cte);

    if data_changed {
        app.thermal.fit_plot_needs_reset = true;
    }

    let mut shaded_buffer = Vec::new();
    let mut segments: Vec<Segment> = Vec::new();
    // Store middle limit line parameters for viewport-clipped rendering
    // Each entry: (middle_limit_at_20C, cte)
    let mut middle_lines: Vec<(f64, f64)> = Vec::new();

    let (mut y0, mut y1) = (f64::MAX, f64::MIN);

    let formatter = |name: &str, point: &PlotPoint| {
        let position = point.to_pos2();
        let temp = fix_dp(position.x, 1);
        let size = fix_dp(position.y, 3);
        if name.is_empty() {
            format!("{size} mm\n{temp} °C")
        } else {
            format!("{name}\n{size} mm @ {temp}°C")
        }
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
        // Store middle limit parameters for viewport-clipped rendering
        middle_lines.push((feature.middle_limit(), cte));

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

    let mut component_intersection_points: Vec<(PlotPoint, String)> = Vec::new();
    let mut temp_intersection_points: Vec<(PlotPoint, String)> = Vec::new();

    let hub_name = &hub.name;
    let shaft_name = &shaft.name;

    // Segments layout:
    // 0 = hub upper, 1 = hub mid, 2 = hub lower
    // 3 = shaft upper, 4 = shaft mid, 5 = shaft lower
    let (hub_upper, hub_mid, hub_lower) = (&segments[0], &segments[1], &segments[2]);
    let (shaft_upper, shaft_mid, shaft_lower) = (&segments[3], &segments[4], &segments[5]);

    // Temperature lines as vertical segments
    let temp_lower = Segment::new(
        RedPoint::new(x0, y0 - 1000.0),
        RedPoint::new(x0, y1 + 1000.0),
    );
    let temp_upper = Segment::new(
        RedPoint::new(x1, y0 - 1000.0),
        RedPoint::new(x1, y1 + 1000.0),
    );

    if app.thermal.show_intersections {
        // Component limit intersections (circle markers)
        // Hub limits vs shaft limits
        for (hub_seg, hub_label) in [(hub_upper, "least material"), (hub_lower, "max material")] {
            for (shaft_seg, shaft_label) in [
                (shaft_upper, "max material"),
                (shaft_lower, "least material"),
            ] {
                if let Some(pt) = hub_seg.intersect(shaft_seg) {
                    let label = if hub_label == shaft_label {
                        format!("Both {hub_label}")
                    } else {
                        format!("{hub_name} {hub_label}\n{shaft_name} {shaft_label}")
                    };
                    component_intersection_points.push((PlotPoint::new(pt.x, pt.y), label));
                }
            }
        }

        // Mid-limit intersection (circle marker) - only mid vs mid
        if let Some(pt) = hub_mid.intersect(shaft_mid) {
            component_intersection_points
                .push((PlotPoint::new(pt.x, pt.y), "Both middle limit".to_string()));
        }

        // Temperature line intersections with all component lines (square markers)
        let component_lines = [
            (hub_upper, format!("{hub_name} upper limit")),
            (hub_mid, format!("{hub_name} middle limit")),
            (hub_lower, format!("{hub_name} lower limit")),
            (shaft_upper, format!("{shaft_name} upper limit")),
            (shaft_mid, format!("{shaft_name} middle limit")),
            (shaft_lower, format!("{shaft_name} lower limit")),
        ];

        for (seg, label) in &component_lines {
            // Lower temperature line intersection
            if let Some(pt) = temp_lower.intersect(seg) {
                temp_intersection_points.push((PlotPoint::new(pt.x, pt.y), label.clone()));
            }
            // Upper temperature line intersection
            if let Some(pt) = temp_upper.intersect(seg) {
                temp_intersection_points.push((PlotPoint::new(pt.x, pt.y), label.clone()));
            }
        }
    }

    // Calculate the default bounds for reset
    let padding = 0.1 * (x1 - x0);
    let default_bounds = PlotBounds::from_min_max([x0 - padding, y0], [x1 + padding, y1]);
    let needs_reset = app.thermal.fit_plot_needs_reset;

    let response = Plot::new("fit_temp_plot")
        .label_formatter(formatter)
        .show_grid(false)
        .show_background(false)
        .show(ui, |plot_ui: &mut egui_plot::PlotUi| {
            if needs_reset {
                plot_ui.set_plot_bounds(default_bounds);
            }

            shaded_buffer.into_iter().for_each(|p| plot_ui.polygon(p));

            // Render middle limit lines clipped to viewport for performance
            let bounds = plot_ui.plot_bounds();
            let view_x_min = bounds.min()[0];
            let view_x_max = bounds.max()[0];
            let margin = (view_x_max - view_x_min) * 0.1;
            let clip_x0 = (view_x_min - margin).max(t0);
            let clip_x1 = (view_x_max + margin).min(t1);

            for (middle_limit, cte) in &middle_lines {
                let y_at_clip_x0 = at_temp(*middle_limit, clip_x0, *cte);
                let y_at_clip_x1 = at_temp(*middle_limit, clip_x1, *cte);
                plot_ui.line(
                    Line::new(
                        "",
                        PlotPoints::Owned(vec![
                            PlotPoint::new(clip_x0, y_at_clip_x0),
                            PlotPoint::new(clip_x1, y_at_clip_x1),
                        ]),
                    )
                    .color(line_colour)
                    .style(LineStyle::dashed_loose()),
                );
            }

            // Temperature lines as vertical lines
            plot_ui.vline(
                egui_plot::VLine::new("lower_temp", x0)
                    .color(line_colour)
                    .style(LineStyle::dashed_dense()),
            );
            plot_ui.vline(
                egui_plot::VLine::new("upper_temp", x1)
                    .color(line_colour)
                    .style(LineStyle::dashed_dense()),
            );

            // Plot component intersection points (where hub and shaft lines cross)
            for (point, label) in component_intersection_points {
                plot_ui.points(
                    Points::new(&label, PlotPoints::Owned(vec![point]))
                        .shape(egui_plot::MarkerShape::Circle)
                        .radius(4.0)
                        .color(line_colour),
                );
            }

            // Plot temperature intersection points (each with its label for tooltip)
            for (point, label) in temp_intersection_points {
                plot_ui.points(
                    Points::new(&label, PlotPoints::Owned(vec![point]))
                        .shape(egui_plot::MarkerShape::Diamond)
                        .radius(4.0)
                        .color(line_colour),
                );
            }
        });

    // Clear the reset flag and update tracking values after applying
    if needs_reset {
        app.thermal.fit_plot_needs_reset = false;
        // Update tracking values so we don't reset again until data actually changes
        app.thermal.last_hub_id = Some(current_hub_id);
        app.thermal.last_shaft_id = Some(current_shaft_id);
        app.thermal.last_hub_upper = Some(current_hub_upper);
        app.thermal.last_hub_lower = Some(current_hub_lower);
        app.thermal.last_shaft_upper = Some(current_shaft_upper);
        app.thermal.last_shaft_lower = Some(current_shaft_lower);
        app.thermal.last_hub_cte = Some(current_hub_cte);
        app.thermal.last_shaft_cte = Some(current_shaft_cte);
    }

    // Reset bounds on double-click
    if response.response.double_clicked() {
        app.thermal.fit_plot_needs_reset = true;
    }
}
