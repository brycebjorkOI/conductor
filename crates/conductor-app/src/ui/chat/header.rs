use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::state::*;

use crate::bridge::SharedState;
use crate::theme::Theme;

pub fn show(
    ui: &mut egui::Ui,
    shared: &SharedState,
    tx: &mpsc::UnboundedSender<Action>,
    sidebar_open: &mut bool,
) {
    let dark = ui.visuals().dark_mode;
    let state = shared.read();
    let active_sid = state.active_session_id.clone();

    let session = match state.sessions.get(&active_sid) {
        Some(s) => s,
        None => return,
    };

    let current_backend = session.backend_id.clone();
    let current_model = session.model_id.clone().unwrap_or_default();

    let backends: Vec<(String, String, DiscoveryState)> = state
        .backend_registry
        .iter()
        .map(|b| (b.backend_id.clone(), b.display_name.clone(), b.discovery_state))
        .collect();

    let models: Vec<(String, String)> = state
        .backend_registry
        .iter()
        .find(|b| b.backend_id == current_backend)
        .map(|b| {
            b.available_models
                .iter()
                .map(|m| (m.model_id.clone(), m.display_name.clone()))
                .collect()
        })
        .unwrap_or_default();
    drop(state);

    ui.horizontal(|ui| {
        ui.add_space(8.0);

        // Sidebar toggle (hamburger) — left side, like Claude.
        if !*sidebar_open {
            if ui
                .add(
                    egui::Button::new(
                        egui::RichText::new("\u{2630}")
                            .size(16.0)
                            .color(Theme::text_secondary(dark)),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .frame(false),
                )
                .clicked()
            {
                *sidebar_open = true;
            }
            ui.add_space(4.0);
        }

        // Push everything else to the right.
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);

            // Model selector — small, right-aligned, like Claude's "Opus 4.6 Extended..." text.
            let model_label = if models.is_empty() {
                backends
                    .iter()
                    .find(|(id, _, _)| id == &current_backend)
                    .map(|(_, name, _)| name.clone())
                    .unwrap_or_else(|| current_backend.clone())
            } else {
                models
                    .iter()
                    .find(|(id, _)| id == &current_model)
                    .map(|(_, name)| name.clone())
                    .unwrap_or_else(|| {
                        if current_model.is_empty() {
                            "default".to_string()
                        } else {
                            current_model.clone()
                        }
                    })
            };

            let combo = egui::ComboBox::from_id_salt("model_header")
                .selected_text(
                    egui::RichText::new(&model_label)
                        .size(Theme::SMALL_FONT_SIZE)
                        .color(Theme::text_secondary(dark)),
                )
                .width(180.0);

            combo.show_ui(ui, |ui| {
                // Backends section.
                ui.label(
                    egui::RichText::new("BACKEND")
                        .size(10.0)
                        .strong()
                        .color(Theme::text_muted(dark)),
                );
                for (id, name, disc) in &backends {
                    if *disc != DiscoveryState::Found {
                        continue;
                    }
                    if ui.selectable_label(id == &current_backend, name).clicked() {
                        let _ = tx.send(Action::SwitchBackend {
                            backend_id: id.clone(),
                        });
                    }
                }

                if !models.is_empty() {
                    ui.separator();
                    ui.label(
                        egui::RichText::new("MODEL")
                            .size(10.0)
                            .strong()
                            .color(Theme::text_muted(dark)),
                    );
                    for (id, name) in &models {
                        if ui.selectable_label(id == &current_model, name).clicked() {
                            let _ = tx.send(Action::SwitchModel {
                                model_id: id.clone(),
                            });
                        }
                    }
                }
            });
        });
    });
}
