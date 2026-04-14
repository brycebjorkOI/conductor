use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::state::*;
use egui_swift::prelude::*;

use crate::bridge::SharedState;

pub fn show(
    ui: &mut egui::Ui,
    shared: &SharedState,
    tx: &mpsc::UnboundedSender<Action>,
    sidebar_open: &mut bool,
    selected_backend_idx: &mut usize,
) {
    let state = shared.read();

    let active_sid = state.active_session_id.clone();
    let session = match state.sessions.get(&active_sid) {
        Some(s) => s,
        None => return,
    };
    let current_backend = session.backend_id.clone();

    let found_backends: Vec<(String, String)> = state
        .backend_registry
        .iter()
        .filter(|b| b.discovery_state == DiscoveryState::Found)
        .map(|b| (b.backend_id.clone(), b.display_name.clone()))
        .collect();

    if let Some(pos) = found_backends.iter().position(|(id, _)| id == &current_backend) {
        *selected_backend_idx = pos;
    }

    drop(state);

    ui.horizontal(|ui| {
        ui.add_space(8.0);

        if !*sidebar_open {
            if Button::new(icons::HAMBURGER)
                .style(ButtonStyle::Borderless)
                .small(true)
                .show(ui)
                .clicked()
            {
                *sidebar_open = true;
            }
            ui.add_space(4.0);
        }

        if found_backends.len() >= 2 {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    let labels: Vec<&str> =
                        found_backends.iter().map(|(_, n)| n.as_str()).collect();
                    let prev = *selected_backend_idx;
                    SegmentedControl::new(&labels, selected_backend_idx).show(ui);
                    if *selected_backend_idx != prev {
                        if let Some((id, _)) = found_backends.get(*selected_backend_idx) {
                            let _ = tx.send(Action::SwitchBackend {
                                backend_id: id.clone(),
                            });
                        }
                    }
                },
            );
        } else if found_backends.len() == 1 {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    Label::new(&found_backends[0].1)
                        .font(Font::Callout)
                        .secondary()
                        .show(ui);
                },
            );
        } else {
            ui.with_layout(
                egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                |ui| {
                    Label::new("No backends detected")
                        .font(Font::Callout)
                        .muted()
                        .show(ui);
                },
            );
        }

        ui.add_space(8.0);
    });
}
