use egui_swift::button::{Button, ButtonStyle};
use egui_swift::colors;

pub fn show(ui: &mut egui::Ui) {
    let p = colors::palette(ui);

    ui.label(
        egui::RichText::new("Debug & Diagnostics")
            .size(22.0)
            .strong()
            .color(p.text_primary),
    );
    ui.add_space(12.0);

    ui.label(
        egui::RichText::new("Log viewer and diagnostics will be available here.")
            .size(13.0)
            .color(p.text_secondary),
    );
    ui.add_space(16.0);

    if Button::new("Export Diagnostics")
        .style(ButtonStyle::Bordered)
        .show(ui)
        .clicked()
    {
        tracing::info!("diagnostics export requested (not yet implemented)");
    }
}
