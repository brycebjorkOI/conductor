use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::session;
use conductor_core::state::SettingsTab;

use crate::bridge::SharedState;
use crate::theme::Theme;

pub fn show(
    ui: &mut egui::Ui,
    shared: &SharedState,
    tx: &mpsc::UnboundedSender<Action>,
) {
    let dark = ui.visuals().dark_mode;
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

    // Leave room for macOS traffic lights (close/minimize/fullscreen).
    ui.add_space(38.0);

    // -- New Chat --
    sidebar_row(ui, dark, false, false, |ui| {
        ui.label(
            egui::RichText::new("+")
                .size(14.0)
                .color(Theme::text_secondary(dark)),
        );
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new("New chat")
                .size(13.5)
                .color(Theme::text_primary(dark)),
        );
    }, || {
        let _ = tx.send(Action::NewSession);
    });

    ui.add_space(6.0);

    // -- Nav items --
    sidebar_row(ui, dark, false, false, |ui| {
        ui.label(
            egui::RichText::new("Settings")
                .size(13.0)
                .color(Theme::text_secondary(dark)),
        );
    }, || {
        let _ = tx.send(Action::OpenSettings { tab: Some(SettingsTab::General) });
    });

    ui.add_space(16.0);

    // -- Conversation list --
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            for (id, name) in &sessions {
                let is_active = id == &active_sid;
                let tx_clone = tx.clone();
                let id_clone = id.clone();

                sidebar_row(ui, dark, is_active, true, |ui| {
                    let truncated = if name.len() > 30 {
                        format!("{}...", &name[..27])
                    } else {
                        name.clone()
                    };
                    let color = if is_active {
                        Theme::text_primary(dark)
                    } else {
                        Theme::text_secondary(dark)
                    };
                    ui.label(
                        egui::RichText::new(truncated)
                            .size(13.0)
                            .color(color),
                    );
                }, move || {
                    let _ = tx_clone.send(Action::SwitchSession {
                        session_id: id_clone.clone(),
                    });
                });
            }
        });
}

/// Renders a sidebar row with hover/active states and click handling.
fn sidebar_row(
    ui: &mut egui::Ui,
    dark: bool,
    is_active: bool,
    is_session: bool,
    content: impl FnOnce(&mut egui::Ui),
    on_click: impl FnOnce(),
) {
    let height = if is_session { 32.0 } else { 30.0 };
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::click(),
    );

    let bg = if is_active {
        Theme::sidebar_active(dark)
    } else if response.hovered() {
        Theme::sidebar_hover(dark)
    } else {
        egui::Color32::TRANSPARENT
    };

    ui.painter().rect_filled(
        rect.shrink2(egui::vec2(4.0, 1.0)),
        egui::CornerRadius::same(6),
        bg,
    );

    // Content positioned inside the rect.
    let content_rect = rect.shrink2(egui::vec2(16.0, 0.0));
    let mut child_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(content_rect)
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );
    content(&mut child_ui);

    if response.clicked() {
        on_click();
    }
}
