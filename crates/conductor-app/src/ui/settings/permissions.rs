use tokio::sync::mpsc;

use conductor_core::events::Action;
use egui_swift::prelude::*;

use crate::bridge::SharedState;

pub fn show(
    ui: &mut egui::Ui,
    shared: &SharedState,
    tx: &mpsc::UnboundedSender<Action>,
) {
    Label::heading("Permissions").show(ui);
    Spacer::fixed(16.0).show(ui);

    let mut config = shared.read().config.clone();
    let mut changed = false;

    Section::new()
        .header("Execution Approval Mode")
        .show(ui, |ui| {
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

    Spacer::fixed(12.0).show(ui);

    Section::new().header("Allowlist Rules").show(ui, |ui| {
        if config.security.allow_rules.is_empty() {
            Label::new("No rules configured.")
                .font(Font::Subheadline)
                .muted()
                .show(ui);
        } else {
            let mut to_remove = None;
            for (i, rule) in config.security.allow_rules.iter().enumerate() {
                HStack::new().show(ui, |ui| {
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

        Spacer::fixed(4.0).show(ui);
        Label::new(
            "Format: tool_name glob_pattern (e.g. \"file_read *\", \"shell_exec cargo *\")",
        )
        .font(Font::Caption)
        .muted()
        .show(ui);
    });

    if changed {
        shared.mutate(|s| s.config = config);
        let _ = tx.send(Action::SaveConfig);
    }
}
