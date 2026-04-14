//! SwiftUI-style Form Section with optional header and footer.
//!
//! ```ignore
//! FormSection::new().header("Behavior").show(ui, |ui| {
//!     Toggle::new(&mut val).label("Auto-hide").show(ui);
//! });
//! ```

use crate::colors;
use crate::theme::Layout;

pub struct FormSection<'a> {
    header: Option<&'a str>,
    footer: Option<&'a str>,
}

impl<'a> FormSection<'a> {
    pub fn new() -> Self {
        Self {
            header: None,
            footer: None,
        }
    }

    pub fn header(mut self, h: &'a str) -> Self {
        self.header = Some(h);
        self
    }

    pub fn footer(mut self, f: &'a str) -> Self {
        self.footer = Some(f);
        self
    }

    pub fn show(
        self,
        ui: &mut egui::Ui,
        content: impl FnOnce(&mut egui::Ui),
    ) -> egui::Response {
        let p = colors::palette(ui);

        // Header (uppercase, small, muted — like macOS Settings sections).
        if let Some(header) = self.header {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new(header.to_uppercase())
                        .size(Layout::CAPTION_FONT_SIZE)
                        .strong()
                        .color(p.text_muted),
                );
            });
            ui.add_space(4.0);
        }

        // Inset grouped background.
        let rounding = egui::CornerRadius::same(Layout::CARD_RADIUS as u8);
        let resp = egui::Frame::NONE
            .fill(p.surface_raised)
            .corner_radius(rounding)
            .stroke(egui::Stroke::new(0.5, p.border_subtle))
            .inner_margin(egui::Margin::symmetric(16, 8))
            .show(ui, |ui| {
                content(ui);
            })
            .response;

        // Footer.
        if let Some(footer) = self.footer {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new(footer)
                        .size(Layout::CAPTION_FONT_SIZE)
                        .color(p.text_muted),
                );
            });
        }

        resp
    }
}

impl<'a> Default for FormSection<'a> {
    fn default() -> Self {
        Self::new()
    }
}
