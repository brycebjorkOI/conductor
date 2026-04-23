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
    message_list: ui::chat::message_list::MessageListView,
    input_bar: ui::chat::input_bar::InputBarView,
    settings: ui::settings::SettingsView,
    notifications: ui::notifications::NotificationsView,
    schedules: ui::schedules::SchedulesView,
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
            message_list: ui::chat::message_list::MessageListView::new(),
            input_bar: ui::chat::input_bar::InputBarView::new(tx.clone()),
            settings: ui::settings::SettingsView::new(shared.clone(), tx.clone()),
            notifications: ui::notifications::NotificationsView::new(shared.clone(), tx.clone()),
            schedules: ui::schedules::SchedulesView::new(shared.clone(), tx),
        }
    }

    /// Show sidebar + central panel with standard padding. Used by schedules, notifications, etc.
    fn show_with_sidebar(
        &mut self,
        ctx: &egui::Context,
        content: impl FnOnce(&mut Self, &mut egui::Ui),
    ) {
        let p = ctx.palette();

        if self.header.sidebar_open {
            SidebarPanel::new()
                .width(Layout::SIDEBAR_WIDTH)
                .show(ctx, |ui| {
                    self.sidebar.show(ui);
                });
        }

        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(p.surface)
                    .inner_margin(egui::Margin::symmetric(24, 24)),
            )
            .show(ctx, |ui| {
                content(self, ui);
            });
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

        // Schedules view.
        if self.shared.read().current_view == conductor_core::state::ViewMode::Schedules {
            self.show_with_sidebar(ctx, |app, ui| {
                app.schedules.show(ui);
            });
            return;
        }

        // Notifications view.
        if self.shared.read().notifications_open {
            self.show_with_sidebar(ctx, |app, ui| {
                app.notifications.show(ui);
            });
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

        // Update input bar fields so body() has what it needs.
        self.input_bar.is_streaming = is_streaming;
        self.input_bar.active_session_id = active_sid;

        // Update message list session.
        self.message_list.session = session_clone.clone();

        if has_messages {
            egui::TopBottomPanel::bottom("input_bar")
                .frame(
                    egui::Frame::NONE
                        .fill(p.surface)
                        .inner_margin(egui::Margin::ZERO),
                )
                .show(ctx, |ui| {
                    self.input_bar.show(ui);
                });

            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.fill(p.surface))
                .show(ctx, |ui| {
                    self.message_list.show(ui);
                });
        } else {
            // --- Empty state ---
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.fill(p.surface))
                .show(ctx, |ui| {
                    let available_height = ui.available_height();
                    let available_width = ui.available_width();
                    let top_space = (available_height * 0.18).max(40.0);
                    let min_margin = 24.0;
                    let content_width = (available_width - min_margin * 2.0).min(Layout::MAX_CONTENT_WIDTH).max(200.0);
                    let side = (available_width - content_width) / 2.0;

                    ui.add_space(top_space);

                    ui.horizontal(|ui| {
                        ui.add_space(side);
                        ui.vertical(|ui| {
                            ui.set_max_width(content_width);

                            ui.vertical_centered(|ui| {
                                let greeting = time_greeting();
                                Label::new(&format!("{}  {greeting}", icons::SPARKLE))
                                    .font(Font::LargeTitle)
                                    .show(ui);
                            });

                            egui_swift::spacer!(ui, 16.0);

                            self.input_bar.show(ui);
                        });
                        ui.add_space(side);
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
