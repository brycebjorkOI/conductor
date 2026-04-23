use tokio::sync::mpsc;

use conductor_core::commands;
use conductor_core::events::Action;
use conductor_core::state::SessionId;
use egui_swift::prelude::*;

/// Chat input bar with autocomplete and send/stop functionality.
pub struct InputBarView {
    tx: mpsc::UnboundedSender<Action>,
    pub input_text: String,
    pub show_autocomplete: bool,
    pub is_streaming: bool,
    pub active_session_id: SessionId,
}

impl InputBarView {
    pub fn new(tx: mpsc::UnboundedSender<Action>) -> Self {
        Self {
            tx,
            input_text: String::new(),
            show_autocomplete: false,
            is_streaming: false,
            active_session_id: String::new(),
        }
    }
}

impl View for InputBarView {
    fn body(&mut self, ui: &mut egui::Ui) {
        // Autocomplete popup.
        if self.show_autocomplete && self.input_text.starts_with('/') {
            let prefix = &self.input_text[1..];
            let suggestions = commands::autocomplete(prefix);
            if !suggestions.is_empty() {
                let available_width = ui.available_width();
                let content_width = available_width.min(Layout::MAX_CONTENT_WIDTH);
                let side = ((available_width - content_width) / 2.0).max(20.0);

                egui_swift::hstack!(ui, {
                    egui_swift::spacer!(ui, side);
                    Card::new()
                        .padding(egui::Margin::symmetric(8, 4))
                        .show(ui, |ui| {
                            for (cmd, desc) in suggestions {
                                let label = format!("/{cmd}  \u{2014}  {desc}");
                                if ui
                                    .selectable_label(
                                        false,
                                        egui::RichText::new(label).size(Font::Callout.size()),
                                    )
                                    .clicked()
                                {
                                    self.input_text = format!("/{cmd} ");
                                    self.show_autocomplete = false;
                                }
                            }
                        });
                });
                egui_swift::spacer!(ui, 4.0);
            }
        }

        egui_swift::spacer!(ui, 4.0);

        let resp = ChatInput::new(&mut self.input_text)
            .placeholder("Type / for commands")
            .streaming(self.is_streaming)
            .max_width(Layout::MAX_CONTENT_WIDTH)
            .show(ui);

        if resp.text_response.changed() {
            self.show_autocomplete = self.input_text.starts_with('/');
        }

        if resp.submitted {
            let text = std::mem::take(&mut self.input_text);
            let text = text.trim_end_matches('\n').to_string();
            if !text.is_empty() {
                let _ = self.tx.send(Action::SendMessage {
                    session_id: self.active_session_id.clone(),
                    text,
                    attachments: Vec::new(),
                });
            }
        }

        if resp.stopped {
            let _ = self.tx.send(Action::AbortGeneration {
                session_id: self.active_session_id.clone(),
            });
        }

        egui_swift::spacer!(ui, 8.0);
    }
}
