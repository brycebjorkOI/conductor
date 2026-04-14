use crate::events::SessionCommand;
use crate::state::ThinkingLevel;

/// All known session commands for autocomplete.
pub const COMMAND_LIST: &[(&str, &str)] = &[
    ("new", "Start a new session"),
    ("reset", "Clear current session history"),
    ("compact", "Compress conversation context"),
    ("think", "Set thinking depth (off/low/medium/high)"),
    ("verbose", "Toggle verbose output"),
    ("usage", "Show session usage statistics"),
    ("model", "Switch the active model"),
    ("cli", "Switch the active backend"),
    ("status", "Show current session status"),
];

/// Try to parse a session command from user input.
/// Returns `None` if the input is not a command (doesn't start with `/`).
pub fn parse_command(input: &str) -> Option<SessionCommand> {
    let trimmed = input.trim();
    if !trimmed.starts_with('/') {
        return None;
    }

    let without_slash = &trimmed[1..];
    let mut parts = without_slash.splitn(2, ' ');
    let cmd_word = parts.next()?.to_lowercase();
    let arg = parts.next().map(|s| s.trim().to_string());

    match cmd_word.as_str() {
        "new" => Some(SessionCommand::New),
        "reset" => Some(SessionCommand::Reset),
        "compact" => Some(SessionCommand::Compact),
        "verbose" => Some(SessionCommand::Verbose),
        "usage" => Some(SessionCommand::Usage),
        "status" => Some(SessionCommand::Status),
        "think" => {
            let level = match arg.as_deref() {
                Some("off") | Some("none") => ThinkingLevel::Off,
                Some("minimal") => ThinkingLevel::Minimal,
                Some("low") => ThinkingLevel::Low,
                Some("medium") | Some("med") => ThinkingLevel::Medium,
                Some("high") => ThinkingLevel::High,
                Some("xhigh") | Some("extra") | Some("max") => ThinkingLevel::ExtraHigh,
                _ => ThinkingLevel::Medium,
            };
            Some(SessionCommand::Think(level))
        }
        "model" => {
            let name = arg.unwrap_or_default();
            if name.is_empty() {
                Some(SessionCommand::Unknown("model requires a name".into()))
            } else {
                Some(SessionCommand::Model(name))
            }
        }
        "cli" => {
            let name = arg.unwrap_or_default();
            if name.is_empty() {
                Some(SessionCommand::Unknown("cli requires a backend name".into()))
            } else {
                Some(SessionCommand::Cli(name))
            }
        }
        other => Some(SessionCommand::Unknown(format!(
            "unknown command: /{other}"
        ))),
    }
}

/// Return command suggestions that match a given prefix (without the leading `/`).
pub fn autocomplete(prefix: &str) -> Vec<(&'static str, &'static str)> {
    let lower = prefix.to_lowercase();
    COMMAND_LIST
        .iter()
        .filter(|(name, _)| name.starts_with(&lower))
        .copied()
        .collect()
}
