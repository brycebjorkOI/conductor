//! SwiftUI-style ScrollView wrapper around `egui::ScrollArea`.
//!
//! ```ignore
//! ScrollView::vertical().show(ui, |ui| {
//!     for item in &items {
//!         Label::new(item).show(ui);
//!     }
//! });
//! ```

/// A scrollable container with SwiftUI naming and sensible defaults.
pub struct ScrollView {
    horizontal: bool,
    vertical: bool,
    stick_to_bottom: bool,
}

impl ScrollView {
    /// Vertical scroll (most common).
    pub fn vertical() -> Self {
        Self {
            horizontal: false,
            vertical: true,
            stick_to_bottom: false,
        }
    }

    /// Horizontal scroll.
    pub fn horizontal() -> Self {
        Self {
            horizontal: true,
            vertical: false,
            stick_to_bottom: false,
        }
    }

    /// Both axes.
    pub fn both() -> Self {
        Self {
            horizontal: true,
            vertical: true,
            stick_to_bottom: false,
        }
    }

    /// Auto-scroll to bottom when new content appears (useful for chat).
    pub fn stick_to_bottom(mut self, stick: bool) -> Self {
        self.stick_to_bottom = stick;
        self
    }

    pub fn show(self, ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) -> egui::Response {
        let mut area = if self.horizontal && self.vertical {
            egui::ScrollArea::both()
        } else if self.horizontal {
            egui::ScrollArea::horizontal()
        } else {
            egui::ScrollArea::vertical()
        };

        area = area.auto_shrink([false; 2]);

        if self.stick_to_bottom {
            area = area.stick_to_bottom(true);
        }

        let output = area.show(ui, |ui| {
            content(ui);
        });

        ui.interact(output.inner_rect, ui.auto_id_with("scroll_view"), egui::Sense::hover())
    }
}
