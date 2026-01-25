use crate::Studio;
use crate::modules::{
    material::Material,
    utils::{at_temp, decimals_for_sig_figs, fix_dp, fix_sf},
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
    // Display toggles for component intersection points (where hub and shaft lines cross)
    pub show_component_limit_intersections: bool,
    pub show_component_mid_intersections: bool,
    // Display toggles for temperature intersection points (where temp lines cross component lines)
    pub show_temp_limit_intersections: bool,
    pub show_temp_mid_intersections: bool,
    // Component colours (with gamma pre-applied for plot fill)
    pub hub_colour: Color32,
    pub shaft_colour: Color32,
    pub colour_gamma: f32,
    #[serde(skip)]
    pub output_view: View,
    #[serde(skip)]
    pub temp_lines_view: View,
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
            upper: 120.0,
            lower: 20.0,
            hub_series: vec![20.0, 120.0],
            shaft_series: vec![20.0, 120.0],
            show_component_limit_intersections: true,
            show_component_mid_intersections: false,
            show_temp_limit_intersections: false,
            show_temp_mid_intersections: false,
            hub_colour: Color32::RED.gamma_multiply(0.2),
            shaft_colour: Color32::BLUE.gamma_multiply(0.2),
            colour_gamma: 0.2,
            output_view: View::new(),
            temp_lines_view: View::new(),
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
    let (t0, t1) = (-273.0, 1000.0);
    let line_colour = ui.visuals().text_color();
    let (Some(hub), Some(shaft)) = (app.library.get_hub(), app.library.get_shaft()) else {
        return;
    };

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
    let mut line_buffer = Vec::new();
    let mut segments: Vec<Segment> = Vec::new();

    let (x0, x1) = (app.thermal.lower, app.thermal.upper);
    let (mut y0, mut y1) = (f64::MAX, f64::MIN);

    let formatter = |name: &str, point: &PlotPoint| {
        let position = point.to_pos2();
        let temp = position.x;
        let size = position.y;
        let temp_dp = decimals_for_sig_figs(temp as f64, 3);
        let size_dp = decimals_for_sig_figs(size as f64, 4);
        if name.is_empty() {
            // format!("{size:.size_dp$} mm\n{temp:.temp_dp$}°C")
            format!("{} mm\n{}°C", fix_dp(position.y, 3), fix_dp(position.x, 2))
        } else {
            // For intersection tooltips, name contains component and limit info
            format!("{name}\n{size:.size_dp$} mm @ {temp:.temp_dp$}°C")
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

    // Build/rebuild the draggable temperature lines view
    // Rebuild if view is empty or if values changed externally (not from dragging)
    let y_lim = 10.0 * hub.outer_diameter.size;
    let y_mid = (y0 + y1) / 2.0; // Middle of visible range for the draggable handle
    let view_needs_rebuild = if app.thermal.temp_lines_view.len() != 2 {
        true
    } else if !app.thermal.temp_lines_view.is_dragging() {
        // Check if the view positions differ from current values (external change)
        let lower_mismatch = app
            .thermal
            .temp_lines_view
            .get(0)
            .and_then(|c| c.get_point("handle"))
            .map(|pt| (pt.x - x0).abs() > 0.001)
            .unwrap_or(true);
        let upper_mismatch = app
            .thermal
            .temp_lines_view
            .get(1)
            .and_then(|c| c.get_point("handle"))
            .map(|pt| (pt.x - x1).abs() > 0.001)
            .unwrap_or(true);
        lower_mismatch || upper_mismatch
    } else {
        false
    };

    if view_needs_rebuild {
        app.thermal.temp_lines_view = View::new();

        // Lower temperature line (x0) with draggable handle in visible range
        let lower_line = RedComponent::builder("lower_temp")
            .cull(false)
            .add_path()
            .named_point("lower_top", RedPoint::new(x0, y_lim))
            .named_point("handle", RedPoint::new(x0, y_mid))
            .named_point("lower_bottom", RedPoint::new(x0, 0.0))
            .constrain(
                "fix_lower_y_top",
                &["lower_top"],
                ConstraintSpec::FixedY { y: y_lim },
            )
            .constrain(
                "fix_handle_y",
                &["handle"],
                ConstraintSpec::FixedY { y: y_mid },
            )
            .constrain(
                "fix_lower_y_bottom",
                &["lower_bottom"],
                ConstraintSpec::FixedY { y: 0.0 },
            )
            .constrain(
                "sync_lower_x",
                &["lower_top", "handle", "lower_bottom"],
                ConstraintSpec::VerticalAlign,
            )
            .build();

        // Upper temperature line (x1) with draggable handle in visible range
        let upper_line = RedComponent::builder("upper_temp")
            .cull(false)
            .add_path()
            .named_point("upper_top", RedPoint::new(x1, y_lim))
            .named_point("handle", RedPoint::new(x1, y_mid))
            .named_point("upper_bottom", RedPoint::new(x1, 0.0))
            .constrain(
                "fix_upper_y_top",
                &["upper_top"],
                ConstraintSpec::FixedY { y: y_lim },
            )
            .constrain(
                "fix_handle_y",
                &["handle"],
                ConstraintSpec::FixedY { y: y_mid },
            )
            .constrain(
                "fix_upper_y_bottom",
                &["upper_bottom"],
                ConstraintSpec::FixedY { y: 0.0 },
            )
            .constrain(
                "sync_upper_x",
                &["upper_top", "handle", "upper_bottom"],
                ConstraintSpec::VerticalAlign,
            )
            .build();

        app.thermal.temp_lines_view.add(lower_line);
        app.thermal.temp_lines_view.add(upper_line);
    }

    // Find intersections between hub and shaft segments based on toggles
    // Segments are stored as: [hub_upper, hub_middle, hub_lower, shaft_upper, shaft_middle, shaft_lower]
    // Limit lines = upper and lower (indices 0, 2 for hub; 3, 5 for shaft)
    // Mid limit lines = middle (indices 1 for hub; 4 for shaft)
    // At temp lines are the vertical dashed lines at x0 and x1
    // For a hole (hub inner diameter): upper limit = max material, lower limit = min material
    // For a shaft (shaft outer diameter): upper limit = max material, lower limit = min material
    let mut component_intersection_points: Vec<(PlotPoint, String)> = Vec::new();
    let mut temp_intersection_points: Vec<(PlotPoint, String)> = Vec::new();

    let hub_name = &hub.name;
    let shaft_name = &shaft.name;

    // Helper to describe hub limit condition (hole: upper=max material, lower=min material)
    let hub_limit_desc = |idx: usize| -> &'static str {
        match idx {
            0 => "max material",   // upper limit
            2 => "least material", // lower limit
            _ => "mid-limit",
        }
    };
    // Helper to describe shaft limit condition (shaft: upper=max material, lower=min material)
    let shaft_limit_desc = |idx: usize| -> &'static str {
        match idx {
            3 => "max material",   // upper limit
            5 => "least material", // lower limit
            _ => "mid-limit",
        }
    };

    // Component limit line intersections (upper/lower vs upper/lower)
    if app.thermal.show_component_limit_intersections {
        for hub_idx in [0, 2] {
            for shaft_idx in [3, 5] {
                if let Some(pt) = segments[hub_idx].intersect(&segments[shaft_idx]) {
                    let hub_cond = hub_limit_desc(hub_idx);
                    let shaft_cond = shaft_limit_desc(shaft_idx);
                    let label = if hub_cond == shaft_cond {
                        // Both at same condition (both max or both min material)
                        format!("Both {hub_cond}")
                    } else {
                        format!("{hub_name} {hub_cond}\n{shaft_name} {shaft_cond}")
                    };
                    component_intersection_points.push((PlotPoint::new(pt.x, pt.y), label));
                }
            }
        }
    }

    // Component mid limit line intersections (middle vs middle, and middle vs limits)
    if app.thermal.show_component_mid_intersections {
        // Middle vs middle
        if let Some(pt) = segments[1].intersect(&segments[4]) {
            component_intersection_points
                .push((PlotPoint::new(pt.x, pt.y), "Both mid-limit".to_string()));
        }
        // Hub middle vs shaft limits
        for shaft_idx in [3, 5] {
            if let Some(pt) = segments[1].intersect(&segments[shaft_idx]) {
                let shaft_cond = shaft_limit_desc(shaft_idx);
                let label = format!("{hub_name} mid-limit\n{shaft_name} {shaft_cond}");
                component_intersection_points.push((PlotPoint::new(pt.x, pt.y), label));
            }
        }
        // Shaft middle vs hub limits
        for hub_idx in [0, 2] {
            if let Some(pt) = segments[hub_idx].intersect(&segments[4]) {
                let hub_cond = hub_limit_desc(hub_idx);
                let label = format!("{hub_name} {hub_cond}\n{shaft_name} mid-limit");
                component_intersection_points.push((PlotPoint::new(pt.x, pt.y), label));
            }
        }
    }

    // Temperature limit intersections (where vertical lines at x0/x1 cross limit lines)
    if app.thermal.show_temp_limit_intersections {
        // Hub upper limit (index 0)
        let seg = &segments[0];
        let y_at_x0 = seg.p1.y + (seg.p2.y - seg.p1.y) * (x0 - seg.p1.x) / (seg.p2.x - seg.p1.x);
        temp_intersection_points.push((
            PlotPoint::new(x0, y_at_x0),
            format!("{hub_name} upper limit"),
        ));
        let y_at_x1 = seg.p1.y + (seg.p2.y - seg.p1.y) * (x1 - seg.p1.x) / (seg.p2.x - seg.p1.x);
        temp_intersection_points.push((
            PlotPoint::new(x1, y_at_x1),
            format!("{hub_name} upper limit"),
        ));
        // Hub lower limit (index 2)
        let seg = &segments[2];
        let y_at_x0 = seg.p1.y + (seg.p2.y - seg.p1.y) * (x0 - seg.p1.x) / (seg.p2.x - seg.p1.x);
        temp_intersection_points.push((
            PlotPoint::new(x0, y_at_x0),
            format!("{hub_name} lower limit"),
        ));
        let y_at_x1 = seg.p1.y + (seg.p2.y - seg.p1.y) * (x1 - seg.p1.x) / (seg.p2.x - seg.p1.x);
        temp_intersection_points.push((
            PlotPoint::new(x1, y_at_x1),
            format!("{hub_name} lower limit"),
        ));
        // Shaft upper limit (index 3)
        let seg = &segments[3];
        let y_at_x0 = seg.p1.y + (seg.p2.y - seg.p1.y) * (x0 - seg.p1.x) / (seg.p2.x - seg.p1.x);
        temp_intersection_points.push((
            PlotPoint::new(x0, y_at_x0),
            format!("{shaft_name} upper limit"),
        ));
        let y_at_x1 = seg.p1.y + (seg.p2.y - seg.p1.y) * (x1 - seg.p1.x) / (seg.p2.x - seg.p1.x);
        temp_intersection_points.push((
            PlotPoint::new(x1, y_at_x1),
            format!("{shaft_name} upper limit"),
        ));
        // Shaft lower limit (index 5)
        let seg = &segments[5];
        let y_at_x0 = seg.p1.y + (seg.p2.y - seg.p1.y) * (x0 - seg.p1.x) / (seg.p2.x - seg.p1.x);
        temp_intersection_points.push((
            PlotPoint::new(x0, y_at_x0),
            format!("{shaft_name} lower limit"),
        ));
        let y_at_x1 = seg.p1.y + (seg.p2.y - seg.p1.y) * (x1 - seg.p1.x) / (seg.p2.x - seg.p1.x);
        temp_intersection_points.push((
            PlotPoint::new(x1, y_at_x1),
            format!("{shaft_name} lower limit"),
        ));
    }

    // Temperature mid-limit intersections (where vertical lines cross mid-limit lines)
    if app.thermal.show_temp_mid_intersections {
        // Hub mid-limit line (index 1)
        let seg = &segments[1];
        let y_at_x0 = seg.p1.y + (seg.p2.y - seg.p1.y) * (x0 - seg.p1.x) / (seg.p2.x - seg.p1.x);
        temp_intersection_points
            .push((PlotPoint::new(x0, y_at_x0), format!("{hub_name} mid-limit")));
        let y_at_x1 = seg.p1.y + (seg.p2.y - seg.p1.y) * (x1 - seg.p1.x) / (seg.p2.x - seg.p1.x);
        temp_intersection_points
            .push((PlotPoint::new(x1, y_at_x1), format!("{hub_name} mid-limit")));
        // Shaft mid-limit line (index 4)
        let seg = &segments[4];
        let y_at_x0 = seg.p1.y + (seg.p2.y - seg.p1.y) * (x0 - seg.p1.x) / (seg.p2.x - seg.p1.x);
        temp_intersection_points.push((
            PlotPoint::new(x0, y_at_x0),
            format!("{shaft_name} mid-limit"),
        ));
        let y_at_x1 = seg.p1.y + (seg.p2.y - seg.p1.y) * (x1 - seg.p1.x) / (seg.p2.x - seg.p1.x);
        temp_intersection_points.push((
            PlotPoint::new(x1, y_at_x1),
            format!("{shaft_name} mid-limit"),
        ));
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
        .allow_drag(!app.thermal.temp_lines_view.is_dragging())
        .show(ui, |plot_ui| {
            // Only set bounds on first render or explicit reset
            if needs_reset {
                plot_ui.set_plot_bounds(default_bounds);
            }

            shaded_buffer.into_iter().for_each(|p| plot_ui.polygon(p));
            line_buffer.into_iter().for_each(|l| plot_ui.line(l));

            // Render draggable temperature lines
            app.thermal.temp_lines_view.render_all_in(plot_ui);

            // Plot component intersection points (where hub and shaft lines cross)
            for (point, label) in component_intersection_points {
                plot_ui.points(
                    Points::new(&label, PlotPoints::Owned(vec![point]))
                        .shape(egui_plot::MarkerShape::Asterisk)
                        .radius(5.0)
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

    // Handle temperature line dragging interaction
    let drag_result = app.thermal.temp_lines_view.show_interaction(ui, &response);
    if drag_result.changed {
        // Update lower temperature from the lower line component's handle
        if let Some(lower_comp) = app.thermal.temp_lines_view.get(0) {
            if let Some(pt) = lower_comp.get_point("handle") {
                app.thermal.lower = pt.x;
            }
        }
        // Update upper temperature from the upper line component's handle
        if let Some(upper_comp) = app.thermal.temp_lines_view.get(1) {
            if let Some(pt) = upper_comp.get_point("handle") {
                app.thermal.upper = pt.x;
            }
        }
        ui.ctx().request_repaint();
    }

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
