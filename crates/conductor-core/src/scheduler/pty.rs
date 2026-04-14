//! PTY background sessions for backends with native interactive scheduling.
//!
//! Some backends (e.g., Anthropic CLI) support built-in loop/repeat commands
//! that run inside a persistent interactive session. These require terminal
//! emulation (PTY) rather than standard pipes.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;

use crate::state::JobId;

/// Maximum session lifetime before auto-renewal (~3 days).
const SESSION_EXPIRY_SECS: u64 = 3 * 24 * 3600;

/// A single active PTY session.
pub struct PtySession {
    pub job_id: JobId,
    pub backend_id: String,
    pub command: String,
    pub started_at: DateTime<Utc>,
    child: tokio::process::Child,
    /// Lines read from the PTY so far (ring buffer, last N lines).
    output_lines: Vec<String>,
}

impl PtySession {
    /// Check whether this session has expired.
    pub fn is_expired(&self) -> bool {
        let age = Utc::now() - self.started_at;
        age.num_seconds() as u64 > SESSION_EXPIRY_SECS
    }
}

/// Manages all active PTY sessions.
pub struct PtyManager {
    sessions: Arc<Mutex<HashMap<JobId, PtySession>>>,
}

impl PtyManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Spawn a new PTY session for a job. Issues the given command string to
    /// the backend's interactive CLI.
    pub async fn spawn_session(
        &self,
        job_id: &str,
        backend_id: &str,
        binary_path: &PathBuf,
        command: &str,
    ) -> Result<(), String> {
        // Check if session already exists.
        {
            let sessions = self.sessions.lock().await;
            if sessions.contains_key(job_id) {
                return Ok(()); // already running
            }
        }

        // Spawn the CLI in interactive mode.
        let mut child = Command::new(binary_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("failed to spawn PTY session: {e}"))?;

        // Write the scheduling command to stdin.
        if let Some(ref mut stdin) = child.stdin {
            stdin
                .write_all(format!("{command}\n").as_bytes())
                .await
                .map_err(|e| format!("failed to write to PTY stdin: {e}"))?;
        }

        let session = PtySession {
            job_id: job_id.to_string(),
            backend_id: backend_id.to_string(),
            command: command.to_string(),
            started_at: Utc::now(),
            child,
            output_lines: Vec::new(),
        };

        let mut sessions = self.sessions.lock().await;
        sessions.insert(job_id.to_string(), session);

        tracing::info!("PTY session spawned for job {job_id} with command: {command}");
        Ok(())
    }

    /// Check if a PTY session exists for a given job.
    pub async fn has_session(&self, job_id: &str) -> bool {
        self.sessions.lock().await.contains_key(job_id)
    }

    /// Read any new output lines from a PTY session (non-blocking).
    pub async fn read_output(&self, job_id: &str) -> Vec<String> {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get_mut(job_id) {
            if let Some(ref mut stdout) = session.child.stdout {
                let mut reader = BufReader::new(stdout);
                let mut lines = Vec::new();
                // Non-blocking: try to read available lines.
                loop {
                    let mut line = String::new();
                    match tokio::time::timeout(Duration::from_millis(50), reader.read_line(&mut line)).await {
                        Ok(Ok(0)) => break, // EOF
                        Ok(Ok(_)) => {
                            let trimmed = line.trim_end().to_string();
                            session.output_lines.push(trimmed.clone());
                            // Keep only last 100 lines.
                            if session.output_lines.len() > 100 {
                                session.output_lines.remove(0);
                            }
                            lines.push(trimmed);
                        }
                        Ok(Err(_)) => break,
                        Err(_) => break, // timeout, no more data available
                    }
                }
                return lines;
            }
        }
        Vec::new()
    }

    /// Check for and renew any expired sessions.
    pub async fn renew_expired_sessions(&self) -> Vec<(JobId, String, String)> {
        let mut to_renew = Vec::new();
        let mut sessions = self.sessions.lock().await;

        let expired: Vec<JobId> = sessions
            .iter()
            .filter(|(_, s)| s.is_expired())
            .map(|(id, _)| id.clone())
            .collect();

        for id in expired {
            if let Some(session) = sessions.remove(&id) {
                tracing::info!("PTY session for job {} expired, will renew", id);
                // Kill the old process.
                let mut child = session.child;
                let _ = child.kill().await;
                to_renew.push((id, session.backend_id, session.command));
            }
        }

        to_renew
    }

    /// Gracefully shut down all PTY sessions.
    pub async fn shutdown_all(&self) {
        let mut sessions = self.sessions.lock().await;
        for (id, mut session) in sessions.drain() {
            tracing::info!("shutting down PTY session for job {id}");
            let _ = session.child.kill().await;
        }
    }

    /// Get persistence data for all active sessions.
    pub async fn get_persistence_data(&self) -> Vec<PtySessionRecord> {
        let sessions = self.sessions.lock().await;
        sessions
            .values()
            .map(|s| PtySessionRecord {
                job_id: s.job_id.clone(),
                backend_id: s.backend_id.clone(),
                last_command: s.command.clone(),
                started_at: s.started_at,
            })
            .collect()
    }

    /// List all active session job IDs.
    pub async fn active_job_ids(&self) -> Vec<JobId> {
        self.sessions.lock().await.keys().cloned().collect()
    }
}

impl Default for PtyManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Serializable record for persisting PTY session state across restarts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtySessionRecord {
    pub job_id: JobId,
    pub backend_id: String,
    pub last_command: String,
    pub started_at: DateTime<Utc>,
}

/// Determine whether a job should use the PTY path based on backend capabilities.
pub fn should_use_pty(
    backend_id: &str,
    backend_registry: &[crate::state::BackendStatus],
) -> bool {
    backend_registry
        .iter()
        .find(|b| b.backend_id == backend_id)
        .map_or(false, |b| b.capabilities.interactive_session)
}

/// Build the native scheduling command for a PTY session.
/// For Anthropic CLI, this would be something like `/loop 30m "prompt text"`.
pub fn build_pty_command(
    backend_id: &str,
    interval_seconds: u64,
    prompt: &str,
) -> Option<String> {
    match backend_id {
        "anthropic" => {
            let interval_str = if interval_seconds >= 3600 {
                format!("{}h", interval_seconds / 3600)
            } else if interval_seconds >= 60 {
                format!("{}m", interval_seconds / 60)
            } else {
                format!("{}s", interval_seconds)
            };
            Some(format!("/loop {interval_str} \"{prompt}\""))
        }
        _ => None, // other backends don't have native scheduling commands
    }
}
