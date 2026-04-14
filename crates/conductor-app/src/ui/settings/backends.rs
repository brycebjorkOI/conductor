use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::state::*;
use egui_swift::button::{Button, ButtonStyle};
use egui_swift::card::Card;
use egui_swift::colors;
use egui_swift::divider::Divider;
use egui_swift::form_section::FormSection;
use egui_swift::progress_indicator::ProgressIndicator;
use egui_swift::status_dot::StatusDot;

use crate::bridge::SharedState;

fn discovery_color(state: DiscoveryState, p: &egui_swift::colors::Palette) -> egui::Color32 {
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
    let p = colors::palette(ui);
    let state = shared.read();
    let registry = state.backend_registry.clone();
    let default_id = state.default_backend_id.clone();
    let fallback = state.fallback_order.clone();
    drop(state);

    // Header with rescan button.
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("Backends")
                .size(22.0)
                .strong()
                .color(p.text_primary),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
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

    // -- Backend cards --
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
                ui.label(egui::RichText::new(title).strong().size(13.0));
            });

            match backend.discovery_state {
                DiscoveryState::Found => {
                    if let Some(ref path) = backend.binary_path {
                        ui.label(
                            egui::RichText::new(format!("Path: {}", path.display()))
                                .size(12.0)
                                .color(p.text_secondary),
                        );
                    }
                    ui.label(
                        egui::RichText::new(format!("Auth: {:?}", backend.auth_state))
                            .size(12.0)
                            .color(p.text_secondary),
                    );
                    if !backend.available_models.is_empty() {
                        let model_names: Vec<&str> = backend
                            .available_models
                            .iter()
                            .map(|m| m.display_name.as_str())
                            .collect();
                        ui.label(
                            egui::RichText::new(format!("Models: {}", model_names.join(", ")))
                                .size(12.0)
                                .color(p.text_secondary),
                        );
                    }
                }
                DiscoveryState::NotFound => {
                    ui.label(
                        egui::RichText::new("Not installed")
                            .size(12.0)
                            .color(p.text_muted),
                    );
                }
                DiscoveryState::Scanning => {
                    ui.horizontal(|ui| {
                        ProgressIndicator::spinner().size(16.0).show(ui);
                        ui.label(
                            egui::RichText::new("Scanning...")
                                .size(12.0)
                                .color(p.text_muted),
                        );
                    });
                }
                DiscoveryState::Error => {
                    ui.label(
                        egui::RichText::new("Error during discovery")
                            .size(12.0)
                            .color(p.status_red),
                    );
                }
                DiscoveryState::NotScanned => {
                    ui.label(
                        egui::RichText::new("Not yet scanned")
                            .size(12.0)
                            .color(p.text_muted),
                    );
                }
            }
        });
    }

    ui.add_space(12.0);
    Divider::new().show(ui);
    ui.add_space(12.0);

    // -- Default backend picker --
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

    // -- Fallback order --
    ui.add_space(8.0);
    FormSection::new().header("Fallback Order").show(ui, |ui| {
        if fallback.is_empty() {
            ui.label(
                egui::RichText::new("No fallback backends configured.")
                    .size(12.0)
                    .color(p.text_muted),
            );
        } else {
            for (i, id) in fallback.iter().enumerate() {
                let name = registry
                    .iter()
                    .find(|b| &b.backend_id == id)
                    .map(|b| b.display_name.as_str())
                    .unwrap_or(id.as_str());
                ui.label(
                    egui::RichText::new(format!("{}. {}", i + 1, name))
                        .size(13.0)
                        .color(p.text_primary),
                );
            }
        }
    });
}
