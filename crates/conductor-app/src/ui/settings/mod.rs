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
use egui_swift::button::{Button, ButtonStyle};
use egui_swift::colors;
use egui_swift::divider::Divider;
use egui_swift::nav_row::NavRow;

use crate::bridge::SharedState;

pub fn show(
    ctx: &egui::Context,
    shared: &SharedState,
    tx: &mpsc::UnboundedSender<Action>,
    schedules_state: &mut schedules::SchedulesTabState,
) {
    let current_tab = shared.read().settings_tab;
    let p = colors::palette_from_ctx(ctx);

    egui::SidePanel::left("settings_tabs")
        .resizable(false)
        .default_width(160.0)
        .frame(
            egui::Frame::NONE
                .fill(p.sidebar_bg)
                .inner_margin(egui::Margin::symmetric(8, 12)),
        )
        .show(ctx, |ui| {
            ui.add_space(28.0); // traffic light clearance

            ui.label(
                egui::RichText::new("Settings")
                    .size(18.0)
                    .strong()
                    .color(p.text_primary),
            );
            ui.add_space(8.0);
            Divider::new().show(ui);
            ui.add_space(8.0);

            let tabs = [
                (SettingsTab::About, "About", "\u{2139}"),
                (SettingsTab::General, "General", "\u{2699}"),
                (SettingsTab::Backends, "Backends", "\u{1f5a5}"),
                (SettingsTab::Channels, "Channels", "\u{1f4ac}"),
                (SettingsTab::Schedules, "Schedules", "\u{1f4c5}"),
                (SettingsTab::Sessions, "Sessions", "\u{1f4cb}"),
                (SettingsTab::Plugins, "Plugins", "\u{1f9e9}"),
                (SettingsTab::Skills, "Skills", "\u{1f4da}"),
                (SettingsTab::McpServers, "MCP Servers", "\u{1f310}"),
                (SettingsTab::Permissions, "Permissions", "\u{1f512}"),
                (SettingsTab::Debug, "Debug", "\u{1f41b}"),
            ];

            for (tab, label, icon) in tabs {
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

    egui::CentralPanel::default()
        .frame(
            egui::Frame::NONE
                .fill(p.surface)
                .inner_margin(egui::Margin::symmetric(24, 20)),
        )
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
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
