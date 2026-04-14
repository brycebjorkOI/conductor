use std::path::{Path, PathBuf};

use crate::config;
use crate::state::{Session, SessionType};

/// Create a new primary session with a fresh UUID.
pub fn create_session(backend_id: &str) -> Session {
    let id = uuid::Uuid::new_v4().to_string();
    Session::new(id, SessionType::Primary, backend_id)
}

/// Return the directory where session files are stored.
pub fn sessions_dir() -> PathBuf {
    let dir = config::config_dir().join("sessions");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}

/// Persist a session to disk as JSON.
pub fn save_session(session: &Session) -> Result<(), std::io::Error> {
    let dir = sessions_dir();
    let path = dir.join(format!("{}.json", session.id));
    let json =
        serde_json::to_string_pretty(session).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, json)
}

/// Load a single session from disk.
pub fn load_session(path: &Path) -> Option<Session> {
    let contents = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&contents).ok()
}

/// Load all persisted sessions.
pub fn load_all_sessions() -> Vec<Session> {
    let dir = sessions_dir();
    let mut sessions = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Some(session) = load_session(&path) {
                    sessions.push(session);
                }
            }
        }
    }
    sessions
}

/// Auto-generate a display name from the first user message.
pub fn auto_display_name(session: &Session) -> String {
    for msg in &session.messages {
        if msg.role == crate::state::MessageRole::User {
            let text = msg.content.trim();
            if text.len() > 40 {
                return format!("{}...", &text[..37]);
            } else if !text.is_empty() {
                return text.to_string();
            }
        }
    }
    "New Chat".to_string()
}
