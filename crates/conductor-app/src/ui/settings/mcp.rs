use egui_swift::prelude::*;

use crate::bridge::SharedState;

pub fn show(ui: &mut egui::Ui, shared: &SharedState) {
    egui_swift::text!(ui, "MCP Server Configuration", .title);
    egui_swift::spacer!(ui, 12.0);

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
        egui_swift::section!(ui, &format!("Backend: {backend_id}"), {
            let transports = conductor_core::mcp::supported_transports(backend_id);
            egui_swift::text!(ui, &format!(
                "Transports: {}",
                transports.iter().map(|t| format!("{t:?}")).collect::<Vec<_>>().join(", ")
            ), .caption, .secondary);

            let servers = mcp_servers.get(*backend_id);
            if let Some(servers) = servers {
                for server in servers {
                    Card::new().show(ui, |ui| {
                        egui_swift::hstack!(ui, {
                            Label::new(&server.name).font(Font::Callout).bold(true).show(ui);
                            Label::new(&format!("{:?}", server.transport))
                                .font(Font::Caption).monospace(true).muted().show(ui);
                        });
                        if let Some(ref cmd) = server.command {
                            Label::new(&format!("Command: {cmd}"))
                                .font(Font::Caption).monospace(true).secondary().show(ui);
                        }
                        if let Some(ref url) = server.url {
                            Label::new(&format!("URL: {url}"))
                                .font(Font::Caption).monospace(true).secondary().show(ui);
                        }
                    });
                }
            } else {
                egui_swift::text!(ui, "No MCP servers configured for this backend.", .subheadline, .muted);
            }

            egui_swift::spacer!(ui, 4.0);
            if Button::new("+ Add MCP Server")
                .style(ButtonStyle::Bordered)
                .small(true)
                .show(ui)
                .clicked()
            {
                // TODO: open add server form
            }
        });

        egui_swift::spacer!(ui, 8.0);
    }
}
