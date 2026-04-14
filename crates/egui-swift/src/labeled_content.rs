//! SwiftUI-style LabeledContent — label on left, value/control on right.
//!
//! ```ignore
//! // Simple key-value
//! LabeledContent::new("Email", "user@example.com").show(ui);
//!
//! // With a custom control on the right
//! LabeledContent::labeled("Status").show(ui, |ui| {
//!     StatusDot::new(p.status_green).label("Online").show(ui);
//! });
//! ```

use crate::colors;
use crate::theme::Layout;
use crate::typography::Font;

/// A row with a label on the left and a value or control on the right.
pub struct LabeledContent<'a> {
    label: &'a str,
    value: Option<&'a str>,
}

impl<'a> LabeledContent<'a> {
    /// Create a labeled row displaying a string value.
    pub fn new(label: &'a str, value: &'a str) -> Self {
        Self {
            label,
            value: Some(value),
        }
    }

    /// Create a labeled row where the right side is a custom closure.
    pub fn labeled(label: &'a str) -> Self {
        Self { label, value: None }
    }

    /// Show with the string value (set via `new()`).
    /// If you used `labeled()`, call `show_with()` instead.
    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let p = colors::palette(ui);
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), Layout::FORM_ROW_HEIGHT),
            egui::Sense::hover(),
        );

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();

            // Label on left.
            painter.text(
                egui::pos2(rect.left(), rect.center().y),
                egui::Align2::LEFT_CENTER,
                self.label,
                egui::FontId::proportional(Font::Callout.size()),
                p.text_primary,
            );

            // Value on right.
            if let Some(value) = self.value {
                painter.text(
                    egui::pos2(rect.right(), rect.center().y),
                    egui::Align2::RIGHT_CENTER,
                    value,
                    egui::FontId::proportional(Font::Callout.size()),
                    p.text_secondary,
                );
            }
        }

        response
    }

    /// Show with a custom control rendered on the right side.
    pub fn show_with(
        self,
        ui: &mut egui::Ui,
        control: impl FnOnce(&mut egui::Ui),
    ) -> egui::Response {
        let p = colors::palette(ui);

        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), Layout::FORM_ROW_HEIGHT),
            egui::Sense::hover(),
        );

        if ui.is_rect_visible(rect) {
            // Label on the left.
            ui.painter().text(
                egui::pos2(rect.left(), rect.center().y),
                egui::Align2::LEFT_CENTER,
                self.label,
                egui::FontId::proportional(Font::Callout.size()),
                p.text_primary,
            );
        }

        // Control on the right.
        let control_rect = egui::Rect::from_min_max(
            egui::pos2(rect.center().x, rect.min.y),
            rect.max,
        );
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(control_rect));
        child.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            control(ui);
        });

        response
    }
}
