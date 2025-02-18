use egui::{DragValue, Ui};

use crate::modules::utils::dynamic_precision;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct Material {
    pub temp: f64,
    pub cte: f64,
    pub poissons: f64,
    pub youngs: f64,
    pub ys: f64,
    pub uts: f64,
}

impl Material {
    pub fn steel() -> Self {
        Material {
            temp: 20.0,
            cte: 11.5,
            poissons: 0.29,
            youngs: 200_000.0,
            ys: 300.0,
            uts: 500.0,
        }
    }

    pub fn brass() -> Self {
        Material {
            temp: 20.0,
            cte: 19.5,
            poissons: 0.34,
            youngs: 110_000.0,
            ys: 300.0,
            uts: 450.0,
        }
    }

    pub fn aluminium() -> Self {
        Material {
            temp: 20.0,
            cte: 23.5,
            poissons: 0.34,
            youngs: 69_000.0,
            ys: 260.0,
            uts: 500.0,
        }
    }

    pub fn input(&mut self, ui: &mut Ui, id: &str) {
        ui.add_space(5.0);

        // ui.horizontal(|ui| {
        //     ui.add_sized(
        //         [45.0, 18.0],
        //         DragValue::new(&mut self.temp)
        //             .custom_formatter(|t, _| format!("{t} ºC"))
        //             .custom_parser(|t| {
        //                 let to_parse = t
        //                     .chars()
        //                     .filter(|c| c.is_ascii_digit() || c == &'.' || c == &'-')
        //                     .collect::<String>();
        //                 to_parse.parse::<f64>().ok()
        //             })
        //             .speed(1.0)
        //             .range(-273.15..=10_000.0)
        //             .min_decimals(1),
        //     )
        //     .on_hover_text("Temperature");

        //     ui.add_sized(
        //         [60.0, 18.0],
        //         DragValue::new(&mut self.cte)
        //             .custom_formatter(|e, _| format!("{e:.1} ¹/k")) // /ºC")) ¹/k
        //             .custom_parser(|t| {
        //                 let parsed = t
        //                     .chars()
        //                     .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        //                     .collect::<String>();
        //                 parsed.parse::<f64>().ok()
        //             })
        //             .speed(0.1)
        //             .range(0.0..=f64::MAX)
        //             .min_decimals(1),
        //     )
        //     .on_hover_text("Thermal expansion coefficient");
        // });

        let drag_width = 61.0;

        egui::Grid::new(id).striped(false).show(ui, |ui| {
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

            ui.label("Temp");
            ui.add_sized(
                [drag_width, 18.0],
                DragValue::new(&mut self.temp)
                    .custom_formatter(|temp, _| {
                        let precision = dynamic_precision(temp, 2);
                        format!("{temp:.precision$} ºC")
                    })
                    .custom_parser(|temp| {
                        let to_parse = temp
                            .chars()
                            .filter(|c| c.is_ascii_digit() || c == &'.' || c == &'-')
                            .collect::<String>();
                        to_parse.parse::<f64>().ok()
                    })
                    .speed(1.0)
                    .range(-273.15..=10_000.0)
                    .min_decimals(1),
            )
            .on_hover_text("Temperature");
            ui.end_row();

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
        });
    }
}
