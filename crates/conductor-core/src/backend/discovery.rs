use std::path::PathBuf;
use std::time::Duration;

use tokio::process::Command;

use crate::backend::definitions::all_known_backends;
use crate::backend::BackendDefinition;
use crate::state::{AuthState, BackendStatus, DiscoveryState};

/// Discover a single backend: check PATH, version, auth, models.
async fn probe_backend(def: &dyn BackendDefinition) -> BackendStatus {
    let binary_name = def.binary_name();

    // 1. Check PATH for binary.
    let binary_path = which_binary(binary_name);
    if binary_path.is_none() {
        return BackendStatus {
            backend_id: def.backend_id().to_string(),
            display_name: def.display_name().to_string(),
            discovery_state: DiscoveryState::NotFound,
            binary_path: None,
            version: None,
            auth_state: AuthState::Unknown,
            available_models: def.static_models(),
            default_model: def.static_models().first().map(|m| m.model_id.clone()),
            capabilities: def.capabilities(),
            enabled: true,
            custom_args: Vec::new(),
            env_overrides: Default::default(),
            timeout_seconds: 300,
        };
    }

    let binary_path = binary_path.unwrap();

    // 2. Version query.
    let version = {
        let cmd = def.version_command();
        run_probe(&cmd, Duration::from_secs(10))
            .await
            .and_then(|(stdout, _, exit_code)| def.parse_version(&stdout, exit_code))
    };

    // 3. Auth check.
    let auth_state = match def.auth_check_command() {
        Some(cmd) => match run_probe(&cmd, Duration::from_secs(10)).await {
            Some((stdout, stderr, exit_code)) => {
                def.parse_auth(&stdout, &stderr, exit_code)
            }
            None => AuthState::Unknown,
        },
        None => AuthState::Authenticated, // No auth needed (e.g., Ollama).
    };

    // 4. Model enumeration.
    let mut models = match def.model_list_command() {
        Some(cmd) => match run_probe(&cmd, Duration::from_secs(10)).await {
            Some((stdout, _, _)) => def.parse_models(&stdout),
            None => Vec::new(),
        },
        None => Vec::new(),
    };
    if models.is_empty() {
        models = def.static_models();
    }

    let default_model = models.first().map(|m| m.model_id.clone());

    BackendStatus {
        backend_id: def.backend_id().to_string(),
        display_name: def.display_name().to_string(),
        discovery_state: DiscoveryState::Found,
        binary_path: Some(binary_path),
        version,
        auth_state,
        available_models: models,
        default_model,
        capabilities: def.capabilities(),
        enabled: true,
        custom_args: Vec::new(),
        env_overrides: Default::default(),
        timeout_seconds: 300,
    }
}

/// Scan all known backends concurrently. Returns results as they complete.
pub async fn scan_all_backends() -> Vec<BackendStatus> {
    let defs = all_known_backends();
    let mut handles = Vec::new();

    for def in defs {
        handles.push(tokio::spawn(async move { probe_backend(def.as_ref()).await }));
    }

    let mut results = Vec::new();
    for handle in handles {
        match handle.await {
            Ok(status) => results.push(status),
            Err(e) => tracing::warn!("backend probe task failed: {e}"),
        }
    }
    results
}

/// Look up a binary by name in the system PATH.
fn which_binary(name: &str) -> Option<PathBuf> {
    let path_var = std::env::var("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Run a probe command with a timeout. Returns (stdout, stderr, exit_code).
async fn run_probe(
    cmd: &[String],
    timeout: Duration,
) -> Option<(String, String, i32)> {
    if cmd.is_empty() {
        return None;
    }
    let result = tokio::time::timeout(timeout, async {
        let output = Command::new(&cmd[0])
            .args(&cmd[1..])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await;
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                let code = out.status.code().unwrap_or(-1);
                Some((stdout, stderr, code))
            }
            Err(_) => None,
        }
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(_) => None, // timeout
    }
}
