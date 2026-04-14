use egui_swift::prelude::*;

pub fn show(ui: &mut egui::Ui) {
    Label::heading("Debug & Diagnostics").show(ui);
    ui.add_space(12.0);

    Label::new("Log viewer and diagnostics will be available here.")
        .font(Font::Callout)
        .secondary()
        .show(ui);
    ui.add_space(16.0);

    if Button::new("Export Diagnostics")
        .style(ButtonStyle::Bordered)
        .show(ui)
        .clicked()
    {
        tracing::info!("diagnostics export requested (not yet implemented)");
    }
}
