//! Rounded card container with optional border, shadow, and hover feedback.

use crate::colors;
use crate::ext::ColorExt;
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

        ui.add_space(3.0);

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

        if ui.is_rect_visible(resp.rect) {
            // Hover: lighten background.
            if resp.hovered() {
                ui.painter().rect_filled(
                    resp.rect,
                    rounding,
                    p.hover_bg.opacity(0.3),
                );
            }

            // Shadow (paint behind — egui draws back-to-front so this overlaps,
            // but at low alpha it's a subtle glow effect).
            if self.shadow {
                let spread = if resp.hovered() { 6.0 } else { 4.0 };
                helpers::paint_shadow(ui, resp.rect, rounding, spread, p.shadow);
            }
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
