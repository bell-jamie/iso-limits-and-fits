use super::{feature::Feature, fit::Fit, utils::decimals};
use egui::{Align2, Color32, RichText, Stroke};
use egui_plot::{Line, LineStyle, PlotPoint, PlotPoints, Polygon, Text};
use std::f64::consts::{PI, TAU};

// Must be even
const RESOLUTION: usize = 1_000;

pub struct VisualFit {
    pub scale: f64,
    pub display: bool,
}

// Generate lmc on the left, mmc on the right
// Semiannular shapes
// Line down the middle separating?
// Scaling from anchor dimension

// Linear fit view
// Needs to somehow scale dynamically... Maybe this is automatically handled by the graph scaling?
//

impl VisualFit {
    pub fn default() -> Self {
        Self {
            scale: 1.0,
            display: false,
        }
    }

    pub fn show(&self, ui: &mut egui::Ui, fit: &Fit, id: &str) {
        let outline_colour = if ui.visuals().dark_mode {
            egui::Color32::LIGHT_GRAY
        } else {
            egui::Color32::DARK_GRAY
        };

        let zero_line = -fit.lower / (fit.upper - fit.lower); // This can fall off the bottom at the moment
        let text_lmc = format!("{:.} µm", decimals(1000.0 * fit.upper, -1));
        let text_mmc = format!("{:.} µm", decimals(1000.0 * fit.lower, -1));
        let text_mid = format!("{:.} µm", decimals(1000.0 * fit.target, -1));

        egui::Frame::none().show(ui, |ui| {
            ui.set_min_size(egui::vec2(200.0, 300.0)); // Set width to 400 and height to 300
            ui.set_max_size(egui::vec2(200.0, 300.0));

            egui_plot::Plot::new([id, "visual"].concat())
                .view_aspect(0.5)
                .show_axes(false)
                .show_grid(false)
                .show_x(false)
                .show_y(false)
                .allow_drag(false)
                .allow_zoom(false)
                .allow_scroll(false)
                .include_x(1.5)
                .show_background(false)
                .show(ui, |plot_ui| {
                    for shape in zones(fit, zero_line) {
                        plot_ui.polygon(shape);
                    }
                    for line in scales(outline_colour, zero_line) {
                        plot_ui.line(line);
                    }

                    // Instead of plotting polygons, lines and text separately
                    // i should send &mut plot_ui to functions and plot multiple things
                    // together
                    // This would mean that the zero line can have 0.0 sandwiched in the middle
                    // of two lines for instance, and text will be easier to add in general
                    // There can then be standalone functions for the individual elements

                    plot_ui.text(
                        Text::new(PlotPoint::new(1.025, 1.0), RichText::new(text_lmc).strong())
                            .anchor(Align2::LEFT_CENTER),
                    );
                    plot_ui.text(
                        Text::new(PlotPoint::new(1.025, 0.5), RichText::new(text_mid).strong())
                            .anchor(Align2::LEFT_CENTER),
                    );
                    plot_ui.text(
                        Text::new(PlotPoint::new(1.025, 0.0), RichText::new(text_mmc).strong())
                            .anchor(Align2::LEFT_CENTER),
                    );
                });
        });
    }
}

fn scales(colour: Color32, zero: f64) -> Vec<Line> {
    let mut lines = Vec::new();

    // Right scale
    lines.push(Line::new(PlotPoints::from(vec![[1.0, 1.0], [1.0, 0.0]])).color(colour));
    lines.push(Line::new(PlotPoints::from(vec![[0.9, 1.0], [1.0, 1.0]])).color(colour));
    lines.push(Line::new(PlotPoints::from(vec![[0.9, 0.0], [1.0, 0.0]])).color(colour));
    lines.push(Line::new(PlotPoints::from(vec![[0.95, 0.5], [1.0, 0.5]])).color(colour));

    // Zero line
    if zero >= 0.0 && zero <= 1.0 {
        lines.push(
            Line::new(PlotPoints::from(vec![[0.0, zero], [1.0, zero]]))
                .color(colour)
                .style(LineStyle::dashed_dense()),
        );
    }

    // Left scale
    lines.push(Line::new(PlotPoints::from(vec![[0.0, 1.0], [0.0, 0.0]])).color(colour));
    lines.push(Line::new(PlotPoints::from(vec![[0.1, 1.0], [0.0, 1.0]])).color(colour));
    lines.push(Line::new(PlotPoints::from(vec![[0.1, 0.0], [0.0, 0.0]])).color(colour));
    lines.push(Line::new(PlotPoints::from(vec![[0.05, 0.5], [0.0, 0.5]])).color(colour));

    lines
}

