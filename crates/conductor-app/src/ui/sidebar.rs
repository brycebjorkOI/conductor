use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::session;
use conductor_core::state::SettingsTab;
use egui_swift::prelude::*;

use crate::bridge::SharedState;

/// Sidebar view with conversation list, navigation, and user profile.
pub struct SidebarView {
    shared: SharedState,
    tx: mpsc::UnboundedSender<Action>,
    pub search_query: String,
}

impl SidebarView {
    pub fn new(shared: SharedState, tx: mpsc::UnboundedSender<Action>) -> Self {
        Self {
            shared,
            tx,
            search_query: String::new(),
        }
    }
}

impl View for SidebarView {
    fn body(&mut self, ui: &mut egui::Ui) {
        let state = self.shared.read();
        let active_sid = state.active_session_id.clone();

        let mut sessions: Vec<(String, String)> = state
            .sessions
            .iter()
            .map(|(id, s)| {
                let name = s
                    .display_name
                    .clone()
                    .unwrap_or_else(|| session::auto_display_name(s));
                (id.clone(), name)
            })
            .collect();
        sessions.sort_by(|a, b| a.1.cmp(&b.1));
        drop(state);

        if NavRow::new("New Conversation")
            .icon(icons::PLUS)
            .show(ui)
            .clicked()
        {
            let _ = self.tx.send(Action::NewSession);
        }

        egui_swift::spacer!(ui, 6.0);

        egui_swift::hstack!(ui, {
            egui_swift::spacer!(ui, 8.0);
            ui.scope(|ui| {
                ui.set_width(ui.available_width() - 16.0);
                SearchField::new(&mut self.search_query).show(ui);
            });
        });

        egui_swift::spacer!(ui, 8.0);

        if NavRow::new("Chats")
            .icon(icons::SPEECH_BALLOON)
            .active(true)
            .show(ui)
            .clicked()
        {}
        if NavRow::new("Projects")
            .icon(icons::FOLDER)
            .show(ui)
            .clicked()
        {}
        if NavRow::new("Schedules")
            .icon(icons::CALENDAR)
            .show(ui)
            .clicked()
        {
            let _ = self.tx.send(Action::OpenSettings {
                tab: Some(SettingsTab::Schedules),
            });
        }
        {
            let unread = {
                let state = self.shared.read();
                state.notifications.iter().filter(|n| !n.dismissed).count() as u32
            };
            let mut row = NavRow::new("Notifications").icon(icons::BELL);
            if unread > 0 {
                row = row.badge(unread);
            }
            if row.show(ui).clicked() {
                let _ = self.tx.send(Action::ToggleNotifications);
            }
        }
        if NavRow::new("Trash")
            .icon(icons::WASTEBASKET)
            .show(ui)
            .clicked()
        {}

        egui_swift::spacer!(ui, 8.0);
        Divider::new().inset(8.0).show(ui);
        egui_swift::spacer!(ui, 4.0);

        SectionHeader::new("Recent").show(ui);
        egui_swift::spacer!(ui, 4.0);

        ScrollView::vertical().show(ui, |ui| {
            let query = self.search_query.to_lowercase();
            for (id, name) in &sessions {
                if !query.is_empty() && !name.to_lowercase().contains(&query) {
                    continue;
                }
                let is_active = id == &active_sid;
                if ConversationItem::new(id, name)
                    .active(is_active)
                    .show(ui)
                    .clicked()
                {
                    let _ = self.tx.send(Action::SwitchSession {
                        session_id: id.clone(),
                    });
                }
            }
        });

        Spacer::bottom(ui, |ui| {
            egui_swift::spacer!(ui, 8.0);
            let resp = UserProfile::new("User")
                .version(concat!("v", env!("CARGO_PKG_VERSION")))
                .show(ui);
            if resp.settings_clicked {
                let _ = self.tx.send(Action::OpenSettings {
                    tab: Some(SettingsTab::General),
                });
            }
            Divider::new().inset(8.0).show(ui);
            egui_swift::spacer!(ui, 4.0);
        });
    }
}
