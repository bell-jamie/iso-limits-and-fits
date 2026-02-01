use crate::Studio;
use egui::{self, Button, Context, Id, Key, RichText};

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

/// Standard warning modal with consistent styling.
/// Returns `true` if confirmed, `false` if cancelled, `None` if still open.
fn warning_modal(
    ctx: &Context,
    id: Id,
    title: &str,
    message: &str,
    confirm_text: &str,
) -> Option<bool> {
    let mut result = None;

    egui::Modal::new(id).show(ctx, |ui| {
        ui.set_width(200.0);
        ui.horizontal(|ui| {
            ui.heading(title);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.heading(RichText::new("⚠").color(ui.visuals().warn_fg_color));
            });
        });
        ui.separator();
        ui.add_space(5.0);
        ui.vertical_centered(|ui| {
            ui.label(message);
        });
        ui.add_space(10.0);

        let enter_pressed = ui.input(|i| i.key_pressed(Key::Enter));
        let esc_pressed = ui.input(|i| i.key_pressed(Key::Escape));

        ui.with_layout(
            egui::Layout::top_down_justified(egui::Align::Center),
            |ui| {
                let confirm_btn = Button::new(RichText::new(confirm_text).strong())
                    .fill(ui.visuals().selection.bg_fill);
                if ui.add(confirm_btn).clicked() || enter_pressed {
                    result = Some(true);
                }
                if ui.button("Cancel").clicked() || esc_pressed {
                    result = Some(false);
                }
            },
        );
    });

    result
}

fn delete_modal(ctx: &Context, app: &mut Studio, target: DeleteTarget) {
    let pending_delete: Option<usize> = ctx.data(|d| d.get_temp(target.pending_id()));
    if let Some(idx) = pending_delete {
        let name = target.get_name(app, idx);
        let message = format!("Are you sure you want to delete \"{}\"?", name);

        if let Some(confirmed) = warning_modal(ctx, target.modal_id(), "Delete", &message, "Delete")
        {
            if confirmed {
                target.perform_delete(app, idx);
            }
            ctx.data_mut(|d| d.remove::<usize>(target.pending_id()));
        }
    }
}

pub fn delete_component(ctx: &Context, app: &mut Studio) {
    delete_modal(ctx, app, DeleteTarget::Component);
}

pub fn delete_material(ctx: &Context, app: &mut Studio) {
    delete_modal(ctx, app, DeleteTarget::Material);
}

pub fn reset_confirm(ctx: &Context, app: &mut Studio) {
    let pending_reset: Option<bool> = ctx.data(|d| d.get_temp(Id::new("pending_reset")));
    if pending_reset.unwrap_or(false) {
        if let Some(confirmed) = warning_modal(
            ctx,
            Id::new("reset_confirm"),
            "Reset",
            "Are you sure you want to reset to default settings?",
            "Reset",
        ) {
            if confirmed {
                let current_scale = ctx.zoom_factor();
                app.state = crate::modules::state::State::default();
                app.thermal = crate::modules::thermal::Thermal::default();
                app.state.scale.value = current_scale;
                ctx.set_zoom_factor(current_scale);
            }
            ctx.data_mut(|d| d.remove::<bool>(Id::new("pending_reset")));
        }
    }
}

pub fn library_reset_confirm(ctx: &Context, app: &mut Studio) {
    let pending_reset: Option<bool> = ctx.data(|d| d.get_temp(Id::new("pending_library_reset")));
    if pending_reset.unwrap_or(false) {
        if let Some(confirmed) = warning_modal(
            ctx,
            Id::new("library_reset_confirm"),
            "Reset Library",
            "Are you sure you want to reset the library?",
            "Reset",
        ) {
            if confirmed {
                app.library = crate::modules::library::Library::default();
            }
            ctx.data_mut(|d| d.remove::<bool>(Id::new("pending_library_reset")));
        }
    }
}

pub fn thermal_colours(ctx: &Context, app: &mut Studio) {
    let show_modal: Option<bool> = ctx.data(|d| d.get_temp(Id::new("show_thermal_colours")));
    if show_modal.unwrap_or(false) {
        let hub_name = app.library.get_hub_name().unwrap_or("Hub").to_string();
        let shaft_name = app.library.get_shaft_name().unwrap_or("Shaft").to_string();

        egui::Modal::new(Id::new("thermal_colours_modal")).show(ctx, |ui| {
            ui.set_width(200.0);
            ui.heading("Thermal Colours");
            ui.separator();
            ui.add_space(5.0);

            // Hub colour picker
            ui.horizontal(|ui| {
                ui.color_edit_button_srgba(&mut app.thermal.hub_colour);
                if ui
                    .add(Button::new("↺").frame(false))
                    .on_hover_text(format!("Reset {} colour", hub_name))
                    .clicked()
                {
                    app.thermal.hub_colour =
                        egui::Color32::RED.gamma_multiply(app.thermal.colour_gamma);
                }
                ui.label(&hub_name);
            });

            // Shaft colour picker
            ui.horizontal(|ui| {
                ui.color_edit_button_srgba(&mut app.thermal.shaft_colour);
                if ui
                    .add(Button::new("↺").frame(false))
                    .on_hover_text(format!("Reset {} colour", shaft_name))
                    .clicked()
                {
                    app.thermal.shaft_colour =
                        egui::Color32::BLUE.gamma_multiply(app.thermal.colour_gamma);
                }
                ui.label(&shaft_name);
            });

            ui.add_space(10.0);

            let esc_pressed = ui.input(|i| i.key_pressed(Key::Escape));
            let enter_pressed = ui.input(|i| i.key_pressed(Key::Enter));

            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    let done_btn = Button::new(RichText::new("Done").strong())
                        .fill(ui.visuals().selection.bg_fill);
                    if ui.add(done_btn).clicked() || esc_pressed || enter_pressed {
                        ctx.data_mut(|d| d.remove::<bool>(Id::new("show_thermal_colours")));
                    }
                },
            );
        });
    }
}
