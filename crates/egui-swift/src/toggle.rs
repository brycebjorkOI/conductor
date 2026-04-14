//! iOS / macOS-style toggle switch.
//!
//! ```ignore
//! Toggle::new(&mut value).label("Launch at login").show(ui);
//! ```

use crate::colors;
use crate::ext::ColorExt;
use crate::helpers;

const TRACK_WIDTH: f32 = 51.0;
const TRACK_HEIGHT: f32 = 31.0;
const THUMB_DIAMETER: f32 = 27.0;
const THUMB_INSET: f32 = 2.0;
const ANIMATION_SECS: f32 = 0.15;

pub struct Toggle<'a> {
    value: &'a mut bool,
    label: Option<&'a str>,
}

impl<'a> Toggle<'a> {
    pub fn new(value: &'a mut bool) -> Self {
        Self { value, label: None }
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let p = colors::palette(ui);
        let id = ui.auto_id_with("toggle");

        let desired_size = if self.label.is_some() {
            egui::vec2(ui.available_width(), TRACK_HEIGHT)
        } else {
            egui::vec2(TRACK_WIDTH, TRACK_HEIGHT)
        };

        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

        if response.clicked() {
            *self.value = !*self.value;
        }

        let t = helpers::animate_bool(ui, id, *self.value, ANIMATION_SECS);

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Label on the left.
            if let Some(label) = self.label {
                let text_pos = egui::pos2(rect.left(), rect.center().y - 7.0);
                painter.text(
                    text_pos,
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(13.0),
                    p.text_primary,
                );
            }

            // Track (right-aligned).
            let track_rect = egui::Rect::from_min_size(
                egui::pos2(rect.right() - TRACK_WIDTH, rect.center().y - TRACK_HEIGHT / 2.0),
                egui::vec2(TRACK_WIDTH, TRACK_HEIGHT),
            );

            let track_color = helpers::lerp_color(p.toggle_off, p.toggle_on, t);
            let track_rounding = egui::CornerRadius::same((TRACK_HEIGHT / 2.0) as u8);
            painter.rect_filled(track_rect, track_rounding, track_color);

            // Thumb.
            let thumb_left = track_rect.left() + THUMB_INSET;
            let thumb_right = track_rect.right() - THUMB_INSET - THUMB_DIAMETER;
            let thumb_x = thumb_left + (thumb_right - thumb_left) * t + THUMB_DIAMETER / 2.0;
            let thumb_center = egui::pos2(thumb_x, track_rect.center().y);

            // Subtle thumb shadow.
            painter.circle_filled(
                thumb_center + egui::vec2(0.0, 1.0),
                THUMB_DIAMETER / 2.0 + 0.5,
                egui::Color32::BLACK.opacity(0.1),
            );
            // White thumb.
            painter.circle_filled(thumb_center, THUMB_DIAMETER / 2.0, egui::Color32::WHITE);
        }

        response
    }
}
