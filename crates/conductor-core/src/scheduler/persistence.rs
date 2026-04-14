//! Job persistence: save/load scheduled jobs, execution history, and PTY
//! session state to disk.

use std::path::PathBuf;

use crate::config;
use crate::state::ScheduledJob;
use super::pty::PtySessionRecord;

/// Directory for scheduler data.
fn scheduler_dir() -> PathBuf {
    let dir = config::config_dir().join("schedules");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    dir
}

fn jobs_file() -> PathBuf {
    scheduler_dir().join("jobs.json")
}

fn pty_sessions_file() -> PathBuf {
    scheduler_dir().join("pty_sessions.json")
}

// ---------------------------------------------------------------------------
// Job persistence
// ---------------------------------------------------------------------------

/// Save all jobs to disk (atomic write).
pub fn save_jobs(jobs: &[ScheduledJob]) -> Result<(), std::io::Error> {
    let path = jobs_file();
    let json = serde_json::to_string_pretty(jobs)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

/// Load jobs from disk.
pub fn load_jobs() -> Vec<ScheduledJob> {
    let path = jobs_file();
    match std::fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|e| {
            tracing::warn!("failed to parse jobs.json: {e}");
            Vec::new()
        }),
        Err(_) => Vec::new(),
    }
}

/// Save a single job's execution history to the history directory.
pub fn save_job_history(
    job_id: &str,
    history: &[crate::state::JobRun],
) -> Result<(), std::io::Error> {
    let dir = scheduler_dir().join("history");
    if !dir.exists() {
        let _ = std::fs::create_dir_all(&dir);
    }
    let path = dir.join(format!("{job_id}.json"));
    let json = serde_json::to_string_pretty(history)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, json)
}

// ---------------------------------------------------------------------------
// PTY session persistence (7.8)
// ---------------------------------------------------------------------------

/// Save active PTY session records so they can be restored after restart.
pub fn save_pty_sessions(records: &[PtySessionRecord]) -> Result<(), std::io::Error> {
    let path = pty_sessions_file();
    let json = serde_json::to_string_pretty(records)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, &path)?;
    Ok(())
}

/// Load PTY session records from the previous run.
pub fn load_pty_sessions() -> Vec<PtySessionRecord> {
    let path = pty_sessions_file();
    match std::fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|e| {
            tracing::warn!("failed to parse pty_sessions.json: {e}");
            Vec::new()
        }),
        Err(_) => Vec::new(),
    }
}

/// Clear the PTY sessions file after restoration.
pub fn clear_pty_sessions() {
    let path = pty_sessions_file();
    let _ = std::fs::remove_file(path);
}
