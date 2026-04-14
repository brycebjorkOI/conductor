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
use egui_swift::prelude::*;

use crate::bridge::SharedState;

pub fn show(
    ctx: &egui::Context,
    shared: &SharedState,
    tx: &mpsc::UnboundedSender<Action>,
    schedules_state: &mut schedules::SchedulesTabState,
) {
    let current_tab = shared.read().settings_tab;

    NavigationSplitView::new("settings_nav")
        .sidebar_width(160.0)
        .show(ctx, |sidebar, detail| {
            sidebar.show(|ui| {
                Label::heading("Settings").show(ui);
                ui.add_space(8.0);
                Divider::new().show(ui);
                ui.add_space(8.0);

                let tabs = [
                    (SettingsTab::About, "About", "info.circle"),
                    (SettingsTab::General, "General", "gear"),
                    (SettingsTab::Backends, "Backends", "desktopcomputer"),
                    (SettingsTab::Channels, "Channels", "bubble.left"),
                    (SettingsTab::Schedules, "Schedules", "calendar"),
                    (SettingsTab::Sessions, "Sessions", "doc.text"),
                    (SettingsTab::Plugins, "Plugins", "puzzlepiece"),
                    (SettingsTab::Skills, "Skills", "books.vertical"),
                    (SettingsTab::McpServers, "MCP Servers", "globe"),
                    (SettingsTab::Permissions, "Permissions", "lock"),
                    (SettingsTab::Debug, "Debug", "ant"),
                ];

                for (tab, label, sf_name) in tabs {
                    let icon = egui_swift::image::sf_symbol(sf_name);
                    let selected = current_tab == tab;
                    if NavRow::new(label)
                        .icon(icon)
                        .active(selected)
                        .show(ui)
                        .clicked()
                    {
                        shared.mutate(|s| s.settings_tab = tab);
                    }
                }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(8.0);
                    if Button::new("Close Settings")
                        .style(ButtonStyle::Bordered)
                        .show(ui)
                        .clicked()
                    {
                        let _ = tx.send(Action::CloseSettings);
                    }
                    ui.add_space(4.0);
                });
            });

            detail.show(|ui| {
                match current_tab {
                    SettingsTab::About => about::show(ui),
                    SettingsTab::General => general::show(ui, shared, tx),
                    SettingsTab::Backends => backends::show(ui, shared, tx),
                    SettingsTab::Channels => channels::show(ui, shared),
                    SettingsTab::Schedules => {
                        schedules::show(ui, shared, tx, schedules_state)
                    }
                    SettingsTab::Sessions => sessions::show(ui, shared, tx),
                    SettingsTab::Plugins => plugins::show(ui),
                    SettingsTab::Skills => skills::show(ui),
                    SettingsTab::McpServers => mcp::show(ui, shared),
                    SettingsTab::Permissions => permissions::show(ui, shared, tx),
                    SettingsTab::Debug => debug::show(ui),
                }
            });
        });
}
