use super::{feature::Feature, state::State, utils::lerp_untimed};
use egui::Ui;

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Focus {
    Inner,
    Outer,
    Material,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Component {
    pub name: String,
    pub inner_diameter: Feature,
    pub outer_diameter: Feature,
    pub material_id: usize,
    pub focus: Focus,
}

impl Component {
    pub fn default_hub() -> Self {
        Component {
            name: "Hub".to_owned(),
            inner_diameter: Feature::default_hole(),
            outer_diameter: Feature::default_outer(),
            material_id: 1, // PB104 (index 1 in default material_list)
            focus: Focus::Inner,
        }
    }

    pub fn default_shaft() -> Self {
        Component {
            name: "Shaft".to_owned(),
            inner_diameter: Feature::default_inner(),
            outer_diameter: Feature::default_shaft(),
            material_id: 0, // Steel 4340 (index 0 in default material_list)
            focus: Focus::Outer,
        }
    }

    /// Not thrilled with this clone()
    pub fn primary_feature(&self) -> Feature {
        if self.inner_diameter.primary {
            self.inner_diameter.clone()
        } else {
            self.outer_diameter.clone()
        }
    }

    /// Handle automatic size synchronization
    pub fn handle_sync(&mut self, state: State) {
        if state.sync_size {
            if self.inner_diameter.primary {
                self.inner_diameter.size = state.synced_size;
            } else {
                self.outer_diameter.size = state.synced_size;
            }
        }
    }

    /// Handle automatic outer diameter scaling when not enabled
    pub fn handle_auto_scale(&mut self, ui: &Ui) {
        if !self.outer_diameter.enabled {
            let target = (1.8 * self.inner_diameter.size).max(1.0);

            if self.outer_diameter.size != target {
                let rate = 0.1;
                let tolerance = (0.005 * self.inner_diameter.size).max(0.01);

                if let Some(size) = lerp_untimed(self.outer_diameter.size, target, rate, tolerance)
                {
                    self.outer_diameter.size = size;
                    ui.ctx().request_repaint();
                }
            }
        }
    }

    // pub fn view(&self, ui: &mut Ui) {
    //     let outline_colour = if ui.visuals().dark_mode {
    //         egui::Color32::LIGHT_GRAY
    //     } else {
    //         egui::Color32::DARK_GRAY
    //     };
    //     let centre = [0.0; 2];
    //     let scale = 1.0;

    //     Frame::group(ui.style())
    //         .inner_margin(10.0)
    //         .rounding(10.0)
    //         .show(ui, |ui| {
    //             ui.set_min_size(vec2(200.0, 200.0));
    //             ui.set_max_size(vec2(200.0, 200.0));

    //             Plot::new(&self.name)
    //                 .data_aspect(1.0)
    //                 .include_x(20.0)
    //                 .show_axes(false)
    //                 .show_grid(false)
    //                 .show_background(false)
    //                 .show_x(false)
    //                 .show_y(false)
    //                 .allow_drag(false)
    //                 .allow_zoom(false)
    //                 .allow_scroll(false)
    //                 .allow_boxed_zoom(false)
    //                 .show(ui, |ui| {
    //                     // Outer circle
    //                     plot_circle(
    //                         ui,
    //                         centre,
    //                         self.outer_diameter.size,
    //                         1.0,
    //                         outline_colour,
    //                         Color32::TRANSPARENT,
    //                     );

    //                     // Inner circle
    //                     plot_circle(
    //                         ui,
    //                         centre,
    //                         self.inner_diameter.size,
    //                         1.0,
    //                         outline_colour,
    //                         Color32::TRANSPARENT,
    //                     );

    //                     // Centre mark
    //                     plot_centre_mark(
    //                         ui,
    //                         centre,
    //                         self.outer_diameter.size,
    //                         0.0,
    //                         1.0,
    //                         outline_colour,
    //                     );

    //                     // Outer diameter dimension
    //                     plot_diameter_limits(
    //                         ui,
    //                         &self.outer_diameter,
    //                         45.0,
    //                         5.0,
    //                         scale,
    //                         1.5,
    //                         outline_colour,
    //                     );

    //                     // Inner diameter dimension
    //                     plot_diameter_limits(
    //                         ui,
    //                         &self.inner_diameter,
    //                         -45.0,
    //                         5.0,
    //                         scale,
    //                         1.5,
    //                         outline_colour,
    //                     );
    //                 });
    //         });
    // }
}
