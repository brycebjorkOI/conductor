use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::oneshot;

use crate::backend::{BackendDefinition, ChatParams, CliCommand, StreamEvent};
use crate::security;
use crate::state::*;

/// Outcome of a completed (or failed) send-message operation.
#[derive(Debug)]
pub enum SendOutcome {
    Completed {
        events: Vec<StreamEvent>,
        duration_ms: u64,
    },
    Cancelled,
    Error(String),
}

/// Spawn the backend CLI and stream events incrementally via a callback.
///
/// Each parsed event is passed to `on_event` AS IT ARRIVES — the UI sees
/// tokens, tool starts, and thinking chunks in real time. Returns the
/// final outcome (duration, or cancel/error).
pub async fn run_chat_streaming(
    backend_def: &dyn BackendDefinition,
    binary_path: &PathBuf,
    params: ChatParams,
    working_dir: Option<PathBuf>,
    env_overrides: &HashMap<String, String>,
    mut cancel_rx: oneshot::Receiver<()>,
    mut on_event: impl FnMut(StreamEvent),
) -> SendOutcome {
    let mut cli_cmd = backend_def.build_chat_command(binary_path, &params);
    if let Some(ref wd) = working_dir {
        cli_cmd.working_dir = Some(wd.clone());
    }

    let env = security::sanitize_env(env_overrides, security::SanitizeMode::Standard);
    cli_cmd.env = env;

    let mut child = match spawn_process(&cli_cmd) {
        Ok(c) => c,
        Err(e) => return SendOutcome::Error(format!("failed to spawn process: {e}")),
    };

    let stdout = match child.stdout.take() {
        Some(s) => s,
        None => return SendOutcome::Error("no stdout handle".into()),
    };

    let start = Instant::now();
    let mut parser = backend_def.create_parser();
    let mut reader = BufReader::new(stdout).lines();

    loop {
        tokio::select! {
            line_result = reader.next_line() => {
                match line_result {
                    Ok(Some(line)) => {
                        let events = parser.parse_line(&line);
                        for event in events {
                            on_event(event);
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        on_event(StreamEvent::Error(format!("read error: {e}")));
                        break;
                    }
                }
            }
            _ = &mut cancel_rx => {
                let _ = child.kill().await;
                return SendOutcome::Cancelled;
            }
        }
    }

    let status = child.wait().await;
    let exit_code = status.map(|s| s.code().unwrap_or(-1)).unwrap_or(-1);

    let stderr_text = if let Some(mut stderr) = child.stderr.take() {
        let mut buf = String::new();
        let _ = tokio::io::AsyncReadExt::read_to_string(&mut stderr, &mut buf).await;
        buf
    } else {
        String::new()
    };

    let final_events = parser.finish(exit_code, &stderr_text);
    for event in final_events {
        on_event(event);
    }

    let duration_ms = start.elapsed().as_millis() as u64;
    SendOutcome::Completed {
        events: Vec::new(), // events already delivered via callback
        duration_ms,
    }
}

/// Spawn the backend CLI, collect all events, and return them.
/// Used by the scheduler where real-time streaming isn't needed.
pub async fn run_chat(
    backend_def: &dyn BackendDefinition,
    binary_path: &PathBuf,
    params: ChatParams,
    working_dir: Option<PathBuf>,
    env_overrides: &HashMap<String, String>,
    cancel_rx: oneshot::Receiver<()>,
) -> SendOutcome {
    let mut all_events = Vec::new();
    let outcome = run_chat_streaming(
        backend_def,
        binary_path,
        params,
        working_dir,
        env_overrides,
        cancel_rx,
        |event| {
            all_events.push(event);
        },
    )
    .await;

    match outcome {
        SendOutcome::Completed { duration_ms, .. } => SendOutcome::Completed {
            events: all_events,
            duration_ms,
        },
        other => other,
    }
}

fn spawn_process(
    cmd: &CliCommand,
) -> Result<tokio::process::Child, std::io::Error> {
    let mut builder = Command::new(&cmd.binary);
    builder
        .args(&cmd.args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .env_clear();

    for (k, v) in &cmd.env {
        builder.env(k, v);
    }

    if let Some(ref wd) = cmd.working_dir {
        builder.current_dir(wd);
    }

    builder.spawn()
}

/// Determine the best available backend for a message.
pub fn select_backend<'a>(
    registry: &'a [BackendStatus],
    preferred_id: &str,
    fallback_order: &[String],
) -> Option<(&'a BackendStatus, &'a PathBuf)> {
    if let Some(bs) = registry
        .iter()
        .find(|b| b.backend_id == preferred_id && is_available(b))
    {
        if let Some(ref path) = bs.binary_path {
            return Some((bs, path));
        }
    }

    for id in fallback_order {
        if let Some(bs) = registry.iter().find(|b| &b.backend_id == id && is_available(b)) {
            if let Some(ref path) = bs.binary_path {
                return Some((bs, path));
            }
        }
    }

    for bs in registry {
        if is_available(bs) {
            if let Some(ref path) = bs.binary_path {
                return Some((bs, path));
            }
        }
    }

    None
}

fn is_available(b: &BackendStatus) -> bool {
    b.enabled
        && b.discovery_state == DiscoveryState::Found
        && b.auth_state != AuthState::NotAuthenticated
}
