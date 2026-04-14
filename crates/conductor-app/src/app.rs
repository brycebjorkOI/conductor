use conductor_core::config;
use conductor_core::session;
use conductor_core::state::AppState;
use egui_swift::prelude::*;

use crate::bridge::SharedState;
use crate::runtime::RuntimeHandle;
use crate::ui;

pub struct ConductorApp {
    shared: SharedState,
    _runtime: RuntimeHandle,

    // Views (each owns its own state + action sender).
    sidebar: ui::sidebar::SidebarView,
    header: ui::chat::header::HeaderView,
    input_bar: ui::chat::input_bar::InputBarView,
    settings: ui::settings::SettingsView,
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
        let tx = runtime.action_tx.clone();

        Self {
            shared: shared.clone(),
            _runtime: runtime,
            sidebar: ui::sidebar::SidebarView::new(shared.clone(), tx.clone()),
            header: ui::chat::header::HeaderView::new(shared.clone(), tx.clone()),
            input_bar: ui::chat::input_bar::InputBarView::new(tx.clone()),
            settings: ui::settings::SettingsView::new(shared.clone(), tx),
        }
    }
}

impl eframe::App for ConductorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let p = ctx.palette();

        // Settings view.
        if self.shared.read().settings_open {
            self.settings.show_ctx(ctx);
            return;
        }

        // -- Sidebar --
        if self.header.sidebar_open {
            SidebarPanel::new()
                .width(Layout::SIDEBAR_WIDTH)
                .show(ctx, |ui| {
                    self.sidebar.show(ui);
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
                        top: 30,
                        bottom: 6,
                    }),
            )
            .show(ctx, |ui| {
                self.header.show(ui);
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
            egui::TopBottomPanel::bottom("input_bar")
                .frame(
                    egui::Frame::NONE
                        .fill(p.surface)
                        .inner_margin(egui::Margin::ZERO),
                )
                .show(ctx, |ui| {
                    self.input_bar
                        .show_for_session(ui, is_streaming, &active_sid);
                });

            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.fill(p.surface))
                .show(ctx, |ui| {
                    if let Some(ref session) = session_clone {
                        ui::chat::message_list::show(ui, session);
                    }
                });
        } else {
            // --- Empty state ---
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.fill(p.surface))
                .show(ctx, |ui| {
                    let available_height = ui.available_height();
                    VStack::new().show(ui, |ui| {
                        Spacer::fixed(available_height * 0.28).show(ui);

                        ui.centered_content(Layout::MAX_CONTENT_WIDTH, |ui| {
                            ui.vertical_centered(|ui| {
                                let greeting = time_greeting();
                                Label::new(&format!("{}  {greeting}", icons::SPARKLE))
                                    .font(Font::LargeTitle)
                                    .show(ui);
                            });
                        });

                        Spacer::fixed(24.0).show(ui);

                        self.input_bar
                            .show_for_session(ui, is_streaming, &active_sid);

                        Spacer::fixed(16.0).show(ui);

                        ui.vertical_centered(|ui| {
                            suggestion_chip::chip_row(
                                ui,
                                &[
                                    ("\u{270f}", "Write"),
                                    ("\u{1f4d6}", "Learn"),
                                    ("</>", "Code"),
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
