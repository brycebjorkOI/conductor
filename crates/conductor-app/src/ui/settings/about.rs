use egui_swift::prelude::*;

pub fn show(ui: &mut egui::Ui) {
    Label::heading("Conductor").show(ui);
    Spacer::fixed(16.0).show(ui);

    Section::new().show(ui, |ui| {
        LabeledContent::new("Version", env!("CARGO_PKG_VERSION")).show(ui);
        LabeledContent::new("Target", std::env::consts::ARCH).show(ui);
    });

    Spacer::fixed(12.0).show(ui);

    Label::new(
        "A unified, provider-agnostic conversational interface to multiple AI language-model backends.",
    )
    .font(Font::Callout)
    .secondary()
    .show(ui);

    Spacer::fixed(8.0).show(ui);
    Label::new("Built with Rust + egui")
        .font(Font::Subheadline)
        .muted()
        .show(ui);

    Spacer::fixed(16.0).show(ui);
    Divider::new().show(ui);
    Spacer::fixed(8.0).show(ui);

    Label::new("Licensed under the MIT License.")
        .font(Font::Subheadline)
        .muted()
        .show(ui);
}
