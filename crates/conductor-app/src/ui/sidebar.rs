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

        Spacer::fixed(6.0).show(ui);

        HStack::new().show(ui, |ui| {
            Spacer::fixed(8.0).show(ui);
            ui.scope(|ui| {
                ui.set_width(ui.available_width() - 16.0);
                SearchField::new(&mut self.search_query).show(ui);
            });
        });

        Spacer::fixed(8.0).show(ui);

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
        if NavRow::new("Notifications")
            .icon(icons::BELL)
            .show(ui)
            .clicked()
        {}
        if NavRow::new("Trash")
            .icon(icons::WASTEBASKET)
            .show(ui)
            .clicked()
        {}

        Spacer::fixed(8.0).show(ui);
        Divider::new().inset(8.0).show(ui);
        Spacer::fixed(4.0).show(ui);

        SectionHeader::new("Recent").show(ui);
        Spacer::fixed(4.0).show(ui);

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
            Spacer::fixed(8.0).show(ui);
            let resp = UserProfile::new("User")
                .version(concat!("v", env!("CARGO_PKG_VERSION")))
                .show(ui);
            if resp.settings_clicked {
                let _ = self.tx.send(Action::OpenSettings {
                    tab: Some(SettingsTab::General),
                });
            }
            Divider::new().inset(8.0).show(ui);
            Spacer::fixed(4.0).show(ui);
        });
    }
}
