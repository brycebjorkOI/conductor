//! Large rounded chat input with placeholder, action icons (attach, mic),
//! and a send button.
//!
//! ```no_run
//! # let ui: &mut egui::Ui = todo!();
//! let mut text = String::new();
//! let resp = conductor_ui::chat_input::ChatInput::new(&mut text).show(ui);
//! if resp.submitted {
//!     // handle send
//! }
//! ```

use crate::colors;

/// Outcome of showing the chat input.
pub struct ChatInputResponse {
    /// The user pressed Enter or clicked Send.
    pub submitted: bool,
    /// The user clicked the Stop button (only while streaming).
    pub stopped: bool,
    /// The text field response.
    pub text_response: egui::Response,
}

pub struct ChatInput<'a> {
    text: &'a mut String,
    placeholder: &'a str,
    is_streaming: bool,
    max_width: f32,
}

impl<'a> ChatInput<'a> {
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            placeholder: "Type / for commands",
            is_streaming: false,
            max_width: 640.0,
        }
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn streaming(mut self, is_streaming: bool) -> Self {
        self.is_streaming = is_streaming;
        self
    }

    pub fn max_width(mut self, w: f32) -> Self {
        self.max_width = w;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> ChatInputResponse {
        let p = colors::palette(ui);
        let rounding = 20u8;
        let mut submitted = false;
        let mut stopped = false;

        // Center the input.
        let available = ui.available_width();
        let width = available.min(self.max_width);
        let side = ((available - width) / 2.0).max(0.0);

        let mut text_response_out: Option<egui::Response> = None;

        ui.horizontal(|ui| {
            ui.add_space(side);

            let frame = egui::Frame::NONE
                .fill(p.input_bg)
                .corner_radius(egui::CornerRadius::same(rounding))
                .stroke(egui::Stroke::new(1.0, p.border))
                .inner_margin(egui::Margin {
                    left: 16,
                    right: 10,
                    top: 12,
                    bottom: 12,
                });

            frame.show(ui, |ui| {
                ui.set_width(width - 4.0);

                // Top row: text input.
                let text_resp = ui.add_sized(
                    egui::vec2(ui.available_width(), 32.0),
                    egui::TextEdit::multiline(self.text)
                        .hint_text(
                            egui::RichText::new(self.placeholder)
                                .size(14.0)
                                .color(p.text_placeholder),
                        )
                        .desired_rows(1)
                        .frame(false)
                        .text_color(p.text_primary)
                        .margin(egui::Margin::ZERO)
                        .return_key(Some(egui::KeyboardShortcut::new(
                            egui::Modifiers::NONE,
                            egui::Key::Enter,
                        ))),
                );

                // Bottom row: action icons left, send button right.
                ui.horizontal(|ui| {
                    // Attach button.
                    ui.add(
                        egui::Button::new(
                            egui::RichText::new("+")
                                .size(16.0)
                                .color(p.text_muted),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .frame(false),
                    );

                    // Mic button.
                    ui.add(
                        egui::Button::new(
                            egui::RichText::new("\u{1f399}")
                                .size(14.0)
                                .color(p.text_muted),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .frame(false),
                    );

                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            if self.is_streaming {
                                // Stop button.
                                let btn = egui::Button::new(
                                    egui::RichText::new("\u{25a0}")
                                        .size(12.0)
                                        .color(p.text_on_accent),
                                )
                                .fill(p.status_red)
                                .corner_radius(egui::CornerRadius::same(14));

                                if ui.add_sized(egui::vec2(28.0, 28.0), btn).clicked()
                                {
                                    stopped = true;
                                }
                            } else {
                                // Send button.
                                let has_text = !self.text.trim().is_empty();
                                let fill = if has_text {
                                    p.text_primary
                                } else {
                                    p.border
                                };

                                let btn = egui::Button::new(
                                    egui::RichText::new("\u{2191}")
                                        .size(14.0)
                                        .strong()
                                        .color(if has_text {
                                            p.surface
                                        } else {
                                            p.text_muted
                                        }),
                                )
                                .fill(fill)
                                .corner_radius(egui::CornerRadius::same(14));

                                if ui
                                    .add_sized(egui::vec2(28.0, 28.0), btn)
                                    .clicked()
                                    && has_text
                                {
                                    submitted = true;
                                }
                            }
                        },
                    );
                });

                text_response_out = Some(text_resp);
            });

            ui.add_space(side);
        });

        // Enter key detection.
        let enter = ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift);
        if enter && !self.is_streaming && !self.text.trim().is_empty() {
            submitted = true;
        }

        // Escape -> stop.
        let esc = ui.input(|i| i.key_pressed(egui::Key::Escape));
        if esc && self.is_streaming {
            stopped = true;
        }

        ChatInputResponse {
            submitted,
            stopped,
            text_response: text_response_out
                .unwrap_or_else(|| ui.label("")),
        }
    }
}
