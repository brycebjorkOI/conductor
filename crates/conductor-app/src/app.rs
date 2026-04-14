use tokio::sync::mpsc;

use conductor_core::config;
use conductor_core::events::Action;
use conductor_core::session;
use conductor_core::state::AppState;
use egui_swift::suggestion_chip;

use crate::bridge::SharedState;
use crate::runtime::RuntimeHandle;
use crate::theme::Theme;
use crate::ui;

pub struct ConductorApp {
    shared: SharedState,
    action_tx: mpsc::UnboundedSender<Action>,
    _runtime: RuntimeHandle,
    input_text: String,
    show_autocomplete: bool,
    sidebar_open: bool,
    sidebar_state: ui::sidebar::SidebarState,
    selected_backend_idx: usize,
    schedules_state: ui::settings::schedules::SchedulesTabState,
}

impl ConductorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        crate::theme::apply(&cc.egui_ctx);

        let config_path = config::config_file_path();
        let cfg = config::load_config(&config_path);

        let mut state = AppState::default();
        state.config = cfg;

        let saved = session::load_all_sessions();
        if !saved.is_empty() {
            for s in saved {
                state.sessions.insert(s.id.clone(), s);
            }
        }

        let shared = SharedState::new(state, cc.egui_ctx.clone());
        let runtime = RuntimeHandle::start(shared.clone());

        Self {
            shared: shared.clone(),
            action_tx: runtime.action_tx.clone(),
            _runtime: runtime,
            input_text: String::new(),
            show_autocomplete: false,
            sidebar_open: true,
            sidebar_state: ui::sidebar::SidebarState::default(),
            selected_backend_idx: 0,
            schedules_state: ui::settings::schedules::SchedulesTabState::default(),
        }
    }
}

impl eframe::App for ConductorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dark = ctx.style().visuals.dark_mode;
        let p = if dark {
            egui_swift::colors::dark()
        } else {
            egui_swift::colors::light()
        };

        // Settings view.
        {
            let state = self.shared.read();
            if state.settings_open {
                drop(state);
                ui::settings::show(ctx, &self.shared, &self.action_tx, &mut self.schedules_state);
                return;
            }
        }

        // -- Sidebar (using egui-swift SidebarPanel) --
        if self.sidebar_open {
            egui_swift::sidebar::SidebarPanel::new()
                .width(Theme::SIDEBAR_WIDTH)
                .show(ctx, |ui| {
                    ui::sidebar::show(
                        ui,
                        &self.shared,
                        &self.action_tx,
                        &mut self.sidebar_state,
                    );
                });
        }

        // -- Header --
        egui::TopBottomPanel::top("header")
            .frame(
                egui::Frame::NONE
                    .fill(p.surface)
                    .inner_margin(egui::Margin {
                        left: 0,
                        right: 0,
                        top: 30, // traffic light clearance
                        bottom: 6,
                    }),
            )
            .show(ctx, |ui| {
                ui::chat::header::show(
                    ui,
                    &self.shared,
                    &self.action_tx,
                    &mut self.sidebar_open,
                    &mut self.selected_backend_idx,
                );
            });

        // Gather session info.
        let state = self.shared.read();
        let is_streaming = state
            .sessions
            .get(&state.active_session_id)
            .and_then(|s| s.streaming.as_ref())
            .map_or(false, |st| st.is_active);
        let active_sid = state.active_session_id.clone();
        let has_messages = state
            .sessions
            .get(&state.active_session_id)
            .map_or(false, |s| !s.messages.is_empty());
        let session_clone = state.sessions.get(&state.active_session_id).cloned();
        drop(state);

        if has_messages {
            // --- Normal chat mode: input at bottom, messages above ---
            egui::TopBottomPanel::bottom("input_bar")
                .frame(
                    egui::Frame::NONE
                        .fill(p.surface)
                        .inner_margin(egui::Margin::ZERO),
                )
                .show(ctx, |ui| {
                    ui::chat::input_bar::show(
                        ui,
                        &mut self.input_text,
                        &mut self.show_autocomplete,
                        is_streaming,
                        &active_sid,
                        &self.action_tx,
                    );
                });

            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.fill(p.surface))
                .show(ctx, |ui| {
                    if let Some(ref session) = session_clone {
                        ui::chat::message_list::show(ui, session);
                    }
                });
        } else {
            // --- Empty state: greeting + centered input + suggestion chips ---
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.fill(p.surface))
                .show(ctx, |ui| {
                    let available_width = ui.available_width();
                    let content_width = available_width.min(Theme::MAX_CONTENT_WIDTH);
                    let side_padding = ((available_width - content_width) / 2.0).max(24.0);
                    let available_height = ui.available_height();

                    ui.vertical(|ui| {
                        ui.add_space(available_height * 0.28);

                        // Greeting.
                        ui.horizontal(|ui| {
                            ui.add_space(side_padding);
                            ui.vertical(|ui| {
                                ui.set_max_width(content_width);
                                ui.vertical_centered(|ui| {
                                    let greeting = time_greeting();
                                    ui.label(
                                        egui::RichText::new(format!("\u{2728}  {greeting}"))
                                            .size(26.0)
                                            .color(p.text_primary),
                                    );
                                });
                            });
                        });

                        ui.add_space(24.0);

                        // Chat input (centered).
                        ui::chat::input_bar::show(
                            ui,
                            &mut self.input_text,
                            &mut self.show_autocomplete,
                            is_streaming,
                            &active_sid,
                            &self.action_tx,
                        );

                        ui.add_space(16.0);

                        // Suggestion chips (using egui-swift).
                        ui.vertical_centered(|ui| {
                            suggestion_chip::chip_row(
                                ui,
                                &[
                                    ("\u{270f}", "Write"),
                                    ("\u{1f4d6}", "Learn"),
                                    ("</>" , "Code"),
                                    ("\u{1f4a1}", "Brainstorm"),
                                ],
                            );
                        });
                    });
                });
        }
    }
}

fn time_greeting() -> &'static str {
    let hour = chrono::Local::now()
        .format("%H")
        .to_string()
        .parse::<u32>()
        .unwrap_or(12);
    if hour < 12 {
        "Good morning"
    } else if hour < 18 {
        "Good afternoon"
    } else {
        "Good evening"
    }
}
