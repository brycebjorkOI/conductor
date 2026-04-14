use egui_swift::button::{Button, ButtonStyle};
use egui_swift::card::Card;
use egui_swift::colors;
use egui_swift::empty_state::EmptyState;
use egui_swift::form_section::FormSection;

use crate::bridge::SharedState;

pub fn show(ui: &mut egui::Ui, shared: &SharedState) {
    let p = colors::palette(ui);

    ui.label(
        egui::RichText::new("MCP Server Configuration")
            .size(22.0)
            .strong()
            .color(p.text_primary),
    );
    ui.add_space(12.0);

    let state = shared.read();
    let registry = state.backend_registry.clone();
    let mcp_servers = state.mcp_servers.clone();
    drop(state);

    let supported: Vec<&str> = registry
        .iter()
        .filter(|b| conductor_core::mcp::backend_supports_mcp(&b.backend_id))
        .map(|b| b.backend_id.as_str())
        .collect();

    if supported.is_empty() {
        EmptyState::new("No backends with MCP support detected")
            .icon("\u{1f310}")
            .subtitle(
                "MCP is supported by: Anthropic CLI (stdio, SSE, streamable-HTTP) and Gemini CLI (stdio only).",
            )
            .show(ui);
        return;
    }

    for backend_id in &supported {
        FormSection::new()
            .header(&format!("Backend: {backend_id}"))
            .show(ui, |ui| {
                let transports = conductor_core::mcp::supported_transports(backend_id);
                ui.label(
                    egui::RichText::new(format!(
                        "Transports: {}",
                        transports
                            .iter()
                            .map(|t| format!("{t:?}"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                    .size(11.0)
                    .color(p.text_secondary),
                );

                let servers = mcp_servers.get(*backend_id);
                if let Some(servers) = servers {
                    for server in servers {
                        Card::new().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(&server.name)
                                        .strong()
                                        .size(13.0),
                                );
                                ui.label(
                                    egui::RichText::new(format!("{:?}", server.transport))
                                        .size(11.0)
                                        .monospace()
                                        .color(p.text_muted),
                                );
                            });
                            if let Some(ref cmd) = server.command {
                                ui.label(
                                    egui::RichText::new(format!("Command: {cmd}"))
                                        .size(11.0)
                                        .monospace()
                                        .color(p.text_secondary),
                                );
                            }
                            if let Some(ref url) = server.url {
                                ui.label(
                                    egui::RichText::new(format!("URL: {url}"))
                                        .size(11.0)
                                        .monospace()
                                        .color(p.text_secondary),
                                );
                            }
                        });
                    }
                } else {
                    ui.label(
                        egui::RichText::new("No MCP servers configured for this backend.")
                            .size(12.0)
                            .color(p.text_muted),
                    );
                }

                ui.add_space(4.0);
                if Button::new("+ Add MCP Server")
                    .style(ButtonStyle::Bordered)
                    .small(true)
                    .show(ui)
                    .clicked()
                {
                    // TODO: open add server form
                }
            });

        ui.add_space(8.0);
    }
}
