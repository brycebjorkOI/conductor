use egui_swift::prelude::*;

pub fn show(ui: &mut egui::Ui) {
    Label::heading("Plugins").show(ui);
    ui.add_space(8.0);

    Label::new("Plugins are npm-based extensions running in the companion server.")
        .font(Font::Callout)
        .secondary()
        .show(ui);
    ui.add_space(12.0);

    FormSection::new().header("Installed Plugins").show(ui, |ui| {
        EmptyState::new("No plugins installed")
            .subtitle("Connect to a companion server to manage plugins.")
            .show(ui);
    });

    ui.add_space(12.0);

    FormSection::new().header("Install Plugin").show(ui, |ui| {
        Label::new(
            "Enter an npm package name to install a plugin on the companion server.",
        )
        .font(Font::Subheadline)
        .secondary()
        .show(ui);
    });
}
