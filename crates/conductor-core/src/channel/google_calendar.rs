//! Google Calendar adapter using `gcloud` for auth and the Calendar REST API.
//!
//! Uses `gcloud auth print-access-token` to get short-lived tokens from the
//! user's existing gcloud authentication. No separate OAuth flow needed.

use std::process::Stdio;

use chrono::{DateTime, Utc};
use tokio::process::Command;

/// A calendar event from the Google Calendar API.
#[derive(Debug, Clone)]
pub struct CalendarEvent {
    pub id: String,
    pub summary: String,
    pub description: Option<String>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub calendar_id: String,
    pub html_link: Option<String>,
    pub attendees: Vec<String>,
}

/// A calendar entry from the calendar list.
#[derive(Debug, Clone)]
pub struct CalendarListEntry {
    pub id: String,
    pub summary: String,
    pub primary: bool,
}

/// Check if gcloud is available and authenticated.
pub async fn check_gcloud() -> Result<String, String> {
    let gcloud = which::which("gcloud").map_err(|_| "gcloud not found in PATH")?;

    let output = Command::new(&gcloud)
        .args(["auth", "print-access-token", "--scopes=https://www.googleapis.com/auth/calendar.readonly"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("failed to run gcloud: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gcloud auth failed: {stderr}"));
    }

    // Get the active account name.
    let account_output = Command::new(&gcloud)
        .args(["config", "get-value", "account"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("failed to get account: {e}"))?;

    let account = String::from_utf8_lossy(&account_output.stdout).trim().to_string();
    Ok(account)
}

/// Get a fresh access token from gcloud.
async fn get_access_token() -> Result<String, String> {
    let gcloud = which::which("gcloud").map_err(|_| "gcloud not found")?;

    let output = Command::new(&gcloud)
        .args(["auth", "print-access-token", "--scopes=https://www.googleapis.com/auth/calendar.readonly"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("gcloud auth: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gcloud auth failed: {stderr}"));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// List the user's calendars.
pub async fn list_calendars() -> Result<Vec<CalendarListEntry>, String> {
    let token = get_access_token().await?;

    let client = reqwest::Client::new();
    let resp = client
        .get("https://www.googleapis.com/calendar/v3/users/me/calendarList")
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|e| format!("calendar list request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("calendar list failed ({status}): {body}"));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("parse calendar list: {e}"))?;

    let mut calendars = Vec::new();
    if let Some(items) = json.get("items").and_then(|v| v.as_array()) {
        for item in items {
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let summary = item
                .get("summary")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let primary = item
                .get("primary")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if !id.is_empty() {
                calendars.push(CalendarListEntry {
                    id,
                    summary,
                    primary,
                });
            }
        }
    }

    Ok(calendars)
}

/// Fetch upcoming events from a specific calendar within a time window.
pub async fn fetch_upcoming_events(
    calendar_id: &str,
    time_min: DateTime<Utc>,
    time_max: DateTime<Utc>,
) -> Result<Vec<CalendarEvent>, String> {
    let token = get_access_token().await?;

    let url = format!(
        "https://www.googleapis.com/calendar/v3/calendars/{}/events",
        urlencod(calendar_id),
    );

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .bearer_auth(&token)
        .query(&[
            ("timeMin", time_min.to_rfc3339()),
            ("timeMax", time_max.to_rfc3339()),
            ("singleEvents", "true".into()),
            ("orderBy", "startTime".into()),
            ("maxResults", "50".into()),
        ])
        .send()
        .await
        .map_err(|e| format!("events request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("events fetch failed ({status}): {body}"));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("parse events: {e}"))?;

    let mut events = Vec::new();
    if let Some(items) = json.get("items").and_then(|v| v.as_array()) {
        for item in items {
            if let Some(event) = parse_event(item, calendar_id) {
                events.push(event);
            }
        }
    }

    Ok(events)
}

fn parse_event(item: &serde_json::Value, calendar_id: &str) -> Option<CalendarEvent> {
    let id = item.get("id")?.as_str()?.to_string();
    let summary = item
        .get("summary")
        .and_then(|v| v.as_str())
        .unwrap_or("(No title)")
        .to_string();
    let description = item
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let html_link = item
        .get("htmlLink")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Parse start time — could be dateTime (timed event) or date (all-day).
    let start_obj = item.get("start")?;
    let start = parse_datetime(start_obj)?;

    let end_obj = item.get("end")?;
    let end = parse_datetime(end_obj).unwrap_or(start);

    let attendees = item
        .get("attendees")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|a| a.get("email").and_then(|e| e.as_str()))
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    Some(CalendarEvent {
        id,
        summary,
        description,
        start,
        end,
        calendar_id: calendar_id.to_string(),
        html_link,
        attendees,
    })
}

fn parse_datetime(obj: &serde_json::Value) -> Option<DateTime<Utc>> {
    // Try dateTime first (timed event), then date (all-day).
    if let Some(dt) = obj.get("dateTime").and_then(|v| v.as_str()) {
        return DateTime::parse_from_rfc3339(dt)
            .ok()
            .map(|d| d.with_timezone(&Utc));
    }
    if let Some(d) = obj.get("date").and_then(|v| v.as_str()) {
        // All-day event: "2026-04-24" → midnight UTC.
        return chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .ok()
            .and_then(|nd| nd.and_hms_opt(0, 0, 0))
            .map(|ndt| ndt.and_utc());
    }
    None
}

/// Simple percent-encoding for calendar IDs (which may contain @ and .).
fn urlencod(s: &str) -> String {
    s.replace('@', "%40").replace('#', "%23")
}
