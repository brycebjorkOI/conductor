use egui_swift::colors;
use egui_swift::divider::Divider;
use egui_swift::form_section::FormSection;

pub fn show(ui: &mut egui::Ui) {
    let p = colors::palette(ui);

    ui.label(
        egui::RichText::new("Conductor")
            .size(22.0)
            .strong()
            .color(p.text_primary),
    );
    ui.add_space(16.0);

    FormSection::new().show(ui, |ui| {
        ui.label(
            egui::RichText::new(format!("Version: {}", env!("CARGO_PKG_VERSION")))
                .size(13.0)
                .color(p.text_primary),
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(format!("Target: {}", std::env::consts::ARCH))
                .size(13.0)
                .color(p.text_secondary),
        );
    });

    ui.add_space(12.0);

    ui.label(
        egui::RichText::new(
            "A unified, provider-agnostic conversational interface to multiple AI language-model backends.",
        )
        .size(13.0)
        .color(p.text_secondary),
    );
    ui.add_space(8.0);
    ui.label(
        egui::RichText::new("Built with Rust + egui")
            .size(12.0)
            .color(p.text_muted),
    );

    ui.add_space(16.0);
    Divider::new().show(ui);
    ui.add_space(8.0);

    ui.label(
        egui::RichText::new("Licensed under the MIT License.")
            .size(12.0)
            .color(p.text_muted),
    );
}
