//! SwiftUI-style Form container.
//!
//! Wraps child `Section`s (or any content) in a scrollable, inset-grouped
//! appearance matching macOS System Settings.
//!
//! ```ignore
//! Form::new().show(ui, |ui| {
//!     Section::new().header("General").show(ui, |ui| {
//!         Toggle::new(&mut dark_mode).label("Dark mode").show(ui);
//!         Picker::new("Language", &mut lang, &langs).show(ui);
//!     });
//!
//!     Section::new().header("Account").show(ui, |ui| {
//!         TextField::new(&mut name).label("Name").show(ui);
//!     });
//! });
//! ```

use crate::colors;

/// A Form container that applies the inset-grouped appearance to its content.
///
/// In SwiftUI, `Form { Section { ... } }` auto-styles child sections with
/// the grouped list appearance. This component does the same: it sets up
/// a scrollable area with appropriate padding and background, then lets
/// you place `Section`/`FormSection` blocks inside.
pub struct Form {
    max_width: Option<f32>,
}

impl Form {
    pub fn new() -> Self {
        Self { max_width: None }
    }

    /// Constrain the form's content width (useful for wide windows).
    pub fn max_width(mut self, w: f32) -> Self {
        self.max_width = Some(w);
        self
    }

    pub fn show(self, ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) -> egui::Response {
        let p = colors::palette(ui);

        egui::Frame::NONE
            .fill(p.surface)
            .inner_margin(egui::Margin::symmetric(0, 8))
            .show(ui, |ui| {
                if let Some(max_w) = self.max_width {
                    ui.set_max_width(max_w);
                }

                // Set spacing between sections.
                ui.spacing_mut().item_spacing.y = 12.0;

                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        content(ui);
                    });
            })
            .response
    }
}

impl Default for Form {
    fn default() -> Self {
        Self::new()
    }
}
