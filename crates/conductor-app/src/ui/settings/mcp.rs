use crate::bridge::SharedState;

pub fn show(ui: &mut egui::Ui, shared: &SharedState) {
    let p = egui_swift::colors::palette(ui);

    ui.heading("MCP Server Configuration");
    ui.add_space(12.0);

    let state = shared.read();
    let registry = state.backend_registry.clone();
    let mcp_servers = state.mcp_servers.clone();
    drop(state);

    // Backend selector.
    let supported: Vec<&str> = registry
        .iter()
        .filter(|b| conductor_core::mcp::backend_supports_mcp(&b.backend_id))
        .map(|b| b.backend_id.as_str())
        .collect();

    if supported.is_empty() {
        ui.label(
            egui::RichText::new("No backends with MCP support detected.")
                .color(p.text_muted),
        );
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("MCP is supported by: Anthropic CLI (stdio, SSE, streamable-HTTP) and Gemini CLI (stdio only).")
                .size(12.0)
                .color(p.text_secondary),
        );
        return;
    }

    for backend_id in &supported {
        ui.add_space(8.0);
        ui.label(egui::RichText::new(format!("Backend: {backend_id}")).strong());

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
                egui::Frame::NONE
                    .fill(p.surface_raised)
                    .corner_radius(egui::CornerRadius::same(6))
                    .stroke(egui::Stroke::new(0.5, p.border_subtle))
                    .inner_margin(egui::Margin::symmetric(10, 6))
                    .outer_margin(egui::Margin::symmetric(0, 2))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(&server.name).strong().size(13.0));
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
        if ui.button("+ Add MCP Server").clicked() {
            // TODO: open add server form
        }
    }
}
