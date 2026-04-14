use egui_swift::card::Card;
use egui_swift::colors;
use egui_swift::status_dot::StatusDot;

use crate::bridge::SharedState;

pub fn show(ui: &mut egui::Ui, shared: &SharedState) {
    let p = colors::palette(ui);

    ui.label(
        egui::RichText::new("Messaging Channels")
            .size(22.0)
            .strong()
            .color(p.text_primary),
    );
    ui.add_space(12.0);

    let state = shared.read();
    let channels = state.channels.clone();
    drop(state);

    if channels.is_empty() {
        let platforms = [
            ("Telegram", "Bot token via BotFather"),
            ("Slack", "Socket Mode (Bot + App-level tokens)"),
            ("Discord", "Gateway WebSocket (Bot token)"),
            ("Matrix", "Client-Server API (access token + homeserver)"),
            ("Mattermost", "WebSocket (Personal access token)"),
            ("Mastodon", "Streaming API (OAuth2 token)"),
            ("Zulip", "Event queue (Bot email + API key)"),
            ("Rocket.Chat", "DDP WebSocket (userId + token)"),
            ("Twitch", "EventSub (OAuth2 + Client ID)"),
        ];

        for (name, auth_hint) in platforms {
            Card::new().show(ui, |ui| {
                ui.horizontal(|ui| {
                    StatusDot::new(p.text_muted).show(ui);
                    ui.label(egui::RichText::new(name).strong().size(13.0));
                    ui.label(
                        egui::RichText::new("Not configured")
                            .size(12.0)
                            .color(p.text_muted),
                    );
                });
                ui.label(
                    egui::RichText::new(format!("Auth: {auth_hint}"))
                        .size(11.0)
                        .color(p.text_secondary),
                );
            });
        }
    } else {
        for (_id, ch) in &channels {
            let status_color = match ch.connection_state {
                conductor_core::state::ChannelConnectionState::Connected => p.status_green,
                conductor_core::state::ChannelConnectionState::Connecting
                | conductor_core::state::ChannelConnectionState::Reconnecting => p.status_yellow,
                conductor_core::state::ChannelConnectionState::Error => p.status_red,
                _ => p.text_muted,
            };

            Card::new().border_color(status_color).show(ui, |ui| {
                ui.horizontal(|ui| {
                    StatusDot::new(status_color).show(ui);
                    ui.label(egui::RichText::new(&ch.display_name).strong().size(13.0));
                    ui.label(
                        egui::RichText::new(format!("{:?}", ch.connection_state))
                            .size(12.0)
                            .color(p.text_secondary),
                    );
                });
                ui.label(
                    egui::RichText::new(format!(
                        "Recv: {} | Sent: {}",
                        ch.stats.messages_received, ch.stats.messages_sent
                    ))
                    .size(11.0)
                    .color(p.text_muted),
                );
            });
        }
    }
}
