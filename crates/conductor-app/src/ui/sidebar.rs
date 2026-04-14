use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::session;
use conductor_core::state::SettingsTab;
use conductor_ui::conversation_item::ConversationItem;
use conductor_ui::nav_row::NavRow;
use conductor_ui::search_field::SearchField;
use conductor_ui::section_header::SectionHeader;
use conductor_ui::user_profile::UserProfile;

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

    // -- New Conversation button --
    if NavRow::new("New Conversation").icon("+").show(ui).clicked() {
        let _ = tx.send(Action::NewSession);
    }

    ui.add_space(6.0);

    // -- Search --
    ui.horizontal(|ui| {
        ui.add_space(8.0);
        ui.scope(|ui| {
            ui.set_width(ui.available_width() - 16.0);
            SearchField::new(&mut sidebar_state.search_query).show(ui);
        });
    });

    ui.add_space(8.0);

    // -- Nav items --
    // The screenshot shows: Chats, Projects, Schedules, Notifications, Trash
    if NavRow::new("Chats").icon("\u{1f4ac}").active(true).show(ui).clicked() {
        // already on chats
    }
    if NavRow::new("Projects").icon("\u{1f4c1}").show(ui).clicked() {
        // future
    }
    if NavRow::new("Schedules").icon("\u{1f4c5}").show(ui).clicked() {
        let _ = tx.send(Action::OpenSettings { tab: Some(SettingsTab::Schedules) });
    }
    if NavRow::new("Notifications").icon("\u{1f514}").show(ui).clicked() {
        // future
    }
    if NavRow::new("Trash").icon("\u{1f5d1}").show(ui).clicked() {
        // future
    }

    ui.add_space(12.0);

    // -- Recent section header --
    SectionHeader::new("Recent").show(ui);

    ui.add_space(4.0);

    // -- Conversation list --
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            // Filter by search query.
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

    // -- User profile at bottom --
    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
        ui.add_space(8.0);
        let resp = UserProfile::new("User")
            .version(concat!("v", env!("CARGO_PKG_VERSION")))
            .show(ui);
        if resp.settings_clicked {
            let _ = tx.send(Action::OpenSettings { tab: Some(SettingsTab::General) });
        }
        ui.add_space(4.0);
    });
}
