use std::collections::HashMap;

use tokio::sync::{mpsc, oneshot};

use conductor_core::backend::definitions::all_known_backends;
use conductor_core::backend::orchestrator::{self, SendOutcome};
use conductor_core::backend::{ChatParams, StreamEvent};
use conductor_core::commands;
use conductor_core::events::{Action, SessionCommand};
use conductor_core::session;
use conductor_core::state::*;

use crate::bridge::SharedState;

/// Handles for communicating with the async runtime.
pub struct RuntimeHandle {
    pub action_tx: mpsc::UnboundedSender<Action>,
    _runtime: tokio::runtime::Runtime,
}

impl RuntimeHandle {
    pub fn start(shared: SharedState) -> Self {
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("failed to create tokio runtime");

        let shared_clone = shared.clone();
        let tx_clone = action_tx.clone();
        runtime.spawn(async move {
            dispatcher(action_rx, shared_clone, tx_clone).await;
        });

        // Kick off initial backend scan.
        let shared_scan = shared.clone();
        runtime.spawn(async move {
            run_backend_scan(shared_scan).await;
        });

        // Auto-connect to Slack if slackdump is available.
        let shared_slack = shared.clone();
        let tx_slack = action_tx.clone();
        runtime.spawn(async move {
            slack_connect(shared_slack, tx_slack).await;
        });

        // Load persisted automation rules.
        {
            let saved_rules = conductor_core::automation::load_rules();
            if !saved_rules.is_empty() {
                tracing::info!("restored {} automation rules", saved_rules.len());
                shared.mutate(|s| {
                    s.automation_rules = saved_rules;
                });
            }
        }

        // Load persisted jobs and start the scheduler loop.
        let shared_sched = shared.clone();
        let tx_sched = action_tx.clone();
        runtime.spawn(async move {
            use conductor_core::scheduler::{pty, persistence};

            // Load persisted jobs.
            let saved_jobs = persistence::load_jobs();
            if !saved_jobs.is_empty() {
                tracing::info!("restored {} scheduled jobs", saved_jobs.len());
                shared_sched.mutate(|s| {
                    s.scheduler.jobs = saved_jobs;
                    let now = chrono::Utc::now();
                    for job in &mut s.scheduler.jobs {
                        job.next_run = conductor_core::scheduler::compute_next_run(job, now);
                    }
                });
            }

            // Restore PTY sessions from previous run.
            let pty_manager = pty::PtyManager::new();
            let saved_pty = persistence::load_pty_sessions();
            for record in &saved_pty {
                let backend_path = {
                    let state = shared_sched.read();
                    state.backend_registry.iter()
                        .find(|b| b.backend_id == record.backend_id)
                        .and_then(|b| b.binary_path.clone())
                };
                match backend_path {
                    Some(path) => {
                        match pty_manager.spawn_session(
                            &record.job_id,
                            &record.backend_id,
                            &path,
                            &record.last_command,
                        ).await {
                            Ok(()) => {
                                tracing::info!("restored PTY session for job {}", record.job_id);
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "failed to restore PTY session for job {}: {e} — falling back to local scheduler",
                                    record.job_id
                                );
                                // Add a notification for the user.
                                shared_sched.mutate(|s| {
                                    s.notifications.push(conductor_core::state::Notification {
                                        id: uuid::Uuid::new_v4().to_string(),
                                        title: "PTY Session Restore Failed".into(),
                                        body: format!(
                                            "Job '{}' could not restore its PTY session: {e}. Using local scheduler instead.",
                                            record.job_id
                                        ),
                                        severity: conductor_core::state::NotificationSeverity::Warning,
                                        timestamp: chrono::Utc::now(),
                                        dismissed: false,
                                    });
                                });
                            }
                        }
                    }
                    None => {
                        tracing::warn!(
                            "cannot restore PTY for job {}: backend '{}' binary not found — falling back to local scheduler",
                            record.job_id, record.backend_id
                        );
                    }
                }
            }
            persistence::clear_pty_sessions();

            // Scheduler loop: evaluate triggers every 30 seconds.
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                let now = chrono::Utc::now();

                // Check for expired PTY sessions and renew them.
                let to_renew = pty_manager.renew_expired_sessions().await;
                for (job_id, backend_id, command) in to_renew {
                    let backend_path = {
                        let state = shared_sched.read();
                        state.backend_registry.iter()
                            .find(|b| b.backend_id == backend_id)
                            .and_then(|b| b.binary_path.clone())
                    };
                    if let Some(path) = backend_path {
                        let _ = pty_manager.spawn_session(&job_id, &backend_id, &path, &command).await;
                    }
                }

                // Evaluate triggers.
                let triggered = {
                    let state = shared_sched.read();
                    conductor_core::scheduler::evaluate_triggers(&state.scheduler.jobs, now)
                };
                for job_id in triggered {
                    // Determine whether to use PTY or local scheduler path.
                    let use_pty = {
                        let state = shared_sched.read();
                        state.scheduler.jobs.iter()
                            .find(|j| j.job_id == job_id)
                            .and_then(|j| {
                                let bid = j.payload.backend_override.as_deref()
                                    .unwrap_or(state.default_backend_id.as_deref().unwrap_or(""));
                                Some(pty::should_use_pty(bid, &state.backend_registry))
                            })
                            .unwrap_or(false)
                    };

                    if use_pty && !pty_manager.has_session(&job_id).await {
                        // Spawn a PTY session for this job.
                        let (backend_id, prompt, interval) = {
                            let state = shared_sched.read();
                            let job = state.scheduler.jobs.iter().find(|j| j.job_id == job_id);
                            match job {
                                Some(j) => {
                                    let bid = j.payload.backend_override.clone()
                                        .unwrap_or_else(|| state.default_backend_id.clone().unwrap_or_default());
                                    let interval = match &j.schedule {
                                        conductor_core::state::ScheduleDefinition::Interval { seconds } => *seconds,
                                        _ => 3600,
                                    };
                                    (bid, j.payload.prompt.clone(), interval)
                                }
                                None => continue,
                            }
                        };

                        if let Some(cmd) = pty::build_pty_command(&backend_id, interval, &prompt) {
                            let backend_path = {
                                let state = shared_sched.read();
                                state.backend_registry.iter()
                                    .find(|b| b.backend_id == backend_id)
                                    .and_then(|b| b.binary_path.clone())
                            };
                            if let Some(path) = backend_path {
                                if let Err(e) = pty_manager.spawn_session(&job_id, &backend_id, &path, &cmd).await {
                                    tracing::warn!("PTY spawn failed for {job_id}: {e}, falling back to local");
                                    let _ = tx_sched.send(Action::RunJobNow { job_id });
                                } else {
                                    tracing::info!("PTY session started for job {job_id}");
                                }
                            } else {
                                let _ = tx_sched.send(Action::RunJobNow { job_id });
                            }
                        } else {
                            // Backend doesn't have a known PTY command, fall back to local.
                            let _ = tx_sched.send(Action::RunJobNow { job_id });
                        }
                    } else if !use_pty {
                        // Local scheduler path.
                        tracing::info!("scheduler triggered job (local path): {job_id}");
                        let _ = tx_sched.send(Action::RunJobNow { job_id });
                    }
                    // If use_pty && has_session, the PTY session is already handling it.
                }

                // Periodically save PTY session state for crash recovery.
                let records = pty_manager.get_persistence_data().await;
                if !records.is_empty() {
                    let _ = persistence::save_pty_sessions(&records);
                }
            }
        });

        Self {
            action_tx,
            _runtime: runtime,
        }
    }
}

