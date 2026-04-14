//! Sidebar navigation row with animated active indicator and hover states.

use crate::colors;
use crate::helpers;
use crate::typography::Font;

pub struct NavRow<'a> {
    label: &'a str,
    icon: Option<&'a str>,
    active: bool,
    badge: Option<u32>,
}

impl<'a> NavRow<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            icon: None,
            active: false,
            badge: None,
        }
    }

    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn badge(mut self, count: u32) -> Self {
        self.badge = Some(count);
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let p = colors::palette(ui);
        let row_height = 32.0;
        let id = ui.auto_id_with(self.label);

        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), row_height),
            egui::Sense::click(),
        );

        // Animate active state for smooth bg + indicator transitions.
        let active_t = helpers::animate_bool(ui, id.with("active"), self.active, 0.15);

        // Background — blend between transparent, hover, and active.
        let base_bg = if self.active {
            p.active_bg
        } else if response.hovered() {
            p.hover_bg
        } else {
            egui::Color32::TRANSPARENT
        };

        let rounding = egui::CornerRadius::same(6);
        let padded = rect.shrink2(egui::vec2(6.0, 1.0));
        ui.painter().rect_filled(padded, rounding, base_bg);

        // Active indicator bar (animated height).
        if active_t > 0.01 {
            let bar_height = (padded.height() - 12.0) * active_t;
            let bar_y = padded.center().y - bar_height / 2.0;
            let bar = egui::Rect::from_min_size(
                egui::pos2(padded.left(), bar_y),
                egui::vec2(3.0, bar_height),
            );
            ui.painter()
                .rect_filled(bar, egui::CornerRadius::same(1), p.active_indicator);
        }

        let text_color = if self.active {
            p.text_primary
        } else {
            p.text_secondary
        };

        // Icon.
        let mut x = padded.left() + 14.0;
        if let Some(icon) = self.icon {
            ui.painter().text(
                egui::pos2(x, padded.center().y),
                egui::Align2::LEFT_CENTER,
                icon,
                egui::FontId::proportional(Font::Body.size()),
                p.text_muted,
            );
            x += 22.0;
        }

        // Label.
        ui.painter().text(
            egui::pos2(x, padded.center().y),
            egui::Align2::LEFT_CENTER,
            self.label,
            egui::FontId::proportional(Font::Callout.size()),
            text_color,
        );

        // Badge.
        if let Some(count) = self.badge {
            let badge_text = format!("{count}");
            let badge_x = padded.right() - 12.0;
            let badge_rect = egui::Rect::from_center_size(
                egui::pos2(badge_x, padded.center().y),
                egui::vec2(28.0, 18.0),
            );
            ui.painter().rect_filled(
                badge_rect,
                egui::CornerRadius::same(9),
                p.surface_raised,
            );
            ui.painter().text(
                badge_rect.center(),
                egui::Align2::CENTER_CENTER,
                &badge_text,
                egui::FontId::proportional(Font::Caption.size()),
                p.text_muted,
            );
        }

        response
    }
}