fn zones(fit: &Fit, zero: f64) -> Vec<Polygon> {
    fn clearance(zones: &mut Vec<Polygon>, split: f64, pad: (f64, f64, f64)) {
        if 1.0 - split >= pad.1 + pad.2 {
            zones.push(
                Polygon::new(PlotPoints::from(vec![
                    [pad.0, 1.0 - pad.1],
                    [1.0 - pad.0, 1.0 - pad.1],
                    [
                        1.0 - pad.0,
                        (split + pad.2).clamp(pad.1 + pad.2, 1.0 - pad.1 - pad.2),
                    ],
                    [
                        pad.0,
                        (split + pad.2).clamp(pad.1 + pad.2, 1.0 - pad.1 - pad.2),
                    ],
                    [pad.0, 1.0 - pad.1],
                ]))
                .fill_color(Color32::from_rgb(0, 0, 255).gamma_multiply(0.7))
                .stroke(Stroke {
                    width: 0.0,
                    color: Color32::DARK_BLUE,
                }),
            );
        }
    }

    fn interference(zones: &mut Vec<Polygon>, split: f64, pad: (f64, f64, f64)) {
        if split >= pad.1 + pad.2 {
            zones.push(
                Polygon::new(PlotPoints::from(vec![
                    [pad.0, pad.1],
                    [1.0 - pad.0, pad.1],
                    [
                        1.0 - pad.0,
                        (split - pad.2).clamp(pad.1 + pad.2, 1.0 - pad.1 - pad.2),
                    ],
                    [
                        pad.0,
                        (split - pad.2).clamp(pad.1 + pad.2, 1.0 - pad.1 - pad.2),
                    ],
                    [pad.0, pad.1],
                ]))
                .fill_color(Color32::from_rgb(255, 0, 0).gamma_multiply(0.7))
                .stroke(Stroke {
                    width: 0.0,
                    color: Color32::DARK_RED,
                }),
            );
        }
    }

    let mut zones = Vec::new();
    let pad = (0.075, 0.025, 0.015);

    match fit.kind.as_str() {
        "Clearance" => clearance(&mut zones, zero, pad),
        "Interference" => interference(&mut zones, zero, pad),
        "Transition" => {
            clearance(&mut zones, zero, pad);
            interference(&mut zones, zero, pad);
        }
        _ => (),
    }

    zones
}