/// Map of active subprocess cancel handles per session.
type CancelMap = HashMap<SessionId, oneshot::Sender<()>>;

async fn dispatcher(
    mut rx: mpsc::UnboundedReceiver<Action>,
    shared: SharedState,
    _tx: mpsc::UnboundedSender<Action>,
) {
    let mut cancel_map: CancelMap = HashMap::new();
    let mut slack_poll_handle: Option<tokio::task::JoinHandle<()>> = None;
    let backend_defs = all_known_backends();

    while let Some(action) = rx.recv().await {
        match action {
            Action::SendMessage {
                session_id,
                text,
                attachments,
            } => {
                // Check for session command.
                if let Some(cmd) = commands::parse_command(&text) {
                    handle_session_command(&shared, &session_id, cmd);
                    continue;
                }

                // Add user message to session.
                let (backend_id, model_id) = {
                    let mut state = shared.write();
                    if let Some(session) = state.sessions.get_mut(&session_id) {
                        let user_msg = Message::user(text.clone());
                        session.messages.push(user_msg);

                        // Create streaming assistant message.
                        let mut assistant = Message::assistant_streaming();
                        assistant.backend_id = Some(session.backend_id.clone());
                        assistant.model_id = session.model_id.clone();
                        session.messages.push(assistant);
                        session.streaming = Some(StreamingState {
                            accumulated_text: String::new(),
                            is_active: true,
                            can_cancel: true,
                            active_sub_agent: None,
                        });
                        (session.backend_id.clone(), session.model_id.clone())
                    } else {
                        continue;
                    }
                };
                shared.ctx().request_repaint();

                // Update tray state.
                shared.mutate(|s| s.tray_state = TrayState::Working);

                // Select backend.
                let (bs, binary_path) = {
                    let state = shared.read();
                    match orchestrator::select_backend(
                        &state.backend_registry,
                        &backend_id,
                        &state.fallback_order,
                    ) {
                        Some((bs, path)) => (bs.clone(), path.clone()),
                        None => {
                            drop(state);
                            shared.mutate(|s| {
                                if let Some(session) = s.sessions.get_mut(&session_id) {
                                    if let Some(msg) = session.messages.last_mut() {
                                        msg.content =
                                            "No AI backends available. Please check your backend settings.".into();
                                        msg.status = MessageStatus::Error;
                                    }
                                    session.streaming = None;
                                }
                                s.tray_state = TrayState::Idle;
                            });
                            continue;
                        }
                    }
                };

                // Find the backend definition.
                let def = backend_defs
                    .iter()
                    .find(|d| d.backend_id() == bs.backend_id);
                let Some(_def) = def else {
                    shared.mutate(|s| {
                        if let Some(session) = s.sessions.get_mut(&session_id) {
                            if let Some(msg) = session.messages.last_mut() {
                                msg.content = format!(
                                    "Backend '{}' not found in definitions.",
                                    bs.backend_id
                                );
                                msg.status = MessageStatus::Error;
                            }
                            session.streaming = None;
                        }
                        s.tray_state = TrayState::Idle;
                    });
                    continue;
                };

                let params = ChatParams {
                    message: text,
                    model: model_id,
                    attachments,
                    thinking_level: None,
                    extra_args: bs.custom_args.clone(),
                    system_prompt: None,
                };

                let (cancel_tx, cancel_rx) = oneshot::channel();
                cancel_map.insert(session_id.clone(), cancel_tx);

                let shared_stream = shared.clone();
                let sid = session_id.clone();
                let env_overrides = bs.env_overrides.clone();
                let working_dir = {
                    let state = shared.read();
                    state
                        .sessions
                        .get(&session_id)
                        .and_then(|s| s.project_binding.as_ref())
                        .map(|p| p.working_directory.clone())
                };

                // We need to call run_chat with a trait object. Since
                // `all_known_backends` returns owned boxes, we rebuild the
                // concrete backend here.
                let backend_id_for_spawn = bs.backend_id.clone();
                let binary_for_spawn = binary_path.clone();

                tokio::spawn(async move {
                    let defs = all_known_backends();
                    let def = defs
                        .iter()
                        .find(|d| d.backend_id() == backend_id_for_spawn)
                        .unwrap();

                    // Stream events incrementally — each event is applied to
                    // state immediately so the UI updates in real time.
                    let shared_cb = shared_stream.clone();
                    let sid_cb = sid.clone();
                    let outcome = orchestrator::run_chat_streaming(
                        def.as_ref(),
                        &binary_for_spawn,
                        params,
                        working_dir,
                        &env_overrides,
                        cancel_rx,
                        |event| {
                            apply_single_event(&shared_cb, &sid_cb, event);
                        },
                    )
                    .await;

                    match outcome {
                        SendOutcome::Completed { duration_ms, .. } => {
                            // Finalize: set duration on the message.
                            shared_stream.mutate(|s| {
                                if let Some(session) = s.sessions.get_mut(&sid) {
                                    if let Some(msg) = session.messages.last_mut() {
                                        msg.duration_ms = Some(duration_ms);
                                    }
                                }
                            });
                        }
                        SendOutcome::Cancelled => {
                            shared_stream.mutate(|s| {
                                if let Some(session) = s.sessions.get_mut(&sid) {
                                    if let Some(msg) = session.messages.last_mut() {
                                        msg.status = MessageStatus::Cancelled;
                                        if let Some(ref st) = session.streaming {
                                            msg.content = st.accumulated_text.clone();
                                        }
                                    }
                                    session.streaming = None;
                                }
                                s.tray_state = TrayState::Idle;
                            });
                        }
                        SendOutcome::Error(e) => {
                            shared_stream.mutate(|s| {
                                if let Some(session) = s.sessions.get_mut(&sid) {
                                    if let Some(msg) = session.messages.last_mut() {
                                        msg.content = e;
                                        msg.status = MessageStatus::Error;
                                    }
                                    session.streaming = None;
                                }
                                s.tray_state = TrayState::Idle;
                            });
                        }
                    }

                    // Persist session after completion.
                    let session_clone = {
                        let state = shared_stream.read();
                        state.sessions.get(&sid).cloned()
                    };
                    if let Some(s) = session_clone {
                        let _ = session::save_session(&s);
                    }
                });
            }

            Action::AbortGeneration { session_id } => {
                if let Some(cancel_tx) = cancel_map.remove(&session_id) {
                    let _ = cancel_tx.send(());
                }
            }

            Action::NewSession => {
                let default_backend = {
                    let state = shared.read();
                    state
                        .default_backend_id
                        .clone()
                        .unwrap_or_else(|| "anthropic".into())
                };
                let new_session = session::create_session(&default_backend);
                let new_id = new_session.id.clone();
                shared.mutate(|s| {
                    s.sessions.insert(new_id.clone(), new_session);
                    s.active_session_id = new_id;
                });
            }

            Action::SwitchSession { session_id } => {
                shared.mutate(|s| {
                    if s.sessions.contains_key(&session_id) {
                        s.active_session_id = session_id;
                    }
                });
            }

            Action::SwitchBackend { backend_id } => {
                shared.mutate(|s| {
                    if let Some(session) = s.sessions.get_mut(&s.active_session_id.clone()) {
                        session.backend_id = backend_id;
                        session.model_id = None;
                    }
                });
            }

            Action::SwitchModel { model_id } => {
                shared.mutate(|s| {
                    if let Some(session) = s.sessions.get_mut(&s.active_session_id.clone()) {
                        session.model_id = Some(model_id);
                    }
                });
            }

            Action::ResetSession { session_id } => {
                shared.mutate(|s| {
                    if let Some(session) = s.sessions.get_mut(&session_id) {
                        session.messages.clear();
                        session.streaming = None;
                        session.active_tool_cards.clear();
                        session.usage_totals = UsageTotals::default();
                        session.messages.push(Message::system("Session reset.".into()));
                    }
                });
            }

            Action::RescanBackends => {
                let shared_clone = shared.clone();
                tokio::spawn(async move {
                    run_backend_scan(shared_clone).await;
                });
            }

            Action::TogglePanel => {
                shared.mutate(|s| s.panel_visible = !s.panel_visible);
            }

            Action::OpenSettings { tab } => {
                shared.mutate(|s| {
                    s.settings_open = true;
                    s.current_view = ViewMode::Chat;
                    s.notifications_open = false;
                    if let Some(tab) = tab {
                        s.settings_tab = tab;
                    }
                });
            }

            Action::CloseSettings => {
                shared.mutate(|s| s.settings_open = false);
            }

            Action::DismissNotification { id } => {
                shared.mutate(|s| {
                    if let Some(n) = s.notifications.iter_mut().find(|n| n.id == id) {
                        n.dismissed = true;
                    }
                });
            }

            Action::SaveConfig => {
                let config = shared.read().config.clone();
                let path = conductor_core::config::config_file_path();
                if let Err(e) = conductor_core::config::save_config(&path, &config) {
                    tracing::error!("failed to save config: {e}");
                }
            }

            Action::Quit => {
                // Persist all sessions.
                let sessions: Vec<Session> = shared.read().sessions.values().cloned().collect();
                for s in &sessions {
                    let _ = session::save_session(s);
                }
                std::process::exit(0);
            }

            // -- Scheduling actions --
            Action::CreateJob { definition } => {
                shared.mutate(|s| {
                    s.scheduler.jobs.push(definition);
                });
                persist_jobs(&shared);
            }

            Action::DeleteJob { job_id } => {
                shared.mutate(|s| {
                    s.scheduler.jobs.retain(|j| j.job_id != job_id);
                });
                persist_jobs(&shared);
            }

            Action::ToggleJob { job_id, enabled } => {
                shared.mutate(|s| {
                    if let Some(job) = s.scheduler.jobs.iter_mut().find(|j| j.job_id == job_id) {
                        job.enabled = enabled;
                    }
                });
                persist_jobs(&shared);
            }

            Action::RunJobNow { job_id } => {
                let shared_run = shared.clone();
                tokio::spawn(async move {
                    run_job_now(shared_run, job_id).await;
                });
            }

            // -- Automation actions --
            Action::CreateAutomation { rule } => {
                shared.mutate(|s| {
                    s.automation_rules.push(rule);
                });
                persist_automations(&shared);
                sync_slack_monitors(&shared, &mut slack_poll_handle, &_tx);
            }

            Action::DeleteAutomation { rule_id } => {
                shared.mutate(|s| {
                    s.automation_rules.retain(|r| r.rule_id != rule_id);
                });
                persist_automations(&shared);
                sync_slack_monitors(&shared, &mut slack_poll_handle, &_tx);
            }

            Action::ToggleAutomation { rule_id, enabled } => {
                shared.mutate(|s| {
                    if let Some(rule) = s.automation_rules.iter_mut().find(|r| r.rule_id == rule_id) {
                        rule.enabled = enabled;
                    }
                });
                persist_automations(&shared);
                sync_slack_monitors(&shared, &mut slack_poll_handle, &_tx);
            }

            Action::RunAutomation { rule_id, event_context } => {
                let shared_run = shared.clone();
                let tx_run = _tx.clone();
                tokio::spawn(async move {
                    run_automation(shared_run, tx_run, rule_id, event_context).await;
                });
            }

            // -- Slack actions --
            Action::SlackConnect => {
                let shared_slack = shared.clone();
                let tx_slack = _tx.clone();
                tokio::spawn(async move {
                    slack_connect(shared_slack, tx_slack).await;
                });
            }

            Action::SlackDisconnect => {
                if let Some(handle) = slack_poll_handle.take() {
                    handle.abort();
                }
                shared.mutate(|s| {
                    s.slack.status = SlackStatus::Disconnected;
                    s.slack.workspace_name = None;
                    s.slack.channels.clear();
                    s.slack.monitored_channels.clear();
                    s.slack.error = None;
                });
            }

            Action::SlackRefreshChannels => {
                let shared_slack = shared.clone();
                tokio::spawn(async move {
                    slack_refresh_channels(shared_slack).await;
                });
            }

            Action::SlackMonitorChannel { channel_id } => {
                shared.mutate(|s| {
                    if !s.slack.monitored_channels.contains(&channel_id) {
                        s.slack.monitored_channels.push(channel_id.clone());
                    }
                });
                tracing::info!("monitoring Slack channel {channel_id}");

                // Start the unified poll loop if not already running.
                if slack_poll_handle.is_none() {
                    let shared_poll = shared.clone();
                    let tx_poll = _tx.clone();
                    slack_poll_handle = Some(tokio::spawn(async move {
                        slack_poll_loop(shared_poll, tx_poll).await;
                    }));
                    tracing::info!("started Slack poll loop");
                }
            }

            Action::SlackUnmonitorChannel { channel_id } => {
                shared.mutate(|s| {
                    s.slack.monitored_channels.retain(|id| id != &channel_id);
                });
                tracing::info!("unmonitored Slack channel {channel_id}");

                // Stop the loop if no more channels are monitored.
                let still_monitoring = !shared.read().slack.monitored_channels.is_empty();
                if !still_monitoring {
                    if let Some(handle) = slack_poll_handle.take() {
                        handle.abort();
                        tracing::info!("stopped Slack poll loop (no monitored channels)");
                    }
                }
            }

            Action::ToggleNotifications => {
                shared.mutate(|s| {
                    s.notifications_open = !s.notifications_open;
                    s.settings_open = false;
                    s.current_view = ViewMode::Chat;
                });
            }

            Action::ToggleSchedules => {
                shared.mutate(|s| {
                    if s.current_view == ViewMode::Schedules {
                        s.current_view = ViewMode::Chat;
                    } else {
                        s.current_view = ViewMode::Schedules;
                    }
                    s.settings_open = false;
                    s.notifications_open = false;
                });
            }

            Action::DismissNotification { id } => {
                shared.mutate(|s| {
                    if let Some(n) = s.notifications.iter_mut().find(|n| n.id == id) {
                        n.dismissed = true;
                    }
                });
            }

            Action::DismissAllNotifications => {
                shared.mutate(|s| {
                    for n in &mut s.notifications {
                        n.dismissed = true;
                    }
                });
            }

            _ => {
                tracing::debug!("unhandled action: {:?}", std::mem::discriminant(&action));
            }
        }
    }
}

