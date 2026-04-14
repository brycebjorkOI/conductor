use egui_swift::prelude::*;

pub fn show(ui: &mut egui::Ui) {
    egui_swift::text!(ui, "Plugins", .title);
    egui_swift::spacer!(ui, 8.0);
    egui_swift::text!(ui, "Plugins are npm-based extensions running in the companion server.", .callout, .secondary);
    egui_swift::spacer!(ui, 12.0);

    egui_swift::section!(ui, "Installed Plugins", {
        EmptyState::new("No plugins installed")
            .subtitle("Connect to a companion server to manage plugins.")
            .show(ui);
    });

    egui_swift::spacer!(ui, 12.0);

    egui_swift::section!(ui, "Install Plugin", {
        egui_swift::text!(ui, "Enter an npm package name to install a plugin on the companion server.", .subheadline, .secondary);
    });
}