fn generate_polygons(fit: &Fit, outline: Color32, scale: f64) -> Vec<Polygon> {
    fn apply_scale(diameter: f64, anchor: f64, scale: f64) -> f64 {
        diameter * (scale * (diameter - anchor) - 1.0)
    }

    fn quick_semicircle(diameter: f64, color: Color32, outline: Color32, angle: f64) -> Polygon {
        Polygon::new(PlotPoints::from(generate_semicircle(
            (0.0, 0.0),
            diameter,
            angle,
        )))
        .fill_color(color)
        .stroke(Stroke {
            width: 1.0,
            color: outline,
        })
    }

    let mut polygons = Vec::new();
    let od = 1.5 * fit.hole.size.basic;
    let mut od_lmc = fit.hole.size.basic + fit.hole.tolerance.upper * scale;
    let mut od_mmc = fit.hole.size.basic + fit.hole.tolerance.lower * scale;
    let mut id_lmc = fit.shaft.size.basic + fit.shaft.tolerance.lower * scale;
    let mut id_mmc = fit.shaft.size.basic + fit.shaft.tolerance.upper * scale;
    // let (fill_lmc, fill_mmc) = if fit.lower >= 0.0 {
    //     (Color32::BLUE, Color32::RED)
    // } else {
    //     (Color32::RED, Color32::BLUE)
    // };

    let (mut fill_lmc, mut fill_mmc) = (Color32::BLACK, Color32::BLACK);

    if fit.hole.size.upper - fit.shaft.size.lower >= 0.0 {
        fill_lmc = Color32::BLUE;
    } else {
        fill_lmc = Color32::RED;
        std::mem::swap(&mut od_lmc, &mut id_lmc);
    }

    if fit.hole.size.lower - fit.shaft.size.upper >= 0.0 {
        fill_mmc = Color32::BLUE;
    } else {
        fill_mmc = Color32::RED;
        std::mem::swap(&mut od_mmc, &mut id_mmc);
    }

    // Generate lmc hole
    polygons.push(quick_semicircle(od, Color32::GRAY, outline, 90.0));
    polygons.push(quick_semicircle(od_lmc, fill_lmc, outline, 90.0));
    polygons.push(quick_semicircle(id_lmc, Color32::GRAY, outline, 90.0));

    // Generate mmc hole
    polygons.push(quick_semicircle(od, Color32::GRAY, outline, 270.0));
    polygons.push(quick_semicircle(od_mmc, fill_mmc, outline, 270.0));
    polygons.push(quick_semicircle(id_mmc, Color32::GRAY, outline, 270.0));

    polygons
}

// Auto generates on x axis, angle value rotates according to polar coords
fn generate_semi_annulus(
    centre: (f64, f64),
    diameter_inner: f64,
    diameter_outer: f64,
    angle: f64,
) -> Vec<[f64; 2]> {
    let mut points = Vec::with_capacity(RESOLUTION + 1);
    let angle_rad = angle.to_radians();
    let angle_delta = TAU / (RESOLUTION as f64 - 1.0);
    let radius_inner = diameter_inner / 2.0;
    let radius_outer = diameter_outer / 2.0;

    // Outer arc
    points.extend((0..RESOLUTION / 2).map(|i| {
        let theta = angle_rad + i as f64 * angle_delta;
        let x = centre.0 + radius_outer * theta.cos();
        let y = centre.1 + radius_outer * theta.sin();
        [x, y]
    }));

    // Inner arc (reversed)
    points.extend((0..RESOLUTION / 2).rev().map(|i| {
        let theta = angle_rad + i as f64 * angle_delta;
        let x = centre.0 + radius_inner * theta.cos();
        let y = centre.1 + radius_inner * theta.sin();
        [x, y]
    }));

    // Connect arcs
    points.push(points[0]);
    points
}

fn generate_semicircle(centre: (f64, f64), diameter: f64, angle: f64) -> Vec<[f64; 2]> {
    let mut points = Vec::with_capacity(RESOLUTION + 1);
    let radius = diameter / 2.0;
    let angle_rad = angle.to_radians();
    let angle_delta = PI / (RESOLUTION as f64 - 1.0);

    points.extend((0..RESOLUTION).map(|i| {
        let theta = angle_rad + i as f64 * angle_delta;
        let x = centre.0 + radius * theta.cos();
        let y = centre.1 + radius * theta.sin();
        [x, y]
    }));

    points.push(points[0]);
    points
}

fn generate_circle(centre: (f64, f64), diameter: f64) -> Vec<[f64; 2]> {
    let radius = diameter / 2.0;
    let mut points = Vec::with_capacity(RESOLUTION);

    for i in 0..RESOLUTION {
        let theta = 2.0 * std::f64::consts::PI * (i as f64 / RESOLUTION as f64);
        let x = centre.0 + radius * theta.cos();
        let y = centre.1 + radius * theta.sin();
        points.push([x, y]);
    }

    points
}
