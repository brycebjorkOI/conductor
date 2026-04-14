//! Global egui style overrides for a macOS / SwiftUI-native look.
//!
//! Call [`apply_macos_style`] once at app startup to set corner radii,
//! spacing, selection colors, and load Inter + JetBrains Mono fonts.

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

// Embedded font data (Inter for proportional, JetBrains Mono for monospace).
// Inter: SIL Open Font License. JetBrains Mono: SIL Open Font License.
const INTER_REGULAR: &[u8] = include_bytes!("../fonts/Inter-Regular.ttf");
const INTER_BOLD: &[u8] = include_bytes!("../fonts/Inter-Bold.ttf");
const JETBRAINS_MONO: &[u8] = include_bytes!("../fonts/JetBrainsMono-Regular.ttf");

/// Apply the macOS / SwiftUI-inspired style to an egui context.
///
/// This sets:
/// - **Inter** as the proportional font (close match for SF Pro)
/// - **JetBrains Mono** as the monospace font
/// - Rounded corner radii, generous spacing, subtle widget backgrounds
///
/// Call once at app startup, e.g. in `eframe::CreationContext`.
pub fn apply_macos_style(ctx: &egui::Context) {
    // -- Fonts --
    let mut fonts = egui::FontDefinitions::default();

    // Insert Inter Regular as the primary proportional font.
    fonts.font_data.insert(
        "Inter-Regular".to_owned(),
        egui::FontData::from_static(INTER_REGULAR).into(),
    );
    // Insert Inter Bold.
    fonts.font_data.insert(
        "Inter-Bold".to_owned(),
        egui::FontData::from_static(INTER_BOLD).into(),
    );
    // Insert JetBrains Mono as the primary monospace font.
    fonts.font_data.insert(
        "JetBrainsMono".to_owned(),
        egui::FontData::from_static(JETBRAINS_MONO).into(),
    );

    // Set Inter as the first proportional font (before the egui defaults as fallback).
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "Inter-Regular".to_owned());

    // Register Inter Bold as a custom "Bold" family for true bold weight.
    fonts.families.insert(
        egui::FontFamily::Name("Bold".into()),
        vec!["Inter-Bold".to_owned(), "Inter-Regular".to_owned()],
    );

    // Set JetBrains Mono as the first monospace font.
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "JetBrainsMono".to_owned());

    ctx.set_fonts(fonts);

    // -- Style --
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

    // Text selection uses the palette accent at 25% opacity.
    let palette = crate::colors::palette_from_ctx(ctx);
    style.visuals.selection.bg_fill =
        Color32::from_rgba_unmultiplied(palette.accent.r(), palette.accent.g(), palette.accent.b(), 64);
    style.visuals.selection.stroke = egui::Stroke::new(1.0, palette.accent);

    // Thin scrollbar styling (macOS overlay style).
    style.spacing.scroll.bar_width = 6.0;
    style.spacing.scroll.bar_inner_margin = 2.0;
    style.spacing.scroll.bar_outer_margin = 2.0;

    ctx.set_style(style);
}
