//! SwiftUI-style Alert / ConfirmationDialog.
//!
//! ```ignore
//! let action = Alert::new("Delete item?", &mut show_alert)
//!     .message("This action cannot be undone.")
//!     .destructive_action("Delete")
//!     .cancel()
//!     .show(ctx);
//!
//! match action {
//!     AlertAction::Destructive => { delete_item(); }
//!     AlertAction::Cancel | AlertAction::None => {}
//!     AlertAction::Primary => {}
//! }
//! ```

use crate::button::{Button, ButtonStyle};
use crate::colors;
use crate::divider::Divider;
use crate::ext::ColorExt;
use crate::helpers;
use crate::typography::Font;

/// The action the user took on the alert.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertAction {
    /// Alert is not visible or no action taken this frame.
    None,
    /// The primary (non-destructive) action was clicked.
    Primary,
    /// The destructive action was clicked.
    Destructive,
    /// Cancel was clicked (or backdrop dismissed).
    Cancel,
}

pub struct Alert<'a> {
    title: &'a str,
    open: &'a mut bool,
    message: Option<&'a str>,
    primary_label: Option<&'a str>,
    destructive_label: Option<&'a str>,
    cancel: bool,
}

const ANIMATION_SECS: f32 = 0.15;
const ALERT_WIDTH: f32 = 320.0;

impl<'a> Alert<'a> {
    pub fn new(title: &'a str, open: &'a mut bool) -> Self {
        Self {
            title,
            open,
            message: None,
            primary_label: None,
            destructive_label: None,
            cancel: false,
        }
    }

    /// Optional explanatory message below the title.
    pub fn message(mut self, msg: &'a str) -> Self {
        self.message = Some(msg);
        self
    }

    /// Add a primary (non-destructive) action button.
    pub fn primary_action(mut self, label: &'a str) -> Self {
        self.primary_label = Some(label);
        self
    }

    /// Add a destructive action button (red).
    pub fn destructive_action(mut self, label: &'a str) -> Self {
        self.destructive_label = Some(label);
        self
    }

    /// Add a cancel button (always shown last).
    pub fn cancel(mut self) -> Self {
        self.cancel = true;
        self
    }

    pub fn show(self, ctx: &egui::Context) -> AlertAction {
        let p = colors::palette_from_ctx(ctx);
        let id = egui::Id::new("alert").with(self.title);
        let t = ctx.animate_bool_with_time(id, *self.open, ANIMATION_SECS);

        if t <= 0.0 {
            return AlertAction::None;
        }

        let mut action = AlertAction::None;
        let screen = ctx.screen_rect();

        // Backdrop.
        egui::Area::new(id.with("backdrop"))
            .fixed_pos(screen.min)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(screen.size(), egui::Sense::click());

                if response.clicked() {
                    *self.open = false;
                    action = AlertAction::Cancel;
                }

                ui.painter().rect_filled(
                    rect,
                    egui::CornerRadius::ZERO,
                    egui::Color32::BLACK.opacity(0.3 * t),
                );
            });

        // Alert card.
        let alert_height = 160.0; // approximate — egui will expand as needed
        egui::Area::new(id.with("card"))
            .fixed_pos(egui::pos2(
                (screen.width() - ALERT_WIDTH) / 2.0,
                (screen.height() - alert_height) / 2.0,
            ))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                helpers::paint_shadow(
                    ui,
                    egui::Rect::from_min_size(
                        ui.cursor().min,
                        egui::vec2(ALERT_WIDTH, alert_height),
                    ),
                    egui::CornerRadius::same(14),
                    6.0,
                    p.shadow,
                );

                egui::Frame::NONE
                    .fill(p.surface_raised)
                    .corner_radius(egui::CornerRadius::same(14))
                    .stroke(egui::Stroke::new(0.5, p.border))
                    .inner_margin(egui::Margin::symmetric(24, 20))
                    .show(ui, |ui| {
                        ui.set_width(ALERT_WIDTH - 48.0);

                        // Title.
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new(self.title)
                                    .size(Font::Headline.size())
                                    .strong()
                                    .color(p.text_primary),
                            );
                        });

                        // Message.
                        if let Some(msg) = self.message {
                            ui.add_space(8.0);
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    egui::RichText::new(msg)
                                        .size(Font::Callout.size())
                                        .color(p.text_secondary),
                                );
                            });
                        }

                        ui.add_space(16.0);
                        Divider::new().show(ui);
                        ui.add_space(8.0);

                        // Buttons — centered.
                        ui.vertical_centered(|ui| {
                            if let Some(label) = self.destructive_label {
                                if Button::new(label)
                                    .style(ButtonStyle::Destructive)
                                    .show(ui)
                                    .clicked()
                                {
                                    *self.open = false;
                                    action = AlertAction::Destructive;
                                }
                                ui.add_space(4.0);
                            }

                            if let Some(label) = self.primary_label {
                                if Button::new(label)
                                    .style(ButtonStyle::BorderedProminent)
                                    .show(ui)
                                    .clicked()
                                {
                                    *self.open = false;
                                    action = AlertAction::Primary;
                                }
                                ui.add_space(4.0);
                            }

                            if self.cancel {
                                if Button::new("Cancel")
                                    .style(ButtonStyle::Bordered)
                                    .show(ui)
                                    .clicked()
                                {
                                    *self.open = false;
                                    action = AlertAction::Cancel;
                                }
                            }
                        });
                    });
            });

        action
    }
}
