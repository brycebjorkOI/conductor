use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::session;
use conductor_core::state::SettingsTab;
use egui_swift::prelude::*;

use crate::bridge::SharedState;

/// Persistent sidebar state (lives across frames).
pub struct SidebarState {
    pub search_query: String,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
        }
    }
}

pub fn show(
    ui: &mut egui::Ui,
    shared: &SharedState,
    tx: &mpsc::UnboundedSender<Action>,
    sidebar_state: &mut SidebarState,
) {
    let state = shared.read();
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
        let _ = tx.send(Action::NewSession);
    }

    ui.add_space(6.0);

    ui.horizontal(|ui| {
        ui.add_space(8.0);
        ui.scope(|ui| {
            ui.set_width(ui.available_width() - 16.0);
            SearchField::new(&mut sidebar_state.search_query).show(ui);
        });
    });

    ui.add_space(8.0);

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
        let _ = tx.send(Action::OpenSettings {
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

    ui.add_space(8.0);
    Divider::new().inset(8.0).show(ui);
    ui.add_space(4.0);

    SectionHeader::new("Recent").show(ui);
    ui.add_space(4.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            let query = sidebar_state.search_query.to_lowercase();
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
                    let _ = tx.send(Action::SwitchSession {
                        session_id: id.clone(),
                    });
                }
            }
        });

    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        ui.add_space(8.0);
        let resp = UserProfile::new("User")
            .version(concat!("v", env!("CARGO_PKG_VERSION")))
            .show(ui);
        if resp.settings_clicked {
            let _ = tx.send(Action::OpenSettings {
                tab: Some(SettingsTab::General),
            });
        }
        Divider::new().inset(8.0).show(ui);
        ui.add_space(4.0);
    });
}
