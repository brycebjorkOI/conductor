use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::session;
use egui_swift::button::{Button, ButtonStyle};
use egui_swift::colors;
use egui_swift::data_table::DataTable;

use crate::bridge::SharedState;

pub fn show(ui: &mut egui::Ui, shared: &SharedState, tx: &mpsc::UnboundedSender<Action>) {
    let p = colors::palette(ui);

    ui.label(
        egui::RichText::new("Sessions")
            .size(22.0)
            .strong()
            .color(p.text_primary),
    );
    ui.add_space(12.0);

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
            ui.label(egui::RichText::new(&label).size(12.0));
            ui.label(egui::RichText::new(format!("{msg_count}")).size(12.0));
            ui.label(egui::RichText::new(created).size(12.0));

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
                ui.label(
                    egui::RichText::new("-")
                        .size(12.0)
                        .color(p.text_muted),
                );
            }
            ui.end_row();
        }
    });
}
