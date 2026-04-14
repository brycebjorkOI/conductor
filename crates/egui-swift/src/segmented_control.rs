//! Pill-shaped segmented control (two or more segments).
//!
//! ```no_run
//! # let ui: &mut egui::Ui = todo!();
//! let mut selected = 0usize;
//! conductor_ui::segmented_control::SegmentedControl::new(
//!     &["Claude CLI", "Gemini CLI"],
//!     &mut selected,
//! ).show(ui);
//! ```

use crate::colors;

pub struct SegmentedControl<'a> {
    labels: &'a [&'a str],
    selected: &'a mut usize,
}

impl<'a> SegmentedControl<'a> {
    pub fn new(labels: &'a [&'a str], selected: &'a mut usize) -> Self {
        Self { labels, selected }
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let p = colors::palette(ui);

        let pill_height = 30.0;
        let seg_padding_h = 16.0;
        let rounding = egui::CornerRadius::same((pill_height / 2.0) as u8);

        // Outer pill background.
        let outer_frame = egui::Frame::NONE
            .fill(p.surface_raised)
            .corner_radius(rounding)
            .inner_margin(egui::Margin::symmetric(3, 3));

        let resp = outer_frame
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    for (i, label) in self.labels.iter().enumerate() {
                        let is_selected = i == *self.selected;

                        let text_color = if is_selected {
                            p.text_on_accent
                        } else {
                            p.text_secondary
                        };

                        let bg = if is_selected {
                            p.accent
                        } else {
                            egui::Color32::TRANSPARENT
                        };

                        let btn = egui::Button::new(
                            egui::RichText::new(*label)
                                .size(12.5)
                                .color(text_color),
                        )
                        .fill(bg)
                        .corner_radius(egui::CornerRadius::same(
                            ((pill_height - 6.0) / 2.0) as u8,
                        ))
                        .stroke(egui::Stroke::NONE);

                        let size = egui::vec2(
                            label.len() as f32 * 7.5 + seg_padding_h * 2.0,
                            pill_height - 6.0,
                        );
                        if ui.add_sized(size, btn).clicked() {
                            *self.selected = i;
                        }
                    }
                });
            })
            .response;

        resp
    }
}
