use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::session;
use egui_swift::prelude::*;

use crate::bridge::SharedState;

pub fn show(ui: &mut egui::Ui, shared: &SharedState, tx: &mpsc::UnboundedSender<Action>) {
    egui_swift::text!(ui, "Sessions", .title);
    egui_swift::spacer!(ui, 12.0);

    let state = shared.read();
    let active_sid = state.active_session_id.clone();
    let mut sessions: Vec<(String, String, usize, String)> = state
        .sessions
        .iter()
        .map(|(id, s)| {
            let name = s
                .display_name
                .clone()
                .unwrap_or_else(|| session::auto_display_name(s));
            let created = s.created_at.format("%Y-%m-%d %H:%M").to_string();
            (id.clone(), name, s.messages.len(), created)
        })
        .collect();
    sessions.sort_by(|a, b| a.3.cmp(&b.3));
    drop(state);

    DataTable::new(&[
        ("Name", 200.0),
        ("Messages", 80.0),
        ("Created", 120.0),
        ("Actions", 80.0),
    ])
    .striped(true)
    .show(ui, |ui| {
        for (id, name, msg_count, created) in &sessions {
            let is_active = id == &active_sid;
            let label = if is_active {
                format!("{name} (active)")
            } else {
                name.clone()
            };
            egui_swift::text!(ui, &label, .subheadline);
            egui_swift::text!(ui, &format!("{msg_count}"), .subheadline);
            egui_swift::text!(ui, created, .subheadline);

            if !is_active {
                if Button::new("Switch")
                    .style(ButtonStyle::Bordered)
                    .small(true)
                    .show(ui)
                    .clicked()
                {
                    let _ = tx.send(Action::SwitchSession {
                        session_id: id.clone(),
                    });
                }
            } else {
                egui_swift::text!(ui, "-", .subheadline, .muted);
            }
            ui.end_row();
        }
    });
}
