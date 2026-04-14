use tokio::sync::mpsc;

use conductor_core::events::Action;
use egui_swift::colors;
use egui_swift::form_section::FormSection;
use egui_swift::radio_group::RadioGroup;
use egui_swift::toggle::Toggle;

use crate::bridge::SharedState;

pub fn show(
    ui: &mut egui::Ui,
    shared: &SharedState,
    tx: &mpsc::UnboundedSender<Action>,
) {
    let p = colors::palette(ui);

    ui.label(
        egui::RichText::new("General")
            .size(22.0)
            .strong()
            .color(p.text_primary),
    );
    ui.add_space(16.0);

    let mut config = shared.read().config.clone();
    let mut changed = false;

    // -- Connection Mode --
    FormSection::new().header("Connection Mode").show(ui, |ui| {
        let modes: Vec<(String, &str)> = vec![
            ("standalone".into(), "Standalone"),
            ("local_server".into(), "Local Server"),
            ("remote_server".into(), "Remote Server"),
        ];
        RadioGroup::new(&mut config.general.connection_mode, &modes).show(ui);
        changed = true; // RadioGroup mutates in place; we always sync
    });

    ui.add_space(12.0);

    // -- Behavior --
    FormSection::new().header("Behavior").show(ui, |ui| {
        if Toggle::new(&mut config.general.auto_hide_panel)
            .label("Auto-hide panel on focus loss")
            .show(ui)
            .clicked()
        {
            changed = true;
        }
        ui.add_space(4.0);
        if Toggle::new(&mut config.general.launch_at_login)
            .label("Launch at login")
            .show(ui)
            .clicked()
        {
            changed = true;
        }
        ui.add_space(4.0);
        if Toggle::new(&mut config.general.check_updates)
            .label("Check for updates")
            .show(ui)
            .clicked()
        {
            changed = true;
        }
    });

    ui.add_space(12.0);

    // -- Logging --
    FormSection::new().header("Logging").show(ui, |ui| {
        let levels: Vec<(String, &str)> = vec![
            ("trace".into(), "Trace"),
            ("debug".into(), "Debug"),
            ("info".into(), "Info"),
            ("warn".into(), "Warn"),
            ("error".into(), "Error"),
        ];
        egui_swift::picker::Picker::new("Log level", &mut config.logging.level, &levels)
            .show(ui);
        changed = true;
    });

    ui.add_space(16.0);

    if changed {
        shared.mutate(|s| s.config = config);
        let _ = tx.send(Action::SaveConfig);
    }
}
