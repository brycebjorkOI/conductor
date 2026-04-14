pub fn show(ui: &mut egui::Ui) {
    ui.heading("Debug & Diagnostics");
    ui.add_space(8.0);

    ui.label("Log viewer and diagnostics will be available here.");
    ui.add_space(16.0);

    ui.horizontal(|ui| {
        if ui.button("Export Diagnostics").clicked() {
            tracing::info!("diagnostics export requested (not yet implemented)");
        }
    });
}
