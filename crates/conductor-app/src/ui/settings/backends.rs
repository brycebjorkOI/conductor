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

pub fn show(
    ui: &mut egui::Ui,
    shared: &SharedState,
    tx: &mpsc::UnboundedSender<Action>,
) {
    let p = ui.palette();
    let state = shared.read();
    let registry = state.backend_registry.clone();
    let default_id = state.default_backend_id.clone();
    let fallback = state.fallback_order.clone();
    drop(state);

    HStack::new().show(ui, |ui| {
        Label::heading("Backends").show(ui);
        Spacer::trailing(ui, |ui| {
            if Button::new("Rescan")
                .style(ButtonStyle::Bordered)
                .small(true)
                .show(ui)
                .clicked()
            {
                let _ = tx.send(Action::RescanBackends);
            }
        });
    });
    ui.add_space(12.0);

    for backend in &registry {
        let color = discovery_color(backend.discovery_state, &p);

        Card::new().border_color(color).show(ui, |ui| {
            ui.horizontal(|ui| {
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
                        Label::new(&format!("Path: {}", path.display()))
                            .font(Font::Subheadline)
                            .secondary()
                            .show(ui);
                    }
                    Label::new(&format!("Auth: {:?}", backend.auth_state))
                        .font(Font::Subheadline)
                        .secondary()
                        .show(ui);
                    if !backend.available_models.is_empty() {
                        let names: Vec<&str> = backend
                            .available_models
                            .iter()
                            .map(|m| m.display_name.as_str())
                            .collect();
                        Label::new(&format!("Models: {}", names.join(", ")))
                            .font(Font::Subheadline)
                            .secondary()
                            .show(ui);
                    }
                }
                DiscoveryState::NotFound => {
                    Label::new("Not installed")
                        .font(Font::Subheadline)
                        .muted()
                        .show(ui);
                }
                DiscoveryState::Scanning => {
                    ui.horizontal(|ui| {
                        ProgressIndicator::spinner().size(16.0).show(ui);
                        Label::new("Scanning...")
                            .font(Font::Subheadline)
                            .muted()
                            .show(ui);
                    });
                }
                DiscoveryState::Error => {
                    Label::new("Error during discovery")
                        .font(Font::Subheadline)
                        .destructive()
                        .show(ui);
                }
                DiscoveryState::NotScanned => {
                    Label::new("Not yet scanned")
                        .font(Font::Subheadline)
                        .muted()
                        .show(ui);
                }
            }
        });
    }

    ui.add_space(12.0);
    Divider::new().show(ui);
    ui.add_space(12.0);

    FormSection::new().header("Default Backend").show(ui, |ui| {
        let current = default_id.as_deref().unwrap_or("none");
        egui::ComboBox::from_id_salt("default_backend")
            .selected_text(current)
            .show_ui(ui, |ui| {
                for b in &registry {
                    if b.discovery_state == DiscoveryState::Found {
                        if ui
                            .selectable_label(
                                Some(b.backend_id.as_str()) == default_id.as_deref(),
                                &b.display_name,
                            )
                            .clicked()
                        {
                            shared.mutate(|s| {
                                s.default_backend_id = Some(b.backend_id.clone());
                            });
                        }
                    }
                }
            });
    });

    ui.add_space(8.0);
    FormSection::new().header("Fallback Order").show(ui, |ui| {
        if fallback.is_empty() {
            Label::new("No fallback backends configured.")
                .font(Font::Subheadline)
                .muted()
                .show(ui);
        } else {
            for (i, id) in fallback.iter().enumerate() {
                let name = registry
                    .iter()
                    .find(|b| &b.backend_id == id)
                    .map(|b| b.display_name.as_str())
                    .unwrap_or(id.as_str());
                Label::new(&format!("{}. {}", i + 1, name))
                    .font(Font::Callout)
                    .show(ui);
            }
        }
    });
}
