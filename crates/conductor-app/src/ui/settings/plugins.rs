use egui_swift::colors;
use egui_swift::empty_state::EmptyState;
use egui_swift::form_section::FormSection;

pub fn show(ui: &mut egui::Ui) {
    let p = colors::palette(ui);

    ui.label(
        egui::RichText::new("Plugins")
            .size(22.0)
            .strong()
            .color(p.text_primary),
    );
    ui.add_space(8.0);

    ui.label(
        egui::RichText::new("Plugins are npm-based extensions running in the companion server.")
            .size(13.0)
            .color(p.text_secondary),
    );
    ui.add_space(12.0);

    FormSection::new().header("Installed Plugins").show(ui, |ui| {
        EmptyState::new("No plugins installed")
            .subtitle("Connect to a companion server to manage plugins.")
            .show(ui);
    });

    ui.add_space(12.0);

    FormSection::new().header("Install Plugin").show(ui, |ui| {
        ui.label(
            egui::RichText::new(
                "Enter an npm package name to install a plugin on the companion server.",
            )
            .size(12.0)
            .color(p.text_secondary),
        );
    });
}
