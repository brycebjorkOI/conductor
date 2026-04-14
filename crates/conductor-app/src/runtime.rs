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

                    let outcome = orchestrator::run_chat(
                        def.as_ref(),
                        &binary_for_spawn,
                        params,
                        working_dir,
                        &env_overrides,
                        cancel_rx,
                    )
                    .await;

                    match outcome {
                        SendOutcome::Completed { events, duration_ms } => {
                            apply_stream_events(&shared_stream, &sid, &events, duration_ms);
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

            _ => {
                tracing::debug!("unhandled action: {:?}", std::mem::discriminant(&action));
            }
        }
    }
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

fn apply_stream_events(
    shared: &SharedState,
    session_id: &str,
    events: &[StreamEvent],
    duration_ms: u64,
) {
    for event in events {
        match event {
            StreamEvent::TextChunk(text) => {
                shared.mutate(|s| {
                    if let Some(session) = s.sessions.get_mut(session_id) {
                        if let Some(ref mut streaming) = session.streaming {
                            streaming.accumulated_text.push_str(text);
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
            StreamEvent::ToolStart { name, args } => {
                shared.mutate(|s| {
                    if let Some(session) = s.sessions.get_mut(session_id) {
                        session.active_tool_cards.push(ToolCard {
                            tool_name: name.clone(),
                            phase: ToolPhase::Started,
                            arguments: args.clone(),
                            result: None,
                            metadata: None,
                            timestamp: chrono::Utc::now(),
                        });
                    }
                });
            }
            StreamEvent::ToolResult {
                name,
                result,
                success,
            } => {
                shared.mutate(|s| {
                    if let Some(session) = s.sessions.get_mut(session_id) {
                        if let Some(card) = session
                            .active_tool_cards
                            .iter_mut()
                            .rev()
                            .find(|c| c.tool_name == *name)
                        {
                            card.phase = if *success {
                                ToolPhase::Completed
                            } else {
                                ToolPhase::Failed
                            };
                            card.result = Some(result.clone());
                        }
                    }
                });
            }
            StreamEvent::ThinkingChunk(text) => {
                shared.mutate(|s| {
                    if let Some(session) = s.sessions.get_mut(session_id) {
                        if let Some(msg) = session.messages.last_mut() {
                            let thinking = msg.thinking_content.get_or_insert_with(String::new);
                            thinking.push_str(text);
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
                        // Accumulate totals.
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
                                msg.duration_ms = Some(duration_ms);
                            }
                        }
                        // Move tool cards to the message.
                        if let Some(msg) = session.messages.last_mut() {
                            msg.tool_cards
                                .extend(session.active_tool_cards.drain(..));
                        }
                        session.streaming = None;
                        session.usage_totals.message_count += 1;
                    }
                    s.tray_state = TrayState::Idle;
                });
            }
        }
    }
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