fn persist_jobs(shared: &SharedState) {
    let jobs = shared.read().scheduler.jobs.clone();
    if let Err(e) = conductor_core::scheduler::persistence::save_jobs(&jobs) {
        tracing::error!("failed to persist jobs: {e}");
    }
}

fn persist_automations(shared: &SharedState) {
    let rules = shared.read().automation_rules.clone();
    if let Err(e) = conductor_core::automation::save_rules(&rules) {
        tracing::error!("failed to persist automations: {e}");
    }
}

async fn run_job_now(shared: SharedState, job_id: String) {
    let (job, registry, fallback, default_backend) = {
        let state = shared.read();
        let job = state.scheduler.jobs.iter().find(|j| j.job_id == job_id).cloned();
        let registry = state.backend_registry.clone();
        let fallback = state.fallback_order.clone();
        let default = state.default_backend_id.clone().unwrap_or_else(|| "anthropic".into());
        (job, registry, fallback, default)
    };

    let Some(job) = job else {
        tracing::warn!("RunJobNow: job {job_id} not found");
        return;
    };

    tracing::info!("running job '{}' now", job.name);

    // Mark a run as started.
    let run = conductor_core::scheduler::new_job_run();
    let run_id = run.run_id.clone();
    shared.mutate(|s| {
        if let Some(j) = s.scheduler.jobs.iter_mut().find(|j| j.job_id == job_id) {
            j.history.push(run.clone());
        }
    });

    // Query connector data bindings (if any are configured on the job payload).
    // In the future, this would call real connector APIs. For now, it's a
    // placeholder that logs what would be fetched.
    let connector_data: Option<String> = None;
    // TODO: When connector runtime is wired, iterate job.payload connector
    // bindings and call the appropriate fetch actions, collecting results.

    // Execute (with connector data injected into the prompt if available).
    let completed_run = conductor_core::scheduler::execute_job(
        &job,
        &registry,
        &fallback,
        &default_backend,
        connector_data.as_deref(),
    )
    .await;

    // Deliver result.
    if let Some(ref summary) = completed_run.output_summary {
        conductor_core::scheduler::deliver_result(&job.delivery, summary).await;
    }

    // Update the run in history.
    shared.mutate(|s| {
        if let Some(j) = s.scheduler.jobs.iter_mut().find(|j| j.job_id == job_id) {
            if let Some(r) = j.history.iter_mut().find(|r| r.run_id == run_id) {
                *r = completed_run.clone();
            }
            // Recompute next_run.
            j.next_run = conductor_core::scheduler::compute_next_run(j, chrono::Utc::now());
        }
    });

    persist_jobs(&shared);
    tracing::info!("job '{}' completed with status {:?}", job.name, completed_run.status);
}

