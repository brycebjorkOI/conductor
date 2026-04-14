//! Suggestion chip — a small bordered pill with icon + label.
//!
//! ```no_run
//! # let ui: &mut egui::Ui = todo!();
//! use conductor_ui::suggestion_chip::SuggestionChip;
//! if SuggestionChip::new("Write").icon("\u{270f}").show(ui).clicked() {
//!     // handle
//! }
//! ```

use crate::colors;

pub struct SuggestionChip<'a> {
    label: &'a str,
    icon: Option<&'a str>,
}

impl<'a> SuggestionChip<'a> {
    pub fn new(label: &'a str) -> Self {
        Self { label, icon: None }
    }

    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let p = colors::palette(ui);

        let text = if let Some(icon) = self.icon {
            format!("{icon}  {}", self.label)
        } else {
            self.label.to_string()
        };

        let btn = egui::Button::new(
            egui::RichText::new(text)
                .size(12.5)
                .color(p.text_secondary),
        )
        .fill(egui::Color32::TRANSPARENT)
        .stroke(egui::Stroke::new(0.5, p.border))
        .corner_radius(egui::CornerRadius::same(16));

        ui.add(btn)
    }
}

/// Show a horizontal row of suggestion chips, centered.
pub fn chip_row(ui: &mut egui::Ui, chips: &[(&str, &str)]) {
    ui.horizontal(|ui| {
        for (icon, label) in chips {
            SuggestionChip::new(label).icon(icon).show(ui);
            ui.add_space(4.0);
        }
    });
}
