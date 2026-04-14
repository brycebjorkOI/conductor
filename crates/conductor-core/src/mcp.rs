//! MCP (Model Context Protocol) server configuration management.
//!
//! Reads and writes MCP server configs to each backend's native config location.

use std::path::PathBuf;


use crate::state::{McpServerEntry, McpTransport, McpScope};

/// Known config file locations per backend.
pub fn config_path_for_backend(backend_id: &str) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    match backend_id {
        "anthropic" => Some(home.join(".claude").join("mcp_servers.json")),
        "gemini" => Some(home.join(".config").join("gemini").join("settings.json")),
        _ => None,
    }
}

/// Load MCP server entries from a backend's config file.
pub fn load_mcp_servers(backend_id: &str) -> Vec<McpServerEntry> {
    let path = match config_path_for_backend(backend_id) {
        Some(p) => p,
        None => return Vec::new(),
    };

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let value: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    // Parse the "mcpServers" key (Anthropic CLI format).
    let servers = value
        .get("mcpServers")
        .or_else(|| value.get("mcp_servers"))
        .and_then(|v| v.as_object());

    let Some(servers) = servers else {
        return Vec::new();
    };

    servers
        .iter()
        .filter_map(|(name, cfg)| {
            let transport = match cfg.get("transport").and_then(|v| v.as_str()) {
                Some("stdio") | None => McpTransport::Stdio,
                Some("sse") => McpTransport::Sse,
                Some("streamable-http") | Some("streamableHttp") => McpTransport::StreamableHttp,
                _ => McpTransport::Stdio,
            };

            Some(McpServerEntry {
                name: name.clone(),
                transport,
                command: cfg.get("command").and_then(|v| v.as_str()).map(|s| s.to_string()),
                args: cfg.get("args").and_then(|v| {
                    v.as_array().map(|arr| {
                        arr.iter().filter_map(|a| a.as_str().map(|s| s.to_string())).collect()
                    })
                }),
                url: cfg.get("url").and_then(|v| v.as_str()).map(|s| s.to_string()),
                env_vars: cfg
                    .get("env")
                    .and_then(|v| v.as_object())
                    .map(|obj| {
                        obj.iter()
                            .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                            .collect()
                    })
                    .unwrap_or_default(),
                scope: McpScope::User,
            })
        })
        .collect()
}

/// Check whether a backend supports MCP.
pub fn backend_supports_mcp(backend_id: &str) -> bool {
    matches!(backend_id, "anthropic" | "gemini")
}

/// Supported transports for a given backend.
pub fn supported_transports(backend_id: &str) -> Vec<McpTransport> {
    match backend_id {
        "anthropic" => vec![McpTransport::Stdio, McpTransport::Sse, McpTransport::StreamableHttp],
        "gemini" => vec![McpTransport::Stdio],
        _ => Vec::new(),
    }
}
