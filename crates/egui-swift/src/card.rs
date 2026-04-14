//! Rounded card container with optional border color and shadow.
//!
//! ```ignore
//! Card::new().border_color(green).shadow(true).show(ui, |ui| { ... });
//! ```

use crate::colors;
use crate::helpers;
use crate::theme::Layout;

pub struct Card {
    border_color: Option<egui::Color32>,
    shadow: bool,
    padding: egui::Margin,
}

impl Card {
    pub fn new() -> Self {
        Self {
            border_color: None,
            shadow: false,
            padding: egui::Margin::symmetric(12, 8),
        }
    }

    pub fn border_color(mut self, c: egui::Color32) -> Self {
        self.border_color = Some(c);
        self
    }

    pub fn shadow(mut self, enabled: bool) -> Self {
        self.shadow = enabled;
        self
    }

    pub fn padding(mut self, m: egui::Margin) -> Self {
        self.padding = m;
        self
    }

    pub fn show(
        self,
        ui: &mut egui::Ui,
        content: impl FnOnce(&mut egui::Ui),
    ) -> egui::Response {
        let p = colors::palette(ui);
        let rounding = egui::CornerRadius::same(Layout::CARD_RADIUS as u8);
        let stroke_color = self.border_color.unwrap_or(p.border_subtle);

        // Reserve space for the outer margin between stacked cards.
        ui.add_space(3.0);

        if self.shadow {
            // We need to paint the shadow before the frame. Allocate a rect first
            // by measuring the content inside a temporary scope. Since we can't
            // easily do two-pass in egui, we apply shadow as a background paint
            // after the frame via the response rect.
        }

        let frame = egui::Frame::NONE
            .fill(p.card_bg)
            .corner_radius(rounding)
            .stroke(egui::Stroke::new(0.5, stroke_color))
            .inner_margin(self.padding);

        let resp = frame
            .show(ui, |ui| {
                content(ui);
            })
            .response;

        if self.shadow && ui.is_rect_visible(resp.rect) {
            helpers::paint_shadow(ui, resp.rect, rounding, 4.0, p.shadow);
        }

        ui.add_space(3.0);

        resp
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}
