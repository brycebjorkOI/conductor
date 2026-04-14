//! Git directive parsing and safety filtering.
//!
//! Supports `@git(command)` (auto-executed reads) and
//! `@git-confirm(command)` (write directives requiring user confirmation).


/// A parsed git directive from an AI response.
#[derive(Debug, Clone)]
pub enum GitDirective {
    /// Read-only command, auto-executed without confirmation.
    Read(String),
    /// Write command, requires user confirmation before execution.
    Write(String),
}

/// Git subcommands that are safe to auto-execute (read-only).
const READ_COMMANDS: &[&str] = &[
    "log", "diff", "status", "show", "blame", "branch", "tag", "remote",
    "ls-files", "shortlog", "describe", "rev-parse", "cat-file",
];

/// Commands that are ALWAYS blocked from auto-execution, even in auto-approve mode.
const BLOCKED_PATTERNS: &[&str] = &[
    "push --force",
    "push -f",
    "reset --hard",
    "clean -f",
    "clean -fd",
    "checkout -- .",
    "stash drop",
    "stash clear",
];

/// Parse `@git(...)` and `@git-confirm(...)` directives from AI response text.
pub fn parse_directives(text: &str) -> Vec<GitDirective> {
    let mut directives = Vec::new();

    // @git(command)
    for (start, _) in text.match_indices("@git(") {
        if text[..start].ends_with('-') {
            continue; // skip @git-confirm matched partially
        }
        if let Some(cmd) = extract_parens(&text[start + 5..]) {
            directives.push(GitDirective::Read(cmd));
        }
    }

    // @git-confirm(command)
    for (start, _) in text.match_indices("@git-confirm(") {
        if let Some(cmd) = extract_parens(&text[start + 13..]) {
            directives.push(GitDirective::Write(cmd));
        }
    }

    directives
}

fn extract_parens(s: &str) -> Option<String> {
    let end = s.find(')')?;
    let inner = s[..end].trim().to_string();
    if inner.is_empty() {
        None
    } else {
        Some(inner)
    }
}

/// Check whether a git command is a safe read-only operation.
pub fn is_read_safe(command: &str) -> bool {
    let first_word = command.split_whitespace().next().unwrap_or("");
    READ_COMMANDS.contains(&first_word)
}

/// Check whether a command matches the blocked patterns list.
pub fn is_blocked(command: &str) -> bool {
    let lower = command.to_lowercase();
    BLOCKED_PATTERNS.iter().any(|pat| lower.contains(pat))
}

/// Check if a command is destructive (rebase, force-push, reset).
pub fn is_destructive(command: &str) -> bool {
    let lower = command.to_lowercase();
    lower.contains("rebase")
        || lower.contains("--force")
        || lower.contains("-f ")
        || lower.contains("reset --hard")
        || lower.contains("branch -D")
        || lower.contains("branch -d")
}

/// Filter sensitive content from git output (API keys, tokens, passwords).
pub fn redact_sensitive(text: &str) -> String {
    let patterns = [
        "password", "token", "secret", "api_key", "apikey", "auth",
        "credential", "private_key", "access_key",
    ];
    let mut result = text.to_string();
    for line in text.lines() {
        let lower = line.to_lowercase();
        for pattern in &patterns {
            if lower.contains(pattern) {
                result = result.replace(line, "[REDACTED]");
                break;
            }
        }
    }
    result
}

/// Truncate git output to the maximum allowed size.
pub fn truncate_output(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        return text.to_string();
    }
    // Break at the last newline before the limit.
    let truncated = &text[..max_chars];
    let last_nl = truncated.rfind('\n').unwrap_or(max_chars);
    format!("{}\n\n[Output truncated at {} characters]", &text[..last_nl], max_chars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_read_directive() {
        let text = "Let me check: @git(log --oneline -5)";
        let dirs = parse_directives(text);
        assert_eq!(dirs.len(), 1);
        assert!(matches!(&dirs[0], GitDirective::Read(cmd) if cmd == "log --oneline -5"));
    }

    #[test]
    fn test_parse_write_directive() {
        let text = "I'll push: @git-confirm(push origin main)";
        let dirs = parse_directives(text);
        assert_eq!(dirs.len(), 1);
        assert!(matches!(&dirs[0], GitDirective::Write(cmd) if cmd == "push origin main"));
    }

    #[test]
    fn test_blocked_command() {
        assert!(is_blocked("push --force origin main"));
        assert!(is_blocked("reset --hard HEAD~1"));
        assert!(!is_blocked("push origin main"));
    }

    #[test]
    fn test_read_safe() {
        assert!(is_read_safe("log --oneline"));
        assert!(is_read_safe("diff HEAD~1"));
        assert!(!is_read_safe("push origin main"));
    }
}
