use egui::{emath::vec2, Button, Color32, DragValue, Frame, Modal, Slider, Ui};
use std::{cmp::Ordering, collections::BTreeSet};

use super::{
    component::Component,
    plot,
    utils::{self, dynamic_precision, State},
};

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Material {
    pub name: String,
    pub temp: f64,
    pub cte: f64,
    pub poissons: f64,
    pub youngs: f64,
    pub ys: f64,
    pub uts: f64,
}

/// This is all required to use the BTreeSet to store materials
/// Only sorts based on the name String for now
impl PartialOrd for Material {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for Material {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialEq for Material {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Material {}

impl Material {
    pub fn steel4340() -> Self {
        Material {
            name: "4340 Steel - Annealed".to_owned(),
            temp: 20.0,
            cte: 12.3,
            poissons: 0.30,
            youngs: 129_000.0,
            ys: 470.0,
            uts: 745.0,
        }
    }

    pub fn pb104() -> Self {
        Material {
            name: "Phosphor Bronze — PB104".to_owned(),
            temp: 20.0,
            cte: 17.0,
            poissons: 0.34,
            youngs: 105_000.0,
            ys: 360.0,
            uts: 500.0,
        }
    }

    // pub fn aluminium() -> Self {
    //     Material {
    //         temp: 20.0,
    //         cte: 23.5,
    //         poissons: 0.34,
    //         youngs: 69_000.0,
    //         ys: 260.0,
    //         uts: 500.0,
    //     }
    // }

