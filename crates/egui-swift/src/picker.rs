//! SwiftUI-style Picker (styled combo box / dropdown).
//!
//! ```ignore
//! let options = [("trace", "Trace"), ("debug", "Debug"), ("info", "Info")];
//! Picker::new("Log Level", &mut level, &options).show(ui);
//! ```

use crate::colors;
use crate::theme::Layout;

pub struct Picker<'a, T: PartialEq + Clone + 'a> {
    label: &'a str,
    selected: &'a mut T,
    options: &'a [(T, &'a str)],
}

impl<'a, T: PartialEq + Clone + std::fmt::Debug> Picker<'a, T> {
    pub fn new(label: &'a str, selected: &'a mut T, options: &'a [(T, &'a str)]) -> Self {
        Self {
            label,
            selected,
            options,
        }
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
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

        // ComboBox on the right.
        let selected_text = self
            .options
            .iter()
            .find(|(v, _)| v == self.selected)
            .map(|(_, name)| *name)
            .unwrap_or("—");

        let combo_rect = egui::Rect::from_min_max(
            egui::pos2(rect.center().x, rect.min.y),
            rect.max,
        );
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(combo_rect));
        child.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            egui::ComboBox::from_id_salt(self.label)
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for (value, name) in self.options {
                        ui.selectable_value(self.selected, value.clone(), *name);
                    }
                });
        });

        response
    }
}
