//! Automation rule persistence: save/load rules to disk.
//! Includes migration from legacy single-action format to multi-step.

use std::io::Write;
use std::path::PathBuf;

use crate::config;
use crate::state::{AutomationAction, AutomationRule, AutomationStep, StepAction};

fn automations_file() -> PathBuf {
    let dir = config::config_dir();
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir.join("automations.json")
}

/// Save all automation rules to disk (atomic write via tmp+rename).
pub fn save_rules(rules: &[AutomationRule]) -> Result<(), std::io::Error> {
    let path = automations_file();
    let json = serde_json::to_string_pretty(rules)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

/// Load automation rules from disk, migrating legacy single-action rules to steps.
pub fn load_rules() -> Vec<AutomationRule> {
    let path = automations_file();
    match std::fs::read_to_string(&path) {
        Ok(contents) => {
            let mut rules: Vec<AutomationRule> =
                serde_json::from_str(&contents).unwrap_or_else(|e| {
                    tracing::warn!("failed to parse automations.json: {e}");
                    Vec::new()
                });
            migrate_rules(&mut rules);
            rules
        }
        Err(_) => Vec::new(),
    }
}

/// Migrate legacy rules that have `action` but no `steps`.
fn migrate_rules(rules: &mut [AutomationRule]) {
    for rule in rules.iter_mut() {
        if rule.steps.is_empty() {
            if let Some(action) = rule.action.take() {
                tracing::info!(
                    "migrating automation '{}' from single action to steps",
                    rule.name
                );
                rule.steps = vec![AutomationStep {
                    step_id: uuid::Uuid::new_v4().to_string(),
                    name: "Step 1".into(),
                    position: 0,
                    step_type: convert_legacy_action(action),
                    enabled: true,
                }];
            }
        }
    }
}

fn convert_legacy_action(action: AutomationAction) -> StepAction {
    match action {
        AutomationAction::RunPrompt {
            prompt,
            include_event_context,
            backend_override,
            model_override,
        } => StepAction::RunPrompt {
            prompt,
            include_event_context,
            include_previous_output: false,
            backend_override,
            model_override,
            sandbox: None,
        },
        AutomationAction::RunJob { job_id } => StepAction::RunJob { job_id },
        AutomationAction::Notify { message } => StepAction::Notify { message },
    }
}

// ---------------------------------------------------------------------------
// Run logging (NDJSON append-only log per run)
// ---------------------------------------------------------------------------

fn logs_dir(rule_id: &str) -> PathBuf {
    let dir = config::config_dir()
        .join("automation_logs")
        .join(rule_id);
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}

/// Path to the log file for a given run.
pub fn run_log_path(rule_id: &str, run_id: &str) -> PathBuf {
    logs_dir(rule_id).join(format!("{run_id}.ndjson"))
}

fn append_log_line(path: &PathBuf, value: &serde_json::Value) {
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = writeln!(f, "{}", value);
    }
}

/// Write the initial run_start event to the log file.
pub fn log_run_start(
    rule_id: &str,
    run_id: &str,
    rule_name: &str,
    trigger_event: Option<&str>,
) {
    let path = run_log_path(rule_id, run_id);
    let event = serde_json::json!({
        "event": "run_start",
        "run_id": run_id,
        "rule_id": rule_id,
        "rule_name": rule_name,
        "started_at": chrono::Utc::now().to_rfc3339(),
        "trigger_event": trigger_event,
    });
    append_log_line(&path, &event);
}

/// Write a step_start event with the input being fed to this step.
pub fn log_step_start(
    rule_id: &str,
    run_id: &str,
    step_id: &str,
    step_name: &str,
    step_type: &str,
    input: Option<&str>,
) {
    let path = run_log_path(rule_id, run_id);
    let event = serde_json::json!({
        "event": "step_start",
        "step_id": step_id,
        "step_name": step_name,
        "step_type": step_type,
        "input": input,
        "started_at": chrono::Utc::now().to_rfc3339(),
    });
    append_log_line(&path, &event);
}

/// Write a step_complete event with full (untruncated) output.
pub fn log_step_complete(
    rule_id: &str,
    run_id: &str,
    step_id: &str,
    step_name: &str,
    status: &str,
    duration_ms: u64,
    output: Option<&str>,
    error: Option<&str>,
    skipped: bool,
) {
    let path = run_log_path(rule_id, run_id);
    let event = serde_json::json!({
        "event": "step_complete",
        "step_id": step_id,
        "step_name": step_name,
        "status": status,
        "duration_ms": duration_ms,
        "output": output,
        "error": error,
        "skipped": skipped,
        "completed_at": chrono::Utc::now().to_rfc3339(),
    });
    append_log_line(&path, &event);
}

/// Write the final run_complete event.
pub fn log_run_complete(
    rule_id: &str,
    run_id: &str,
    status: &str,
    duration_ms: u64,
) {
    let path = run_log_path(rule_id, run_id);
    let event = serde_json::json!({
        "event": "run_complete",
        "run_id": run_id,
        "status": status,
        "duration_ms": duration_ms,
        "completed_at": chrono::Utc::now().to_rfc3339(),
    });
    append_log_line(&path, &event);
}
