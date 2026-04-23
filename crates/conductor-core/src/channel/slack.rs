//! Slack adapter backed by `slackdump` CLI.
//!
//! Uses the credentials already stored by slackdump (encrypted in
//! `~/Library/Caches/slackdump/`). No separate token management needed.

use std::path::PathBuf;
use std::process::Stdio;

use tokio::process::Command;

/// Result of listing Slack channels via slackdump.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SlackChannel {
    pub id: String,
    pub name: String,
    pub is_private: bool,
    pub num_members: u64,
    pub topic: String,
}

/// Result of fetching recent messages from a channel.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SlackMessage {
    pub user: String,
    pub user_name: String,
    pub text: String,
    pub ts: String,
    pub thread_ts: Option<String>,
    pub channel_id: String,
}

/// Find the slackdump binary.
pub fn find_slackdump() -> Option<PathBuf> {
    which::which("slackdump").ok()
}

/// Check if slackdump has a workspace configured.
pub async fn has_workspace() -> bool {
    let Some(bin) = find_slackdump() else {
        return false;
    };
    let output = Command::new(&bin)
        .args(["workspace", "list"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;
    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            // Look for "=>" which marks the current workspace.
            stdout.contains("=>")
        }
        Err(_) => false,
    }
}

/// Get the current workspace name from slackdump.
pub async fn current_workspace() -> Result<String, String> {
    let bin = find_slackdump().ok_or("slackdump not found")?;
    let output = Command::new(&bin)
        .args(["workspace", "list"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("failed to run slackdump: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        if line.contains("=>") {
            // Format: "=> ionq (file: ionq.bin, ...)"
            let name = line
                .trim_start_matches("=>")
                .trim()
                .split_whitespace()
                .next()
                .unwrap_or("unknown");
            return Ok(name.to_string());
        }
    }
    Err("no current workspace found".into())
}

/// List channels in the current Slack workspace.
pub async fn list_channels() -> Result<Vec<SlackChannel>, String> {
    let bin = find_slackdump().ok_or("slackdump not found")?;

    let output = Command::new(&bin)
        .args(["list", "channels", "-no-json", "-member-only", "-format", "Text"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("failed to run slackdump: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("slackdump list channels failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_channel_list(&stdout)
}

fn parse_channel_list(output: &str) -> Result<Vec<SlackChannel>, String> {
    let mut channels = Vec::new();
    let mut lines = output.lines();

    // Skip header line ("ID  Arch  What")
    if let Some(header) = lines.next() {
        if !header.contains("ID") {
            // No header — maybe it's a different format. Try parsing anyway.
            if let Some(ch) = parse_channel_line(header) {
                channels.push(ch);
            }
        }
    }

    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(ch) = parse_channel_line(line) {
            channels.push(ch);
        }
    }

    Ok(channels)
}

fn parse_channel_line(line: &str) -> Option<SlackChannel> {
    // Format: "C08B54W6WTD  -     #general"
    // or:     "D0AV8CWJHUY  -     @<external>:U08DDFM0L8L"
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let id = parts[0].to_string();
    let name_raw = parts[2..].join(" ");

    // Skip DMs and external contacts.
    if name_raw.starts_with('@') {
        return None;
    }

    // slackdump prefixes private channels with "🔒 " and public with "#".
    let is_private = name_raw.contains('\u{1f512}') || !name_raw.starts_with('#');
    let name = name_raw
        .trim_start_matches('#')
        .trim_start_matches('\u{1f512}')
        .trim()
        .to_string();

    Some(SlackChannel {
        id,
        name,
        is_private,
        num_members: 0,
        topic: String::new(),
    })
}

/// Fetch recent messages from one or more channels in a single slackdump call.
///
/// Passes all channel IDs to `slackdump dump` at once. Uses `-time-from` to
/// only grab messages since the given timestamp, `-files=false` to skip file
/// downloads, and `-legacy` for JSON output.
pub async fn fetch_recent_messages(
    channel_ids: &[String],
    since: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<SlackMessage>, String> {
    if channel_ids.is_empty() {
        return Ok(Vec::new());
    }

    let bin = find_slackdump().ok_or("slackdump not found")?;

    let time_from = since.format("%Y-%m-%dT%H:%M:%S").to_string();
    let tmp_dir = tempfile::tempdir().map_err(|e| format!("tmpdir: {e}"))?;
    let out_path = tmp_dir.path().join("dump.zip");

    let mut args: Vec<&str> = vec![
        "dump",
        "-time-from",
        &time_from,
        "-files=false",
        "-legacy",
        "-o",
        out_path.to_str().unwrap_or("dump.zip"),
        "-y",
    ];
    // Append all channel IDs.
    for cid in channel_ids {
        args.push(cid.as_str());
    }

    let output = Command::new(&bin)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("slackdump dump failed: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("slackdump dump failed: {stderr}"));
    }

    // Parse the zip — each channel produces a {channel_id}.json file.
    parse_dump_output(&out_path)
}

fn parse_dump_output(zip_path: &std::path::Path) -> Result<Vec<SlackMessage>, String> {
    use std::io::Read;

    let file =
        std::fs::File::open(zip_path).map_err(|e| format!("open zip: {e}"))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("read zip: {e}"))?;

    let mut messages = Vec::new();

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("zip entry: {e}"))?;

        let entry_name = entry.name().to_string();
        if !entry_name.ends_with(".json") {
            continue;
        }

        // Extract channel_id from filename: "{channel_id}.json" or
        // "{channel_id}-{thread_ts}.json".
        let channel_id = entry_name
            .trim_end_matches(".json")
            .split('-')
            .next()
            .unwrap_or("")
            .to_string();

        let mut contents = String::new();
        entry
            .read_to_string(&mut contents)
            .map_err(|e| format!("read entry: {e}"))?;

        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&contents) {
            if let Some(msgs) = val.get("messages").and_then(|m| m.as_array()) {
                for msg in msgs {
                    let text = msg
                        .get("text")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let user = msg
                        .get("user")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let ts = msg
                        .get("ts")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let thread_ts = msg
                        .get("thread_ts")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    if !text.is_empty() {
                        messages.push(SlackMessage {
                            user,
                            user_name: String::new(),
                            text,
                            ts,
                            thread_ts,
                            channel_id: channel_id.clone(),
                        });
                    }
                }
            }
        }
    }

    Ok(messages)
}

