use tokio::sync::mpsc;

use conductor_core::events::Action;
use egui_swift::button::{Button, ButtonStyle};
use egui_swift::colors;
use egui_swift::form_section::FormSection;
use egui_swift::radio_group::RadioGroup;

use crate::bridge::SharedState;

pub fn show(
    ui: &mut egui::Ui,
    shared: &SharedState,
    tx: &mpsc::UnboundedSender<Action>,
) {
    let p = colors::palette(ui);

    ui.label(
        egui::RichText::new("Permissions")
            .size(22.0)
            .strong()
            .color(p.text_primary),
    );
    ui.add_space(16.0);

    let mut config = shared.read().config.clone();
    let mut changed = false;

    // -- Execution Approval Mode --
    FormSection::new()
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

    ui.add_space(12.0);

    // -- Allowlist Rules --
    FormSection::new().header("Allowlist Rules").show(ui, |ui| {
        if config.security.allow_rules.is_empty() {
            ui.label(
                egui::RichText::new("No rules configured.")
                    .size(12.0)
                    .color(p.text_muted),
            );
        } else {
            let mut to_remove = None;
            for (i, rule) in config.security.allow_rules.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(rule).monospace().size(12.0),
                    );
                    if Button::new("Remove")
                        .style(ButtonStyle::Destructive)
                        .small(true)
                        .show(ui)
                        .clicked()
                    {
                        to_remove = Some(i);
                    }
                });
            }
            if let Some(i) = to_remove {
                config.security.allow_rules.remove(i);
                changed = true;
            }
        }

        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(
                "Format: tool_name glob_pattern (e.g. \"file_read *\", \"shell_exec cargo *\")",
            )
            .size(11.0)
            .color(p.text_muted),
        );
    });

    if changed {
        shared.mutate(|s| s.config = config);
        let _ = tx.send(Action::SaveConfig);
    }
}
