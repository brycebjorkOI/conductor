//! SwiftUI-style radio button group.
//!
//! ```ignore
//! RadioGroup::new(&mut mode, &[
//!     ("standalone", "Standalone"),
//!     ("local", "Local Server"),
//! ]).show(ui);
//! ```

use crate::colors;
use crate::helpers;

const ROW_HEIGHT: f32 = 32.0;
const OUTER_RADIUS: f32 = 8.0;
const INNER_RADIUS: f32 = 4.0;

pub struct RadioGroup<'a, T: PartialEq + Clone + 'a> {
    selected: &'a mut T,
    options: &'a [(T, &'a str)],
    label: Option<&'a str>,
}

impl<'a, T: PartialEq + Clone> RadioGroup<'a, T> {
    pub fn new(selected: &'a mut T, options: &'a [(T, &'a str)]) -> Self {
        Self {
            selected,
            options,
            label: None,
        }
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
                    .strong()
                    .size(13.0)
                    .color(p.text_primary),
            );
            ui.add_space(4.0);
        }

        let mut changed = false;

        for (value, name) in self.options {
            let is_selected = value == self.selected;
            let row_id = ui.auto_id_with(name);

            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(ui.available_width(), ROW_HEIGHT), egui::Sense::click());

            if response.clicked() {
                *self.selected = value.clone();
                changed = true;
            }

            if ui.is_rect_visible(rect) {
                let painter = ui.painter();

                // Hover background.
                if response.hovered() {
                    painter.rect_filled(
                        rect,
                        egui::CornerRadius::same(6),
                        p.hover_bg,
                    );
                }

                // Radio circle.
                let circle_center = egui::pos2(rect.left() + 12.0, rect.center().y);
                painter.circle_stroke(
                    circle_center,
                    OUTER_RADIUS,
                    egui::Stroke::new(1.5, if is_selected { p.accent } else { p.border }),
                );

                // Filled inner circle (animated).
                let t = helpers::animate_bool(ui, row_id, is_selected, 0.12);
                if t > 0.0 {
                    painter.circle_filled(
                        circle_center,
                        INNER_RADIUS * t,
                        p.accent,
                    );
                }

                // Label.
                painter.text(
                    egui::pos2(rect.left() + 28.0, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    *name,
                    egui::FontId::proportional(13.0),
                    p.text_primary,
                );
            }
        }

        // Return a dummy response for the whole group.
        let (_, response) = ui.allocate_exact_size(egui::vec2(0.0, 0.0), egui::Sense::hover());
        if changed {
            response.ctx.request_repaint();
        }
        response
    }
}
