use crate::{
    Studio,
    modules::{
        cards::{ComponentDrag, MaterialDrag},
        component::Component,
        material::Material,
        utils,
    },
};
use egui::{Button, RichText, Ui};

pub fn render(app: &mut Studio, ui: &mut Ui) {
    components(app, ui);
    ui.separator();
    materials(app, ui);
}

fn components(app: &mut Studio, ui: &mut Ui) {
    utils::accordion(ui, "components_accordion", "Components", false, |ui| {
        let mut new_hub_id = app.hub_id;
        let mut new_shaft_id = app.shaft_id;

        for (i, component) in app.component_library.iter().enumerate() {
            let selected = ui.visuals().selection.bg_fill;
            let unselected = ui.visuals().text_color();
            let is_hub = app.hub_id == i;
            let is_shaft = app.shaft_id == i;
            let hub_button_colour = if is_hub { selected } else { unselected };
            let shaft_button_colour = if is_shaft { selected } else { unselected };

            ui.horizontal(|ui| {
                // Drag source for the component
                let drag_id = egui::Id::new(("component_drag", i));
                ui.dnd_drag_source(drag_id, ComponentDrag(i), |ui| {
                    ui.label(&component.name);
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Delete button (only if more than 2 components)
                    if app.component_library.len() > 2 {
                        let delete_btn = Button::new(RichText::new("🗑")).frame(false);
                        if ui.add(delete_btn).on_hover_text("Delete").clicked() {
                            ui.ctx().data_mut(|d| {
                                d.insert_temp(egui::Id::new("pending_delete"), i);
                            });
                        }
                    }

                    // Shaft selection button
                    let shaft_btn =
                        Button::new(RichText::new("S").color(shaft_button_colour)).frame(false);
                    if ui.add(shaft_btn).on_hover_text("Set as Shaft").clicked() {
                        new_shaft_id = i;
                    }

                    // Hub selection button
                    let hub_btn =
                        Button::new(RichText::new("H").color(hub_button_colour)).frame(false);
                    if ui.add(hub_btn).on_hover_text("Set as Hub").clicked() {
                        new_hub_id = i;
                    }
                });
            });
        }

        app.hub_id = new_hub_id;
        app.shaft_id = new_shaft_id;

        if ui
            .add(Button::new(RichText::new("New component").small().italics()).frame(false))
            .clicked()
        {
            app.component_library.push(Component::default());
        }
    });
}

fn materials(app: &mut Studio, ui: &mut Ui) {
    utils::accordion(ui, "materials_accordion", "Materials", false, |ui| {
        let mut new_hub_mat_id = app.get_hub().map(|hub| hub.material_id);
        let mut new_shaft_mat_id = app.get_shaft().map(|shaft| shaft.material_id);

        for (i, material) in app.material_library.iter().enumerate() {
            let selected = ui.visuals().selection.bg_fill;
            let unselected = ui.visuals().text_color();
            let is_hub = app
                .get_hub()
                .map(|hub| hub.material_id)
                .unwrap_or(usize::MAX)
                == i;
            let is_shaft = app
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
                    ui.label(&material.name);
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Delete button (only if more than 2 components)
                    if app.material_library.len() > 2 {
                        let delete_btn = Button::new(RichText::new("🗑")).frame(false);
                        if ui.add(delete_btn).on_hover_text("Delete").clicked() {
                            ui.ctx().data_mut(|d| {
                                d.insert_temp(egui::Id::new("pending_delete"), i);
                            });
                        }
                    }

                    // Shaft selection button
                    let shaft_btn =
                        Button::new(RichText::new("S").color(shaft_button_colour)).frame(false);
                    if ui.add(shaft_btn).on_hover_text("Shaft Material").clicked() {
                        new_shaft_mat_id = Some(i);
                    }

                    // Hub selection button
                    let hub_btn =
                        Button::new(RichText::new("H").color(hub_button_colour)).frame(false);
                    if ui.add(hub_btn).on_hover_text("Hub Material").clicked() {
                        new_hub_mat_id = Some(i);
                    }
                });
            });
        }

        if let (Some(hub), Some(id)) = (app.get_hub_mut(), new_hub_mat_id) {
            hub.material_id = id;
        }
        if let (Some(shaft), Some(id)) = (app.get_shaft_mut(), new_shaft_mat_id) {
            shaft.material_id = id;
        }

        if ui
            .add(Button::new(RichText::new("New material").small().italics()).frame(false))
            .clicked()
        {
            app.material_library.push(Material::default());
        }
    });
}
