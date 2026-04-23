use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::state::*;
use egui_swift::prelude::*;

use crate::bridge::SharedState;

fn discovery_color(state: DiscoveryState, p: &Palette) -> egui::Color32 {
    match state {
        DiscoveryState::Found => p.status_green,
        DiscoveryState::Scanning => p.status_yellow,
        DiscoveryState::NotFound => p.text_muted,
        DiscoveryState::Error => p.status_red,
        DiscoveryState::NotScanned => p.text_muted,
    }
}

pub struct BackendsView {
    shared: SharedState,
    tx: mpsc::UnboundedSender<Action>,
}

impl BackendsView {
    pub fn new(shared: SharedState, tx: mpsc::UnboundedSender<Action>) -> Self {
        Self { shared, tx }
    }
}

impl View for BackendsView {
    fn body(&mut self, ui: &mut egui::Ui) {
        let p = ui.palette();
        let state = self.shared.read();
        let registry = state.backend_registry.clone();
        let default_id = state.default_backend_id.clone();
        let fallback = state.fallback_order.clone();
        drop(state);

        egui_swift::hstack!(ui, {
            Label::heading("Backends").show(ui);
            Spacer::trailing(ui, |ui| {
                if Button::new("Rescan")
                    .style(ButtonStyle::Bordered)
                    .small(true)
                    .show(ui)
                    .clicked()
                {
                    let _ = self.tx.send(Action::RescanBackends);
                }
            });
        });
        egui_swift::spacer!(ui, 12.0);

        for backend in &registry {
            let color = discovery_color(backend.discovery_state, &p);

            Card::new().border_color(color).show(ui, |ui| {
                egui_swift::hstack!(ui, {
                    StatusDot::new(color).show(ui);
                    let version = backend.version.as_deref().unwrap_or("");
                    let title = if version.is_empty() {
                        backend.display_name.clone()
                    } else {
                        format!("{} v{}", backend.display_name, version)
                    };
                    Label::new(&title).font(Font::Callout).bold(true).show(ui);
                });

                match backend.discovery_state {
                    DiscoveryState::Found => {
                        if let Some(ref path) = backend.binary_path {
                            egui_swift::text!(ui, &format!("Path: {}", path.display()), .subheadline, .secondary);
                        }
                        egui_swift::text!(ui, &format!("Auth: {:?}", backend.auth_state), .subheadline, .secondary);
                        if !backend.available_models.is_empty() {
                            let names: Vec<&str> = backend
                                .available_models
                                .iter()
                                .map(|m| m.display_name.as_str())
                                .collect();
                            egui_swift::text!(ui, &format!("Models: {}", names.join(", ")), .subheadline, .secondary);
                        }
                    }
                    DiscoveryState::NotFound => {
                        egui_swift::text!(ui, "Not installed", .subheadline, .muted);
                    }
                    DiscoveryState::Scanning => {
                        egui_swift::hstack!(ui, {
                            ProgressIndicator::spinner().size(16.0).show(ui);
                            egui_swift::text!(ui, "Scanning...", .subheadline, .muted);
                        });
                    }
                    DiscoveryState::Error => {
                        egui_swift::text!(ui, "Error during discovery", .subheadline, .muted);
                    }
                    DiscoveryState::NotScanned => {
                        egui_swift::text!(ui, "Not yet scanned", .subheadline, .muted);
                    }
                }
            });
        }

        egui_swift::spacer!(ui, 12.0);
        Divider::new().show(ui);
        egui_swift::spacer!(ui, 12.0);

        egui_swift::section!(ui, "Default Backend", {
            let options: Vec<(String, &str)> = registry
                .iter()
                .filter(|b| b.discovery_state == DiscoveryState::Found)
                .map(|b| (b.backend_id.clone(), b.display_name.as_str()))
                .collect();
            let mut selected = default_id.clone().unwrap_or_default();
            if Picker::new("Default Backend", &mut selected, &options).show(ui).changed() {
                self.shared.mutate(|s| {
                    s.default_backend_id = Some(selected);
                });
            }
        });

        egui_swift::spacer!(ui, 8.0);
        egui_swift::section!(ui, "Fallback Order", {
            if fallback.is_empty() {
                egui_swift::text!(ui, "No fallback backends configured.", .subheadline, .muted);
            } else {
                for (i, id) in fallback.iter().enumerate() {
                    let name = registry
                        .iter()
                        .find(|b| &b.backend_id == id)
                        .map(|b| b.display_name.as_str())
                        .unwrap_or(id.as_str());
                    egui_swift::text!(ui, &format!("{}. {}", i + 1, name), .callout);
                }
            }
        });
    }
}
