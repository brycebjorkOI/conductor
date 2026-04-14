use conductor_core::state::*;
use egui_swift::prelude::*;

pub fn show(ui: &mut egui::Ui, session: &Session) {
    let is_streaming = session
        .streaming
        .as_ref()
        .map_or(false, |s| s.is_active);

    ScrollView::vertical()
        .stick_to_bottom(is_streaming || !session.messages.is_empty())
        .show(ui, |ui| {
            ui.centered_content(Layout::MAX_CONTENT_WIDTH, |ui| {
                if session.messages.is_empty() {
                    render_empty_state(ui);
                    return;
                }

                Spacer::fixed(24.0).show(ui);

                for msg in &session.messages {
                    render_message(ui, msg);
                    Spacer::fixed(Layout::MESSAGE_SPACING).show(ui);
                }

                if is_streaming {
                    let p = ui.palette();
                    let time = ui.input(|i| i.time);
                    let visible = (time * 2.5) as u64 % 2 == 0;
                    if visible {
                        ui.label(
                            egui::RichText::new("\u{2588}")
                                .size(Layout::BODY_FONT_SIZE)
                                .color(p.accent),
                        );
                    } else {
                        Spacer::fixed(Layout::BODY_FONT_SIZE + 4.0).show(ui);
                    }
                    ui.ctx().request_repaint();
                }

                Spacer::fixed(24.0).show(ui);
            });
        });
}

fn render_empty_state(ui: &mut egui::Ui) {
    let available_height = ui.available_height();
    let top_space = (available_height * 0.30).max(60.0);
    Spacer::fixed(top_space).show(ui);

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

        Label::new(&format!("{}  {greeting}", icons::SPARKLE))
            .font(Font::LargeTitle)
            .show(ui);

        Spacer::fixed(8.0).show(ui);

        Label::new("How can I help you today?")
            .font(Font::Body)
            .muted()
            .show(ui);
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
    let p = ui.palette();
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
                    Label::new(&msg.content).font(Font::Body).show(ui);
                },
            );
        });
    });
}

fn render_assistant_message(ui: &mut egui::Ui, msg: &Message) {
    let _p = ui.palette();

    // -- Thinking section (shown above tool cards and response text) --
    if let Some(ref thinking) = msg.thinking_content {
        if !thinking.is_empty() {
            render_thinking(ui, &msg.id, thinking, msg.status == MessageStatus::Streaming);
            Spacer::fixed(8.0).show(ui);
        }
    } else if msg.status == MessageStatus::Streaming && msg.content.is_empty() && msg.tool_cards.is_empty() {
        // Streaming but no content yet — show a subtle "thinking" indicator.
        HStack::new().show(ui, |ui| {
            ui.spinner();
            Label::new("Thinking...")
                .font(Font::Subheadline)
                .italic(true)
                .muted()
                .show(ui);
        });
        Spacer::fixed(4.0).show(ui);
    }

    // -- Tool cards --
    for (idx, card) in msg.tool_cards.iter().enumerate() {
        render_tool_card(ui, card, &msg.id, idx);
        Spacer::fixed(6.0).show(ui);
    }

    // -- Response text (Markdown) --
    if !msg.content.is_empty() {
        let mut cache = egui_commonmark::CommonMarkCache::default();
        egui_commonmark::CommonMarkViewer::new().show(ui, &mut cache, &msg.content);
    }

    if msg.status == MessageStatus::Cancelled {
        Spacer::fixed(4.0).show(ui);
        Label::new("Stopped generating")
            .font(Font::Subheadline)
            .italic(true)
            .muted()
            .show(ui);
    }

    if let Some(ref usage) = msg.usage {
        Spacer::fixed(6.0).show(ui);
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
            Label::new(&parts.join(" \u{00b7} "))
                .font(Font::Caption)
                .muted()
                .show(ui);
        }
    }
}

fn render_system_message(ui: &mut egui::Ui, msg: &Message) {
    ui.vertical_centered(|ui| {
        Label::new(&msg.content)
            .font(Font::Subheadline)
            .italic(true)
            .muted()
            .show(ui);
    });
}

fn render_error_message(ui: &mut egui::Ui, msg: &Message) {
    let p = ui.palette();
    Card::new()
        .border_color(p.status_red)
        .padding(egui::Margin::symmetric(16, 10))
        .show(ui, |ui| {
            ui.set_max_width(Layout::MAX_CONTENT_WIDTH);
            Label::new(&msg.content)
                .font(Font::Body)
                .destructive()
                .show(ui);
        });
}

