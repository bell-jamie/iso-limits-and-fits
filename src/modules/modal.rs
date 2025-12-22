use crate::Studio;
use egui::{self, Context, RichText};

pub fn delete_component(ctx: &Context, app: &mut Studio) {
    let pending_delete: Option<usize> = ctx.data(|d| d.get_temp(egui::Id::new("pending_delete")));
    if let Some(idx) = pending_delete {
        let component_name = app
            .component_library
            .get(idx)
            .map(|c| c.name.clone())
            .unwrap_or_default();

        egui::Modal::new("delete_confirm".into()).show(ctx, |ui| {
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
                    ui.label(format!(
                        "Are you sure you want to delete \"{}\"?",
                        component_name
                    ));
                    ui.add_space(10.0);
                    if ui.button("Delete").clicked() {
                        app.component_library.remove(idx);
                        // Adjust hub_id and shaft_id if needed
                        if app.hub_id >= idx && app.hub_id > 0 {
                            app.hub_id -= 1;
                        }
                        if app.shaft_id >= idx && app.shaft_id > 0 {
                            app.shaft_id -= 1;
                        }
                        ctx.data_mut(|d| d.remove::<usize>(egui::Id::new("pending_delete")));
                    }
                    if ui.button("Cancel").clicked() {
                        ctx.data_mut(|d| d.remove::<usize>(egui::Id::new("pending_delete")));
                    }
                },
            );
        });
    }
}
