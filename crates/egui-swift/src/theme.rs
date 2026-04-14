//! Global egui style overrides for a macOS / SwiftUI-native look.
//!
//! Call [`apply_macos_style`] once at app startup to set corner radii,
//! spacing, and selection colors that match macOS Sonoma conventions.

use egui::Color32;

/// Layout constants calibrated from macOS / Claude desktop screenshots.
pub struct Layout;

impl Layout {
    pub const MAX_CONTENT_WIDTH: f32 = 640.0;
    pub const MESSAGE_SPACING: f32 = 20.0;
    pub const BODY_FONT_SIZE: f32 = 14.5;
    pub const SMALL_FONT_SIZE: f32 = 12.0;
    pub const CAPTION_FONT_SIZE: f32 = 11.0;
    pub const USER_BUBBLE_RADIUS: f32 = 20.0;
    pub const INPUT_RADIUS: f32 = 24.0;
    pub const CARD_RADIUS: f32 = 10.0;
    pub const CONTROL_RADIUS: f32 = 8.0;
    pub const PILL_RADIUS: f32 = 16.0;
    pub const SIDEBAR_WIDTH: f32 = 240.0;
    pub const TOOLBAR_HEIGHT: f32 = 44.0;
    pub const FORM_ROW_HEIGHT: f32 = 36.0;
    pub const NAV_ROW_HEIGHT: f32 = 32.0;
}

/// Apply the macOS / SwiftUI-inspired style to an egui context.
pub fn apply_macos_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.spacing.item_spacing = egui::vec2(8.0, 4.0);
    style.spacing.window_margin = egui::Margin::same(0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);

    // Rounder widgets throughout.
    style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(8);
    style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(8);
    style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(8);

    // Subtle widget backgrounds.
    style.visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
    style.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;

    // Subtle selection highlight.
    style.visuals.selection.bg_fill = Color32::from_rgba_premultiplied(191, 87, 0, 40);

    ctx.set_style(style);
}
