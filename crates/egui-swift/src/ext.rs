//! Extension trait on `egui::Ui` for ergonomic access to palette and layout.
//!
//! ```ignore
//! use egui_swift::prelude::*;
//!
//! fn my_view(ui: &mut egui::Ui) {
//!     let p = ui.palette();                  // no more `colors::palette(ui)`
//!
//!     ui.centered_content(640.0, |ui| {      // no more 3-line centering math
//!         Label::heading("Title").show(ui);
//!     });
//! }
//! ```

use crate::colors::{self, Palette};

/// Convenience methods added to `egui::Ui`.
pub trait UiExt {
    /// Get the current color palette (auto-detects dark/light mode).
    fn palette(&self) -> Palette;

    /// Render content centered horizontally with a maximum width.
    /// Adds symmetric padding on both sides.
    fn centered_content(&mut self, max_width: f32, content: impl FnOnce(&mut egui::Ui));
}

impl UiExt for egui::Ui {
    fn palette(&self) -> Palette {
        colors::palette(self)
    }

    fn centered_content(&mut self, max_width: f32, content: impl FnOnce(&mut egui::Ui)) {
        let available_width = self.available_width();
        let content_width = available_width.min(max_width);
        let side_padding = ((available_width - content_width) / 2.0).max(24.0);

        self.horizontal(|ui| {
            ui.add_space(side_padding);
            ui.vertical(|ui| {
                ui.set_max_width(content_width);
                content(ui);
            });
            ui.add_space(side_padding);
        });
    }
}

/// Convenience methods added to `egui::Context`.
pub trait CtxExt {
    /// Get the current color palette from an egui context.
    fn palette(&self) -> Palette;
}

impl CtxExt for egui::Context {
    fn palette(&self) -> Palette {
        colors::palette_from_ctx(self)
    }
}
