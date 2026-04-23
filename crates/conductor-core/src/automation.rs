//! Automation rule persistence: save/load rules to disk.

use std::path::PathBuf;

use crate::config;
use crate::state::AutomationRule;

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

/// Load automation rules from disk.
pub fn load_rules() -> Vec<AutomationRule> {
    let path = automations_file();
    match std::fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|e| {
            tracing::warn!("failed to parse automations.json: {e}");
            Vec::new()
        }),
        Err(_) => Vec::new(),
    }
}
