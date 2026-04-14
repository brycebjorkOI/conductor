mod about;
mod backends;
mod channels;
mod debug;
mod general;
mod mcp;
mod permissions;
mod plugins;
pub mod schedules;
mod sessions;
mod skills;

use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::state::SettingsTab;

use crate::bridge::SharedState;

pub fn show(
    ctx: &egui::Context,
    shared: &SharedState,
    tx: &mpsc::UnboundedSender<Action>,
    schedules_state: &mut schedules::SchedulesTabState,
) {
    let current_tab = shared.read().settings_tab;
    let p = egui_swift::colors::palette_from_ctx(ctx);

    egui::SidePanel::left("settings_tabs")
        .resizable(false)
        .default_width(140.0)
        .frame(
            egui::Frame::NONE
                .fill(p.sidebar_bg)
                .inner_margin(egui::Margin::symmetric(8, 12)),
        )
        .show(ctx, |ui| {
            ui.add_space(28.0); // traffic light clearance
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

    egui::CentralPanel::default()
        .frame(
            egui::Frame::NONE
                .fill(p.surface)
                .inner_margin(egui::Margin::symmetric(20, 20)),
        )
        .show(ctx, |ui| {
            match current_tab {
                SettingsTab::About => about::show(ui),
                SettingsTab::General => general::show(ui, shared, tx),
                SettingsTab::Backends => backends::show(ui, shared, tx),
                SettingsTab::Channels => channels::show(ui, shared),
                SettingsTab::Schedules => schedules::show(ui, shared, tx, schedules_state),
                SettingsTab::Sessions => sessions::show(ui, shared, tx),
                SettingsTab::Plugins => plugins::show(ui),
                SettingsTab::Skills => skills::show(ui),
                SettingsTab::McpServers => mcp::show(ui, shared),
                SettingsTab::Permissions => permissions::show(ui, shared),
                SettingsTab::Debug => debug::show(ui),
            }
        });
}