/// Ensure all Slack channels referenced by enabled automation rules are monitored,
/// and start/stop the poll loop as needed.
fn sync_slack_monitors(
    shared: &SharedState,
    slack_poll_handle: &mut Option<tokio::task::JoinHandle<()>>,
    tx: &mpsc::UnboundedSender<Action>,
) {
    let state = shared.read();

    // Collect channel names needed by enabled automation rules.
    let mut needed_channels: Vec<String> = Vec::new();
    for rule in &state.automation_rules {
        if !rule.enabled {
            continue;
        }
        match &rule.trigger {
            TriggerCondition::SlackMessage { channel_name, .. } => {
                let name = channel_name.trim_start_matches('#').to_lowercase();
                // Find the channel ID for this name.
                if let Some(ch) = state.slack.channels.iter().find(|c| c.name.to_lowercase() == name) {
                    if !needed_channels.contains(&ch.id) {
                        needed_channels.push(ch.id.clone());
                    }
                }
            }
            TriggerCondition::ChannelMessage { platform_id, channel_filter, .. } => {
                if platform_id == "slack" {
                    if let Some(filter) = channel_filter {
                        let name = filter.trim_start_matches('#').to_lowercase();
                        if let Some(ch) = state.slack.channels.iter().find(|c| c.name.to_lowercase() == name) {
                            if !needed_channels.contains(&ch.id) {
                                needed_channels.push(ch.id.clone());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let current = state.slack.monitored_channels.clone();
    drop(state);

    // Add any missing channels to the monitored list.
    for cid in &needed_channels {
        if !current.contains(cid) {
            shared.mutate(|s| {
                if !s.slack.monitored_channels.contains(cid) {
                    s.slack.monitored_channels.push(cid.clone());
                }
            });
            tracing::info!("auto-monitoring Slack channel {cid} (used by automation)");
        }
    }

    // Start the poll loop if we now have monitored channels and it's not running.
    let has_monitored = !shared.read().slack.monitored_channels.is_empty();
    if has_monitored && slack_poll_handle.is_none() {
        let shared_poll = shared.clone();
        let tx_poll = tx.clone();
        *slack_poll_handle = Some(tokio::spawn(async move {
            slack_poll_loop(shared_poll, tx_poll).await;
        }));
        tracing::info!("started Slack poll loop (auto-monitor)");
    }

    // Stop the loop if nothing is monitored anymore.
    if !has_monitored {
        if let Some(handle) = slack_poll_handle.take() {
            handle.abort();
            tracing::info!("stopped Slack poll loop (no monitored channels)");
        }
    }
}

/// Single poll loop that fetches all monitored Slack channels every 60 seconds.
/// Matches incoming messages against automation rules and fires triggers.
async fn slack_poll_loop(shared: SharedState, tx: mpsc::UnboundedSender<Action>) {
    use conductor_core::channel::slack;

    let poll_interval = std::time::Duration::from_secs(60);
    let mut last_ts = chrono::Utc::now();

    loop {
        tokio::time::sleep(poll_interval).await;

        // Read current monitored channels + channel name map.
        let (monitored, channel_map) = {
            let state = shared.read();
            let monitored = state.slack.monitored_channels.clone();
            let map: HashMap<String, String> = state
                .slack
                .channels
                .iter()
                .map(|c| (c.id.clone(), c.name.clone()))
                .collect();
            (monitored, map)
        };

        if monitored.is_empty() {
            continue;
        }

        // Single slackdump call for all monitored channels.
        match slack::fetch_recent_messages(&monitored, last_ts).await {
            Ok(messages) => {
                if !messages.is_empty() {
                    tracing::debug!("Slack poll: {} new messages across {} channels", messages.len(), monitored.len());
                }

                // Advance watermark.
                for msg in &messages {
                    if let Some(t) = slack::parse_slack_ts(&msg.ts) {
                        if t > last_ts {
                            last_ts = t;
                        }
                    }
                }

                // Match each message against automation rules.
                let rules = shared.read().automation_rules.clone();

                for msg in &messages {
                    let channel_name = channel_map
                        .get(&msg.channel_id)
                        .cloned()
                        .unwrap_or_else(|| msg.channel_id.clone());

                    for rule in &rules {
                        if !rule.enabled {
                            continue;
                        }

                        if matches_slack_trigger(rule, &channel_name, &msg.text) {
                            tracing::info!(
                                "Slack message matched automation '{}' in #{}",
                                rule.name,
                                channel_name
                            );

                            let event_context = format!(
                                "Slack message in #{channel_name} from user {}:\n{}",
                                msg.user, msg.text
                            );

                            let _ = tx.send(Action::RunAutomation {
                                rule_id: rule.rule_id.clone(),
                                event_context: Some(event_context),
                            });
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Slack poll failed: {e}");
            }
        }
    }
}

/// Check if an automation rule's trigger matches a Slack message.
fn matches_slack_trigger(rule: &AutomationRule, channel_name: &str, text: &str) -> bool {
    match &rule.trigger {
        TriggerCondition::SlackMessage {
            channel_name: trigger_channel,
            keyword_filter,
        } => {
            let trigger_ch = trigger_channel.trim_start_matches('#');
            if !trigger_ch.eq_ignore_ascii_case(channel_name) {
                return false;
            }
            match keyword_filter {
                None => true,
                Some(kw) => {
                    let lower = text.to_lowercase();
                    kw.split(',')
                        .map(|k| k.trim().to_lowercase())
                        .any(|k| lower.contains(&k))
                }
            }
        }
        TriggerCondition::ChannelMessage {
            platform_id,
            channel_filter,
            keyword_filter,
        } => {
            if platform_id != "slack" {
                return false;
            }
            if let Some(filter) = channel_filter {
                let f = filter.trim_start_matches('#');
                if !f.eq_ignore_ascii_case(channel_name) {
                    return false;
                }
            }
            match keyword_filter {
                None => true,
                Some(kw) => {
                    let lower = text.to_lowercase();
                    kw.split(',')
                        .map(|k| k.trim().to_lowercase())
                        .any(|k| lower.contains(&k))
                }
            }
        }
        _ => false,
    }
}

async fn slack_connect(shared: SharedState, tx: mpsc::UnboundedSender<Action>) {
    use conductor_core::channel::slack;

    shared.mutate(|s| {
        s.slack.status = SlackStatus::Checking;
        s.slack.error = None;
    });

    // Check if slackdump is installed.
    if slack::find_slackdump().is_none() {
        shared.mutate(|s| {
            s.slack.status = SlackStatus::Error;
            s.slack.error = Some("slackdump not found. Install with: brew install slackdump".into());
        });
        return;
    }

    // Check if a workspace is configured.
    if !slack::has_workspace().await {
        shared.mutate(|s| {
            s.slack.status = SlackStatus::Error;
            s.slack.error = Some("No Slack workspace configured. Run: slackdump workspace new".into());
        });
        return;
    }

    // Get workspace name.
    match slack::current_workspace().await {
        Ok(name) => {
            shared.mutate(|s| {
                s.slack.workspace_name = Some(name);
                s.slack.status = SlackStatus::Connected;
            });
            tracing::info!("Slack connected");
        }
        Err(e) => {
            shared.mutate(|s| {
                s.slack.status = SlackStatus::Error;
                s.slack.error = Some(e);
            });
            return;
        }
    }

    // Load channels.
    slack_refresh_channels(shared.clone()).await;

    // Auto-monitor channels needed by persisted automation rules.
    let needed: Vec<String> = {
        let state = shared.read();
        let mut ids = Vec::new();
        for rule in &state.automation_rules {
            if !rule.enabled {
                continue;
            }
            match &rule.trigger {
                TriggerCondition::SlackMessage { channel_name, .. } => {
                    let name = channel_name.trim_start_matches('#').to_lowercase();
                    if let Some(ch) = state.slack.channels.iter().find(|c| c.name.to_lowercase() == name) {
                        if !ids.contains(&ch.id) {
                            ids.push(ch.id.clone());
                        }
                    }
                }
                _ => {}
            }
        }
        ids
    };

    for cid in needed {
        let _ = tx.send(Action::SlackMonitorChannel { channel_id: cid });
    }
}

async fn slack_refresh_channels(shared: SharedState) {
    use conductor_core::channel::slack;

    match slack::list_channels().await {
        Ok(channels) => {
            shared.mutate(|s| {
                s.slack.channels = channels
                    .into_iter()
                    .map(|ch| SlackChannelInfo {
                        id: ch.id,
                        name: ch.name,
                        is_private: ch.is_private,
                    })
                    .collect();
            });
            tracing::info!("Slack channels refreshed");
        }
        Err(e) => {
            tracing::warn!("Failed to list Slack channels: {e}");
            shared.mutate(|s| {
                s.slack.error = Some(format!("Failed to list channels: {e}"));
            });
        }
    }
}

async fn run_automation(
    shared: SharedState,
    tx: mpsc::UnboundedSender<Action>,
    rule_id: String,
    event_context: Option<String>,
) {
    let rule = {
        let state = shared.read();
        state.automation_rules.iter().find(|r| r.rule_id == rule_id).cloned()
    };

    let Some(rule) = rule else {
        tracing::warn!("RunAutomation: rule {rule_id} not found");
        return;
    };

    tracing::info!("running automation '{}'", rule.name);

    // Create a run entry.
    let run_id = uuid::Uuid::new_v4().to_string();
    let run = AutomationRunEntry {
        run_id: run_id.clone(),
        started_at: chrono::Utc::now(),
        completed_at: None,
        duration_ms: None,
        status: JobRunStatus::Running,
        trigger_event: event_context.clone(),
        error: None,
        output_summary: None,
    };

    shared.mutate(|s| {
        if let Some(r) = s.automation_rules.iter_mut().find(|r| r.rule_id == rule_id) {
            r.history.push(run.clone());
        }
    });

    let start = std::time::Instant::now();

    match &rule.action {
        AutomationAction::RunPrompt {
            prompt,
            include_event_context,
            backend_override: _,
            model_override: _,
        } => {
            // Build the full prompt, optionally injecting event context.
            let full_prompt = if *include_event_context {
                if let Some(ref ctx) = event_context {
                    format!("Event context:\n{ctx}\n\n---\n\n{prompt}")
                } else {
                    prompt.clone()
                }
            } else {
                prompt.clone()
            };

            // Create a temporary job and execute it using existing infra.
            let (registry, fallback, default_backend) = {
                let state = shared.read();
                (
                    state.backend_registry.clone(),
                    state.fallback_order.clone(),
                    state.default_backend_id.clone().unwrap_or_else(|| "anthropic".into()),
                )
            };

            let temp_job = ScheduledJob {
                job_id: format!("auto-{run_id}"),
                name: rule.name.clone(),
                enabled: true,
                schedule: ScheduleDefinition::OneTime { datetime: chrono::Utc::now() },
                payload: JobPayload {
                    prompt: full_prompt,
                    execution_mode: ExecutionMode::Isolated,
                    backend_override: None,
                    model_override: None,
                    thinking_level: None,
                    timeout_seconds: None,
                },
                delivery: DeliveryConfig::Silent,
                retry_policy: RetryPolicy::default(),
                history: Vec::new(),
                next_run: None,
            };

            let completed = conductor_core::scheduler::execute_job(
                &temp_job,
                &registry,
                &fallback,
                &default_backend,
                None,
            )
            .await;

            let elapsed = start.elapsed().as_millis() as u64;

            // Update the run entry and also push to global job_history.
            shared.mutate(|s| {
                if let Some(r) = s.automation_rules.iter_mut().find(|r| r.rule_id == rule_id) {
                    r.trigger_count += 1;
                    r.last_triggered = Some(chrono::Utc::now());
                    if let Some(entry) = r.history.iter_mut().find(|e| e.run_id == run_id) {
                        entry.completed_at = Some(chrono::Utc::now());
                        entry.duration_ms = Some(elapsed);
                        entry.status = completed.status;
                        entry.error = completed.error.clone();
                        entry.output_summary = completed.output_summary.clone();
                    }
                }
                // Also record in the global job history.
                s.job_history.push(JobHistoryEntry {
                    run_id: run_id.clone(),
                    job_name: rule.name.clone(),
                    job_id: Some(rule.rule_id.clone()),
                    trigger: JobTrigger::Automation,
                    started_at: run.started_at,
                    completed_at: Some(chrono::Utc::now()),
                    duration_ms: Some(elapsed),
                    status: completed.status,
                    error: completed.error,
                    output_summary: completed.output_summary,
                    backend_id: None,
                    prompt_preview: Some(rule.name.clone()),
                });
            });
        }

        AutomationAction::RunJob { job_id } => {
            let _ = tx.send(Action::RunJobNow { job_id: job_id.clone() });

            let elapsed = start.elapsed().as_millis() as u64;
            shared.mutate(|s| {
                if let Some(r) = s.automation_rules.iter_mut().find(|r| r.rule_id == rule_id) {
                    r.trigger_count += 1;
                    r.last_triggered = Some(chrono::Utc::now());
                    if let Some(entry) = r.history.iter_mut().find(|e| e.run_id == run_id) {
                        entry.completed_at = Some(chrono::Utc::now());
                        entry.duration_ms = Some(elapsed);
                        entry.status = JobRunStatus::Success;
                        entry.output_summary = Some(format!("Triggered job {job_id}"));
                    }
                }
            });
        }

        AutomationAction::Notify { message } => {
            shared.mutate(|s| {
                s.notifications.push(Notification {
                    id: uuid::Uuid::new_v4().to_string(),
                    title: format!("Automation: {}", rule.name),
                    body: message.clone(),
                    severity: NotificationSeverity::Info,
                    timestamp: chrono::Utc::now(),
                    dismissed: false,
                });

                if let Some(r) = s.automation_rules.iter_mut().find(|r| r.rule_id == rule_id) {
                    r.trigger_count += 1;
                    r.last_triggered = Some(chrono::Utc::now());
                    if let Some(entry) = r.history.iter_mut().find(|e| e.run_id == run_id) {
                        entry.completed_at = Some(chrono::Utc::now());
                        entry.duration_ms = Some(start.elapsed().as_millis() as u64);
                        entry.status = JobRunStatus::Success;
                        entry.output_summary = Some("Notification sent".into());
                    }
                }
            });
        }
    }

    persist_automations(&shared);
    tracing::info!("automation '{}' completed", rule.name);
}

fn handle_session_command(shared: &SharedState, session_id: &str, cmd: SessionCommand) {
    match cmd {
        SessionCommand::New => {
            let default_backend = {
                let state = shared.read();
                state
                    .default_backend_id
                    .clone()
                    .unwrap_or_else(|| "anthropic".into())
            };
            let new_session = session::create_session(&default_backend);
            let new_id = new_session.id.clone();
            shared.mutate(|s| {
                s.sessions.insert(new_id.clone(), new_session);
                s.active_session_id = new_id;
            });
        }
        SessionCommand::Reset => {
            shared.mutate(|s| {
                if let Some(session) = s.sessions.get_mut(session_id) {
                    session.messages.clear();
                    session.messages.push(Message::system("Session reset.".into()));
                }
            });
        }
        SessionCommand::Usage => {
            let text = {
                let state = shared.read();
                if let Some(session) = state.sessions.get(session_id) {
                    let u = &session.usage_totals;
                    format!(
                        "Session usage: {} messages, {} input tokens, {} output tokens, ${:.4} estimated cost",
                        u.message_count, u.total_input_tokens, u.total_output_tokens, u.total_cost
                    )
                } else {
                    "No active session.".into()
                }
            };
            shared.mutate(|s| {
                if let Some(session) = s.sessions.get_mut(session_id) {
                    session.messages.push(Message::system(text));
                }
            });
        }
        SessionCommand::Status => {
            let text = {
                let state = shared.read();
                if let Some(session) = state.sessions.get(session_id) {
                    format!(
                        "Backend: {}, Model: {}, Messages: {}",
                        session.backend_id,
                        session.model_id.as_deref().unwrap_or("default"),
                        session.messages.len()
                    )
                } else {
                    "No active session.".into()
                }
            };
            shared.mutate(|s| {
                if let Some(session) = s.sessions.get_mut(session_id) {
                    session.messages.push(Message::system(text));
                }
            });
        }
        SessionCommand::Model(name) => {
            shared.mutate(|s| {
                let sid = session_id.to_string();
                if let Some(session) = s.sessions.get_mut(&sid) {
                    session.model_id = Some(name.clone());
                    session
                        .messages
                        .push(Message::system(format!("Switched to model: {name}")));
                }
            });
        }
        SessionCommand::Cli(name) => {
            shared.mutate(|s| {
                let sid = session_id.to_string();
                if let Some(session) = s.sessions.get_mut(&sid) {
                    session.backend_id = name.clone();
                    session.model_id = None;
                    session
                        .messages
                        .push(Message::system(format!("Switched to backend: {name}")));
                }
            });
        }
        SessionCommand::Think(level) => {
            shared.mutate(|s| {
                if let Some(session) = s.sessions.get_mut(session_id) {
                    session
                        .messages
                        .push(Message::system(format!("Thinking level set to: {level:?}")));
                }
            });
        }
        SessionCommand::Compact => {
            shared.mutate(|s| {
                if let Some(session) = s.sessions.get_mut(session_id) {
                    session
                        .messages
                        .push(Message::system("Context compaction requested. (Not yet implemented — send /compact to the backend in a future version.)".into()));
                }
            });
        }
        SessionCommand::Verbose => {
            shared.mutate(|s| {
                if let Some(session) = s.sessions.get_mut(session_id) {
                    session
                        .messages
                        .push(Message::system("Verbose mode toggled.".into()));
                }
            });
        }
        SessionCommand::Unknown(msg) => {
            shared.mutate(|s| {
                if let Some(session) = s.sessions.get_mut(session_id) {
                    session.messages.push(Message::system(msg));
                }
            });
        }
    }
}

/// Apply a single stream event to state immediately (for real-time streaming).
fn apply_single_event(shared: &SharedState, session_id: &str, event: StreamEvent) {
    use conductor_core::state::{SubAgentStep, SubStepKind};

    // Check if we're inside a sub-agent scope.
    let in_sub_agent = {
        let state = shared.read();
        state
            .sessions
            .get(session_id)
            .and_then(|s| s.streaming.as_ref())
            .and_then(|st| st.active_sub_agent.clone())
    };

    match event {
        StreamEvent::TextChunk(text) => {
            if let Some(ref _agent_name) = in_sub_agent {
                // Inside sub-agent: add as a reasoning step.
                shared.mutate(|s| {
                    if let Some(session) = s.sessions.get_mut(session_id) {
                        // Find the Agent card and add a reasoning sub-step.
                        if let Some(msg) = session.messages.last_mut() {
                            if let Some(agent_card) = msg.tool_cards.iter_mut().rev()
                                .find(|c| c.tool_name == "Agent" && c.phase == ToolPhase::Started)
                            {
                                agent_card.sub_steps.push(SubAgentStep {
                                    kind: SubStepKind::Reasoning,
                                    content: text.clone(),
                                    timestamp: chrono::Utc::now(),
                                });
                            }
                        }
                    }
                });
            } else {
                // Normal: accumulate text in the streaming buffer.
                shared.mutate(|s| {
                    if let Some(session) = s.sessions.get_mut(session_id) {
                        if let Some(ref mut streaming) = session.streaming {
                            streaming.accumulated_text.push_str(&text);
                        }
                        if let Some(msg) = session.messages.last_mut() {
                            if msg.status == MessageStatus::Streaming {
                                if let Some(ref st) = session.streaming {
                                    msg.content = st.accumulated_text.clone();
                                }
                            }
                        }
                    }
                });
            }
        }
        StreamEvent::ToolStart { name, args } => {
            if name == "Agent" {
                // Entering a sub-agent scope.
                shared.mutate(|s| {
                    if let Some(session) = s.sessions.get_mut(session_id) {
                        if let Some(ref mut streaming) = session.streaming {
                            streaming.active_sub_agent = Some(name.clone());
                        }
                        let card = ToolCard {
                            tool_name: name.clone(),
                            phase: ToolPhase::Started,
                            arguments: args.clone(),
                            result: None,
                            metadata: None,
                            timestamp: chrono::Utc::now(),
                            sub_steps: Vec::new(),
                        };
                        session.active_tool_cards.push(card.clone());
                        if let Some(msg) = session.messages.last_mut() {
                            msg.tool_cards.push(card);
                        }
                    }
                });
            } else if in_sub_agent.is_some() {
                // Inside sub-agent: add tool use as a sub-step.
                shared.mutate(|s| {
                    if let Some(session) = s.sessions.get_mut(session_id) {
                        if let Some(msg) = session.messages.last_mut() {
                            if let Some(agent_card) = msg.tool_cards.iter_mut().rev()
                                .find(|c| c.tool_name == "Agent" && c.phase == ToolPhase::Started)
                            {
                                agent_card.sub_steps.push(SubAgentStep {
                                    kind: SubStepKind::ToolUse(name.clone()),
                                    content: String::new(),
                                    timestamp: chrono::Utc::now(),
                                });
                            }
                        }
                        // Also track in active_tool_cards for result matching.
                        let card = ToolCard {
                            tool_name: name.clone(),
                            phase: ToolPhase::Started,
                            arguments: args.clone(),
                            result: None,
                            metadata: None,
                            timestamp: chrono::Utc::now(),
                            sub_steps: Vec::new(),
                        };
                        session.active_tool_cards.push(card);
                    }
                });
            } else {
                // Normal top-level tool use.
                shared.mutate(|s| {
                    if let Some(session) = s.sessions.get_mut(session_id) {
                        let card = ToolCard {
                            tool_name: name.clone(),
                            phase: ToolPhase::Started,
                            arguments: args.clone(),
                            result: None,
                            metadata: None,
                            timestamp: chrono::Utc::now(),
                            sub_steps: Vec::new(),
                        };
                        session.active_tool_cards.push(card.clone());
                        if let Some(msg) = session.messages.last_mut() {
                            msg.tool_cards.push(card);
                        }
                    }
                });
            }
        }
        StreamEvent::ToolResult {
            name,
            result,
            success,
        } => {
            let phase = if success { ToolPhase::Completed } else { ToolPhase::Failed };

            if in_sub_agent.is_some() && name != "Agent" {
                // Sub-agent internal tool result — add as sub-step.
                shared.mutate(|s| {
                    if let Some(session) = s.sessions.get_mut(session_id) {
                        // Update the sub-tool's card.
                        if let Some(card) = session.active_tool_cards.iter_mut().rev()
                            .find(|c| c.tool_name == name)
                        {
                            card.phase = phase;
                            card.result = Some(result.clone());
                        }
                        // Add result as sub-step to the Agent card.
                        if let Some(msg) = session.messages.last_mut() {
                            if let Some(agent_card) = msg.tool_cards.iter_mut().rev()
                                .find(|c| c.tool_name == "Agent" && c.phase == ToolPhase::Started)
                            {
                                let truncated = if result.len() > 200 {
                                    format!("{}...", &result[..197])
                                } else {
                                    result.clone()
                                };
                                agent_card.sub_steps.push(SubAgentStep {
                                    kind: SubStepKind::ToolResult,
                                    content: truncated,
                                    timestamp: chrono::Utc::now(),
                                });
                            }
                        }
                    }
                });
            } else if name == "Agent" || (in_sub_agent.is_some() && name == "sub_agent_tool") {
                // Agent tool result — complete the sub-agent and exit scope.
                shared.mutate(|s| {
                    if let Some(session) = s.sessions.get_mut(session_id) {
                        if let Some(ref mut streaming) = session.streaming {
                            streaming.active_sub_agent = None;
                        }
                        // Mark Agent card as completed.
                        if let Some(card) = session.active_tool_cards.iter_mut().rev()
                            .find(|c| c.tool_name == "Agent")
                        {
                            card.phase = phase;
                            card.result = Some(result.clone());
                        }
                        if let Some(msg) = session.messages.last_mut() {
                            if let Some(card) = msg.tool_cards.iter_mut().rev()
                                .find(|c| c.tool_name == "Agent")
                            {
                                card.phase = phase;
                                card.result = Some(result.clone());
                                card.sub_steps.push(SubAgentStep {
                                    kind: SubStepKind::Done,
                                    content: "Done".to_string(),
                                    timestamp: chrono::Utc::now(),
                                });
                            }
                        }
                    }
                });
            } else {
                // Normal top-level tool result.
                shared.mutate(|s| {
                    if let Some(session) = s.sessions.get_mut(session_id) {
                        if let Some(card) = session.active_tool_cards.iter_mut().rev()
                            .find(|c| c.tool_name == name)
                        {
                            card.phase = phase;
                            card.result = Some(result.clone());
                        }
                        if let Some(msg) = session.messages.last_mut() {
                            if let Some(card) = msg.tool_cards.iter_mut().rev()
                                .find(|c| c.tool_name == name)
                            {
                                card.phase = phase;
                                card.result = Some(result.clone());
                            }
                        }
                    }
                });
            }
        }
        StreamEvent::ThinkingChunk(text) => {
            shared.mutate(|s| {
                if let Some(session) = s.sessions.get_mut(session_id) {
                    if let Some(msg) = session.messages.last_mut() {
                        let thinking = msg.thinking_content.get_or_insert_with(String::new);
                        thinking.push_str(&text);
                    }
                }
            });
        }
        StreamEvent::UsageData(metrics) => {
            shared.mutate(|s| {
                if let Some(session) = s.sessions.get_mut(session_id) {
                    if let Some(msg) = session.messages.last_mut() {
                        msg.usage = Some(metrics.clone());
                    }
                    if let Some(input) = metrics.input_tokens {
                        session.usage_totals.total_input_tokens += input;
                    }
                    if let Some(output) = metrics.output_tokens {
                        session.usage_totals.total_output_tokens += output;
                    }
                    if let Some(cost) = metrics.estimated_cost {
                        session.usage_totals.total_cost += cost;
                    }
                }
            });
        }
        StreamEvent::Error(msg) => {
            shared.mutate(|s| {
                if let Some(session) = s.sessions.get_mut(session_id) {
                    if let Some(last) = session.messages.last_mut() {
                        if last.status == MessageStatus::Streaming {
                            last.content.push_str(&format!("\n\nError: {msg}"));
                            last.status = MessageStatus::Error;
                        }
                    }
                    session.streaming = None;
                }
                s.tray_state = TrayState::Idle;
            });
        }
        StreamEvent::Done => {
            shared.mutate(|s| {
                if let Some(session) = s.sessions.get_mut(session_id) {
                    if let Some(msg) = session.messages.last_mut() {
                        if msg.status == MessageStatus::Streaming {
                            msg.status = MessageStatus::Complete;
                        }
                    }
                    // Sync final tool card state from active_tool_cards.
                    if let Some(msg) = session.messages.last_mut() {
                        // Replace message tool cards with final active state
                        // (they may have been updated with results).
                        msg.tool_cards = session.active_tool_cards.drain(..).collect();
                    }
                    session.streaming = None;
                    session.usage_totals.message_count += 1;
                }
                s.tray_state = TrayState::Idle;
            });
        }
    }
}

/// Apply a batch of stream events (used by scheduler for non-streaming jobs).
#[allow(dead_code)]
fn apply_stream_events(
    shared: &SharedState,
    session_id: &str,
    events: &[StreamEvent],
    duration_ms: u64,
) {
    for event in events {
        apply_single_event(shared, session_id, event.clone());
    }
    // Set duration after all events applied.
    shared.mutate(|s| {
        if let Some(session) = s.sessions.get_mut(session_id) {
            if let Some(msg) = session.messages.last_mut() {
                msg.duration_ms = Some(duration_ms);
            }
        }
    });
}

async fn run_backend_scan(shared: SharedState) {
    shared.mutate(|s| {
        for b in &mut s.backend_registry {
            b.discovery_state = DiscoveryState::Scanning;
        }
    });

    let results = conductor_core::backend::discovery::scan_all_backends().await;

    shared.mutate(|s| {
        s.backend_registry = results;

        // Set default backend to first found + authenticated if not already set.
        if s.default_backend_id.is_none() {
            if let Some(first) = s.backend_registry.iter().find(|b| {
                b.discovery_state == DiscoveryState::Found
                    && b.auth_state == AuthState::Authenticated
            }) {
                s.default_backend_id = Some(first.backend_id.clone());
                // Update default session's backend too.
                if let Some(session) = s.sessions.get_mut(&s.active_session_id) {
                    session.backend_id = first.backend_id.clone();
                    if let Some(model) = first.default_model.as_ref() {
                        session.model_id = Some(model.clone());
                    }
                }
            }
        }
    });
}
