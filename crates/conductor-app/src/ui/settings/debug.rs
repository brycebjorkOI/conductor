use egui_swift::prelude::*;

pub fn show(ui: &mut egui::Ui) {
    Label::heading("Debug & Diagnostics").show(ui);
    Spacer::fixed(12.0).show(ui);

    Section::new().header("Tools").show(ui, |ui| {
        Label::new("Log viewer and diagnostics will be available here.")
            .font(Font::Callout)
            .secondary()
            .show(ui);
        Spacer::fixed(8.0).show(ui);
        if Button::new("Export Diagnostics")
            .style(ButtonStyle::Bordered)
            .show(ui)
            .clicked()
        {
            tracing::info!("diagnostics export requested (not yet implemented)");
        }
    });
}
