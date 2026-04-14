use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::state::*;
use egui_swift::prelude::*;

use crate::bridge::SharedState;

/// Chat header view with backend selector and sidebar toggle.
pub struct HeaderView {
    shared: SharedState,
    tx: mpsc::UnboundedSender<Action>,
    pub sidebar_open: bool,
    pub selected_backend_idx: usize,
}

impl HeaderView {
    pub fn new(shared: SharedState, tx: mpsc::UnboundedSender<Action>) -> Self {
        Self {
            shared,
            tx,
            sidebar_open: true,
            selected_backend_idx: 0,
        }
    }
}

impl View for HeaderView {
    fn body(&mut self, ui: &mut egui::Ui) {
        let state = self.shared.read();

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

        if let Some(pos) = found_backends
            .iter()
            .position(|(id, _)| id == &current_backend)
        {
            self.selected_backend_idx = pos;
        }

        drop(state);

        egui_swift::hstack!(ui, {
            egui_swift::spacer!(ui, 8.0);

            if !self.sidebar_open {
                if Button::new(icons::HAMBURGER)
                    .style(ButtonStyle::Borderless)
                    .small(true)
                    .show(ui)
                    .clicked()
                {
                    self.sidebar_open = true;
                }
                egui_swift::spacer!(ui, 4.0);
            }

            if found_backends.len() >= 2 {
                ui.with_layout(
                    egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| {
                        let labels: Vec<&str> =
                            found_backends.iter().map(|(_, n)| n.as_str()).collect();
                        let prev = self.selected_backend_idx;
                        SegmentedControl::new(&labels, &mut self.selected_backend_idx)
                            .show(ui);
                        if self.selected_backend_idx != prev {
                            if let Some((id, _)) =
                                found_backends.get(self.selected_backend_idx)
                            {
                                let _ = self.tx.send(Action::SwitchBackend {
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

            egui_swift::spacer!(ui, 8.0);
        });
    }
}
