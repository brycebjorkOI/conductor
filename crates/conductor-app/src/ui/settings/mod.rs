mod about;
mod backends;
mod debug;
mod general;

use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::state::SettingsTab;

use crate::bridge::SharedState;

pub fn show(
    ctx: &egui::Context,
    shared: &SharedState,
    tx: &mpsc::UnboundedSender<Action>,
) {
    let current_tab = shared.read().settings_tab;

    egui::SidePanel::left("settings_tabs")
        .resizable(false)
        .default_width(130.0)
        .show(ctx, |ui| {
            ui.heading("Settings");
            ui.separator();

            let tabs = [
                (SettingsTab::About, "About"),
                (SettingsTab::General, "General"),
                (SettingsTab::Backends, "Backends"),
                (SettingsTab::Channels, "Channels"),
                (SettingsTab::Schedules, "Schedules"),
                (SettingsTab::Sessions, "Sessions"),
                (SettingsTab::Plugins, "Plugins"),
                (SettingsTab::Skills, "Skills"),
                (SettingsTab::McpServers, "MCP Servers"),
                (SettingsTab::Permissions, "Permissions"),
                (SettingsTab::Debug, "Debug"),
            ];

            for (tab, label) in tabs {
                let selected = current_tab == tab;
                if ui.selectable_label(selected, label).clicked() {
                    shared.mutate(|s| s.settings_tab = tab);
                }
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                if ui.button("Close Settings").clicked() {
                    let _ = tx.send(Action::CloseSettings);
                }
            });
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        match current_tab {
            SettingsTab::About => about::show(ui),
            SettingsTab::General => general::show(ui, shared, tx),
            SettingsTab::Backends => backends::show(ui, shared, tx),
            SettingsTab::Debug => debug::show(ui),
            _ => {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.heading(format!("{current_tab:?}"));
                    ui.label("This tab will be available in a future update.");
                });
            }
        }
    });
}
