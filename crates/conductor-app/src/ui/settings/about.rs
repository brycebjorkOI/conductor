pub fn show(ui: &mut egui::Ui) {
    ui.heading("Conductor");
    ui.add_space(8.0);

    ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));
    ui.label(format!("Target: {}", std::env::consts::ARCH));
    ui.add_space(16.0);

    ui.label("A unified, provider-agnostic conversational interface to multiple AI language-model backends.");
    ui.add_space(12.0);

    ui.horizontal(|ui| {
        ui.label("Built with Rust + egui");
    });

    ui.add_space(24.0);
    ui.separator();
    ui.add_space(8.0);
    ui.label("Licensed under the MIT License.");
}
