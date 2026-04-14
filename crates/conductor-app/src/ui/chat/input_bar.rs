use tokio::sync::mpsc;

use conductor_core::commands;
use conductor_core::events::Action;
use conductor_core::state::SessionId;
use egui_swift::card::Card;
use egui_swift::chat_input::ChatInput;
use egui_swift::theme::Layout;

pub fn show(
    ui: &mut egui::Ui,
    input_text: &mut String,
    show_autocomplete: &mut bool,
    is_streaming: bool,
    active_session_id: &SessionId,
    tx: &mpsc::UnboundedSender<Action>,
) {
    // Autocomplete popup (rendered above the input).
    if *show_autocomplete && input_text.starts_with('/') {
        let prefix = &input_text[1..];
        let suggestions = commands::autocomplete(prefix);
        if !suggestions.is_empty() {
            let available_width = ui.available_width();
            let content_width = available_width.min(Layout::MAX_CONTENT_WIDTH);
            let side = ((available_width - content_width) / 2.0).max(20.0);

            ui.horizontal(|ui| {
                ui.add_space(side);
                Card::new()
                    .padding(egui::Margin::symmetric(8, 4))
                    .show(ui, |ui| {
                        for (cmd, desc) in suggestions {
                            let label = format!("/{cmd}  \u{2014}  {desc}");
                            if ui
                                .selectable_label(
                                    false,
                                    egui::RichText::new(label).size(13.0),
                                )
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

    // Use the reusable ChatInput component.
    let resp = ChatInput::new(input_text)
        .placeholder("Type / for commands")
        .streaming(is_streaming)
        .max_width(Layout::MAX_CONTENT_WIDTH)
        .show(ui);

    if resp.text_response.changed() {
        *show_autocomplete = input_text.starts_with('/');
    }

    if resp.submitted {
        let text = std::mem::take(input_text);
        let text = text.trim_end_matches('\n').to_string();
        if !text.is_empty() {
            let _ = tx.send(Action::SendMessage {
                session_id: active_session_id.clone(),
                text,
                attachments: Vec::new(),
            });
        }
    }

    if resp.stopped {
        let _ = tx.send(Action::AbortGeneration {
            session_id: active_session_id.clone(),
        });
    }

    ui.add_space(8.0);
}
