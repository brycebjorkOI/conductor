//! Styled text input with rounded borders and focus ring.
//!
//! ```ignore
//! TextField::new(&mut text).placeholder("Enter name").label("Job Name").show(ui);
//! ```

use crate::colors;
use crate::theme::Layout;

pub struct TextField<'a> {
    text: &'a mut String,
    placeholder: &'a str,
    label: Option<&'a str>,
    multiline: Option<usize>,
    monospace: bool,
}

impl<'a> TextField<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            placeholder: "",
            label: None,
            multiline: None,
            monospace: false,
        }
    }

    pub fn placeholder(mut self, p: &'a str) -> Self {
        self.placeholder = p;
        self
    }

    pub fn label(mut self, l: &'a str) -> Self {
        self.label = Some(l);
        self
    }

    /// Enable multiline mode with the given number of visible rows.
    pub fn multiline(mut self, rows: usize) -> Self {
        self.multiline = Some(rows);
        self
    }

    pub fn monospace(mut self, m: bool) -> Self {
        self.monospace = m;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let p = colors::palette(ui);

        if let Some(label) = self.label {
            ui.label(
                egui::RichText::new(label)
                    .size(13.0)
                    .color(p.text_primary),
            );
            ui.add_space(4.0);
        }

        let mut edit = if let Some(rows) = self.multiline {
            egui::TextEdit::multiline(self.text).desired_rows(rows)
        } else {
            egui::TextEdit::singleline(self.text)
        };

        edit = edit.hint_text(
            egui::RichText::new(self.placeholder).color(p.text_placeholder),
        );

        if self.monospace {
            edit = edit.font(egui::TextStyle::Monospace);
        }

        // Wrap in a styled frame.
        let frame = egui::Frame::NONE
            .fill(p.input_bg)
            .corner_radius(egui::CornerRadius::same(Layout::CONTROL_RADIUS as u8))
            .stroke(egui::Stroke::new(1.0, p.border))
            .inner_margin(egui::Margin::symmetric(10, 6));

        let mut response = None;
        frame.show(ui, |ui| {
            response = Some(ui.add(edit));
        });
        let resp = response.unwrap();

        // Accent focus ring.
        if resp.has_focus() {
            let rect = resp.rect.expand(1.0);
            ui.painter().rect_stroke(
                rect,
                egui::CornerRadius::same(Layout::CONTROL_RADIUS as u8),
                egui::Stroke::new(2.0, p.accent),
                egui::StrokeKind::Outside,
            );
        }

        resp
    }
}
