//! SwiftUI-style List — scrollable list with dividers and selection.
//!
//! ```ignore
//! List::new()
//!     .inset_grouped()
//!     .show(ui, |list| {
//!         for (i, item) in items.iter().enumerate() {
//!             list.row(i == selected_index, |ui| {
//!                 HStack::new().show(ui, |ui| {
//!                     Image::system_name("folder").show(ui);
//!                     Text::new(&item.name).show(ui);
//!                 });
//!             });
//!         }
//!     });
//! ```

use crate::colors;
use crate::divider::Divider;
use crate::theme::Layout;

/// List appearance style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListStyle {
    /// Plain list with hairline dividers.
    Plain,
    /// Grouped with rounded background (like macOS Settings).
    InsetGrouped,
}

/// A styled scrollable list container.
pub struct List {
    style: ListStyle,
    divider_inset: f32,
}

impl List {
    pub fn new() -> Self {
        Self {
            style: ListStyle::Plain,
            divider_inset: 0.0,
        }
    }

    /// Use inset-grouped appearance (rounded background, like macOS Settings).
    pub fn inset_grouped(mut self) -> Self {
        self.style = ListStyle::InsetGrouped;
        self
    }

    /// Set left inset for dividers between rows (default 0).
    pub fn divider_inset(mut self, inset: f32) -> Self {
        self.divider_inset = inset;
        self
    }

    pub fn show(self, ui: &mut egui::Ui, content: impl FnOnce(&mut ListBuilder<'_>)) {
        let p = colors::palette(ui);

        match self.style {
            ListStyle::InsetGrouped => {
                egui::Frame::NONE
                    .fill(p.surface_raised)
                    .corner_radius(egui::CornerRadius::same(Layout::CARD_RADIUS as u8))
                    .stroke(egui::Stroke::new(0.5, p.border_subtle))
                    .inner_margin(egui::Margin::symmetric(0, 4))
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .show(ui, |ui| {
                                let mut builder = ListBuilder {
                                    ui,
                                    divider_inset: self.divider_inset,
                                    row_count: 0,
                                    style: self.style,
                                };
                                content(&mut builder);
                            });
                    });
            }
            ListStyle::Plain => {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        let mut builder = ListBuilder {
                            ui,
                            divider_inset: self.divider_inset,
                            row_count: 0,
                            style: self.style,
                        };
                        content(&mut builder);
                    });
            }
        }
    }
}

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder passed into the List content closure for adding rows.
pub struct ListBuilder<'a> {
    ui: &'a mut egui::Ui,
    divider_inset: f32,
    row_count: usize,
    style: ListStyle,
}

impl<'a> ListBuilder<'a> {
    /// Add a row to the list. If `selected` is true, the row gets a
    /// highlighted background.
    ///
    /// Returns the `egui::Response` for the row (supports `.clicked()`).
    pub fn row(
        &mut self,
        selected: bool,
        content: impl FnOnce(&mut egui::Ui),
    ) -> egui::Response {
        let p = colors::palette(self.ui);

        // Divider between rows (not before the first).
        if self.row_count > 0 {
            Divider::new().inset(self.divider_inset).show(self.ui);
        }
        self.row_count += 1;

        let padding = match self.style {
            ListStyle::InsetGrouped => egui::Margin::symmetric(16, 6),
            ListStyle::Plain => egui::Margin::symmetric(0, 4),
        };

        let bg = if selected {
            p.active_bg
        } else {
            egui::Color32::TRANSPARENT
        };

        let frame_resp = egui::Frame::NONE
            .fill(bg)
            .inner_margin(padding)
            .show(self.ui, |ui| {
                content(ui);
            });

        let response = frame_resp.response;

        // Hover highlight.
        if response.hovered() && !selected {
            self.ui.painter().rect_filled(
                response.rect,
                egui::CornerRadius::ZERO,
                p.hover_bg,
            );
        }

        response
    }

    /// Add a plain (non-selectable) row. Sugar for `row(false, content)`.
    pub fn item(&mut self, content: impl FnOnce(&mut egui::Ui)) -> egui::Response {
        self.row(false, content)
    }
}
