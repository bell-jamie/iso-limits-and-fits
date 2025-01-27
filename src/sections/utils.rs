use egui::{Button, Context, DragValue, Label, Pos2, Sense, Ui, Vec2};

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct State {
    pub debug: bool,
    pub force_valid: bool,
    pub sync_size: bool,
    pub synced_size: f64,
    pub sync_temp: bool,
    pub synced_temp: f64,
    pub thermal: bool,
    pub zoom: Zoom,
}

impl State {
    pub fn default() -> Self {
        State {
            debug: false,
            force_valid: false,
            sync_size: true,
            synced_size: 10.0,
            sync_temp: true,
            synced_temp: 20.0,
            thermal: false,
            zoom: Zoom::default(),
        }
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Zoom {
    pub expand: bool,
    pub scale: f32,
}

impl Zoom {
    pub fn default() -> Self {
        Zoom {
            expand: false,
            scale: 1.0,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, ctx: &Context) {
        ctx.set_zoom_factor(self.scale);

        ui.toggle_value(&mut self.expand, "🔍")
            .on_hover_text("Zoom");

        if self.expand {
            let mut new_scale = self.scale;

            if ui.add(Button::new("➖")).clicked() {
                new_scale -= 0.1;
            }

            let scale_button = ui
                .add(Button::new(format!("{:.1}x", self.scale)))
                .on_hover_text("Scale");

            if scale_button.clicked() {
                new_scale = new_scale.floor() + 1.0;
                if new_scale == 6.0 {
                    new_scale = 1.0;
                }
            }

            if ui.add(Button::new("➕")).clicked() {
                new_scale += 0.1;
            }

            self.scale = new_scale.clamp(0.5, 5.0);
        }

        // let zoom_button = ui.add(Button::new("🔍").sense(Sense {
        //     click: true,
        //     drag: true,
        //     focusable: false,
        // }));

        // if zoom_button.drag_started() {
        //     self.origin = ctx
        //         .pointer_latest_pos()
        //         .unwrap_or(zoom_button.rect.center()); // Save the drag start position
        //     self.factor_last = self.level; // Save the current zoom factor
        // }

        // if zoom_button.dragged() {
        //     if let Some(pointer_pos) = ctx.pointer_latest_pos() {
        //         let distance = pointer_pos - self.origin;
        //         let sensitivity = 0.01; // Zoom sensitivity

        //         self.level =
        //             (self.factor_last + (distance.x + distance.y) * sensitivity).clamp(0.5, 3.0);

        //         // let target = self.factor_last + (distance.x + distance.y) * 0.01;
        //         // let t = ((self.factor - self.factor_last) / (target - self.factor_last))
        //         //     .clamp(-1.0, 1.0);

        //         // self.factor = (self.factor_last + (target - self.factor_last) * t).clamp(0.5, 3.0);
        //     }
        // }

        // if ui.add(Button::new("HD")).clicked() {
        //     self.level = 1.0;
        // }

        // if ui.add(Button::new("4K")).clicked() {
        //     self.level = 2.0;
        // }

        // if zoom_button.clicked() {
        //     self.level = 1.0;
        // }
    }
}

pub fn decimals(num: f64, decimals: i32) -> f64 {
    // Negative decimals inherit the default decimal places value
    let power = if decimals >= 0 { decimals } else { 6 };
    let factor = 10f64.powi(power);
    (num * factor).round() / factor
}

// pub fn linspace(a: f64, b: f64, n: usize) -> Vec<f64> {
//     (0..n)
//         .map(|i| {
//             let t = i as f64 / (n as f64 - 1.0);
//             a + t * (b - a)
//         })
//         .collect()
// }
