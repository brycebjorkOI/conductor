use egui_swift::prelude::*;

egui_swift::view! {
    pub struct AboutView {}
    fn body(&mut self, ui: &mut egui::Ui) {
        egui_swift::text!(ui, "Conductor", .title);
        egui_swift::spacer!(ui, 16.0);

        egui_swift::section!(ui, {
            LabeledContent::new("Version", env!("CARGO_PKG_VERSION")).show(ui);
            LabeledContent::new("Target", std::env::consts::ARCH).show(ui);
        });

        egui_swift::spacer!(ui, 12.0);
        egui_swift::text!(ui, "A unified, provider-agnostic conversational interface to multiple AI language-model backends.", .callout, .secondary);
        egui_swift::spacer!(ui, 8.0);
        egui_swift::text!(ui, "Built with Rust + egui", .subheadline, .muted);
        egui_swift::spacer!(ui, 16.0);
        Divider::new().show(ui);
        egui_swift::spacer!(ui, 8.0);
        egui_swift::text!(ui, "Licensed under the MIT License.", .subheadline, .muted);
    }
}
