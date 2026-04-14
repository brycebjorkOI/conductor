use tokio::sync::mpsc;

use conductor_core::config;
use conductor_core::events::Action;
use conductor_core::session;
use conductor_core::state::AppState;

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
}

impl ConductorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Theme::apply(&cc.egui_ctx);

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
        }
    }
}

impl eframe::App for ConductorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dark = ctx.style().visuals.dark_mode;

        // Settings view.
        {
            let state = self.shared.read();
            if state.settings_open {
                drop(state);
                ui::settings::show(ctx, &self.shared, &self.action_tx);
                return;
            }
        }

        // -- Sidebar --
        if self.sidebar_open {
            egui::SidePanel::left("sidebar")
                .resizable(false)
                .exact_width(Theme::SIDEBAR_WIDTH)
                .frame(
                    egui::Frame::NONE
                        .fill(Theme::sidebar_bg(dark))
                        .inner_margin(egui::Margin::same(0))
                        .stroke(egui::Stroke::new(
                            0.5,
                            if dark {
                                egui::Color32::from_rgb(50, 50, 50)
                            } else {
                                egui::Color32::from_rgb(222, 220, 216)
                            },
                        )),
                )
                .show(ctx, |ui| {
                    ui::sidebar::show(ui, &self.shared, &self.action_tx);
                });
        }

        // -- Header (top) --
        egui::TopBottomPanel::top("header")
            .frame(
                egui::Frame::NONE
                    .fill(Theme::surface(dark))
                    .inner_margin(egui::Margin { left: 0, right: 0, top: 30, bottom: 6 }),
            )
            .show(ctx, |ui| {
                ui::chat::header::show(ui, &self.shared, &self.action_tx, &mut self.sidebar_open);
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
                        .fill(Theme::surface(dark))
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
                .frame(egui::Frame::NONE.fill(Theme::surface(dark)))
                .show(ctx, |ui| {
                    if let Some(ref session) = session_clone {
                        ui::chat::message_list::show(ui, session);
                    }
                });
        } else {
            // --- Empty state: greeting + input centered in the page ---

            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.fill(Theme::surface(dark)))
                .show(ctx, |ui| {
                    let available_width = ui.available_width();
                    let content_width = available_width.min(Theme::MAX_CONTENT_WIDTH);
                    let side_padding = ((available_width - content_width) / 2.0).max(24.0);
                    let available_height = ui.available_height();

                    ui.vertical(|ui| {
                        // Push greeting to ~35% from top.
                        ui.add_space(available_height * 0.28);

                        // Greeting.
                        ui.horizontal(|ui| {
                            ui.add_space(side_padding);
                            ui.vertical(|ui| {
                                ui.set_max_width(content_width);
                                ui.vertical_centered(|ui| {
                                    let greeting = {
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
                                    };
                                    ui.label(
                                        egui::RichText::new(format!("\u{2728}  {greeting}"))
                                            .size(26.0)
                                            .color(Theme::text_primary(dark)),
                                    );
                                });
                            });
                        });

                        ui.add_space(24.0);

                        // Input bar — centered in the page.
                        ui::chat::input_bar::show(
                            ui,
                            &mut self.input_text,
                            &mut self.show_autocomplete,
                            is_streaming,
                            &active_sid,
                            &self.action_tx,
                        );

                        ui.add_space(16.0);

                        // Suggestion chips (like Claude's Write/Learn/Code/etc).
                        ui.vertical_centered(|ui| {
                            ui.horizontal(|ui| {
                                let chips = [
                                    ("\u{270f}", "Write"),
                                    ("\u{2728}", "Learn"),
                                    ("\u{2699}", "Code"),
                                    ("\u{25ef}", "Chat"),
                                ];
                                for (icon, label) in chips {
                                    let btn = egui::Button::new(
                                        egui::RichText::new(format!("{icon}  {label}"))
                                            .size(12.0)
                                            .color(Theme::text_secondary(dark)),
                                    )
                                    .fill(egui::Color32::TRANSPARENT)
                                    .stroke(egui::Stroke::new(
                                        0.5,
                                        Theme::input_border(dark),
                                    ))
                                    .corner_radius(egui::CornerRadius::same(16));

                                    ui.add(btn);
                                    ui.add_space(4.0);
                                }
                            });
                        });
                    });
                });
        }
    }
}
