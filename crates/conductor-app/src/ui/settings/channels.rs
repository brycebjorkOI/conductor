use crate::bridge::SharedState;

pub fn show(ui: &mut egui::Ui, shared: &SharedState) {
    let p = egui_swift::colors::palette(ui);

    ui.heading("Messaging Channels");
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
            egui::Frame::NONE
                .fill(p.surface_raised)
                .corner_radius(egui::CornerRadius::same(8))
                .stroke(egui::Stroke::new(0.5, p.border_subtle))
                .inner_margin(egui::Margin::symmetric(12, 8))
                .outer_margin(egui::Margin::symmetric(0, 3))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Gray dot — not configured.
                        let (dot_rect, _) = ui.allocate_exact_size(
                            egui::vec2(8.0, 8.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().circle_filled(
                            dot_rect.center(),
                            4.0,
                            p.text_muted,
                        );

                        ui.label(
                            egui::RichText::new(name).strong().size(13.0),
                        );
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
                | conductor_core::state::ChannelConnectionState::Reconnecting => {
                    egui::Color32::from_rgb(210, 170, 50)
                }
                conductor_core::state::ChannelConnectionState::Error => p.status_red,
                _ => p.text_muted,
            };

            egui::Frame::NONE
                .fill(p.surface_raised)
                .corner_radius(egui::CornerRadius::same(8))
                .stroke(egui::Stroke::new(0.5, status_color))
                .inner_margin(egui::Margin::symmetric(12, 8))
                .outer_margin(egui::Margin::symmetric(0, 3))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (dot_rect, _) = ui.allocate_exact_size(
                            egui::vec2(8.0, 8.0),
                            egui::Sense::hover(),
                        );
                        ui.painter()
                            .circle_filled(dot_rect.center(), 4.0, status_color);

                        ui.label(egui::RichText::new(&ch.display_name).strong());
                        ui.label(
                            egui::RichText::new(format!("{:?}", ch.connection_state))
                                .size(12.0)
                                .color(p.text_secondary),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "Recv: {} | Sent: {}",
                                ch.stats.messages_received, ch.stats.messages_sent
                            ))
                            .size(11.0)
                            .color(p.text_muted),
                        );
                    });
                });
        }
    }
}
