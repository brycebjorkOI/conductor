use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::session;

use crate::bridge::SharedState;

pub fn show(ui: &mut egui::Ui, shared: &SharedState, tx: &mpsc::UnboundedSender<Action>) {
    let _p = egui_swift::colors::palette(ui);

    ui.heading("Sessions");
    ui.add_space(8.0);

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

    egui::Grid::new("sessions_grid")
        .num_columns(4)
        .spacing([12.0, 6.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Name").strong().size(12.0));
            ui.label(egui::RichText::new("Messages").strong().size(12.0));
            ui.label(egui::RichText::new("Created").strong().size(12.0));
            ui.label(egui::RichText::new("Actions").strong().size(12.0));
            ui.end_row();

            for (id, name, msg_count, created) in &sessions {
                let is_active = id == &active_sid;
                let label = if is_active {
                    format!("{name} (active)")
                } else {
                    name.clone()
                };
                ui.label(&label);
                ui.label(format!("{msg_count}"));
                ui.label(created);

                if !is_active {
                    if ui.small_button("Switch").clicked() {
                        let _ = tx.send(Action::SwitchSession {
                            session_id: id.clone(),
                        });
                    }
                } else {
                    ui.label("-");
                }
                ui.end_row();
            }
        });
}
