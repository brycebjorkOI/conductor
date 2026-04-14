//! Rounded search input field with a leading magnifying glass icon.
//!
//! ```no_run
//! # let ui: &mut egui::Ui = todo!();
//! let mut query = String::new();
//! conductor_ui::search_field::SearchField::new(&mut query).show(ui);
//! ```

use crate::colors;

pub struct SearchField<'a> {
    text: &'a mut String,
    placeholder: &'a str,
}

impl<'a> SearchField<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            placeholder: "Search",
        }
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let p = colors::palette(ui);

        let frame = egui::Frame::NONE
            .fill(p.input_bg)
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(8, 5));

        let outer = frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("\u{1f50d}")
                        .size(13.0)
                        .color(p.text_muted),
                );
                ui.add_sized(
                    egui::vec2(ui.available_width(), 18.0),
                    egui::TextEdit::singleline(self.text)
                        .hint_text(
                            egui::RichText::new(self.placeholder)
                                .color(p.text_placeholder),
                        )
                        .frame(false)
                        .text_color(p.text_primary)
                        .margin(egui::Margin::ZERO),
                )
            })
            .inner
        });

        outer.response
    }
}