/// Search messages across the workspace.
pub async fn search_messages(query: &str) -> Result<Vec<SlackMessage>, String> {
    let bin = find_slackdump().ok_or("slackdump not found")?;

    let tmp_dir = tempfile::tempdir().map_err(|e| format!("tmpdir: {e}"))?;
    let out_path = tmp_dir.path().join("search.zip");

    let output = Command::new(&bin)
        .args([
            "search",
            "messages",
            "-no-channel-users",
            "-o",
            out_path.to_str().unwrap_or("search.zip"),
            query,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("slackdump search failed: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("slackdump search failed: {stderr}"));
    }

    // Search results are in the zip — parse all JSON files.
    let file =
        std::fs::File::open(&out_path).map_err(|e| format!("open zip: {e}"))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("read zip: {e}"))?;

    let mut messages = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("zip entry: {e}"))?;
        if !entry.name().ends_with(".json") {
            continue;
        }
        let mut contents = String::new();
        use std::io::Read;
        entry
            .read_to_string(&mut contents)
            .map_err(|e| format!("read: {e}"))?;

        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&contents) {
            if let Some(msgs) = val.get("messages").and_then(|m| m.as_array()) {
                for msg in msgs {
                    let text = msg
                        .get("text")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let user = msg
                        .get("user")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let ts = msg
                        .get("ts")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let channel = msg
                        .get("channel")
                        .and_then(|c| c.get("id"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    if !text.is_empty() {
                        messages.push(SlackMessage {
                            user,
                            user_name: String::new(),
                            text,
                            ts,
                            thread_ts: None,
                            channel_id: channel,
                        });
                    }
                }
            }
        }
    }

    Ok(messages)
}

/// Parse a Slack message timestamp string into a `DateTime<Utc>`.
pub fn parse_slack_ts(ts: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    let ts_f: f64 = ts.parse().ok()?;
    chrono::DateTime::from_timestamp(ts_f as i64, ((ts_f.fract()) * 1_000_000_000.0) as u32)
}
