//! Themed divider / separator line.
//!
//! ```ignore
//! Divider::new().inset(16.0).show(ui);
//! ```

use crate::colors;

pub struct Divider {
    inset: f32,
}

impl Divider {
    pub fn new() -> Self {
        Self { inset: 0.0 }
    }

    /// Left inset in pixels (SwiftUI form sections use 16px).
    pub fn inset(mut self, px: f32) -> Self {
        self.inset = px;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let p = colors::palette(ui);
        let available = ui.available_width();
        let (rect, _) =
            ui.allocate_exact_size(egui::vec2(available, 0.5), egui::Sense::hover());

        if ui.is_rect_visible(rect) {
            let start = egui::pos2(rect.left() + self.inset, rect.center().y);
            let end = egui::pos2(rect.right(), rect.center().y);
            ui.painter()
                .line_segment([start, end], egui::Stroke::new(0.5, p.divider));
        }
    }
}

impl Default for Divider {
    fn default() -> Self {
        Self::new()
    }
}
