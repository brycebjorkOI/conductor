//! Schedule engine: cron parsing, trigger evaluation, job execution, and
//! persistence.

pub mod persistence;
pub mod pty;

use std::str::FromStr;
use std::time::Instant;

use chrono::{DateTime, Utc};
use cron::Schedule;

use crate::backend::definitions::all_known_backends;
use crate::backend::orchestrator::{self, SendOutcome};
use crate::backend::{ChatParams, StreamEvent};
use crate::connector::fetch;
use crate::state::*;

// ---------------------------------------------------------------------------
// Cron evaluation (7.1)
// ---------------------------------------------------------------------------

/// Parse a cron expression and return the next occurrence after `after`.
pub fn next_cron_occurrence(expression: &str, after: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let schedule = Schedule::from_str(expression).ok()?;
    schedule.after(&after).next()
}

/// Check if a cron expression matches the current time (within a 30-second window).
pub fn cron_matches_now(expression: &str, now: DateTime<Utc>) -> bool {
    let schedule = match Schedule::from_str(expression) {
        Ok(s) => s,
        Err(_) => return false,
    };
    // Check if there's an occurrence within the last 30 seconds.
    let window_start = now - chrono::Duration::seconds(30);
    schedule.after(&window_start).next().map_or(false, |next| next <= now)
}

// ---------------------------------------------------------------------------
// Trigger evaluation (7.2)
// ---------------------------------------------------------------------------

/// Evaluate which jobs should trigger at the current time.
pub fn evaluate_triggers(jobs: &[ScheduledJob], now: DateTime<Utc>) -> Vec<JobId> {
    jobs.iter()
        .filter(|job| job.enabled && should_trigger(job, now))
        .map(|job| job.job_id.clone())
        .collect()
}

fn should_trigger(job: &ScheduledJob, now: DateTime<Utc>) -> bool {
    match &job.schedule {
        ScheduleDefinition::OneTime { datetime } => {
            now >= *datetime
                && !job.history.iter().any(|r| r.status == JobRunStatus::Success)
        }
        ScheduleDefinition::Interval { seconds } => {
            let interval = chrono::Duration::seconds(*seconds as i64);
            let last_run = job.history.iter().filter_map(|r| r.completed_at).max();
            match last_run {
                Some(last) => now >= last + interval,
                None => true,
            }
        }
        ScheduleDefinition::Cron { expression, .. } => {
            cron_matches_now(expression, now)
        }
    }
}

/// Compute the next run time for a job.
pub fn compute_next_run(job: &ScheduledJob, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    match &job.schedule {
        ScheduleDefinition::OneTime { datetime } => {
            if now < *datetime {
                Some(*datetime)
            } else {
                None
            }
        }
        ScheduleDefinition::Interval { seconds } => {
            let interval = chrono::Duration::seconds(*seconds as i64);
            let last = job
                .history
                .iter()
                .filter_map(|r| r.completed_at)
                .max()
                .unwrap_or(now);
            Some(last + interval)
        }
        ScheduleDefinition::Cron { expression, .. } => {
            next_cron_occurrence(expression, now)
        }
    }
}

// ---------------------------------------------------------------------------
// Retry logic (7.6)
// ---------------------------------------------------------------------------

/// Calculate retry delay based on policy and attempt number.
/// Returns `None` if max retries exceeded.
pub fn retry_delay(policy: &RetryPolicy, attempt: u32) -> Option<std::time::Duration> {
    if attempt >= policy.max_retries {
        return None;
    }
    let base = policy.initial_delay_seconds as f64;
    let delay = match policy.backoff_strategy {
        BackoffStrategy::None => base,
        BackoffStrategy::Linear => base * (attempt as f64 + 1.0),
        BackoffStrategy::Exponential => base * 2f64.powi(attempt as i32),
    };
    Some(std::time::Duration::from_secs_f64(delay))
}

/// Create a new job run entry.
pub fn new_job_run() -> JobRun {
    JobRun {
        run_id: uuid::Uuid::new_v4().to_string(),
        started_at: Utc::now(),
        completed_at: None,
        duration_ms: None,
        status: JobRunStatus::Running,
        error: None,
        output_summary: None,
    }
}

// ---------------------------------------------------------------------------
// Job execution — local scheduler path (7.3)
// ---------------------------------------------------------------------------

