use egui_swift::prelude::*;

pub fn show(ui: &mut egui::Ui) {
    Label::heading("Conductor").show(ui);
    ui.add_space(16.0);

    FormSection::new().show(ui, |ui| {
        Label::new(&format!("Version: {}", env!("CARGO_PKG_VERSION")))
            .font(Font::Callout)
            .show(ui);
        ui.add_space(4.0);
        Label::new(&format!("Target: {}", std::env::consts::ARCH))
            .font(Font::Callout)
            .secondary()
            .show(ui);
    });

    ui.add_space(12.0);

    Label::new(
        "A unified, provider-agnostic conversational interface to multiple AI language-model backends.",
    )
    .font(Font::Callout)
    .secondary()
    .show(ui);

    ui.add_space(8.0);
    Label::new("Built with Rust + egui")
        .font(Font::Subheadline)
        .muted()
        .show(ui);

    ui.add_space(16.0);
    Divider::new().show(ui);
    ui.add_space(8.0);

    Label::new("Licensed under the MIT License.")
        .font(Font::Subheadline)
        .muted()
        .show(ui);
}
