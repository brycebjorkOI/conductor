use egui::Color32;

// Re-export the SwiftUI-style theme from egui-swift.
pub use egui_swift::theme::Layout as Theme;

/// Apply the macOS-style theme. Delegates to egui-swift.
pub fn apply(ctx: &egui::Context) {
    egui_swift::theme::apply_macos_style(ctx);
}

/// Map a backend discovery state to a status color.
pub fn discovery_color(state: conductor_core::state::DiscoveryState) -> Color32 {
    use conductor_core::state::DiscoveryState;
    let p = egui_swift::colors::light(); // status colors are the same in both modes
    match state {
        DiscoveryState::Found => p.status_green,
        DiscoveryState::Scanning => p.status_yellow,
        DiscoveryState::NotFound => Color32::from_rgb(160, 160, 160),
        DiscoveryState::Error => p.status_red,
        DiscoveryState::NotScanned => Color32::from_rgb(160, 160, 160),
    }
}
