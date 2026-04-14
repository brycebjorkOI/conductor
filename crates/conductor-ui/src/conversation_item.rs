//! Conversation list item — a truncated text row with hover highlight.
//!
//! ```no_run
//! # let ui: &mut egui::Ui = todo!();
//! use conductor_ui::conversation_item::ConversationItem;
//! if ConversationItem::new("id-1", "What is Rust?").active(true).show(ui).clicked() {
//!     // switch to this conversation
//! }
//! ```

use crate::colors;

#[allow(dead_code)]
pub struct ConversationItem<'a> {
    id: &'a str,
    title: &'a str,
    active: bool,
    max_chars: usize,
}

impl<'a> ConversationItem<'a> {
    pub fn new(id: &'a str, title: &'a str) -> Self {
        Self {
            id,
            title,
            active: false,
            max_chars: 30,
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn max_chars(mut self, n: usize) -> Self {
        self.max_chars = n;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let p = colors::palette(ui);
        let row_height = 30.0;

        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), row_height),
            egui::Sense::click(),
        );

        let bg = if self.active {
            p.active_bg
        } else if response.hovered() {
            p.hover_bg
        } else {
            egui::Color32::TRANSPARENT
        };

        let padded = rect.shrink2(egui::vec2(6.0, 1.0));
        ui.painter()
            .rect_filled(padded, egui::CornerRadius::same(4), bg);

        // Truncate title.
        let display = if self.title.len() > self.max_chars {
            format!("{}...", &self.title[..self.max_chars - 3])
        } else {
            self.title.to_string()
        };

        let color = if self.active {
            p.text_primary
        } else {
            p.text_secondary
        };

        ui.painter().text(
            egui::pos2(padded.left() + 14.0, padded.center().y),
            egui::Align2::LEFT_CENTER,
            &display,
            egui::FontId::proportional(13.0),
            color,
        );

        response
    }
}
