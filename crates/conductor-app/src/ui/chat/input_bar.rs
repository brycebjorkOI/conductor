use tokio::sync::mpsc;

use conductor_core::commands;
use conductor_core::events::Action;
use conductor_core::state::SessionId;

use crate::theme::Theme;

pub fn show(
    ui: &mut egui::Ui,
    input_text: &mut String,
    show_autocomplete: &mut bool,
    is_streaming: bool,
    active_session_id: &SessionId,
    tx: &mpsc::UnboundedSender<Action>,
) {
    let dark = ui.visuals().dark_mode;

    let available_width = ui.available_width();
    let content_width = available_width.min(Theme::MAX_CONTENT_WIDTH);
    let side_padding = ((available_width - content_width) / 2.0).max(20.0);

    // Autocomplete popup.
    if *show_autocomplete && input_text.starts_with('/') {
        let prefix = &input_text[1..];
        let suggestions = commands::autocomplete(prefix);
        if !suggestions.is_empty() {
            ui.horizontal(|ui| {
                ui.add_space(side_padding);
                egui::Frame::NONE
                    .fill(Theme::input_bg(dark))
                    .corner_radius(egui::CornerRadius::same(10))
                    .stroke(egui::Stroke::new(1.0, Theme::input_border(dark)))
                    .inner_margin(egui::Margin::symmetric(8, 4))
                    .show(ui, |ui| {
                        for (cmd, desc) in suggestions {
                            let label = format!("/{cmd}  \u{2014}  {desc}");
                            if ui
                                .selectable_label(false, egui::RichText::new(label).size(13.0))
                                .clicked()
                            {
                                *input_text = format!("/{cmd} ");
                                *show_autocomplete = false;
                            }
                        }
                    });
            });
            ui.add_space(4.0);
        }
    }

    ui.add_space(4.0);

    // Input container — centered, rounded.
    ui.horizontal(|ui| {
        ui.add_space(side_padding);

        let input_frame = egui::Frame::NONE
            .fill(Theme::input_bg(dark))
            .corner_radius(egui::CornerRadius::same(Theme::INPUT_RADIUS as u8))
            .stroke(egui::Stroke::new(1.0, Theme::input_border(dark)))
            .inner_margin(egui::Margin {
                left: 18,
                right: 10,
                top: 12,
                bottom: 12,
            });

        input_frame.show(ui, |ui| {
            ui.set_width(content_width - 4.0);

            ui.horizontal(|ui| {
                let input_width = ui.available_width() - 40.0;

                let response = ui.add_sized(
                    egui::vec2(input_width, 20.0),
                    egui::TextEdit::multiline(input_text)
                        .hint_text(
                            egui::RichText::new("How can I help you today?")
                                .size(Theme::BODY_FONT_SIZE)
                                .color(Theme::text_muted(dark)),
                        )
                        .desired_rows(1)
                        .frame(false)
                        .text_color(Theme::text_primary(dark))
                        .margin(egui::Margin::ZERO)
                        .return_key(Some(egui::KeyboardShortcut::new(
                            egui::Modifiers::NONE,
                            egui::Key::Enter,
                        ))),
                );

                if response.changed() {
                    *show_autocomplete = input_text.starts_with('/');
                }

                let enter_pressed = ui.input(|i| {
                    i.key_pressed(egui::Key::Enter) && !i.modifiers.shift
                });

                let escape_pressed = ui.input(|i| i.key_pressed(egui::Key::Escape));
                if escape_pressed && is_streaming {
                    let _ = tx.send(Action::AbortGeneration {
                        session_id: active_session_id.clone(),
                    });
                }

                // Send/Stop button — small circle inside the input.
                if is_streaming {
                    let stop_btn = egui::Button::new(
                        egui::RichText::new("\u{25a0}")
                            .size(12.0)
                            .color(egui::Color32::WHITE),
                    )
                    .fill(Theme::text_primary(dark))
                    .corner_radius(egui::CornerRadius::same(14));

                    if ui.add_sized(egui::vec2(28.0, 28.0), stop_btn).clicked() {
                        let _ = tx.send(Action::AbortGeneration {
                            session_id: active_session_id.clone(),
                        });
                    }
                } else {
                    let has_text = !input_text.trim().is_empty();
                    let fill = if has_text {
                        Theme::text_primary(dark)
                    } else {
                        Theme::text_muted(dark)
                    };

                    let send_btn = egui::Button::new(
                        egui::RichText::new("\u{2191}")  // up arrow
                            .size(14.0)
                            .strong()
                            .color(if has_text {
                                Theme::surface(dark)
                            } else {
                                Theme::surface(dark)
                            }),
                    )
                    .fill(fill)
                    .corner_radius(egui::CornerRadius::same(14));

                    if ui.add_sized(egui::vec2(28.0, 28.0), send_btn).clicked() && has_text {
                        send_message(input_text, active_session_id, tx);
                    }
                }

                if enter_pressed && !is_streaming && !input_text.trim().is_empty() {
                    send_message(input_text, active_session_id, tx);
                }
            });
        });

        ui.add_space(side_padding);
    });

    ui.add_space(10.0);
}

fn send_message(
    input_text: &mut String,
    active_session_id: &SessionId,
    tx: &mpsc::UnboundedSender<Action>,
) {
    let text = std::mem::take(input_text);
    let text = text.trim_end_matches('\n').to_string();
    if text.is_empty() {
        return;
    }
    let _ = tx.send(Action::SendMessage {
        session_id: active_session_id.clone(),
        text,
        attachments: Vec::new(),
    });
}