/// Execute a scheduled job by spawning the backend CLI.
///
/// If `connector_data` is provided (from bound connector queries), it is
/// prepended to the job's prompt as additional context.
pub async fn execute_job(
    job: &ScheduledJob,
    backend_registry: &[BackendStatus],
    fallback_order: &[String],
    default_backend_id: &str,
    connector_data: Option<&str>,
) -> JobRun {
    let mut run = new_job_run();
    let start = Instant::now();

    let backend_id = job
        .payload
        .backend_override
        .as_deref()
        .unwrap_or(default_backend_id);

    // Find the backend.
    let selected = orchestrator::select_backend(backend_registry, backend_id, fallback_order);
    let Some((bs, binary_path)) = selected else {
        run.status = JobRunStatus::Failure;
        run.error = Some(format!("No available backend for job '{}'", job.name));
        run.completed_at = Some(Utc::now());
        run.duration_ms = Some(start.elapsed().as_millis() as u64);
        return run;
    };

    let defs = all_known_backends();
    let def = match defs.iter().find(|d| d.backend_id() == bs.backend_id) {
        Some(d) => d,
        None => {
            run.status = JobRunStatus::Failure;
            run.error = Some(format!("Backend definition not found: {}", bs.backend_id));
            run.completed_at = Some(Utc::now());
            run.duration_ms = Some(start.elapsed().as_millis() as u64);
            return run;
        }
    };

    // Build the message: inject connector data before the prompt if available.
    let message = match connector_data {
        Some(data) if !data.is_empty() => {
            format!(
                "Context from connected data sources:\n{}\n\n---\n\n{}",
                fetch::truncate_result(data, 4000),
                job.payload.prompt,
            )
        }
        _ => job.payload.prompt.clone(),
    };

    let params = ChatParams {
        message,
        model: job.payload.model_override.clone(),
        attachments: Vec::new(),
        thinking_level: job.payload.thinking_level,
        extra_args: bs.custom_args.clone(),
        system_prompt: None,
    };

    let (_cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();

    let outcome = orchestrator::run_chat(
        def.as_ref(),
        &binary_path.clone(),
        params,
        None,
        &bs.env_overrides,
        cancel_rx,
    )
    .await;

    match outcome {
        SendOutcome::Completed { events, duration_ms } => {
            // Collect text from events.
            let mut text = String::new();
            for event in &events {
                if let StreamEvent::TextChunk(chunk) = event {
                    text.push_str(chunk);
                }
            }
            let summary = if text.len() > 200 {
                format!("{}...", &text[..197])
            } else {
                text
            };

            run.status = JobRunStatus::Success;
            run.output_summary = Some(summary);
            run.duration_ms = Some(duration_ms);
        }
        SendOutcome::Cancelled => {
            run.status = JobRunStatus::Cancelled;
            run.duration_ms = Some(start.elapsed().as_millis() as u64);
        }
        SendOutcome::Error(e) => {
            run.status = JobRunStatus::Failure;
            run.error = Some(e);
            run.duration_ms = Some(start.elapsed().as_millis() as u64);
        }
    }

    run.completed_at = Some(Utc::now());
    run
}

// ---------------------------------------------------------------------------
// Delivery (7.3 continued)
// ---------------------------------------------------------------------------

/// Deliver a job result according to the delivery config.
pub async fn deliver_result(delivery: &DeliveryConfig, output: &str) {
    match delivery {
        DeliveryConfig::Silent => {
            tracing::info!("scheduled job result (silent): {}", &output[..output.len().min(100)]);
        }
        DeliveryConfig::ChannelAnnounce { channel_id } => {
            tracing::info!(
                "scheduled job result -> channel {}: {}",
                channel_id,
                &output[..output.len().min(100)]
            );
            // Channel delivery requires the channel manager to be wired up.
        }
        DeliveryConfig::Webhook { url } => {
            tracing::info!("scheduled job result -> webhook {}: {}", url, &output[..output.len().min(100)]);
            // Webhook delivery requires reqwest.
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: create a new ScheduledJob with defaults
// ---------------------------------------------------------------------------

pub fn create_job(
    name: String,
    schedule: ScheduleDefinition,
    prompt: String,
    execution_mode: ExecutionMode,
    delivery: DeliveryConfig,
) -> ScheduledJob {
    let now = Utc::now();
    let mut job = ScheduledJob {
        job_id: uuid::Uuid::new_v4().to_string(),
        name,
        enabled: true,
        schedule,
        payload: JobPayload {
            prompt,
            execution_mode,
            backend_override: None,
            model_override: None,
            thinking_level: None,
            timeout_seconds: None,
        },
        delivery,
        retry_policy: RetryPolicy::default(),
        history: Vec::new(),
        next_run: None,
    };
    job.next_run = compute_next_run(&job, now);
    job
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_delay_exponential() {
        let policy = RetryPolicy {
            max_retries: 3,
            backoff_strategy: BackoffStrategy::Exponential,
            initial_delay_seconds: 5,
        };
        assert_eq!(retry_delay(&policy, 0).unwrap().as_secs(), 5);
        assert_eq!(retry_delay(&policy, 1).unwrap().as_secs(), 10);
        assert_eq!(retry_delay(&policy, 2).unwrap().as_secs(), 20);
        assert!(retry_delay(&policy, 3).is_none());
    }

    #[test]
    fn test_cron_next_occurrence() {
        // Every minute.
        let next = next_cron_occurrence("* * * * * *", Utc::now());
        assert!(next.is_some());
        assert!(next.unwrap() > Utc::now() - chrono::Duration::seconds(2));
    }

    #[test]
    fn test_compute_next_run_interval() {
        let job = create_job(
            "test".into(),
            ScheduleDefinition::Interval { seconds: 3600 },
            "hello".into(),
            ExecutionMode::Isolated,
            DeliveryConfig::Silent,
        );
        let next = compute_next_run(&job, Utc::now());
        assert!(next.is_some());
    }

    #[test]
    fn test_create_job_defaults() {
        let job = create_job(
            "Daily summary".into(),
            ScheduleDefinition::Interval { seconds: 86400 },
            "Summarize my day".into(),
            ExecutionMode::Isolated,
            DeliveryConfig::Silent,
        );
        assert!(job.enabled);
        assert!(job.next_run.is_some());
        assert_eq!(job.retry_policy.max_retries, 3);
    }
}
