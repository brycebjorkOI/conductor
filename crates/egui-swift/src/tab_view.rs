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

/// A single tab definition.
struct Tab<'a> {
    id: &'a str,
    label: &'a str,
    sf_symbol: &'a str,
    content: Box<dyn FnOnce(&mut egui::Ui) + 'a>,
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
            content: Box::new(content),
        });
        self
    }

    pub fn show(mut self, ui: &mut egui::Ui) {
        let p = colors::palette(ui);

        // If no tab is selected yet, select the first one.
        if self.selected.is_empty() {
            if let Some(first) = self.tabs.first() {
                *self.selected = first.id.to_string();
            }
        }

        let selected_id = self.selected.clone();

        // Find and remove the selected tab's content to call it.
        let tab_idx = self
            .tabs
            .iter()
            .position(|t| t.id == selected_id)
            .unwrap_or(0);

        // -- Content area (everything above the tab bar) --
        let content_tab = self.tabs.remove(tab_idx);
        let remaining_height = ui.available_height() - 56.0; // reserve tab bar height
        let (content_rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), remaining_height.max(0.0)),
            egui::Sense::hover(),
        );

        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(content_rect));
        (content_tab.content)(&mut child);

        // Re-insert a dummy so tab_ids stay aligned for the bar rendering.
        // We only need id/label/sf_symbol for rendering the bar.

        // -- Tab bar --
        Divider::new().show(ui);

        let bar_width = ui.available_width();
        let tab_count = self.tabs.len() + 1; // +1 for the removed one
        let tab_width = bar_width / tab_count as f32;

        // Rebuild the full tab info list (we consumed one for rendering).
        let mut tab_info: Vec<(&str, &str, &str)> = Vec::new();
        for (i, t) in self.tabs.iter().enumerate() {
            if i == tab_idx {
                tab_info.push((content_tab.id, content_tab.label, content_tab.sf_symbol));
            }
            tab_info.push((t.id, t.label, t.sf_symbol));
        }
        if tab_idx >= self.tabs.len() {
            tab_info.push((content_tab.id, content_tab.label, content_tab.sf_symbol));
        }

        ui.horizontal(|ui| {
            for (id, label, sf_name) in &tab_info {
                let is_selected = *id == selected_id;
                let icon_glyph = image::sf_symbol(sf_name);
                let icon_str = if icon_glyph.is_empty() {
                    format!("[{sf_name}]")
                } else {
                    icon_glyph.to_string()
                };

                let color = if is_selected {
                    p.accent
                } else {
                    p.text_muted
                };

                let (rect, response) = ui.allocate_exact_size(
                    egui::vec2(tab_width, 50.0),
                    egui::Sense::click(),
                );

                if response.clicked() {
                    *self.selected = id.to_string();
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

                    // Icon centered above label.
                    let icon_pos = egui::pos2(rect.center().x, rect.min.y + 14.0);
                    painter.text(
                        icon_pos,
                        egui::Align2::CENTER_CENTER,
                        &icon_str,
                        egui::FontId::proportional(20.0),
                        color,
                    );

                    // Label below icon.
                    let label_pos = egui::pos2(rect.center().x, rect.max.y - 10.0);
                    painter.text(
                        label_pos,
                        egui::Align2::CENTER_CENTER,
                        *label,
                        egui::FontId::proportional(Font::Footnote.size()),
                        color,
                    );
                }
            }
        });
    }
}
