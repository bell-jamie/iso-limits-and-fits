use crate::modules::{
    cards::{ComponentDrag, MaterialDrag},
    component::Component,
    mat_data::material_list,
    material::Material,
    state::State,
    utils::{self, truncate_string_to_width},
};
use egui::{Button, RichText, Ui};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Library {
    pub hub_id: usize,
    pub shaft_id: usize,
    pub components: Vec<Component>,
    pub materials: Vec<Material>,
}

impl Library {
    pub fn default() -> Self {
        Self {
            hub_id: 0,
            shaft_id: 1,
            components: vec![Component::default_hub(), Component::default_shaft()],
            materials: material_list().into_iter().collect(),
        }
    }

    pub fn get_hub_name(&self) -> Option<&str> {
        self.components
            .get(self.hub_id)
            .map(|hub| hub.name.as_str())
    }

    pub fn get_hub_name_mut(&mut self) -> Option<&mut String> {
        if let Some(hub) = self.get_hub_mut() {
            Some(&mut hub.name)
        } else {
            None
        }
    }

    pub fn get_shaft_name(&self) -> Option<&str> {
        self.components
            .get(self.shaft_id)
            .map(|shaft| shaft.name.as_str())
    }

    pub fn get_shaft_name_mut(&mut self) -> Option<&mut String> {
        if let Some(shaft) = self.get_shaft_mut() {
            Some(&mut shaft.name)
        } else {
            None
        }
    }

    pub fn get_hub(&self) -> Option<&Component> {
        self.components.get(self.hub_id)
    }

    pub fn get_shaft(&self) -> Option<&Component> {
        self.components.get(self.shaft_id)
    }

    pub fn get_hub_mut(&mut self) -> Option<&mut Component> {
        self.components.get_mut(self.hub_id)
    }

    pub fn get_shaft_mut(&mut self) -> Option<&mut Component> {
        self.components.get_mut(self.shaft_id)
    }

    pub fn get_material(&self, id: usize) -> Option<&Material> {
        self.materials.get(id)
    }

    pub fn get_material_mut(&mut self, id: usize) -> Option<&mut Material> {
        self.materials.get_mut(id)
    }

    pub fn get_hub_material_name(&self) -> Option<&str> {
        self.get_hub()
            .and_then(|hub| self.get_material(hub.material_id))
            .map(|mat| mat.name.as_str())
    }

    pub fn get_shaft_material_name(&self) -> Option<&str> {
        self.get_shaft()
            .and_then(|shaft| self.get_material(shaft.material_id))
            .map(|mat| mat.name.as_str())
    }

    pub fn render(&mut self, state: &mut State, ui: &mut Ui) {
        self.components(ui);
        ui.separator();
        self.materials(state, ui);
    }

    fn components(&mut self, ui: &mut Ui) {
        utils::accordion(ui, "components_accordion", "Components", false, |ui| {
            let mut new_hub_id = self.hub_id;
            let mut new_shaft_id = self.shaft_id;

            // Calculate available width for name (panel width minus buttons and spacing)
            // Buttons: H, S, delete (if >2) ~ 60px, plus spacing ~20px
            let buttons_width = if self.components.len() > 2 {
                60.0
            } else {
                40.0
            };
            let name_width = (ui.available_width() - buttons_width).max(40.0);
            let ctx = ui.ctx().clone();

            for (i, component) in self.components.iter().enumerate() {
                let selected = ui.visuals().selection.bg_fill;
                let unselected = ui.visuals().text_color();
                let is_hub = self.hub_id == i;
                let is_shaft = self.shaft_id == i;
                let hub_button_colour = if is_hub { selected } else { unselected };
                let shaft_button_colour = if is_shaft { selected } else { unselected };

                ui.horizontal(|ui| {
                    // Drag source for the component
                    let drag_id = egui::Id::new(("component_drag", i));
                    ui.dnd_drag_source(drag_id, ComponentDrag(i), |ui| {
                        ui.label(truncate_string_to_width(&ctx, &component.name, name_width));
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Delete button (only if more than 2 components)
                        if self.components.len() > 2 {
                            let delete_btn = Button::new(RichText::new("🗑")).frame(false);
                            if ui.add(delete_btn).on_hover_text("Delete").clicked() {
                                ui.ctx().data_mut(|d| {
                                    d.insert_temp(egui::Id::new("pending_delete_component"), i);
                                });
                            }
                            ui.add_space(-3.0);
                        }

                        // Shaft selection button
                        let shaft_btn =
                            Button::new(RichText::new("🇸").color(shaft_button_colour)).frame(false);
                        if ui.add(shaft_btn).on_hover_text("Set as Shaft").clicked() {
                            new_shaft_id = i;
                        }

                        ui.add_space(-5.0);

                        // Hub selection button
                        let hub_btn =
                            Button::new(RichText::new("🇭").color(hub_button_colour)).frame(false);
                        if ui.add(hub_btn).on_hover_text("Set as Hub").clicked() {
                            new_hub_id = i;
                        }
                    });
                });
            }

            self.hub_id = new_hub_id;
            self.shaft_id = new_shaft_id;

            if ui
                .add(Button::new(RichText::new("New component").small().italics()).frame(false))
                .clicked()
            {
                self.components.push(Component::default());
            }
        });
    }

