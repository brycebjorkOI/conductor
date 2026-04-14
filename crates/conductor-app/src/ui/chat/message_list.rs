use conductor_core::state::*;
use egui_swift::card::Card;
use egui_swift::colors;
use egui_swift::divider::Divider;
use egui_swift::icons;
use egui_swift::theme::Layout;

pub fn show(ui: &mut egui::Ui, session: &Session) {
    let is_streaming = session
        .streaming
        .as_ref()
        .map_or(false, |s| s.is_active);

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .stick_to_bottom(is_streaming || !session.messages.is_empty())
        .show(ui, |ui| {
            let available_width = ui.available_width();
            let content_width = available_width.min(Layout::MAX_CONTENT_WIDTH);
            let side_padding = ((available_width - content_width) / 2.0).max(24.0);

            ui.horizontal(|ui| {
                ui.add_space(side_padding);
                ui.vertical(|ui| {
                    ui.set_max_width(content_width);

                    if session.messages.is_empty() {
                        render_empty_state(ui);
                        return;
                    }

                    ui.add_space(24.0);

                    for msg in &session.messages {
                        render_message(ui, msg);
                        ui.add_space(Layout::MESSAGE_SPACING);
                    }

                    if is_streaming {
                        let p = colors::palette(ui);
                        let time = ui.input(|i| i.time);
                        let visible = (time * 2.5) as u64 % 2 == 0;
                        if visible {
                            ui.label(
                                egui::RichText::new("\u{2588}")
                                    .size(Layout::BODY_FONT_SIZE)
                                    .color(p.accent),
                            );
                        } else {
                            ui.add_space(Layout::BODY_FONT_SIZE + 4.0);
                        }
                        ui.ctx().request_repaint();
                    }

                    ui.add_space(24.0);
                });
                ui.add_space(side_padding);
            });
        });
}

fn render_empty_state(ui: &mut egui::Ui) {
    let p = colors::palette(ui);
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

    let available_height = ui.available_height();
    let top_space = (available_height * 0.30).max(60.0);
    ui.add_space(top_space);

    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new(format!("{}  {greeting}", icons::SPARKLE))
                .size(26.0)
                .color(p.text_primary),
        );

        ui.add_space(8.0);

        ui.label(
            egui::RichText::new("How can I help you today?")
                .size(14.5)
                .color(p.text_muted),
        );
    });
}

fn render_message(ui: &mut egui::Ui, msg: &Message) {
    match msg.role {
        MessageRole::User => render_user_message(ui, msg),
        MessageRole::Assistant => render_assistant_message(ui, msg),
        MessageRole::System => render_system_message(ui, msg),
        MessageRole::Error => render_error_message(ui, msg),
    }
}

fn render_user_message(ui: &mut egui::Ui, msg: &Message) {
    let p = colors::palette(ui);
    ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
        let frame = egui::Frame::NONE
            .fill(p.user_bubble_bg)
            .corner_radius(egui::CornerRadius {
                nw: Layout::USER_BUBBLE_RADIUS as u8,
                ne: 4,
                sw: Layout::USER_BUBBLE_RADIUS as u8,
                se: Layout::USER_BUBBLE_RADIUS as u8,
            })
            .inner_margin(egui::Margin::symmetric(16, 10));

        frame.show(ui, |ui| {
            ui.set_max_width(Layout::MAX_CONTENT_WIDTH * 0.78);
            ui.with_layout(
                egui::Layout::left_to_right(egui::Align::TOP).with_main_wrap(true),
                |ui| {
                    ui.label(
                        egui::RichText::new(&msg.content)
                            .size(Layout::BODY_FONT_SIZE)
                            .color(p.text_primary),
                    );
                },
            );
        });
    });
}

fn render_assistant_message(ui: &mut egui::Ui, msg: &Message) {
    let p = colors::palette(ui);
    for card in &msg.tool_cards {
        render_tool_card(ui, card);
        ui.add_space(6.0);
    }

    if !msg.content.is_empty() {
        let mut cache = egui_commonmark::CommonMarkCache::default();
        egui_commonmark::CommonMarkViewer::new().show(ui, &mut cache, &msg.content);
    }

    if msg.status == MessageStatus::Cancelled {
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Stopped generating")
                .italics()
                .size(Layout::SMALL_FONT_SIZE)
                .color(p.text_muted),
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
                    .size(Layout::CAPTION_FONT_SIZE)
                    .color(p.text_muted),
            );
        }
    }
}

fn render_system_message(ui: &mut egui::Ui, msg: &Message) {
    let p = colors::palette(ui);
    ui.vertical_centered(|ui| {
        ui.label(
            egui::RichText::new(&msg.content)
                .size(Layout::SMALL_FONT_SIZE)
                .italics()
                .color(p.text_muted),
        );
    });
}

fn render_error_message(ui: &mut egui::Ui, msg: &Message) {
    let p = colors::palette(ui);
    Card::new()
        .border_color(p.status_red)
        .padding(egui::Margin::symmetric(16, 10))
        .show(ui, |ui| {
            ui.set_max_width(Layout::MAX_CONTENT_WIDTH);
            ui.label(
                egui::RichText::new(&msg.content)
                    .size(Layout::BODY_FONT_SIZE)
                    .color(p.status_red),
            );
        });
}

fn render_tool_card(ui: &mut egui::Ui, card: &ToolCard) {
    let p = colors::palette(ui);
    let (status_icon, status_color) = match card.phase {
        ToolPhase::Started => (icons::CIRCLE_FILLED, p.status_yellow),
        ToolPhase::Completed => (icons::CHECKMARK, p.status_green),
        ToolPhase::Failed => (icons::XMARK, p.status_red),
    };

    Card::new()
        .padding(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
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
                                .size(Layout::CAPTION_FONT_SIZE)
                                .color(p.text_secondary),
                        );
                        let val_str = match value {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        ui.label(
                            egui::RichText::new(val_str)
                                .monospace()
                                .size(Layout::CAPTION_FONT_SIZE)
                                .color(p.text_primary),
                        );
                    });
                }
                if let Some(ref result) = card.result {
                    ui.add_space(4.0);
                    Divider::new().show(ui);
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(result)
                            .monospace()
                            .size(Layout::CAPTION_FONT_SIZE)
                            .color(p.text_secondary),
                    );
                }
            });
        });
}
