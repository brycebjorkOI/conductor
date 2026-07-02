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
    cancel_rx: oneshot::Receiver<()>,
    on_event: impl FnMut(StreamEvent),
) -> SendOutcome {
    run_chat_streaming_sandboxed(
        backend_def,
        binary_path,
        params,
        working_dir,
        env_overrides,
        cancel_rx,
        on_event,
        None,
    )
    .await
}

/// Like `run_chat_streaming` but with optional Docker sandbox isolation.
pub async fn run_chat_streaming_sandboxed(
    backend_def: &dyn BackendDefinition,
    binary_path: &PathBuf,
    params: ChatParams,
    working_dir: Option<PathBuf>,
    env_overrides: &HashMap<String, String>,
    mut cancel_rx: oneshot::Receiver<()>,
    mut on_event: impl FnMut(StreamEvent),
    sandbox: Option<super::SandboxConfig>,
) -> SendOutcome {
    let mut cli_cmd = backend_def.build_chat_command(binary_path, &params);
    if let Some(ref wd) = working_dir {
        cli_cmd.working_dir = Some(wd.clone());
    }

    let env = security::sanitize_env(env_overrides, security::SanitizeMode::Standard);
    cli_cmd.env = env;

    tracing::debug!(
        "spawning: {} {}{}",
        cli_cmd.binary.display(),
        cli_cmd.args.join(" "),
        if cli_cmd.sandbox.is_some() { " [sandboxed]" } else { "" },
    );
    cli_cmd.sandbox = sandbox;

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

    if exit_code != 0 || !stderr_text.is_empty() {
        tracing::warn!(
            "process exited with code {exit_code}{}",
            if stderr_text.is_empty() {
                String::new()
            } else {
                format!(", stderr: {}", stderr_text.chars().take(500).collect::<String>())
            }
        );
    }

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
    run_chat_sandboxed(backend_def, binary_path, params, working_dir, env_overrides, cancel_rx, None).await
}

/// Like `run_chat` but with optional Docker sandbox.
pub async fn run_chat_sandboxed(
    backend_def: &dyn BackendDefinition,
    binary_path: &PathBuf,
    params: ChatParams,
    working_dir: Option<PathBuf>,
    env_overrides: &HashMap<String, String>,
    cancel_rx: oneshot::Receiver<()>,
    sandbox: Option<super::SandboxConfig>,
) -> SendOutcome {
    let mut all_events = Vec::new();
    let outcome = run_chat_streaming_sandboxed(
        backend_def,
        binary_path,
        params,
        working_dir,
        env_overrides,
        cancel_rx,
        |event| {
            all_events.push(event);
        },
        sandbox,
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
    if let Some(ref sandbox) = cmd.sandbox {
        // Docker sandboxing: only works on Linux where host binaries run natively
        // in containers. On macOS, host binaries are Mach-O and won't run in
        // Linux containers. Fall back to direct execution with a warning.
        if cfg!(target_os = "linux") {
            return spawn_docker_process(cmd, sandbox);
        } else {
            tracing::warn!(
                "Docker sandbox requested but host OS is not Linux — \
                 macOS binaries cannot run in Linux containers. \
                 Running without sandbox. Use CLI --directory flags for scoping."
            );
        }
    }

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

/// Spawn the CLI command inside a Docker container for isolation.
fn spawn_docker_process(
    cmd: &CliCommand,
    sandbox: &crate::backend::SandboxConfig,
) -> Result<tokio::process::Child, std::io::Error> {
    let docker = which::which("docker").map_err(|_| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "docker not found in PATH")
    })?;

    let mut args: Vec<String> = vec![
        "run".into(),
        "--rm".into(),
        "-i".into(),
    ];

    // Network isolation.
    if !sandbox.allow_network {
        args.push("--network".into());
        args.push("none".into());
    }

    // Auto-mount the backend binary into the container so it's available
    // at the same path. Mount the parent dir (e.g. /opt/homebrew/bin) read-only.
    if let Some(bin_dir) = cmd.binary.parent() {
        let dir = bin_dir.display();
        args.push("-v".into());
        args.push(format!("{dir}:{dir}:ro"));
    }

    // Also mount HOME for CLI config files (tokens, etc.) read-only.
    if let Ok(home) = std::env::var("HOME") {
        args.push("-v".into());
        args.push(format!("{home}:{home}:ro"));
    }

    // User-specified volume mounts.
    for (host_path, container_path, read_only) in &sandbox.mounts {
        let host = host_path.display();
        let container = container_path.display();
        if *read_only {
            args.push("-v".into());
            args.push(format!("{host}:{container}:ro"));
        } else {
            args.push("-v".into());
            args.push(format!("{host}:{container}"));
        }
    }

    // Working directory inside the container.
    if let Some(ref wd) = cmd.working_dir {
        args.push("-w".into());
        args.push(wd.display().to_string());
    }

    // Pass through sanitized environment variables.
    for (k, v) in &cmd.env {
        args.push("-e".into());
        args.push(format!("{k}={v}"));
    }

    // Image.
    let image = sandbox
        .image
        .clone()
        .unwrap_or_else(|| "ubuntu:latest".into());
    args.push(image);

    // The actual command to run inside the container.
    args.push(cmd.binary.display().to_string());
    args.extend(cmd.args.iter().cloned());

    let mut builder = Command::new(docker);
    builder
        .args(&args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .env_clear();

    // Docker itself needs minimal host env to function.
    if let Ok(home) = std::env::var("HOME") {
        builder.env("HOME", home);
    }
    if let Ok(path) = std::env::var("PATH") {
        builder.env("PATH", path);
    }
    // Docker socket config.
    if let Ok(host) = std::env::var("DOCKER_HOST") {
        builder.env("DOCKER_HOST", host);
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
