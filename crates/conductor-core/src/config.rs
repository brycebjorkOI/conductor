use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub backends: BackendsConfig,
    #[serde(default)]
    pub security: SecurityConfig,
    #[serde(default)]
    pub learning: LearningConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            backends: BackendsConfig::default(),
            security: SecurityConfig::default(),
            learning: LearningConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_connection_mode")]
    pub connection_mode: String,
    #[serde(default)]
    pub auto_hide_panel: bool,
    #[serde(default)]
    pub launch_at_login: bool,
    #[serde(default = "default_true")]
    pub check_updates: bool,
    #[serde(default = "default_locale")]
    pub locale: String,
    #[serde(default)]
    pub onboarding_completed: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            connection_mode: default_connection_mode(),
            auto_hide_panel: false,
            launch_at_login: false,
            check_updates: true,
            locale: default_locale(),
            onboarding_completed: false,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackendsConfig {
    pub default: Option<String>,
    #[serde(default)]
    pub fallback_order: Vec<String>,
    #[serde(default)]
    pub entries: HashMap<String, BackendEntryConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendEntryConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub binary_path: Option<String>,
    pub default_model: Option<String>,
    #[serde(default)]
    pub extra_args: Vec<String>,
    #[serde(default)]
    pub env_overrides: HashMap<String, String>,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

impl Default for BackendEntryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            binary_path: None,
            default_model: None,
            extra_args: Vec::new(),
            env_overrides: HashMap::new(),
            timeout: default_timeout(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default = "default_execution_mode")]
    pub execution_mode: String,
    #[serde(default)]
    pub allow_rules: Vec<String>,
    #[serde(default = "default_sanitize_patterns")]
    pub sanitize_env_patterns: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            execution_mode: default_execution_mode(),
            allow_rules: Vec::new(),
            sanitize_env_patterns: default_sanitize_patterns(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_confidence_threshold")]
    pub confidence_threshold: f64,
    #[serde(default = "default_decay_rate")]
    pub decay_rate: f64,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            confidence_threshold: default_confidence_threshold(),
            decay_rate: default_decay_rate(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_max_file_size")]
    pub max_file_size_mb: u64,
    #[serde(default = "default_retention_days")]
    pub retention_days: u64,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            max_file_size_mb: default_max_file_size(),
            retention_days: default_retention_days(),
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_true() -> bool {
    true
}
fn default_connection_mode() -> String {
    "standalone".into()
}
fn default_locale() -> String {
    "en".into()
}
fn default_timeout() -> u64 {
    300
}
fn default_execution_mode() -> String {
    "ask".into()
}
fn default_sanitize_patterns() -> Vec<String> {
    vec![
        "*_KEY".into(),
        "*_SECRET".into(),
        "*_TOKEN".into(),
        "*_PASSWORD".into(),
        "*_CREDENTIAL".into(),
        "*_AUTH".into(),
    ]
}
fn default_confidence_threshold() -> f64 {
    0.6
}
fn default_decay_rate() -> f64 {
    0.099
}
fn default_log_level() -> String {
    "info".into()
}
fn default_max_file_size() -> u64 {
    50
}
fn default_retention_days() -> u64 {
    30
}

/// Return the conductor configuration directory, creating it if necessary.
pub fn config_dir() -> PathBuf {
    let dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".conductor");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}

pub fn config_file_path() -> PathBuf {
    config_dir().join("config.json")
}

pub fn load_config(path: &Path) -> AppConfig {
    match std::fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|e| {
            tracing::warn!("config parse error, using defaults: {e}");
            AppConfig::default()
        }),
        Err(_) => AppConfig::default(),
    }
}

pub fn save_config(path: &Path, config: &AppConfig) -> Result<(), std::io::Error> {
    let tmp = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(config).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e)
    })?;
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
