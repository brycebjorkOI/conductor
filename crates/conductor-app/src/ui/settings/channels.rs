use egui_swift::prelude::*;

use crate::bridge::SharedState;

pub fn show(ui: &mut egui::Ui, shared: &SharedState) {
    let p = ui.palette();

    egui_swift::text!(ui, "Messaging Channels", .title);
    egui_swift::spacer!(ui, 12.0);

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
                egui_swift::hstack!(ui, {
                    StatusDot::new(p.text_muted).show(ui);
                    Label::new(name).font(Font::Callout).bold(true).show(ui);
                    egui_swift::text!(ui, "Not configured", .subheadline, .muted);
                });
                egui_swift::text!(ui, &format!("Auth: {auth_hint}"), .caption, .secondary);
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
                egui_swift::hstack!(ui, {
                    StatusDot::new(status_color).show(ui);
                    Label::new(&ch.display_name).font(Font::Callout).bold(true).show(ui);
                    egui_swift::text!(ui, &format!("{:?}", ch.connection_state), .subheadline, .secondary);
                });
                egui_swift::text!(ui, &format!("Recv: {} | Sent: {}", ch.stats.messages_received, ch.stats.messages_sent), .caption, .muted);
            });
        }
    }
}
