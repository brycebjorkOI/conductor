use std::collections::HashMap;

/// Environment sanitization modes per spec 2.13.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SanitizeMode {
    /// Tier 1: Remove dangerous injectors + pattern-matched secrets.
    Standard,
    /// Tier 2: Standard + prevent overrides of critical system variables.
    OverrideBlocked,
    /// Tier 3: Ultra-restrictive — only TERM, LANG, LC_* pass through.
    ShellWrapper,
}

/// Tier 1 blocklist: always removed.
const BLOCKLIST_PREFIXES: &[&str] = &[
    "DYLD_",
    "LD_",
    "NODE_OPTIONS",
    "PYTHONPATH",
    "BASH_FUNC_",
    "PERL5OPT",
    "RUBYOPT",
];

const BLOCKLIST_EXACT: &[&str] = &[
    "LD_PRELOAD",
    "AWS_ACCESS_KEY_ID",
    "AWS_SECRET_ACCESS_KEY",
    "AWS_SESSION_TOKEN",
    "GITHUB_TOKEN",
    "GH_TOKEN",
    "GITLAB_TOKEN",
    "DATABASE_URL",
    "PGPASSWORD",
    "MYSQL_PWD",
    "REDIS_URL",
    "OPENAI_API_KEY",
    "ANTHROPIC_API_KEY",
    "GOOGLE_API_KEY",
    "AZURE_STORAGE_KEY",
];

/// Pattern suffixes that trigger removal in Standard mode.
const SECRET_SUFFIXES: &[&str] = &[
    "_KEY",
    "_SECRET",
    "_TOKEN",
    "_PASSWORD",
    "_CREDENTIAL",
    "_AUTH",
    "_APIKEY",
];

/// Variables that cannot be overridden by backend config (Tier 2).
const OVERRIDE_BLOCKED: &[&str] = &["HOME", "EDITOR", "VISUAL", "GIT_SSH_COMMAND", "SHELL"];

/// Build a sanitized environment map.
pub fn sanitize_env(
    overrides: &HashMap<String, String>,
    mode: SanitizeMode,
) -> HashMap<String, String> {
    match mode {
        SanitizeMode::ShellWrapper => {
            // Tier 3: only pass TERM, LANG, LC_*
            let mut env = HashMap::new();
            for (k, v) in std::env::vars() {
                if k == "TERM" || k == "LANG" || k.starts_with("LC_") {
                    env.insert(k, v);
                }
            }
            env
        }
        SanitizeMode::Standard | SanitizeMode::OverrideBlocked => {
            let mut env: HashMap<String, String> = std::env::vars()
                .filter(|(k, _)| !is_blocked(k))
                .collect();

            // Apply backend-config overrides, respecting Tier 2.
            for (k, v) in overrides {
                if mode == SanitizeMode::OverrideBlocked && OVERRIDE_BLOCKED.contains(&k.as_str())
                {
                    continue; // skip override for protected variables
                }
                if !is_blocked(k) {
                    env.insert(k.clone(), v.clone());
                }
            }

            env
        }
    }
}

fn is_blocked(key: &str) -> bool {
    let upper = key.to_uppercase();

    // Exact match.
    if BLOCKLIST_EXACT.contains(&upper.as_str()) {
        return true;
    }

    // Prefix match.
    for prefix in BLOCKLIST_PREFIXES {
        if upper.starts_with(prefix) {
            return true;
        }
    }

    // Suffix pattern match.
    for suffix in SECRET_SUFFIXES {
        if upper.ends_with(suffix) {
            return true;
        }
    }

    false
}

// ---------------------------------------------------------------------------
// Execution approval
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    Deny,
    Ask,
    Allowlist,
    Auto,
}

/// Check whether a tool invocation matches any rule in the allowlist.
pub fn matches_allowlist(tool_name: &str, args: &str, rules: &[String]) -> bool {
    for rule in rules {
        let parts: Vec<&str> = rule.splitn(2, ' ').collect();
        let rule_tool = parts.first().unwrap_or(&"");
        let rule_pattern = parts.get(1).unwrap_or(&"*");

        if *rule_tool == tool_name || *rule_tool == "*" {
            if glob_match(rule_pattern, args) {
                return true;
            }
        }
    }
    false
}

/// Simple glob matching (supports `*` and `?`).
fn glob_match(pattern: &str, text: &str) -> bool {
    glob::Pattern::new(pattern)
        .map(|p| p.matches(text))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocked_keys() {
        assert!(is_blocked("AWS_SECRET_ACCESS_KEY"));
        assert!(is_blocked("DYLD_INSERT_LIBRARIES"));
        assert!(is_blocked("LD_PRELOAD"));
        assert!(is_blocked("MY_API_KEY"));
        assert!(is_blocked("GITHUB_TOKEN"));
        assert!(!is_blocked("HOME"));
        assert!(!is_blocked("PATH"));
        assert!(!is_blocked("TERM"));
    }

    #[test]
    fn test_allowlist_matching() {
        let rules = vec![
            "file_read *".to_string(),
            "shell_exec cargo *".to_string(),
        ];
        assert!(matches_allowlist("file_read", "src/main.rs", &rules));
        assert!(matches_allowlist("shell_exec", "cargo test", &rules));
        assert!(!matches_allowlist("shell_exec", "rm -rf /", &rules));
    }
}
