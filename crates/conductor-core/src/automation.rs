//! Automation rule persistence: save/load rules to disk.
//! Includes migration from legacy single-action format to multi-step.

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
        },
        AutomationAction::RunJob { job_id } => StepAction::RunJob { job_id },
        AutomationAction::Notify { message } => StepAction::Notify { message },
    }
}
