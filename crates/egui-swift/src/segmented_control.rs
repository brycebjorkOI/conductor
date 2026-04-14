//! Pill-shaped segmented control with animated selection.

use crate::colors;
use crate::ext::ColorExt;
use crate::typography::Font;

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
        let id = ui.auto_id_with("segmented_control");

        let pill_height = 30.0;
        let seg_padding_h = 16.0;
        let inner_height = pill_height - 6.0;
        let rounding = egui::CornerRadius::same((pill_height / 2.0) as u8);
        let inner_rounding = egui::CornerRadius::same((inner_height / 2.0) as u8);

        // Calculate segment widths.
        let seg_widths: Vec<f32> = self
            .labels
            .iter()
            .map(|l| l.len() as f32 * 7.5 + seg_padding_h * 2.0)
            .collect();


        // Outer pill.
        let outer_frame = egui::Frame::NONE
            .fill(p.surface_raised)
            .corner_radius(rounding)
            .inner_margin(egui::Margin::symmetric(3, 3));

        let resp = outer_frame
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;

                    // Animate selection position for smooth sliding.
                    let target_x: f32 = seg_widths[..*self.selected].iter().sum();
                    let anim_id = id.with("sel_x");
                    let prev_x = ui
                        .ctx()
                        .animate_value_with_time(anim_id, target_x, 0.2);

                    // Draw the sliding selection pill behind the segments.
                    let origin = ui.cursor().min;
                    let sel_rect = egui::Rect::from_min_size(
                        egui::pos2(origin.x + prev_x, origin.y),
                        egui::vec2(seg_widths[*self.selected], inner_height),
                    );
                    ui.painter()
                        .rect_filled(sel_rect, inner_rounding, p.accent);

                    // Draw each segment as transparent button overlay.
                    for (i, label) in self.labels.iter().enumerate() {
                        let is_selected = i == *self.selected;

                        let text_color = if is_selected {
                            p.text_on_accent
                        } else {
                            p.text_secondary
                        };

                        let btn = egui::Button::new(
                            egui::RichText::new(*label)
                                .size(Font::Subheadline.size())
                                .color(text_color),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .corner_radius(inner_rounding)
                        .stroke(egui::Stroke::NONE);

                        let size = egui::vec2(seg_widths[i], inner_height);
                        let resp = ui.add_sized(size, btn);

                        // Hover tint on non-selected segments.
                        if resp.hovered() && !is_selected {
                            ui.painter().rect_filled(
                                resp.rect,
                                inner_rounding,
                                p.hover_bg.opacity(0.3),
                            );
                        }

                        if resp.clicked() {
                            *self.selected = i;
                        }
                    }
                });
            })
            .response;

        resp
    }
}
