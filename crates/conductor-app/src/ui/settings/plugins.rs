pub fn show(ui: &mut egui::Ui) {
    let p = egui_swift::colors::palette(ui);

    ui.heading("Plugins");
    ui.add_space(12.0);

    ui.label("Plugins are npm-based extensions running in the companion server.");
    ui.add_space(8.0);

    ui.group(|ui| {
        ui.label(egui::RichText::new("Installed Plugins").strong());
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("No plugins installed. Connect to a companion server to manage plugins.")
                .size(12.0)
                .color(p.text_muted),
        );
    });

    ui.add_space(12.0);

    ui.group(|ui| {
        ui.label(egui::RichText::new("Install Plugin").strong());
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Enter an npm package name to install a plugin on the companion server.")
                .size(12.0)
                .color(p.text_secondary),
        );
    });
}
