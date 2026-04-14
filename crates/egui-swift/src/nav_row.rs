//! Sidebar navigation row — icon + label with active/hover states and optional
//! badge count.
//!
//! ```no_run
//! # let ui: &mut egui::Ui = todo!();
//! use conductor_ui::nav_row::NavRow;
//! if NavRow::new("Chats").icon("\u{1f4ac}").active(true).show(ui).clicked() {
//!     // handle click
//! }
//! NavRow::new("Trash").icon("\u{1f5d1}").badge(20).show(ui);
//! ```

use crate::colors;

/// A single sidebar navigation row.
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

    /// Leading icon text (emoji or Unicode glyph).
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Mark this row as the active/selected item.
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Show a count badge on the trailing edge.
    pub fn badge(mut self, count: u32) -> Self {
        self.badge = Some(count);
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let p = colors::palette(ui);
        let row_height = 32.0;

        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), row_height),
            egui::Sense::click(),
        );

        // Background.
        let bg = if self.active {
            p.active_bg
        } else if response.hovered() {
            p.hover_bg
        } else {
            egui::Color32::TRANSPARENT
        };

        let rounding = egui::CornerRadius::same(6);
        let padded = rect.shrink2(egui::vec2(6.0, 1.0));
        ui.painter().rect_filled(padded, rounding, bg);

        // Active indicator bar (left edge).
        if self.active {
            let bar = egui::Rect::from_min_size(
                padded.left_top() + egui::vec2(0.0, 6.0),
                egui::vec2(3.0, padded.height() - 12.0),
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
                egui::FontId::proportional(14.0),
                p.text_muted,
            );
            x += 22.0;
        }

        // Label.
        ui.painter().text(
            egui::pos2(x, padded.center().y),
            egui::Align2::LEFT_CENTER,
            self.label,
            egui::FontId::proportional(13.5),
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
                egui::FontId::proportional(11.0),
                p.text_muted,
            );
        }

        response
    }
}
