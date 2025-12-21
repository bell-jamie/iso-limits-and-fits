use crate::Studio;

pub fn inputs(ctx: &egui::Context, app: &mut Studio) {
    // Open library panel - ctrl + B
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::B)) {
        app.state.show_library_panel = !app.state.show_library_panel;
    }
}
