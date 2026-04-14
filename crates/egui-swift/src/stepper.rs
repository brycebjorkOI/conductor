//! Numeric stepper with +/- buttons in a capsule.
//!
//! ```ignore
//! Stepper::new(&mut value, 1.0..=10080.0).step(5.0).label("Interval (min)").show(ui);
//! ```

use crate::colors;

pub struct Stepper<'a> {
    value: &'a mut f64,
    range: std::ops::RangeInclusive<f64>,
    step: f64,
    label: Option<&'a str>,
}

impl<'a> Stepper<'a> {
    pub fn new(value: &'a mut f64, range: std::ops::RangeInclusive<f64>) -> Self {
        Self {
            value,
            range,
            step: 1.0,
            label: None,
        }
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let p = colors::palette(ui);

        if let Some(label) = self.label {
            ui.label(
                egui::RichText::new(label)
                    .size(13.0)
                    .color(p.text_primary),
            );
            ui.add_space(4.0);
        }

        let height = 30.0;
        let btn_size = 28.0;
        let rounding = egui::CornerRadius::same((height / 2.0) as u8);

        let resp = egui::Frame::NONE
            .fill(p.input_bg)
            .corner_radius(rounding)
            .stroke(egui::Stroke::new(0.5, p.border))
            .inner_margin(egui::Margin::symmetric(4, 0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.set_height(height);

                    // Minus button.
                    let can_dec = *self.value > *self.range.start();
                    let minus = ui.add_enabled(
                        can_dec,
                        egui::Button::new(
                            egui::RichText::new("\u{2212}").size(16.0).color(p.accent),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .min_size(egui::vec2(btn_size, btn_size)),
                    );
                    if minus.clicked() {
                        *self.value = (*self.value - self.step).max(*self.range.start());
                    }

                    // Value display.
                    let display = if self.step.fract() == 0.0 {
                        format!("{}", *self.value as i64)
                    } else {
                        format!("{:.1}", *self.value)
                    };
                    ui.label(
                        egui::RichText::new(display)
                            .size(14.0)
                            .color(p.text_primary)
                            .strong(),
                    );

                    // Plus button.
                    let can_inc = *self.value < *self.range.end();
                    let plus = ui.add_enabled(
                        can_inc,
                        egui::Button::new(
                            egui::RichText::new("+").size(16.0).color(p.accent),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .min_size(egui::vec2(btn_size, btn_size)),
                    );
                    if plus.clicked() {
                        *self.value = (*self.value + self.step).min(*self.range.end());
                    }
                });
            })
            .response;

        resp
    }
}
