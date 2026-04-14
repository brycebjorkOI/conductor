use tokio::sync::mpsc;

use conductor_core::events::Action;

use crate::bridge::SharedState;

pub fn show(
    ui: &mut egui::Ui,
    shared: &SharedState,
    tx: &mpsc::UnboundedSender<Action>,
) {
    ui.heading("General Settings");
    ui.add_space(12.0);

    let mut config = shared.read().config.clone();
    let mut changed = false;

    // -- Connection Mode --
    ui.group(|ui| {
        ui.label(egui::RichText::new("Connection Mode").strong());
        ui.add_space(4.0);

        let modes = ["standalone", "local_server", "remote_server"];
        let labels = ["Standalone", "Local Server", "Remote Server"];
        for (mode, label) in modes.iter().zip(labels.iter()) {
            if ui
                .radio_value(&mut config.general.connection_mode, mode.to_string(), *label)
                .changed()
            {
                changed = true;
            }
        }
    });

    ui.add_space(12.0);

    // -- Behavior --
    ui.group(|ui| {
        ui.label(egui::RichText::new("Behavior").strong());
        ui.add_space(4.0);

        if ui
            .checkbox(&mut config.general.auto_hide_panel, "Auto-hide panel on focus loss")
            .changed()
        {
            changed = true;
        }

        if ui
            .checkbox(&mut config.general.launch_at_login, "Launch at login")
            .changed()
        {
            changed = true;
        }

        if ui
            .checkbox(&mut config.general.check_updates, "Check for updates")
            .changed()
        {
            changed = true;
        }
    });

    ui.add_space(12.0);

    // -- Logging --
    ui.group(|ui| {
        ui.label(egui::RichText::new("Logging").strong());
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("Log level:");
            let levels = ["trace", "debug", "info", "warn", "error"];
            egui::ComboBox::from_id_salt("log_level")
                .selected_text(&config.logging.level)
                .show_ui(ui, |ui| {
                    for level in levels {
                        if ui
                            .selectable_value(
                                &mut config.logging.level,
                                level.to_string(),
                                level,
                            )
                            .changed()
                        {
                            changed = true;
                        }
                    }
                });
        });
    });

    ui.add_space(16.0);

    if changed {
        shared.mutate(|s| s.config = config);
        let _ = tx.send(Action::SaveConfig);
    }
}
