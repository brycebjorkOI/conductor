use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::state::NotificationSeverity;
use egui_swift::prelude::*;

use crate::bridge::SharedState;

/// Notifications panel — shows all notifications with dismiss controls.
pub struct NotificationsView {
    shared: SharedState,
    tx: mpsc::UnboundedSender<Action>,
}

impl NotificationsView {
    pub fn new(shared: SharedState, tx: mpsc::UnboundedSender<Action>) -> Self {
        Self { shared, tx }
    }

    /// Count of undismissed notifications (for sidebar badge).
    #[allow(dead_code)]
    pub fn unread_count(&self) -> u32 {
        self.shared
            .read()
            .notifications
            .iter()
            .filter(|n| !n.dismissed)
            .count() as u32
    }
}

impl View for NotificationsView {
    fn body(&mut self, ui: &mut egui::Ui) {
        let p = ui.palette();

        let state = self.shared.read();
        let notifications = state.notifications.clone();
        drop(state);

        // Header.
        egui_swift::hstack!(ui, {
            Label::heading("Notifications").show(ui);
            Spacer::trailing(ui, |ui| {
                if Button::new("Dismiss All")
                    .style(ButtonStyle::Bordered)
                    .small(true)
                    .show(ui)
                    .clicked()
                {
                    let _ = self.tx.send(Action::DismissAllNotifications);
                }
                egui_swift::spacer!(ui, 8.0);
                if Button::new("Close")
                    .style(ButtonStyle::Bordered)
                    .small(true)
                    .show(ui)
                    .clicked()
                {
                    let _ = self.tx.send(Action::ToggleNotifications);
                }
            });
        });

        egui_swift::spacer!(ui, 12.0);

        let active: Vec<_> = notifications.iter().filter(|n| !n.dismissed).collect();
        let dismissed: Vec<_> = notifications.iter().filter(|n| n.dismissed).collect();

        if active.is_empty() && dismissed.is_empty() {
            EmptyState::new("No notifications")
                .icon("\u{1f514}")
                .subtitle("System notifications will appear here.")
                .show(ui);
            return;
        }

        ScrollView::vertical().show(ui, |ui| {
            // Active notifications.
            if !active.is_empty() {
                SectionHeader::new("Active").show(ui);
                egui_swift::spacer!(ui, 4.0);

                for notif in &active {
                    let (icon, border_color) = match notif.severity {
                        NotificationSeverity::Info => ("\u{2139}", p.accent),
                        NotificationSeverity::Warning => ("\u{26a0}", p.status_yellow),
                        NotificationSeverity::Error => ("\u{274c}", p.status_red),
                    };

                    Card::new().border_color(border_color).show(ui, |ui| {
                        egui_swift::hstack!(ui, {
                            Label::new(icon).font(Font::Body).show(ui);

                            VStack::new().spacing(2.0).show(ui, |ui| {
                                Label::new(&notif.title)
                                    .font(Font::Callout)
                                    .bold(true)
                                    .show(ui);
                                Label::new(&notif.body)
                                    .font(Font::Subheadline)
                                    .secondary()
                                    .show(ui);
                                Label::new(&notif.timestamp.format("%b %d, %H:%M").to_string())
                                    .font(Font::Caption)
                                    .muted()
                                    .show(ui);
                            });

                            Spacer::trailing(ui, |ui| {
                                if Button::new("\u{2715}") // ✕
                                    .style(ButtonStyle::Borderless)
                                    .small(true)
                                    .show(ui)
                                    .clicked()
                                {
                                    let _ = self.tx.send(Action::DismissNotification {
                                        id: notif.id.clone(),
                                    });
                                }
                            });
                        });
                    });
                    egui_swift::spacer!(ui, 4.0);
                }
            }

            // Dismissed notifications.
            if !dismissed.is_empty() {
                egui_swift::spacer!(ui, 12.0);
                SectionHeader::new("Dismissed").show(ui);
                egui_swift::spacer!(ui, 4.0);

                for notif in dismissed.iter().rev().take(20) {
                    egui_swift::hstack!(ui, {
                        Label::new(&notif.title)
                            .font(Font::Subheadline)
                            .muted()
                            .show(ui);
                        Spacer::trailing(ui, |ui| {
                            Label::new(&notif.timestamp.format("%b %d").to_string())
                                .font(Font::Caption)
                                .muted()
                                .show(ui);
                        });
                    });
                }
            }
        });
    }
}
