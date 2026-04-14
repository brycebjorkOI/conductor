//! Centered empty-state view with icon, title, subtitle, and optional action.
//!
//! ```ignore
//! let resp = EmptyState::new("No scheduled jobs")
//!     .icon("📅")
//!     .subtitle("Create one to get started")
//!     .action("New Job")
//!     .show(ui);
//! if resp.action_clicked { ... }
//! ```

use crate::colors;
use crate::button::{Button, ButtonStyle};

pub struct EmptyState<'a> {
    title: &'a str,
    icon: Option<&'a str>,
    subtitle: Option<&'a str>,
    action_label: Option<&'a str>,
}

pub struct EmptyStateResponse {
    pub action_clicked: bool,
}

impl<'a> EmptyState<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            icon: None,
            subtitle: None,
            action_label: None,
        }
    }

    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn subtitle(mut self, s: &'a str) -> Self {
        self.subtitle = Some(s);
        self
    }

    pub fn action(mut self, label: &'a str) -> Self {
        self.action_label = Some(label);
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> EmptyStateResponse {
        let p = colors::palette(ui);
        let mut action_clicked = false;

        ui.add_space(20.0);

        ui.vertical_centered(|ui| {
            if let Some(icon) = self.icon {
                ui.label(
                    egui::RichText::new(icon)
                        .size(40.0)
                        .color(p.text_muted),
                );
                ui.add_space(8.0);
            }

            ui.label(
                egui::RichText::new(self.title)
                    .size(15.0)
                    .color(p.text_muted),
            );

            if let Some(subtitle) = self.subtitle {
                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(subtitle)
                        .size(12.0)
                        .color(p.text_muted),
                );
            }

            if let Some(label) = self.action_label {
                ui.add_space(12.0);
                if Button::new(label)
                    .style(ButtonStyle::BorderedProminent)
                    .show(ui)
                    .clicked()
                {
                    action_clicked = true;
                }
            }
        });

        EmptyStateResponse { action_clicked }
    }
}
