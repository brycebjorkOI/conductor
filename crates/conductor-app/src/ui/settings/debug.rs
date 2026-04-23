use egui_swift::prelude::*;

egui_swift::view! {
    pub struct DebugView {}
    fn body(&mut self, ui: &mut egui::Ui) {
        egui_swift::text!(ui, "Debug & Diagnostics", .title);
        egui_swift::spacer!(ui, 12.0);

        egui_swift::section!(ui, "Tools", {
            egui_swift::text!(ui, "Log viewer and diagnostics will be available here.", .callout, .secondary);
            egui_swift::spacer!(ui, 8.0);
            if Button::new("Export Diagnostics")
                .style(ButtonStyle::Bordered)
                .show(ui)
                .clicked()
            {
                tracing::info!("diagnostics export requested (not yet implemented)");
            }
        });
    }
}
