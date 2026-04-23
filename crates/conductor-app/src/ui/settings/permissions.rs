use tokio::sync::mpsc;

use conductor_core::events::Action;
use egui_swift::prelude::*;

use crate::bridge::SharedState;

pub struct PermissionsView {
    shared: SharedState,
    tx: mpsc::UnboundedSender<Action>,
}

impl PermissionsView {
    pub fn new(shared: SharedState, tx: mpsc::UnboundedSender<Action>) -> Self {
        Self { shared, tx }
    }
}

impl View for PermissionsView {
    fn body(&mut self, ui: &mut egui::Ui) {
        egui_swift::text!(ui, "Permissions", .title);
        egui_swift::spacer!(ui, 16.0);

        let mut config = self.shared.read().config.clone();
        let mut changed = false;

        egui_swift::section!(ui, "Execution Approval Mode", {
            let modes: Vec<(String, &str)> = vec![
                ("deny".into(), "Deny All \u{2014} reject all tool execution"),
                ("ask".into(), "Ask \u{2014} confirm every tool invocation"),
                (
                    "allowlist".into(),
                    "Allowlist \u{2014} auto-approve matching rules, ask for others",
                ),
                (
                    "auto".into(),
                    "Full Auto \u{2014} approve all (trusted environments only)",
                ),
            ];
            RadioGroup::new(&mut config.security.execution_mode, &modes).show(ui);
            changed = true;
        });

        egui_swift::spacer!(ui, 12.0);

        egui_swift::section!(ui, "Allowlist Rules", {
            if config.security.allow_rules.is_empty() {
                egui_swift::text!(ui, "No rules configured.", .subheadline, .muted);
            } else {
                let mut to_remove = None;
                for (i, rule) in config.security.allow_rules.iter().enumerate() {
                    egui_swift::hstack!(ui, {
                        Label::new(rule)
                            .font(Font::Subheadline)
                            .monospace(true)
                            .show(ui);
                        Spacer::trailing(ui, |ui| {
                            if Button::new("Remove")
                                .style(ButtonStyle::Destructive)
                                .small(true)
                                .show(ui)
                                .clicked()
                            {
                                to_remove = Some(i);
                            }
                        });
                    });
                }
                if let Some(i) = to_remove {
                    config.security.allow_rules.remove(i);
                    changed = true;
                }
            }

            egui_swift::spacer!(ui, 4.0);
            egui_swift::text!(ui, "Format: tool_name glob_pattern (e.g. \"file_read *\", \"shell_exec cargo *\")", .caption, .muted);
        });

        if changed {
            self.shared.mutate(|s| s.config = config);
            let _ = self.tx.send(Action::SaveConfig);
        }
    }
}
