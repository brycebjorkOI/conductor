//! Shared color palette for the component library.
//!
//! Two palettes are provided: `dark()` and `light()`.  Every component reads
//! `ui.visuals().dark_mode` to pick the right palette, so callers never need
//! to pass a palette around.

use egui::Color32;

/// Complete color palette for one appearance mode.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    // Surfaces
    pub sidebar_bg: Color32,
    pub surface: Color32,
    pub surface_raised: Color32,
    pub input_bg: Color32,

    // Borders
    pub border: Color32,
    pub border_subtle: Color32,

    // Text
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub text_placeholder: Color32,
    pub text_on_accent: Color32,

    // Accent
    pub accent: Color32,
    pub accent_bg: Color32,

    // Status
    pub status_green: Color32,
    pub status_red: Color32,

    // Interactive states
    pub hover_bg: Color32,
    pub active_bg: Color32,
    pub active_indicator: Color32,
}

/// Dark-mode palette (matches the McClaw/Claude dark screenshot).
pub fn dark() -> Palette {
    Palette {
        sidebar_bg: Color32::from_rgb(30, 30, 30),
        surface: Color32::from_rgb(38, 38, 38),
        surface_raised: Color32::from_rgb(48, 48, 48),
        input_bg: Color32::from_rgb(55, 55, 55),

        border: Color32::from_rgb(65, 65, 65),
        border_subtle: Color32::from_rgb(50, 50, 50),

        text_primary: Color32::from_rgb(236, 236, 236),
        text_secondary: Color32::from_rgb(170, 170, 170),
        text_muted: Color32::from_rgb(110, 110, 110),
        text_placeholder: Color32::from_rgb(100, 100, 100),
        text_on_accent: Color32::WHITE,

        accent: Color32::from_rgb(80, 130, 220),
        accent_bg: Color32::from_rgb(35, 55, 90),

        status_green: Color32::from_rgb(80, 200, 120),
        status_red: Color32::from_rgb(220, 70, 70),

        hover_bg: Color32::from_rgb(45, 45, 48),
        active_bg: Color32::from_rgb(40, 52, 70),
        active_indicator: Color32::from_rgb(80, 140, 240),
    }
}

/// Light-mode palette.
pub fn light() -> Palette {
    Palette {
        sidebar_bg: Color32::from_rgb(243, 242, 240),
        surface: Color32::WHITE,
        surface_raised: Color32::WHITE,
        input_bg: Color32::WHITE,

        border: Color32::from_rgb(218, 216, 212),
        border_subtle: Color32::from_rgb(230, 228, 224),

        text_primary: Color32::from_rgb(28, 28, 28),
        text_secondary: Color32::from_rgb(100, 100, 100),
        text_muted: Color32::from_rgb(160, 158, 154),
        text_placeholder: Color32::from_rgb(175, 172, 168),
        text_on_accent: Color32::WHITE,

        accent: Color32::from_rgb(50, 110, 210),
        accent_bg: Color32::from_rgb(225, 237, 255),

        status_green: Color32::from_rgb(60, 180, 90),
        status_red: Color32::from_rgb(210, 60, 60),

        hover_bg: Color32::from_rgb(233, 231, 228),
        active_bg: Color32::from_rgb(225, 235, 250),
        active_indicator: Color32::from_rgb(50, 110, 210),
    }
}

/// Return the appropriate palette for the current visuals.
pub fn palette(ui: &egui::Ui) -> Palette {
    if ui.visuals().dark_mode {
        dark()
    } else {
        light()
    }
}
