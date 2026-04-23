use conductor_core::state::*;
use egui_swift::prelude::*;

pub struct MessageListView {
    pub session: Option<Session>,
}

impl MessageListView {
    pub fn new() -> Self {
        Self { session: None }
    }
}

impl View for MessageListView {
    fn body(&mut self, ui: &mut egui::Ui) {
        let Some(ref session) = self.session else {
            return;
        };

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

                    egui_swift::spacer!(ui, 24.0);

                    for msg in &session.messages {
                        render_message(ui, msg);
                        egui_swift::spacer!(ui, Layout::MESSAGE_SPACING);
                    }

                    if is_streaming {
                        let p = ui.palette();
                        let time = ui.input(|i| i.time);
                        let visible = (time * 2.5) as u64 % 2 == 0;
                        if visible {
                            Label::new("\u{2588}")
                                .font(Font::Body)
                                .color(p.accent)
                                .show(ui);
                        } else {
                            egui_swift::spacer!(ui, Layout::BODY_FONT_SIZE + 4.0);
                        }
                        ui.ctx().request_repaint();
                    }

                    egui_swift::spacer!(ui, 24.0);
                });
            });
    }
}

fn render_empty_state(ui: &mut egui::Ui) {
    let available_height = ui.available_height();
    let top_space = (available_height * 0.30).max(60.0);
    egui_swift::spacer!(ui, top_space);

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

        egui_swift::spacer!(ui, 8.0);

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
    // -- Thinking section (shown above tool cards and response text) --
    if let Some(ref thinking) = msg.thinking_content {
        if !thinking.is_empty() {
            render_thinking(ui, &msg.id, thinking, msg.status == MessageStatus::Streaming);
            egui_swift::spacer!(ui, 8.0);
        }
    } else if msg.status == MessageStatus::Streaming && msg.content.is_empty() && msg.tool_cards.is_empty() {
        egui_swift::hstack!(ui, {
            ProgressView::spinner().show(ui);
            Label::new("Thinking...")
                .font(Font::Subheadline)
                .italic(true)
                .muted()
                .show(ui);
        });
        egui_swift::spacer!(ui, 4.0);
    }

    // -- Tool cards --
    for (idx, card) in msg.tool_cards.iter().enumerate() {
        render_tool_card(ui, card, &msg.id, idx);
        egui_swift::spacer!(ui, 6.0);
    }

    // -- Response text (Markdown) --
    if !msg.content.is_empty() {
        let mut cache = egui_commonmark::CommonMarkCache::default();
        egui_commonmark::CommonMarkViewer::new().show(ui, &mut cache, &msg.content);
    }

    if msg.status == MessageStatus::Cancelled {
        egui_swift::spacer!(ui, 4.0);
        Label::new("Stopped generating")
            .font(Font::Subheadline)
            .italic(true)
            .muted()
            .show(ui);
    }

    if let Some(ref usage) = msg.usage {
        egui_swift::spacer!(ui, 6.0);
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
    let (status_icon, _status_color) = match card.phase {
        ToolPhase::Started => (icons::CIRCLE_FILLED, p.status_yellow),
        ToolPhase::Completed => (icons::CHECKMARK, p.status_green),
        ToolPhase::Failed => (icons::XMARK, p.status_red),
    };

    let uid = format!("tc_{msg_id}_{idx}");

    Card::new()
        .padding(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            let header = format!("{status_icon}  {}", card.tool_name);
            let mut open = false;
            DisclosureGroup::new(&header, &mut open)
                .icon(status_icon)
                .show(ui, |ui| {
                    let _ = &uid; // keep uid alive for identity
                    for (key, value) in &card.arguments {
                        egui_swift::hstack!(ui, {
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
                        egui_swift::spacer!(ui, 4.0);
                        Divider::new().show(ui);
                        egui_swift::spacer!(ui, 4.0);
                        Label::new(result)
                            .font(Font::Caption)
                            .monospace(true)
                            .secondary()
                            .show(ui);
                    }
                });
        });
}

/// Render the thinking/reasoning section.
fn render_thinking(ui: &mut egui::Ui, msg_id: &str, thinking: &str, is_streaming: bool) {
    let p = ui.palette();

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

    let header_text = format!("\u{1f4ad} {summary}");
    let mut open = false;

    DisclosureGroup::new(&header_text, &mut open).show(ui, |ui| {
        let _ = msg_id; // keep alive for identity
        let lines: Vec<&str> = thinking.lines().filter(|l| !l.trim().is_empty()).collect();

        for (i, line) in lines.iter().enumerate() {
            let is_last = i == lines.len() - 1;
            egui_swift::hstack!(ui, {
                if is_last && !is_streaming {
                    Label::new(icons::CHECKMARK)
                        .font(Font::Caption)
                        .color(p.status_green)
                        .show(ui);
                } else if is_last && is_streaming {
                    ProgressView::spinner().show(ui);
                } else {
                    Label::new(icons::CIRCLE_FILLED)
                        .font(Font::Footnote)
                        .color(p.text_muted)
                        .show(ui);
                }

                Label::new(*line)
                    .font(Font::Subheadline)
                    .color(p.text_secondary)
                    .show(ui);
            });

            // Connector line between steps.
            if !is_last {
                egui_swift::hstack!(ui, {
                    egui_swift::spacer!(ui, 7.0);
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
