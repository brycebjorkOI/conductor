use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::state::SlackStatus;
use egui_swift::prelude::*;

use crate::bridge::SharedState;

pub struct ChannelsView {
    shared: SharedState,
    tx: mpsc::UnboundedSender<Action>,
}

impl ChannelsView {
    pub fn new(shared: SharedState, tx: mpsc::UnboundedSender<Action>) -> Self {
        Self { shared, tx }
    }
}

impl View for ChannelsView {
    fn body(&mut self, ui: &mut egui::Ui) {
        let p = ui.palette();

        egui_swift::text!(ui, "Messaging Channels", .title);
        egui_swift::spacer!(ui, 12.0);

        // -- Slack section --
        let slack = self.shared.read().slack.clone();

        Section::new().header("Slack (via slackdump)").show(ui, |ui| {
            let (status_color, status_text) = match slack.status {
                SlackStatus::Connected => (p.status_green, "Connected"),
                SlackStatus::Checking => (p.accent, "Connecting..."),
                SlackStatus::Error => (p.status_red, "Error"),
                SlackStatus::Disconnected => (p.text_muted, "Disconnected"),
            };

            Card::new().show(ui, |ui| {
                egui_swift::hstack!(ui, {
                    StatusDot::new(status_color).show(ui);
                    Label::new("Slack").font(Font::Callout).bold(true).show(ui);
                    Label::new(status_text)
                        .font(Font::Subheadline)
                        .color(status_color)
                        .show(ui);
                    if let Some(ref ws) = slack.workspace_name {
                        Label::new(&format!("({ws})"))
                            .font(Font::Subheadline)
                            .secondary()
                            .show(ui);
                    }
                });

                if let Some(ref err) = slack.error {
                    egui_swift::spacer!(ui, 4.0);
                    Label::new(err)
                        .font(Font::Caption)
                        .color(p.status_red)
                        .show(ui);
                }

                egui_swift::spacer!(ui, 4.0);

                egui_swift::hstack!(ui, {
                    match slack.status {
                        SlackStatus::Connected => {
                            if Button::new("Refresh Channels")
                                .style(ButtonStyle::Bordered)
                                .small(true)
                                .show(ui)
                                .clicked()
                            {
                                let _ = self.tx.send(Action::SlackRefreshChannels);
                            }
                            if Button::new("Disconnect")
                                .style(ButtonStyle::Bordered)
                                .small(true)
                                .show(ui)
                                .clicked()
                            {
                                let _ = self.tx.send(Action::SlackDisconnect);
                            }
                        }
                        SlackStatus::Disconnected | SlackStatus::Error => {
                            if Button::new("Connect")
                                .style(ButtonStyle::BorderedProminent)
                                .small(true)
                                .show(ui)
                                .clicked()
                            {
                                let _ = self.tx.send(Action::SlackConnect);
                            }
                        }
                        SlackStatus::Checking => {
                            ProgressView::spinner().show(ui);
                        }
                    }
                });
            });

            // Show channels if connected.
            if slack.status == SlackStatus::Connected && !slack.channels.is_empty() {
                egui_swift::spacer!(ui, 8.0);
                Label::new(&format!("{} channels available", slack.channels.len()))
                    .font(Font::Subheadline)
                    .secondary()
                    .show(ui);
                egui_swift::spacer!(ui, 4.0);

                let monitored = slack.monitored_channels.clone();

                for ch in &slack.channels {
                    let is_monitored = monitored.contains(&ch.id);
                    egui_swift::hstack!(ui, {
                        let prefix = if ch.is_private { "\u{1f512}" } else { "#" };
                        Label::new(&format!("{prefix} {}", ch.name))
                            .font(Font::Callout)
                            .show(ui);
                        Spacer::trailing(ui, |ui| {
                            if is_monitored {
                                if Button::new("Unmonitor")
                                    .style(ButtonStyle::Bordered)
                                    .small(true)
                                    .show(ui)
                                    .clicked()
                                {
                                    let _ = self.tx.send(Action::SlackUnmonitorChannel {
                                        channel_id: ch.id.clone(),
                                    });
                                }
                            } else {
                                if Button::new("Monitor")
                                    .style(ButtonStyle::Bordered)
                                    .small(true)
                                    .show(ui)
                                    .clicked()
                                {
                                    let _ = self.tx.send(Action::SlackMonitorChannel {
                                        channel_id: ch.id.clone(),
                                    });
                                }
                            }
                        });
                    });
                }
            }

            if slack.status == SlackStatus::Disconnected {
                egui_swift::spacer!(ui, 4.0);
                Label::new("Uses credentials from slackdump. Run 'slackdump workspace new' to authenticate.")
                    .font(Font::Caption)
                    .muted()
                    .show(ui);
            }
        });

        egui_swift::spacer!(ui, 16.0);

        // -- Other platforms (stubs) --
        Section::new().header("Other Platforms").show(ui, |ui| {
            let platforms = [
                ("Telegram", "Bot token via BotFather"),
                ("Discord", "Gateway WebSocket (Bot token)"),
                ("Matrix", "Client-Server API (access token + homeserver)"),
            ];

            for (name, auth_hint) in platforms {
                Card::new().show(ui, |ui| {
                    egui_swift::hstack!(ui, {
                        StatusDot::new(p.text_muted).show(ui);
                        Label::new(name).font(Font::Callout).bold(true).show(ui);
                        egui_swift::text!(ui, "Not configured", .subheadline, .muted);
                    });
                    egui_swift::text!(ui, &format!("Auth: {auth_hint}"), .caption, .secondary);
                });
            }
        });
    }
}