    fn materials(&mut self, state: &mut State, ui: &mut Ui) {
        // Track which material to edit (if any clicked this frame)
        let mut edit_material_id: Option<usize> = None;

        utils::accordion(ui, "materials_accordion", "Materials", false, |ui| {
            let mut new_hub_mat_id = self.get_hub().map(|hub| hub.material_id);
            let mut new_shaft_mat_id = self.get_shaft().map(|shaft| shaft.material_id);

            // Calculate available width for name (panel width minus buttons and spacing)
            // Buttons: H, S, edit, delete (if >2) ~ 70px, plus spacing ~20px
            let buttons_width = if self.materials.len() > 2 { 80.0 } else { 60.0 };
            let name_width = (ui.available_width() - buttons_width).max(40.0);
            let ctx = ui.ctx().clone();

            for (i, material) in self.materials.iter().enumerate() {
                let selected = ui.visuals().selection.bg_fill;
                let unselected = ui.visuals().text_color();
                let is_hub = self
                    .get_hub()
                    .map(|hub| hub.material_id)
                    .unwrap_or(usize::MAX)
                    == i;
                let is_shaft = self
                    .get_shaft()
                    .map(|shaft| shaft.material_id)
                    .unwrap_or(usize::MAX)
                    == i;
                let hub_button_colour = if is_hub { selected } else { unselected };
                let shaft_button_colour = if is_shaft { selected } else { unselected };

                ui.horizontal(|ui| {
                    // Drag source for the material
                    let drag_id = egui::Id::new(("material_drag", i));
                    ui.dnd_drag_source(drag_id, MaterialDrag(i), |ui| {
                        ui.label(truncate_string_to_width(&ctx, &material.name, name_width));
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Delete button (only if more than 2 materials)
                        if self.materials.len() > 2 {
                            let delete_btn = Button::new(RichText::new("🗑")).frame(false);
                            if ui.add(delete_btn).on_hover_text("Delete").clicked() {
                                ui.ctx().data_mut(|d| {
                                    d.insert_temp(egui::Id::new("pending_delete_material"), i);
                                });
                            }
                            ui.add_space(-3.0);
                        }

                        // Shaft selection button
                        let shaft_btn =
                            Button::new(RichText::new("🇸").color(shaft_button_colour)).frame(false);
                        if ui.add(shaft_btn).on_hover_text("Shaft material").clicked() {
                            new_shaft_mat_id = Some(i);
                        }

                        ui.add_space(-5.0);

                        // Hub selection button
                        let hub_btn =
                            Button::new(RichText::new("🇭").color(hub_button_colour)).frame(false);
                        if ui.add(hub_btn).on_hover_text("Hub material").clicked() {
                            new_hub_mat_id = Some(i);
                        }

                        ui.add_space(-2.0);

                        // Edit button
                        let edit_btn = Button::new(RichText::new("🗖")).frame(false);
                        if ui.add(edit_btn).on_hover_text("Material editor").clicked() {
                            edit_material_id = Some(i);
                        }
                    });
                });
            }

            if let (Some(hub), Some(id)) = (self.get_hub_mut(), new_hub_mat_id) {
                hub.material_id = id;
            }
            if let (Some(shaft), Some(id)) = (self.get_shaft_mut(), new_shaft_mat_id) {
                shaft.material_id = id;
            }

            if ui
                .add(Button::new(RichText::new("New material").small().italics()).frame(false))
                .clicked()
            {
                self.materials.push(Material::default());
                edit_material_id = Some(self.materials.len() - 1); // immediately edit last material
            }
        });

        // Open material editor if edit was clicked
        if let Some(id) = edit_material_id {
            state.editing_material_id = Some(id);
            state.show_material_editor = true;
        }
    }
}