    pub fn input(&mut self, ui: &mut Ui, materials: &mut BTreeSet<Material>, id: &str) {
        let drag_width = 61.0;
        let id = ui.make_persistent_id(format!("{id}-material_listing"));

        ui.add_space(5.0);

        // Calculate input width
        let material_name_input_width =
            ui.min_rect().width() - 2.0 * ui.style().spacing.item_spacing.x - 21.0;

        let material_name_input = egui::TextEdit::singleline(&mut self.name)
            .desired_width(material_name_input_width)
            .background_color(ui.visuals().widgets.inactive.bg_fill);

        let material_save_button = Button::new("💾");

        // Create input field and save button
        let (save_button, name_input) = ui
            .horizontal(|ui| (ui.add(material_save_button), ui.add(material_name_input)))
            .inner;

        ui.add_space(5.0);

        if save_button.clicked() {
            // let error_message = Modal::new(egui::Id::new("material_exists")).show(|ui| {
            //     ui.vertical_centered(|ui| {
            //         ui.heading("Error");
            //         ui.label("This material already exists.");
            //         if ui.button("OK").clicked() {
            //             modal.close();
            //         }
            //     })
            // });
            // if materials.contains(self) {

            // } else {
            //     materials.insert(self.clone());
            // }

            materials.insert(self.clone());
        }

        // Open popup when the name input is focused
        if name_input.has_focus() && !ui.memory(|mem| mem.is_popup_open(id)) {
            ui.memory_mut(|mem| mem.open_popup(id));
        }

        egui::popup::popup_below_widget(
            ui,
            id,
            &name_input,
            egui::containers::popup::PopupCloseBehavior::CloseOnClickOutside,
            |ui| {
                egui::ScrollArea::vertical()
                    .min_scrolled_height(100.0)
                    .show(ui, |ui| {
                        // ui.set_min_height(60.0);
                        let mut to_remove = None;

                        for material in materials.iter() {
                            let (delete_button, material_listing) = ui
                                .horizontal(|ui| {
                                    (
                                        ui.add(Button::new("🗑")),
                                        ui.add(
                                            // [material_name_input_width, 18.0],
                                            Button::new(material.name.clone()),
                                        ),
                                    )
                                })
                                .inner;

                            if material_listing.clicked() {
                                *self = material.clone();
                                ui.memory_mut(|mem| mem.close_popup()); // close when selected
                            }

                            if delete_button.clicked() {
                                to_remove = Some(material.clone());
                            }
                        }

                        // Material has to be deleted outside iter method
                        if let Some(material) = to_remove {
                            materials.remove(&material);
                        }
                    })
            },
        );

        egui::Grid::new(id).striped(false).show(ui, |ui| {
            ui.label("Youngs");
            ui.add_sized(
                [drag_width, 18.0],
                DragValue::new(&mut self.youngs)
                    .custom_formatter(|mut youngs, _| {
                        youngs /= 1_000.0; // MPa -> GPa
                        let precision = dynamic_precision(youngs, 2);
                        format!("{youngs:.precision$} GPa")
                    })
                    .custom_parser(|youngs| {
                        let to_parse = youngs
                            .chars()
                            .filter(|c| c.is_ascii_digit() || c == &'.')
                            .collect::<String>();

                        if let Ok(parsed_value) = to_parse.parse::<f64>() {
                            Some(parsed_value * 1_000.0)
                        } else {
                            None
                        }
                    })
                    .speed(100.0)
                    .range(0.0..=999_000.0),
            )
            .on_hover_text("Young's modulus");

            ui.label("Poissons");
            ui.add_sized(
                [drag_width, 18.0],
                DragValue::new(&mut self.poissons)
                    .custom_formatter(|poissons, _| {
                        let precision = dynamic_precision(poissons, 2);
                        format!("{poissons:.precision$}")
                    })
                    .speed(0.01)
                    .range(0.0..=1.0),
            )
            .on_hover_text("Poisson's ratio");
            ui.end_row();

            ui.label("UTS");
            ui.add_sized(
                [drag_width, 18.0],
                DragValue::new(&mut self.uts)
                    .custom_formatter(|uts, _| {
                        let precision = dynamic_precision(uts, 2);
                        format!("{uts:.precision$} MPa")
                    })
                    .custom_parser(|uts| {
                        let to_parse = uts
                            .chars()
                            .filter(|c| c.is_ascii_digit() || c == &'.')
                            .collect::<String>();
                        to_parse.parse::<f64>().ok()
                    })
                    .speed(1.0)
                    .range(self.ys..=9_999.0),
            )
            .on_hover_text("Ultimate tensile strength");

            ui.label("Yield");
            ui.add_sized(
                [drag_width, 18.0],
                DragValue::new(&mut self.ys)
                    .custom_formatter(|ys, _| {
                        let precision = dynamic_precision(ys, 2);
                        format!("{ys:.precision$} MPa")
                    })
                    .custom_parser(|ys| {
                        let to_parse = ys
                            .chars()
                            .filter(|c| c.is_ascii_digit() || c == &'.')
                            .collect::<String>();
                        to_parse.parse::<f64>().ok()
                    })
                    .speed(1.0)
                    .range(0.0..=self.uts),
            )
            .on_hover_text("Yield strength");
            ui.end_row();
            ui.label("CTE");
            ui.add_sized(
                [drag_width, 18.0],
                DragValue::new(&mut self.cte)
                    .custom_formatter(|e, _| format!("{e:.1} ¹/k")) // /ºC")) ¹/k
                    .custom_parser(|t| {
                        let parsed = t
                            .chars()
                            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
                            .collect::<String>();
                        parsed.parse::<f64>().ok()
                    })
                    .speed(0.1)
                    .range(0.0..=f64::MAX)
                    .min_decimals(1),
            )
            .on_hover_text("Thermal expansion coefficient");

            // ui.label("Temp");
            // ui.add_sized(
            //     [drag_width, 18.0],
            //     DragValue::new(&mut self.temp)
            //         .custom_formatter(|temp, _| {
            //             let precision = dynamic_precision(temp, 2);
            //             format!("{temp:.precision$} ºC")
            //         })
            //         .custom_parser(|temp| {
            //             let to_parse = temp
            //                 .chars()
            //                 .filter(|c| c.is_ascii_digit() || c == &'.' || c == &'-')
            //                 .collect::<String>();
            //             to_parse.parse::<f64>().ok()
            //         })
            //         .speed(1.0)
            //         .range(-273.15..=10_000.0)
            //         .min_decimals(1),
            // )
            // .on_hover_text("Temperature");
            ui.end_row();
        });
    }
}

pub fn temperature_input(
    ui: &mut Ui,
    state: &mut State,
    left_component: &mut Component,
    right_component: &mut Component,
) {
    Frame::group(ui.style())
        .inner_margin(10.0)
        .rounding(10.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.set_width(514.0);
                let button_size = [30.0, 18.0];
                let drag_size = [54.5, 18.0];
                let centre_spacing = 0.0; // Empirically determined (tweaked with drag now)
                let temp_range = -273.15..=999.9;
                let (mut left_changed, mut right_changed) = (false, false);

                if ui
                    .add_sized(button_size, Button::new("LN₂"))
                    .on_hover_text("Liquid Nitrogen")
                    .clicked()
                {
                    left_component.mat.temp = -196.0;
                    left_changed = true;
                }

                if ui
                    .add_sized(button_size, Button::new("RT"))
                    .on_hover_text("Room temperature")
                    .clicked()
                {
                    left_component.mat.temp = 20.0;
                    left_changed = true;
                }

                left_changed |= ui
                    .add(
                        Slider::new(&mut left_component.mat.temp, temp_range.clone())
                            .trailing_fill(true)
                            .show_value(false)
                            .handle_shape(egui::style::HandleShape::Rect {
                                aspect_ratio: (0.2),
                            }),
                    )
                    .on_hover_text("Hub temperature")
                    .changed();

                left_changed |= ui
                    .add_sized(
                        drag_size,
                        DragValue::new(&mut left_component.mat.temp)
                            .custom_formatter(|t, _| {
                                let precision = dynamic_precision(t, 2);
                                format!("{t:.precision$} ºC")
                            })
                            .custom_parser(|t| {
                                let to_parse = t
                                    .chars()
                                    .filter(|c| c.is_ascii_digit() || c == &'.' || c == &'-')
                                    .collect::<String>();
                                to_parse.parse::<f64>().ok()
                            })
                            .speed(1.0)
                            .range(temp_range.clone()),
                    )
                    .on_hover_text("Hub temperature")
                    .changed();

                ui.add_space(centre_spacing);

                ui.toggle_value(&mut state.sync_temp, "🔃")
                    .on_hover_text("Sync temperature");

                ui.add_space(centre_spacing);

                right_changed |= ui
                    .add_sized(
                        drag_size,
                        DragValue::new(&mut right_component.mat.temp)
                            .custom_formatter(|t, _| {
                                let precision = dynamic_precision(t, 2);
                                format!("{t:.precision$} ºC")
                            })
                            .custom_parser(|t| {
                                let to_parse = t
                                    .chars()
                                    .filter(|c| c.is_ascii_digit() || c == &'.' || c == &'-')
                                    .collect::<String>();
                                to_parse.parse::<f64>().ok()
                            })
                            .speed(1.0)
                            .range(temp_range.clone()),
                    )
                    .on_hover_text("Shaft temperature")
                    .changed();

                right_changed |= ui
                    .add(
                        Slider::new(&mut right_component.mat.temp, temp_range.clone())
                            .trailing_fill(true)
                            .show_value(false)
                            .handle_shape(egui::style::HandleShape::Rect {
                                aspect_ratio: (0.2),
                            }),
                    )
                    .on_hover_text("Shaft temperature")
                    .changed();

                if ui
                    .add_sized(button_size, Button::new("LN₂"))
                    .on_hover_text("Liquid Nitrogen")
                    .clicked()
                {
                    right_component.mat.temp = -196.0;
                    right_changed = true;
                }

                if ui
                    .add_sized(button_size, Button::new("RT"))
                    .on_hover_text("Room temperature")
                    .clicked()
                {
                    right_component.mat.temp = 20.0;
                    right_changed = true;
                }

                // This method of syncing maintains the last changed feature hierachy
                if left_changed {
                    state.synced_temp = left_component.mat.temp;
                } else if right_changed {
                    state.synced_temp = right_component.mat.temp;
                }

                if state.sync_temp {
                    left_component.mat.temp = state.synced_temp;
                    right_component.mat.temp = state.synced_temp;
                }

                // For checking total width
                // let width_text = ui.min_rect().width();
                // ui.label(format!("{width_text}"));
            })
        });
}

pub fn temperature_output(ui: &mut Ui, state: &mut State, hub: &Component, shaft: &Component) {
    Frame::group(ui.style())
        .inner_margin(10.0)
        .rounding(10.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.set_max_size(vec2(514.0, 200.0));

                ui.label("WIP Temp Graph");
                plot::fit_temp_graph(ui, state, hub, shaft);
            })
        });
}
