use egui::Color32;

use crate::sections::{fit::Fit, input::Input};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
// if we add new fields, give them default values when deserializing old state

pub struct LimitsFitsApp {
    #[serde(skip)]
    hole_input: Input,
    #[serde(skip)]
    shaft_input: Input,
    #[serde(skip)]
    fit: Fit,
}

impl Default for LimitsFitsApp {
    fn default() -> Self {
        Self {
            hole_input: Input::default_hole(),
            shaft_input: Input::default_shaft(),
            fit: Fit::default(),
        }
    }
}

impl LimitsFitsApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for LimitsFitsApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("ISO Limits and Fits Tool");

            // ----------------------------------------------------------------------------

            ui.label(egui::RichText::new("Inputs").strong().underline());
            ui.add_space(5.0);

            let hole_option = self.hole_input.show(ui, true, "hole_input");
            let shaft_option = self.shaft_input.show(ui, false, "shaft_input");
            // if ui.toggle_value(&mut self.standard, "Lock size").clicked() sync button?

            ui.separator();

            if let (Some(hole), Some(shaft)) = (hole_option, shaft_option) {
                let fit = Fit::new(&hole, &shaft);
                fit.show(ui, "fit_results");

                hole.size.show(ui, "hole_size", "Hole");
                shaft.size.show(ui, "shaft_size", "Shaft");

                //     // ----------------------------------------------------------------------------

                //     ui.separator();

                //     ui.label(egui::RichText::new("Visualisation").strong().underline());
                //     ui.add_space(5.0);

                //     egui::Grid::new("visuals").show(ui, |ui| {
                //         ui.label("Plot scale ");
                //         ui.add(
                //             egui::DragValue::new(&mut self.visual.scale)
                //                 .range(RangeInclusive::new(1.0, 1000.0)),
                //         );
                //         ui.label(":1");
                //     });

                //     // The concept is that we centre on 0,0 moving upwards in y
                //     // There are black lines for the lower and upper sizes of the features
                //     // Between these black lines are shaded (blue and red) areas representing the tolerance
                //     // When (if) these overlap, the engine renders the overlapping colour
                //     // Always draw the black lines on top to not be affected by the shading
                //     // Might want a cross hatching function
                //     // Also need to decide a standard aspect ratio for the feature (this can change in the future)

                //     // YOU WERE HERE!!!
                //     // You wanted to start splitting into modules from the app.rs file
                //     // This was so that you could start putting the visualisation next to the current outputs
                //     //
                //     // The problem with the current implementation of scaling is that both fits just automatically overlap
                //     // There needs to be a preservation of the relationship between the gaps at LMC and MMC

                //     // Global vis variables

                //     let nominal_width = self.core.basic_size;
                //     let height = 1.5 * nominal_width;
                //     let outer_width = nominal_width;
                //     let outline_color = if ui.visuals().dark_mode {
                //         egui::Color32::WHITE
                //     } else {
                //         egui::Color32::BLACK
                //     };

                //     // Create geometry for the hole

                //     let wall_outer = (result.hole.size.mid_limits
                //         - self.visual.scale * result.hole.tolerance.mid_limits)
                //         / 2.0;
                //     let wall_inner = (result.hole.size.mid_limits
                //         + self.visual.scale * result.hole.tolerance.mid_limits)
                //         / 2.0;

                //     let left_hole_wall_points = vec![
                //         [-outer_width, height],
                //         [-wall_outer, height],
                //         [-wall_outer, 0.0],
                //         [-outer_width, 0.0],
                //         [-outer_width, height],
                //     ];

                //     let left_hole_wall_points_tol = vec![
                //         [-wall_outer, height],
                //         [-wall_inner, height],
                //         [-wall_inner, 0.0],
                //         [-wall_outer, 0.0],
                //         [-wall_outer, height],
                //     ];

                //     let right_hole_wall_points = vec![
                //         [outer_width, height],
                //         [wall_outer, height],
                //         [wall_outer, 0.0],
                //         [outer_width, 0.0],
                //         [outer_width, height],
                //     ];

                //     let right_hole_wall_points_tol = vec![
                //         [wall_outer, height],
                //         [wall_inner, height],
                //         [wall_inner, 0.0],
                //         [wall_outer, 0.0],
                //         [wall_outer, height],
                //     ];

                //     let left_wall_outer_poly =
                //         egui_plot::Polygon::new(egui_plot::PlotPoints::from(left_hole_wall_points))
                //             .stroke(egui::Stroke {
                //                 width: 1.0,
                //                 color: outline_color,
                //             })
                //             .name("left_hole_solid");

                //     let left_wall_inner_poly =
                //         egui_plot::Polygon::new(egui_plot::PlotPoints::from(left_hole_wall_points_tol))
                //             .fill_color(egui::Color32::RED.linear_multiply(0.5))
                //             .stroke(egui::Stroke {
                //                 width: 1.0,
                //                 color: outline_color,
                //             })
                //             .name("left_hole_trans");

                //     let right_wall_outer_poly =
                //         egui_plot::Polygon::new(egui_plot::PlotPoints::from(right_hole_wall_points))
                //             .stroke(egui::Stroke {
                //                 width: 1.0,
                //                 color: outline_color,
                //             })
                //             .name("right_hole_solid");

                //     let right_wall_inner_poly = egui_plot::Polygon::new(egui_plot::PlotPoints::from(
                //         right_hole_wall_points_tol,
                //     ))
                //     .fill_color(egui::Color32::RED.linear_multiply(0.5))
                //     .stroke(egui::Stroke {
                //         width: 1.0,
                //         color: outline_color,
                //     })
                //     .name("right_hole_trans");

                //     // Create geometry for the shaft

                //     let shaft_wall_outer = (result.shaft.size.mid_limits
                //         + self.visual.scale * result.shaft.tolerance.mid_limits)
                //         / 2.0;
                //     let shaft_wall_inner = (result.shaft.size.mid_limits
                //         - self.visual.scale * result.shaft.tolerance.mid_limits)
                //         / 2.0;

                //     let shaft_centre_points = vec![
                //         [-shaft_wall_inner, height],
                //         [shaft_wall_inner, height],
                //         [shaft_wall_inner, 0.0],
                //         [-shaft_wall_inner, 0.0],
                //         [-shaft_wall_inner, height],
                //     ];

                //     let left_shaft_outer_points = vec![
                //         [-shaft_wall_outer, height],
                //         [-shaft_wall_inner, height],
                //         [-shaft_wall_inner, 0.0],
                //         [-shaft_wall_outer, 0.0],
                //         [-shaft_wall_outer, height],
                //     ];

                //     let right_shaft_outer_points = vec![
                //         [shaft_wall_inner, height],
                //         [shaft_wall_outer, height],
                //         [shaft_wall_outer, 0.0],
                //         [shaft_wall_inner, 0.0],
                //         [shaft_wall_inner, height],
                //     ];

                //     let shaft_centre_poly =
                //         egui_plot::Polygon::new(egui_plot::PlotPoints::from(shaft_centre_points))
                //             .stroke(egui::Stroke {
                //                 width: 1.0,
                //                 color: outline_color,
                //             })
                //             .name("shaft_solid");

                //     let left_shaft_poly =
                //         egui_plot::Polygon::new(egui_plot::PlotPoints::from(left_shaft_outer_points))
                //             .fill_color(egui::Color32::BLUE.linear_multiply(0.5))
                //             .stroke(egui::Stroke {
                //                 width: 1.0,
                //                 color: outline_color,
                //             })
                //             .name("left_shaft_trans");

                //     let right_shaft_poly =
                //         egui_plot::Polygon::new(egui_plot::PlotPoints::from(right_shaft_outer_points))
                //             .fill_color(egui::Color32::BLUE.linear_multiply(0.5))
                //             .stroke(egui::Stroke {
                //                 width: 1.0,
                //                 color: outline_color,
                //             })
                //             .name("right_shaft_trans");

                //     egui::Frame::none().show(ui, |ui| {
                //         egui_plot::Plot::new("visualisation")
                //             .view_aspect(1.0)
                //             .show_axes(false)
                //             .show_grid(false)
                //             .show_background(false)
                //             .show(ui, |plot_ui| {
                //                 plot_ui.polygon(left_wall_outer_poly);
                //                 plot_ui.polygon(left_wall_inner_poly);
                //                 plot_ui.polygon(right_wall_outer_poly);
                //                 plot_ui.polygon(right_wall_inner_poly);
                //                 plot_ui.polygon(shaft_centre_poly);
                //                 plot_ui.polygon(left_shaft_poly);
                //                 plot_ui.polygon(right_shaft_poly);
                //             });
                //     });

                //     // ----------------------------------------------------------------------------

                //     ui.separator();

                //     ui.label(egui::RichText::new("Thermals").strong().underline());
                //     ui.add_space(5.0);

                //     egui::Grid::new("thermals").show(ui, |ui| {
                //         ui.label("Hole CTE: ");
                //         ui.add(egui::DragValue::new(&mut self.thermal.hole_cte).speed(0.1));
                //         ui.end_row();

                //         ui.label("Shaft CTE: ");
                //         ui.add(egui::DragValue::new(&mut self.thermal.shaft_cte).speed(0.1));
                //         ui.end_row();

                //         ui.label("Ambient Temperature: ");
                //         ui.add(egui::DragValue::new(&mut self.thermal.ambient).speed(0.1));
                //         ui.end_row();

                //         ui.label("Operating Temperature: ");
                //         ui.add(egui::DragValue::new(&mut self.thermal.operating).speed(0.1));
                //     });

                //     fn linspace(a: f64, b: f64, n: usize) -> Vec<f64> {
                //         (0..n)
                //             .map(|i| {
                //                 let t = i as f64 / (n as f64 - 1.0);
                //                 a + t * (b - a)
                //             })
                //             .collect()
                //     }

                //     fn calculate_thermal_points(result: &Result, thermal: &Thermal) -> Vec<[f64; 2]> {
                //         const RESOLUTION: usize = 1_000;
                //         let delta_temp = thermal.operating - thermal.ambient;
                //         let hole_factor = 1.0 + (delta_temp * thermal.hole_cte / 10_000.0);
                //         let shaft_factor = 1.0 + (delta_temp * thermal.shaft_cte / 10_000.0);
                //         let thermal_fit_lmc = hole_factor * result.hole.size.upper
                //             - shaft_factor * result.shaft.size.lower;
                //         let thermal_fit_mmc = hole_factor * result.hole.size.lower
                //             - shaft_factor * result.shaft.size.upper;
                //         // let thermal_fit_vec = linspace(thermal_fit_lmc, thermal_fit_mmc, RESOLUTION);

                //         let fit_vec = linspace(thermal_fit_lmc, thermal_fit_mmc, RESOLUTION);
                //         let temp_vec = linspace(thermal.ambient, thermal.operating, RESOLUTION);

                //         temp_vec
                //             .into_iter()
                //             .zip(fit_vec.into_iter())
                //             .map(|(f, t)| [f, t])
                //             .collect::<Vec<_>>()
                //     }

                //     let thermal_fit_line =
                //         egui_plot::Line::new(calculate_thermal_points(result, &self.thermal));

                //     egui::Frame::none().show(ui, |ui| {
                //         egui_plot::Plot::new("thermal_plot")
                //             .show(ui, |plot_ui| plot_ui.line(thermal_fit_line))
                //     });

                //     // ----------------------------------------------------------------------------
                // } else {
                //     ui.label(format!("No Results!"));
            }

            ui.separator();

            // ----------------------------------------------------------------------------

            // ----------------------------------------------------------------------------

            // let rectangle_points = vec![
            //     [0.0, 0.0], // Bottom-left corner
            //     [0.0, 2.0], // Top-left corner
            //     [3.0, 2.0], // Top-right corner
            //     [3.0, 0.0], // Bottom-right corner
            //     [0.0, 0.0], // Closing the rectangle (back to the bottom-left corner)
            // ];

            // // Create a line from the points to form the rectangle
            // let rectangle_line = egui_plot::Line::new(rectangle_points);

            // egui::Frame::none()
            //     // .fixed_size(egui::Vec2::new(300.0, 200.0)) // Set the size of the frame
            //     .show(ui, |ui| {
            //         egui_plot::Plot::new("rectangle_plot")
            //             // .allow_interactions(false) // Disable interaction for a static plot
            //             .allow_drag(false)
            //             .allow_zoom(false)
            //             .allow_scroll(false)
            //             .show_x(false)
            //             .show_y(false)
            //             .show(ui, |plot_ui| {
            //                 plot_ui.line(rectangle_line);
            //             });
            //     });

            // ----------------------------------------------------------------------------

            // ui.add(egui::github_link_file!(
            //     "https://github.com/bell-jamie/iso-limits-and-fits",
            //     "Source code."
            // ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        let version_colour = Color32::from_rgb(0, 169, 0);
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(". Created by ");
        ui.hyperlink_to("James Bell", "https://www.linkedin.com/in/bell-jamie/");
        ui.label(".");
        ui.colored_label(version_colour, " v");
        ui.colored_label(version_colour, env!("CARGO_PKG_VERSION"));
        ui.colored_label(version_colour, " alpha");
    });
}
