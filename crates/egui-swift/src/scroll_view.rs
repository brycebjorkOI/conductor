//! SwiftUI-style ScrollView with thin overlay scrollbar styling.

use crate::colors;
use crate::ext::ColorExt;

/// A scrollable container with SwiftUI naming and macOS-style thin scrollbars.
pub struct ScrollView {
    horizontal: bool,
    vertical: bool,
    stick_to_bottom: bool,
}

impl ScrollView {
    pub fn vertical() -> Self {
        Self {
            horizontal: false,
            vertical: true,
            stick_to_bottom: false,
        }
    }

    pub fn horizontal() -> Self {
        Self {
            horizontal: true,
            vertical: false,
            stick_to_bottom: false,
        }
    }

    pub fn both() -> Self {
        Self {
            horizontal: true,
            vertical: true,
            stick_to_bottom: false,
        }
    }

    pub fn stick_to_bottom(mut self, stick: bool) -> Self {
        self.stick_to_bottom = stick;
        self
    }

    pub fn show(self, ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) -> egui::Response {
        let p = colors::palette(ui);

        // Style thin overlay scrollbars before creating the ScrollArea.
        let prev_spacing = ui.spacing().scroll.bar_width;
        ui.spacing_mut().scroll.bar_width = 6.0;
        ui.spacing_mut().scroll.bar_inner_margin = 2.0;
        ui.spacing_mut().scroll.bar_outer_margin = 2.0;

        // Make scrollbar handles semi-transparent and rounded.
        let vis = &mut ui.visuals_mut().widgets;
        vis.inactive.bg_fill = p.text_muted.opacity(0.2);
        vis.hovered.bg_fill = p.text_muted.opacity(0.4);
        vis.active.bg_fill = p.text_muted.opacity(0.5);

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

        // Restore previous spacing.
        ui.spacing_mut().scroll.bar_width = prev_spacing;

        ui.interact(
            output.inner_rect,
            ui.auto_id_with("scroll_view"),
            egui::Sense::hover(),
        )
    }
}
