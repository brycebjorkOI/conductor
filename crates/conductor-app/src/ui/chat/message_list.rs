use conductor_core::state::*;
use crate::theme::Theme;

pub fn show(ui: &mut egui::Ui, session: &Session) {
    let dark = ui.visuals().dark_mode;

    let is_streaming = session
        .streaming
        .as_ref()
        .map_or(false, |s| s.is_active);

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .stick_to_bottom(is_streaming || !session.messages.is_empty())
        .show(ui, |ui| {
            let available_width = ui.available_width();
            let content_width = available_width.min(Theme::MAX_CONTENT_WIDTH);
            let side_padding = ((available_width - content_width) / 2.0).max(24.0);

            ui.horizontal(|ui| {
                ui.add_space(side_padding);
                ui.vertical(|ui| {
                    ui.set_max_width(content_width);

                    if session.messages.is_empty() {
                        render_empty_state(ui, dark);
                        return;
                    }

                    ui.add_space(24.0);

                    for msg in &session.messages {
                        render_message(ui, msg, dark);
                        ui.add_space(Theme::MESSAGE_SPACING);
                    }

                    if is_streaming {
                        let time = ui.input(|i| i.time);
                        let visible = (time * 2.5) as u64 % 2 == 0;
                        if visible {
                            ui.label(
                                egui::RichText::new("\u{2588}")
                                    .size(Theme::BODY_FONT_SIZE)
                                    .color(Theme::accent(dark)),
                            );
                        } else {
                            ui.add_space(Theme::BODY_FONT_SIZE + 4.0);
                        }
                        ui.ctx().request_repaint();
                    }

                    ui.add_space(24.0);
                });
                ui.add_space(side_padding);
            });
        });
}

fn render_empty_state(ui: &mut egui::Ui, dark: bool) {
    // Claude-style: greeting centered vertically with sparkle to the left.
    let greeting = {
        let hour = chrono::Local::now().format("%H").to_string().parse::<u32>().unwrap_or(12);
        if hour < 12 {
            "Good morning"
        } else if hour < 18 {
            "Good afternoon"
        } else {
            "Good evening"
        }
    };

    // Push the greeting to roughly the vertical center of the available space.
    let available_height = ui.available_height();
    let top_space = (available_height * 0.30).max(60.0);
    ui.add_space(top_space);

    // Sparkle + greeting centered — use a single RichText label for perfect centering.
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new(format!("\u{2728}  {greeting}"))
                .size(26.0)
                .color(Theme::text_primary(dark)),
        );

        ui.add_space(8.0);

        ui.label(
            egui::RichText::new("How can I help you today?")
                .size(14.5)
                .color(Theme::text_muted(dark)),
        );
    });
}

fn render_message(ui: &mut egui::Ui, msg: &Message, dark: bool) {
    match msg.role {
        MessageRole::User => render_user_message(ui, msg, dark),
        MessageRole::Assistant => render_assistant_message(ui, msg, dark),
        MessageRole::System => render_system_message(ui, msg, dark),
        MessageRole::Error => render_error_message(ui, msg, dark),
    }
}

fn render_user_message(ui: &mut egui::Ui, msg: &Message, dark: bool) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
        let frame = egui::Frame::NONE
            .fill(Theme::user_bubble_bg(dark))
            .corner_radius(egui::CornerRadius {
                nw: Theme::USER_BUBBLE_RADIUS as u8,
                ne: 4,
                sw: Theme::USER_BUBBLE_RADIUS as u8,
                se: Theme::USER_BUBBLE_RADIUS as u8,
            })
            .inner_margin(egui::Margin::symmetric(16, 10));

        frame.show(ui, |ui| {
            ui.set_max_width(Theme::MAX_CONTENT_WIDTH * 0.78);
            ui.with_layout(
                egui::Layout::left_to_right(egui::Align::TOP).with_main_wrap(true),
                |ui| {
                    ui.label(
                        egui::RichText::new(&msg.content)
                            .size(Theme::BODY_FONT_SIZE)
                            .color(Theme::user_bubble_text(dark)),
                    );
                },
            );
        });
    });
}

fn render_assistant_message(ui: &mut egui::Ui, msg: &Message, dark: bool) {
    for card in &msg.tool_cards {
        render_tool_card(ui, card, dark);
        ui.add_space(6.0);
    }

    if !msg.content.is_empty() {
        let mut cache = egui_commonmark::CommonMarkCache::default();
        egui_commonmark::CommonMarkViewer::new()
            .show(ui, &mut cache, &msg.content);
    }

    if msg.status == MessageStatus::Cancelled {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Stopped generating")
                .italics()
                .size(Theme::SMALL_FONT_SIZE)
                .color(Theme::text_muted(dark)),
        );
    }

    if let Some(ref usage) = msg.usage {
        ui.add_space(6.0);
        let mut parts = Vec::new();
        if let Some(input) = usage.input_tokens {
            parts.push(format!("{input} in"));
        }
        if let Some(output) = usage.output_tokens {
            parts.push(format!("{output} out"));
        }
        if let Some(cost) = usage.estimated_cost {
            parts.push(format!("${cost:.4}"));
        }
        if let Some(ms) = msg.duration_ms {
            parts.push(format!("{:.1}s", ms as f64 / 1000.0));
        }
        if !parts.is_empty() {
            ui.label(
                egui::RichText::new(parts.join(" \u{00b7} "))
                    .size(11.0)
                    .color(Theme::text_muted(dark)),
            );
        }
    }
}

fn render_system_message(ui: &mut egui::Ui, msg: &Message, dark: bool) {
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new(&msg.content)
                .size(Theme::SMALL_FONT_SIZE)
                .italics()
                .color(Theme::text_muted(dark)),
        );
    });
}

fn render_error_message(ui: &mut egui::Ui, msg: &Message, dark: bool) {
    egui::Frame::NONE
        .fill(Theme::error_bg(dark))
        .corner_radius(egui::CornerRadius::same(12))
        .inner_margin(egui::Margin::symmetric(16, 10))
        .show(ui, |ui| {
            ui.set_max_width(Theme::MAX_CONTENT_WIDTH);
            ui.label(
                egui::RichText::new(&msg.content)
                    .size(Theme::BODY_FONT_SIZE)
                    .color(Theme::status_red()),
            );
        });
}

fn render_tool_card(ui: &mut egui::Ui, card: &ToolCard, dark: bool) {
    egui::Frame::NONE
        .fill(Theme::tool_card_bg(dark))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            let (status_icon, status_color) = match card.phase {
                ToolPhase::Started => ("\u{25cf}", Theme::status_yellow()),
                ToolPhase::Completed => ("\u{2713}", Theme::status_green()),
                ToolPhase::Failed => ("\u{2717}", Theme::status_red()),
            };

            egui::CollapsingHeader::new(
                egui::RichText::new(format!("{status_icon}  {}", card.tool_name))
                    .size(12.0)
                    .monospace()
                    .color(status_color),
            )
            .default_open(false)
            .show(ui, |ui| {
                for (key, value) in &card.arguments {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{key}:"))
                                .monospace()
                                .size(11.0)
                                .color(Theme::text_secondary(dark)),
                        );
                        let val_str = match value {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        ui.label(
                            egui::RichText::new(val_str)
                                .monospace()
                                .size(11.0)
                                .color(Theme::text_primary(dark)),
                        );
                    });
                }
                if let Some(ref result) = card.result {
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(result)
                            .monospace()
                            .size(11.0)
                            .color(Theme::text_secondary(dark)),
                    );
                }
            });
        });
}
