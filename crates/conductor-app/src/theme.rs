/// Apply the macOS-style theme. Delegates to egui-swift.
pub fn apply(ctx: &egui::Context) {
    egui_swift::theme::apply_macos_style(ctx);
}
