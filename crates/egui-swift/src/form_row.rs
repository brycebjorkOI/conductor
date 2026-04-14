//! Label + control row for use inside form sections.
//!
//! ```ignore
//! FormRow::new("Log level").show(ui, |ui| { picker.show(ui); });
//! ```

use crate::colors;
use crate::theme::Layout;

pub struct FormRow<'a> {
    label: &'a str,
}

impl<'a> FormRow<'a> {
    pub fn new(label: &'a str) -> Self {
        Self { label }
    }

    pub fn show(
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
                egui::FontId::proportional(13.0),
                p.text_primary,
            );
        }

        // Control on the right — use a child ui right-aligned within the row.
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
