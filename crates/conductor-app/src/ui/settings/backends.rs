use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::state::*;

use crate::bridge::SharedState;
use crate::theme;

pub fn show(
    ui: &mut egui::Ui,
    shared: &SharedState,
    tx: &mpsc::UnboundedSender<Action>,
) {
    let state = shared.read();

    ui.horizontal(|ui| {
        ui.heading("Backends");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Rescan").clicked() {
                let _ = tx.send(Action::RescanBackends);
            }
        });
    });

    ui.add_space(8.0);

    let registry = state.backend_registry.clone();
    let default_id = state.default_backend_id.clone();
    let fallback = state.fallback_order.clone();
    drop(state);

    // -- Backend cards --
    for backend in &registry {
        let color = theme::discovery_color(backend.discovery_state);

        egui::Frame::default()
            .stroke(egui::Stroke::new(1.0, color))
            .corner_radius(egui::CornerRadius::same(6))
            .inner_margin(egui::Margin::same(8))
            .outer_margin(egui::Margin::symmetric(0, 4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Status dot.
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(10.0, 10.0),
                        egui::Sense::hover(),
                    );
                    ui.painter().circle_filled(rect.center(), 5.0, color);

                    // Name + version.
                    let version = backend
                        .version
                        .as_deref()
                        .unwrap_or("");
                    let title = if version.is_empty() {
                        backend.display_name.clone()
                    } else {
                        format!("{} v{}", backend.display_name, version)
                    };
                    ui.label(egui::RichText::new(title).strong());
                });

                // Details.
                match backend.discovery_state {
                    DiscoveryState::Found => {
                        if let Some(ref path) = backend.binary_path {
                            ui.label(format!(
                                "Path: {}",
                                path.display()
                            ));
                        }
                        ui.label(format!("Auth: {:?}", backend.auth_state));

                        if !backend.available_models.is_empty() {
                            ui.horizontal(|ui| {
                                ui.label("Models:");
                                let model_names: Vec<&str> = backend
                                    .available_models
                                    .iter()
                                    .map(|m| m.display_name.as_str())
                                    .collect();
                                ui.label(model_names.join(", "));
                            });
                        }
                    }
                    DiscoveryState::NotFound => {
                        ui.label("Not installed");
                    }
                    DiscoveryState::Scanning => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label("Scanning...");
                        });
                    }
                    DiscoveryState::Error => {
                        ui.label("Error during discovery");
                    }
                    DiscoveryState::NotScanned => {
                        ui.label("Not yet scanned");
                    }
                }
            });
    }

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(8.0);

    // -- Default backend --
    ui.horizontal(|ui| {
        ui.label("Default backend:");
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
    ui.label(egui::RichText::new("Fallback Order").strong());
    if fallback.is_empty() {
        ui.label("No fallback backends configured.");
    } else {
        for (i, id) in fallback.iter().enumerate() {
            let name = registry
                .iter()
                .find(|b| &b.backend_id == id)
                .map(|b| b.display_name.as_str())
                .unwrap_or(id.as_str());
            ui.label(format!("{}. {}", i + 1, name));
        }
    }
}
