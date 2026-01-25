use crate::Studio;
use egui::{self, Context, Id, RichText};

#[derive(Clone, Copy)]
pub enum DeleteTarget {
    Component,
    Material,
}

impl DeleteTarget {
    fn pending_id(&self) -> Id {
        match self {
            DeleteTarget::Component => Id::new("pending_delete_component"),
            DeleteTarget::Material => Id::new("pending_delete_material"),
        }
    }

    fn modal_id(&self) -> Id {
        match self {
            DeleteTarget::Component => Id::new("delete_component_confirm"),
            DeleteTarget::Material => Id::new("delete_material_confirm"),
        }
    }

    fn get_name(&self, app: &Studio, idx: usize) -> String {
        match self {
            DeleteTarget::Component => app
                .library
                .components
                .get(idx)
                .map(|c| c.name.clone())
                .unwrap_or_default(),
            DeleteTarget::Material => app
                .library
                .materials
                .get(idx)
                .map(|m| m.name.clone())
                .unwrap_or_default(),
        }
    }

    fn perform_delete(&self, app: &mut Studio, idx: usize) {
        match self {
            DeleteTarget::Component => {
                app.library.components.remove(idx);
                if app.library.hub_id >= idx && app.library.hub_id > 0 {
                    app.library.hub_id -= 1;
                }
                if app.library.shaft_id >= idx && app.library.shaft_id > 0 {
                    app.library.shaft_id -= 1;
                }
            }
            DeleteTarget::Material => {
                app.library.materials.remove(idx);
                for component in &mut app.library.components {
                    if component.material_id >= idx && component.material_id > 0 {
                        component.material_id -= 1;
                    }
                }
            }
        }
    }
}

fn delete_modal(ctx: &Context, app: &mut Studio, target: DeleteTarget) {
    let pending_delete: Option<usize> = ctx.data(|d| d.get_temp(target.pending_id()));
    if let Some(idx) = pending_delete {
        let name = target.get_name(app, idx);

        egui::Modal::new(target.modal_id()).show(ctx, |ui| {
            ui.set_width(200.0);
            ui.horizontal(|ui| {
                ui.heading("Delete");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.heading(RichText::new("⚠").color(ui.visuals().warn_fg_color));
                });
            });
            ui.add_space(10.0);
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    ui.label(format!("Are you sure you want to delete \"{}\"?", name));
                    ui.add_space(10.0);
                    if ui.button("Delete").clicked() {
                        target.perform_delete(app, idx);
                        ctx.data_mut(|d| d.remove::<usize>(target.pending_id()));
                    }
                    if ui.button("Cancel").clicked() {
                        ctx.data_mut(|d| d.remove::<usize>(target.pending_id()));
                    }
                },
            );
        });
    }
}

pub fn delete_component(ctx: &Context, app: &mut Studio) {
    delete_modal(ctx, app, DeleteTarget::Component);
}

pub fn delete_material(ctx: &Context, app: &mut Studio) {
    delete_modal(ctx, app, DeleteTarget::Material);
}
