use tokio::sync::mpsc;

use conductor_core::events::Action;
use egui_swift::prelude::*;

use crate::bridge::SharedState;

pub struct GeneralView {
    shared: SharedState,
    tx: mpsc::UnboundedSender<Action>,
}

impl GeneralView {
    pub fn new(shared: SharedState, tx: mpsc::UnboundedSender<Action>) -> Self {
        Self { shared, tx }
    }
}

impl View for GeneralView {
    fn body(&mut self, ui: &mut egui::Ui) {
        egui_swift::text!(ui, "General", .title);
        egui_swift::spacer!(ui, 16.0);

        let mut config = self.shared.read().config.clone();
        let mut changed = false;

        egui_swift::section!(ui, "Connection Mode", {
            let modes: Vec<(String, &str)> = vec![
                ("standalone".into(), "Standalone"),
                ("local_server".into(), "Local Server"),
                ("remote_server".into(), "Remote Server"),
            ];
            RadioGroup::new(&mut config.general.connection_mode, &modes).show(ui);
            changed = true;
        });

        egui_swift::spacer!(ui, 12.0);

        egui_swift::section!(ui, "Behavior", {
            if Toggle::new(&mut config.general.auto_hide_panel)
                .label("Auto-hide panel on focus loss")
                .show(ui)
                .clicked()
            {
                changed = true;
            }
            egui_swift::spacer!(ui, 4.0);
            if Toggle::new(&mut config.general.launch_at_login)
                .label("Launch at login")
                .show(ui)
                .clicked()
            {
                changed = true;
            }
            egui_swift::spacer!(ui, 4.0);
            if Toggle::new(&mut config.general.check_updates)
                .label("Check for updates")
                .show(ui)
                .clicked()
            {
                changed = true;
            }
        });

        egui_swift::spacer!(ui, 12.0);

        egui_swift::section!(ui, "Logging", {
            let levels: Vec<(String, &str)> = vec![
                ("trace".into(), "Trace"),
                ("debug".into(), "Debug"),
                ("info".into(), "Info"),
                ("warn".into(), "Warn"),
                ("error".into(), "Error"),
            ];
            Picker::new("Log level", &mut config.logging.level, &levels).show(ui);
            changed = true;
        });

        if changed {
            self.shared.mutate(|s| s.config = config);
            let _ = self.tx.send(Action::SaveConfig);
        }
    }
}
