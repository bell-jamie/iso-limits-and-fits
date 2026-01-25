use std::cmp::Ordering;

use crate::Studio;
use egui::{Button, DragValue, Grid, RichText, TextEdit, Ui};

use super::utils::decimals_for_sig_figs;

/// Shows the material editor window for editing a material's properties.
pub fn show_material_editor(app: &mut Studio, ctx: &egui::Context) {
    let mut open = app.state.show_material_editor;

    egui::Window::new("Material Editor")
        .id(egui::Id::new("material_editor_window"))
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            if let Some(material_id) = app.state.editing_material_id {
                if let Some(material) = app.library.materials.get_mut(material_id) {
                    Grid::new("material_editor_grid")
                        .num_columns(2)
                        .spacing([10.0, 5.0])
                        .show(ui, |ui| {
                            ui.label("Name");
                            ui.add(TextEdit::singleline(&mut material.name).desired_width(150.0));
                            ui.end_row();

                            ui.label("CTE");
                            ui.horizontal(|ui| {
                                let cte_decimals = decimals_for_sig_figs(material.cte, 3);
                                ui.add(
                                    DragValue::new(&mut material.cte)
                                        .range(0.0..=100.0)
                                        .speed(0.1)
                                        .max_decimals(cte_decimals),
                                );
                                ui.label("µm/m·°C");
                            });
                            ui.end_row();

                            ui.label("Poisson's Ratio");
                            let poissons_decimals = decimals_for_sig_figs(material.poissons, 3);
                            ui.add(
                                DragValue::new(&mut material.poissons)
                                    .range(0.0..=0.5)
                                    .speed(0.01)
                                    .max_decimals(poissons_decimals),
                            );
                            ui.end_row();

                            ui.label("Young's Modulus");
                            ui.horizontal(|ui| {
                                ui.add(
                                    DragValue::new(&mut material.youngs)
                                        .range(0.0..=10_000.0)
                                        .speed(100.0)
                                        .max_decimals(0),
                                );
                                ui.label("GPa");
                            });
                            ui.end_row();

                            ui.label("Yield Strength");
                            ui.horizontal(|ui| {
                                ui.add(
                                    DragValue::new(&mut material.ys)
                                        .range(0.0..=10_000.0)
                                        .speed(1.0)
                                        .max_decimals(0),
                                );
                                ui.label("MPa");
                            });
                            ui.end_row();

                            ui.label("UTS");
                            ui.horizontal(|ui| {
                                ui.add(
                                    DragValue::new(&mut material.uts)
                                        .range(0.0..=10_000.0)
                                        .speed(1.0)
                                        .max_decimals(0),
                                );
                                ui.label("MPa");
                            });
                            ui.end_row();
                        });
                } else {
                    ui.label("Material not found");
                }
            } else {
                ui.label("No material selected");
            }
        });

    app.state.show_material_editor = open;

    // Clear editing_material_id when window is closed
    if !open {
        app.state.editing_material_id = None;
    }
}

#[derive(Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Material {
    pub name: String,
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
    pub fn default() -> Self {
        Self::carbon_steel()
    }

    pub fn carbon_steel() -> Self {
        Material {
            name: "Carbon Steel".to_owned(),
            cte: 11.7,
            poissons: 0.29,
            youngs: 210.0,
            ys: 250.0,
            uts: 440.0,
        }
    }

    pub fn steel4340() -> Self {
        Material {
            name: "4340 Steel - Annealed".to_owned(),
            cte: 12.3,
            poissons: 0.30,
            youngs: 129.0,
            ys: 470.0,
            uts: 745.0,
        }
    }

    pub fn pb104() -> Self {
        Material {
            name: "Phosphor Bronze — PB104".to_owned(),
            cte: 17.0,
            poissons: 0.34,
            youngs: 105.0,
            ys: 360.0,
            uts: 500.0,
        }
    }
}