fn render_tool_card(ui: &mut egui::Ui, card: &ToolCard, msg_id: &str, idx: usize) {
    let p = ui.palette();
    let (status_icon, status_color) = match card.phase {
        ToolPhase::Started => (icons::CIRCLE_FILLED, p.status_yellow),
        ToolPhase::Completed => (icons::CHECKMARK, p.status_green),
        ToolPhase::Failed => (icons::XMARK, p.status_red),
    };

    // Unique ID: message UUID + sequential index — guaranteed no collisions.
    let uid = format!("tc_{msg_id}_{idx}");

    Card::new()
        .padding(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            egui::CollapsingHeader::new(
                egui::RichText::new(format!("{status_icon}  {}", card.tool_name))
                    .size(Font::Subheadline.size())
                    .monospace()
                    .color(status_color),
            )
            .id_salt(&uid)
            .default_open(false)
            .show(ui, |ui| {
                for (key, value) in &card.arguments {
                    HStack::new().show(ui, |ui| {
                        Label::new(&format!("{key}:"))
                            .font(Font::Caption)
                            .monospace(true)
                            .secondary()
                            .show(ui);
                        let val_str = match value {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        Label::new(&val_str)
                            .font(Font::Caption)
                            .monospace(true)
                            .show(ui);
                    });
                }
                if let Some(ref result) = card.result {
                    Spacer::fixed(4.0).show(ui);
                    Divider::new().show(ui);
                    Spacer::fixed(4.0).show(ui);
                    Label::new(result)
                        .font(Font::Caption)
                        .monospace(true)
                        .secondary()
                        .show(ui);
                }
            });
        });
}

/// Render the thinking/reasoning section like Claude's "Thinking about..." UI.
fn render_thinking(ui: &mut egui::Ui, msg_id: &str, thinking: &str, is_streaming: bool) {
    let p = ui.palette();

    // Extract a summary from the first line/sentence for the collapsible header.
    let summary = {
        let first_line = thinking.lines().next().unwrap_or("Thinking...");
        let trimmed = first_line.trim();
        if trimmed.len() > 60 {
            format!("{}...", &trimmed[..57])
        } else if trimmed.is_empty() {
            "Thinking...".to_string()
        } else {
            trimmed.to_string()
        }
    };

    // Header: "Thinking about ..." with expand chevron.
    let header_text = if is_streaming {
        format!("\u{1f4ad} {summary}")  // thought balloon emoji while active
    } else {
        format!("\u{1f4ad} {summary}")
    };

    egui::CollapsingHeader::new(
        egui::RichText::new(&header_text)
            .size(Font::Subheadline.size())
            .italics()
            .color(p.text_secondary),
    )
    .id_salt(format!("thinking_{msg_id}"))
    .default_open(false)
    .show(ui, |ui| {
        // Render thinking steps with indicators.
        let lines: Vec<&str> = thinking.lines().filter(|l| !l.trim().is_empty()).collect();

        for (i, line) in lines.iter().enumerate() {
            let is_last = i == lines.len() - 1;
            HStack::new().show(ui, |ui| {
                // Step indicator.
                if is_last && !is_streaming {
                    // Final step: checkmark
                    ui.label(
                        egui::RichText::new(icons::CHECKMARK)
                            .size(Font::Caption.size())
                            .color(p.status_green),
                    );
                } else if is_last && is_streaming {
                    // Currently processing: spinner
                    ui.spinner();
                } else {
                    // Completed step: circle
                    ui.label(
                        egui::RichText::new(icons::CIRCLE_FILLED)
                            .size(8.0)
                            .color(p.text_muted),
                    );
                }

                ui.label(
                    egui::RichText::new(*line)
                        .size(Font::Subheadline.size())
                        .color(p.text_secondary),
                );
            });

            // Connector line between steps (except after the last one).
            if !is_last {
                HStack::new().show(ui, |ui| {
                    Spacer::fixed(7.0).show(ui); // align with the icon center
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(1.0, 12.0),
                        egui::Sense::hover(),
                    );
                    ui.painter().rect_filled(rect, 0.0, p.border_subtle);
                });
            }
        }
    });
}
