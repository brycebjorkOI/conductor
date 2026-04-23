mod about;
mod backends;
mod channels;
mod debug;
mod general;
mod mcp;
mod permissions;
mod plugins;
mod sessions;
mod skills;

use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::state::SettingsTab;
use egui_swift::prelude::*;

use crate::bridge::SharedState;

/// Settings view with sidebar navigation and tab content.
pub struct SettingsView {
    shared: SharedState,
    tx: mpsc::UnboundedSender<Action>,

    // Tab views.
    about: about::AboutView,
    general: general::GeneralView,
    backends: backends::BackendsView,
    channels: channels::ChannelsView,
    sessions: sessions::SessionsView,
    plugins: plugins::PluginsView,
    skills: skills::SkillsView,
    mcp: mcp::McpView,
    permissions: permissions::PermissionsView,
    debug: debug::DebugView,
}

impl SettingsView {
    pub fn new(shared: SharedState, tx: mpsc::UnboundedSender<Action>) -> Self {
        Self {
            about: about::AboutView::default(),
            general: general::GeneralView::new(shared.clone(), tx.clone()),
            backends: backends::BackendsView::new(shared.clone(), tx.clone()),
            channels: channels::ChannelsView::new(shared.clone(), tx.clone()),
            sessions: sessions::SessionsView::new(shared.clone(), tx.clone()),
            plugins: plugins::PluginsView::default(),
            skills: skills::SkillsView::default(),
            mcp: mcp::McpView::new(shared.clone()),
            permissions: permissions::PermissionsView::new(shared.clone(), tx.clone()),
            debug: debug::DebugView::default(),
            shared,
            tx,
        }
    }

    /// Show the settings using the full egui context (needs ctx for NavigationSplitView).
    pub fn show_ctx(&mut self, ctx: &egui::Context) {
        let current_tab = self.shared.read().settings_tab;

        NavigationSplitView::new("settings_nav")
            .sidebar_width(160.0)
            .show(ctx, |sidebar, detail| {
                sidebar.show(|ui| {
                    egui_swift::text!(ui, "Settings", .title);
                    egui_swift::spacer!(ui, 8.0);
                    Divider::new().show(ui);
                    egui_swift::spacer!(ui, 8.0);

                    let tabs = [
                        (SettingsTab::About, "About", "info.circle"),
                        (SettingsTab::General, "General", "gear"),
                        (SettingsTab::Backends, "Backends", "desktopcomputer"),
                        (SettingsTab::Channels, "Channels", "bubble.left"),
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
                            self.shared.mutate(|s| s.settings_tab = tab);
                        }
                    }

                    Spacer::bottom(ui, |ui| {
                        egui_swift::spacer!(ui, 8.0);
                        if Button::new("Close Settings")
                            .style(ButtonStyle::Bordered)
                            .show(ui)
                            .clicked()
                        {
                            let _ = self.tx.send(Action::CloseSettings);
                        }
                        egui_swift::spacer!(ui, 4.0);
                    });
                });

                detail.show(|ui| {
                    match current_tab {
                        SettingsTab::About => self.about.show(ui),
                        SettingsTab::General => self.general.show(ui),
                        SettingsTab::Backends => self.backends.show(ui),
                        SettingsTab::Channels => self.channels.show(ui),
                        SettingsTab::Sessions => self.sessions.show(ui),
                        SettingsTab::Plugins => self.plugins.show(ui),
                        SettingsTab::Skills => self.skills.show(ui),
                        SettingsTab::McpServers => self.mcp.show(ui),
                        SettingsTab::Permissions => self.permissions.show(ui),
                        SettingsTab::Debug => self.debug.show(ui),
                    }
                });
            });
    }
}
