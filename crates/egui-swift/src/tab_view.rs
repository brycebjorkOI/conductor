//! SwiftUI-style TabView — bottom tab bar with icon + label tabs.
//!
//! ```ignore
//! TabView::new(&mut selected_tab)
//!     .tab("home", "Home", "house", |ui| { home_view(ui); })
//!     .tab("settings", "Settings", "gear", |ui| { settings_view(ui); })
//!     .show(ui);
//! ```

use crate::colors;
use crate::divider::Divider;
use crate::ext::ColorExt;
use crate::image;
use crate::typography::Font;

const TAB_BAR_HEIGHT: f32 = 56.0;
const TAB_ITEM_HEIGHT: f32 = 50.0;

/// A single tab definition.
struct Tab<'a> {
    id: &'a str,
    label: &'a str,
    sf_symbol: &'a str,
    content: Option<Box<dyn FnOnce(&mut egui::Ui) + 'a>>,
}

/// SwiftUI-style tab bar interface.
pub struct TabView<'a> {
    selected: &'a mut String,
    tabs: Vec<Tab<'a>>,
}

impl<'a> TabView<'a> {
    pub fn new(selected: &'a mut String) -> Self {
        Self {
            selected,
            tabs: Vec::new(),
        }
    }

    /// Add a tab with an ID, display label, SF Symbol name, and content closure.
    pub fn tab(
        mut self,
        id: &'a str,
        label: &'a str,
        sf_symbol: &'a str,
        content: impl FnOnce(&mut egui::Ui) + 'a,
    ) -> Self {
        self.tabs.push(Tab {
            id,
            label,
            sf_symbol,
            content: Some(Box::new(content)),
        });
        self
    }

    pub fn show(mut self, ui: &mut egui::Ui) {
        if self.tabs.is_empty() {
            return;
        }

        let p = colors::palette(ui);

        // Default to first tab if selected doesn't match any tab.
        let has_match = self.tabs.iter().any(|t| t.id == self.selected.as_str());
        if !has_match {
            *self.selected = self.tabs[0].id.to_string();
        }

        let selected_id = self.selected.clone();

        // Find the selected tab and take its content closure (leaving None).
        let tab_idx = self
            .tabs
            .iter()
            .position(|t| t.id == selected_id)
            .unwrap_or(0);
        let content_fn = self.tabs[tab_idx].content.take();

        // -- Content area --
        if let Some(content) = content_fn {
            let remaining_height = ui.available_height() - TAB_BAR_HEIGHT;
            let (content_rect, _) = ui.allocate_exact_size(
                egui::vec2(ui.available_width(), remaining_height.max(0.0)),
                egui::Sense::hover(),
            );
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(content_rect));
            content(&mut child);
        }

        // -- Tab bar --
        Divider::new().show(ui);

        let bar_width = ui.available_width();
        let tab_count = self.tabs.len();
        let tab_width = bar_width / tab_count as f32;

        ui.horizontal(|ui| {
            for tab in &self.tabs {
                let is_selected = tab.id == selected_id;
                let icon_glyph = image::sf_symbol(tab.sf_symbol);
                let icon_str = if icon_glyph.is_empty() {
                    format!("[{}]", tab.sf_symbol)
                } else {
                    icon_glyph.to_string()
                };

                let color = if is_selected {
                    p.accent
                } else {
                    p.text_muted
                };

                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(tab_width, TAB_ITEM_HEIGHT),
                    egui::Sense::click(),
                );

                if response.clicked() {
                    *self.selected = tab.id.to_string();
                }

                if response.hovered() && !is_selected {
                    ui.painter().rect_filled(
                        rect,
                        egui::CornerRadius::ZERO,
                        p.hover_bg.opacity(0.5),
                    );
                }

                if ui.is_rect_visible(rect) {
                    let painter = ui.painter();

                    painter.text(
                        egui::pos2(rect.center().x, rect.min.y + 14.0),
                        egui::Align2::CENTER_CENTER,
                        &icon_str,
                        egui::FontId::proportional(20.0),
                        color,
                    );

                    painter.text(
                        egui::pos2(rect.center().x, rect.max.y - 10.0),
                        egui::Align2::CENTER_CENTER,
                        tab.label,
                        egui::FontId::proportional(Font::Footnote.size()),
                        color,
                    );
                }
            }
        });
    }
}
