//! Top toolbar / header bar with optional title and leading/trailing content.
//!
//! ```ignore
//! Toolbar::new()
//!     .title("Settings")
//!     .trailing(|ui| { Button::new("Done").show(ui); })
//!     .show(ui);
//! ```

use crate::colors;
use crate::divider::Divider;
use crate::theme::Layout;

pub struct Toolbar<'a> {
    title: Option<&'a str>,
    leading: Option<Box<dyn FnOnce(&mut egui::Ui) + 'a>>,
    trailing: Option<Box<dyn FnOnce(&mut egui::Ui) + 'a>>,
}

impl<'a> Toolbar<'a> {
    pub fn new() -> Self {
        Self {
            title: None,
            leading: None,
            trailing: None,
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn leading(mut self, f: impl FnOnce(&mut egui::Ui) + 'a) -> Self {
        self.leading = Some(Box::new(f));
        self
    }

    pub fn trailing(mut self, f: impl FnOnce(&mut egui::Ui) + 'a) -> Self {
        self.trailing = Some(Box::new(f));
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let p = colors::palette(ui);

        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), Layout::TOOLBAR_HEIGHT),
            egui::Sense::hover(),
        );

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Background.
            painter.rect_filled(rect, egui::CornerRadius::ZERO, p.surface);

            // Title (centered).
            if let Some(title) = self.title {
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    title,
                    egui::FontId::proportional(15.0),
                    p.text_primary,
                );
            }
        }

        // Leading content (left-aligned).
        if let Some(leading) = self.leading {
            let leading_rect = egui::Rect::from_min_size(
                egui::pos2(rect.left() + 8.0, rect.min.y),
                egui::vec2(rect.width() * 0.3, rect.height()),
            );
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(leading_rect));
            child.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                leading(ui);
            });
        }

        // Trailing content (right-aligned).
        if let Some(trailing) = self.trailing {
            let trailing_rect = egui::Rect::from_min_max(
                egui::pos2(rect.right() - rect.width() * 0.3, rect.min.y),
                egui::pos2(rect.right() - 8.0, rect.max.y),
            );
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(trailing_rect));
            child.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                trailing(ui);
            });
        }

        // Bottom divider.
        Divider::new().show(ui)
    }
}

impl<'a> Default for Toolbar<'a> {
    fn default() -> Self {
        Self::new()
    }
}
