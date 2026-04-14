//! Extension traits for `egui::Ui`, `egui::Context`, and `egui::Color32`.
//!
//! ```ignore
//! use egui_swift::prelude::*;
//!
//! fn my_view(ui: &mut egui::Ui) {
//!     let p = ui.palette();
//!     let subtle = p.accent.opacity(0.1);   // semi-transparent color
//!     ui.centered_content(640.0, |ui| { ... });
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

/// SwiftUI-style `.opacity()` on colors.
///
/// ```ignore
/// let subtle_bg = p.accent.opacity(0.1);
/// let dimmed = p.text_primary.opacity(0.5);
/// ```
pub trait ColorExt {
    /// Create a semi-transparent version of this color.
    ///
    /// `opacity` is 0.0 (fully transparent) to 1.0 (fully opaque).
    /// Works correctly with opaque colors from the palette.
    fn opacity(self, opacity: f32) -> egui::Color32;
}

impl ColorExt for egui::Color32 {
    fn opacity(self, opacity: f32) -> egui::Color32 {
        let a = (opacity.clamp(0.0, 1.0) * 255.0) as u8;
        egui::Color32::from_rgba_unmultiplied(self.r(), self.g(), self.b(), a)
    }
}
