//! Uppercase muted section header (e.g. "RECENT").
//!
//! ```no_run
//! # let ui: &mut egui::Ui = todo!();
//! conductor_ui::section_header::SectionHeader::new("RECENT").show(ui);
//! ```

use crate::colors;

pub struct SectionHeader<'a> {
    label: &'a str,
}

impl<'a> SectionHeader<'a> {
    pub fn new(label: &'a str) -> Self {
        Self { label }
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let p = colors::palette(ui);
        ui.horizontal(|ui| {
            ui.add_space(14.0);
            ui.label(
                egui::RichText::new(self.label.to_uppercase())
                    .size(10.5)
                    .strong()
                    .color(p.text_muted),
            )
        })
        .inner
    }
}
