//! Styled context menu (right-click menu) helper.
//!
//! ```ignore
//! let items = [
//!     MenuItem::new("Copy"),
//!     MenuItem::new("Delete").destructive(true),
//! ];
//! if let Some(idx) = context_menu(&response, &items) {
//!     match idx { ... }
//! }
//! ```

use crate::colors;

pub struct MenuItem<'a> {
    pub label: &'a str,
    pub icon: Option<&'a str>,
    pub destructive: bool,
    pub shortcut: Option<&'a str>,
}

impl<'a> MenuItem<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            icon: None,
            destructive: false,
            shortcut: None,
        }
    }

    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn destructive(mut self, d: bool) -> Self {
        self.destructive = d;
        self
    }

    pub fn shortcut(mut self, s: &'a str) -> Self {
        self.shortcut = Some(s);
        self
    }
}

/// Show a styled context menu on the given response. Returns the index
/// of the clicked item, if any.
pub fn context_menu(response: &egui::Response, items: &[MenuItem<'_>]) -> Option<usize> {
    let mut clicked = None;

    response.context_menu(|ui| {
        let p = colors::palette(ui);

        for (i, item) in items.iter().enumerate() {
            let text_color = if item.destructive {
                p.destructive
            } else {
                p.text_primary
            };

            let label = if let Some(icon) = item.icon {
                format!("{icon}  {}", item.label)
            } else {
                item.label.to_string()
            };

            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new(&label).size(13.0).color(text_color),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .frame(false)
                        .min_size(egui::vec2(ui.available_width(), 28.0)),
                    )
                    .clicked()
                {
                    clicked = Some(i);
                    ui.close_menu();
                }

                if let Some(shortcut) = item.shortcut {
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            ui.label(
                                egui::RichText::new(shortcut)
                                    .size(11.0)
                                    .color(p.text_muted),
                            );
                        },
                    );
                }
            });
        }
    });

    clicked
}
