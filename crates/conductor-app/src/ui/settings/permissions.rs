use crate::bridge::SharedState;

pub fn show(ui: &mut egui::Ui, shared: &SharedState) {
    let p = egui_swift::colors::palette(ui);

    ui.heading("Permissions");
    ui.add_space(12.0);

    let mut config = shared.read().config.clone();
    let mut changed = false;

    // -- Execution Approval Mode --
    ui.group(|ui| {
        ui.label(egui::RichText::new("Execution Approval Mode").strong());
        ui.add_space(4.0);

        let modes = ["deny", "ask", "allowlist", "auto"];
        let labels = [
            "Deny All — reject all tool execution",
            "Ask — confirm every tool invocation",
            "Allowlist — auto-approve matching rules, ask for others",
            "Full Auto — approve all (trusted environments only)",
        ];

        for (mode, label) in modes.iter().zip(labels.iter()) {
            if ui
                .radio_value(
                    &mut config.security.execution_mode,
                    mode.to_string(),
                    *label,
                )
                .changed()
            {
                changed = true;
            }
        }
    });

    ui.add_space(12.0);

    // -- Allowlist Rules --
    ui.group(|ui| {
        ui.label(egui::RichText::new("Allowlist Rules").strong());
        ui.add_space(4.0);

        if config.security.allow_rules.is_empty() {
            ui.label(
                egui::RichText::new("No rules configured.")
                    .color(p.text_muted),
            );
        } else {
            let mut to_remove = None;
            for (i, rule) in config.security.allow_rules.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(rule).monospace().size(12.0),
                    );
                    if ui.small_button("Remove").clicked() {
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
            egui::RichText::new("Format: tool_name glob_pattern (e.g. \"file_read *\", \"shell_exec cargo *\")")
                .size(11.0)
                .color(p.text_muted),
        );
    });

    if changed {
        shared.mutate(|s| s.config = config);
    }
}
