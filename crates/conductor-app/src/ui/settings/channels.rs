use egui_swift::prelude::*;

use crate::bridge::SharedState;

pub fn show(ui: &mut egui::Ui, shared: &SharedState) {
    let p = ui.palette();

    Label::heading("Messaging Channels").show(ui);
    Spacer::fixed(12.0).show(ui);

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
                HStack::new().show(ui, |ui| {
                    StatusDot::new(p.text_muted).show(ui);
                    Label::new(name).font(Font::Callout).bold(true).show(ui);
                    Label::new("Not configured")
                        .font(Font::Subheadline)
                        .muted()
                        .show(ui);
                });
                Label::new(&format!("Auth: {auth_hint}"))
                    .font(Font::Caption)
                    .secondary()
                    .show(ui);
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
                HStack::new().show(ui, |ui| {
                    StatusDot::new(status_color).show(ui);
                    Label::new(&ch.display_name)
                        .font(Font::Callout)
                        .bold(true)
                        .show(ui);
                    Label::new(&format!("{:?}", ch.connection_state))
                        .font(Font::Subheadline)
                        .secondary()
                        .show(ui);
                });
                Label::new(&format!(
                    "Recv: {} | Sent: {}",
                    ch.stats.messages_received, ch.stats.messages_sent
                ))
                .font(Font::Caption)
                .muted()
                .show(ui);
            });
        }
    }
}
