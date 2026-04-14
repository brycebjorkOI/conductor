use egui::Color32;

/// Theme closely matching the Claude desktop app on macOS.
#[allow(dead_code)]
pub struct Theme;

#[allow(dead_code)]
impl Theme {
    // -- Surface colors (Claude uses pure white main, light gray sidebar) --

    pub fn surface(dark: bool) -> Color32 {
        if dark {
            Color32::from_rgb(32, 32, 32)
        } else {
            Color32::WHITE                        // Claude uses pure white, not cream
        }
    }

    pub fn sidebar_bg(dark: bool) -> Color32 {
        if dark {
            Color32::from_rgb(25, 25, 25)
        } else {
            Color32::from_rgb(243, 242, 240)      // Very light warm gray
        }
    }

    pub fn sidebar_hover(dark: bool) -> Color32 {
        if dark {
            Color32::from_rgb(40, 40, 40)
        } else {
            Color32::from_rgb(233, 231, 228)
        }
    }

    pub fn sidebar_active(dark: bool) -> Color32 {
        if dark {
            Color32::from_rgb(45, 45, 45)
        } else {
            Color32::from_rgb(234, 232, 228)      // Very subtle — just barely visible
        }
    }

    // -- Message bubbles --

    pub fn user_bubble_bg(dark: bool) -> Color32 {
        if dark {
            Color32::from_rgb(60, 60, 60)         // Dark mode: subtle gray
        } else {
            Color32::from_rgb(238, 237, 234)      // Light: warm light gray (Claude uses light bg for user)
        }
    }

    pub fn user_bubble_text(dark: bool) -> Color32 {
        Self::text_primary(dark)                   // Same as body text
    }

    pub fn error_bg(dark: bool) -> Color32 {
        if dark {
            Color32::from_rgb(60, 30, 30)
        } else {
            Color32::from_rgb(254, 236, 236)
        }
    }

    // -- Text hierarchy --

    pub fn text_primary(dark: bool) -> Color32 {
        if dark {
            Color32::from_rgb(236, 236, 236)
        } else {
            Color32::from_rgb(28, 28, 28)          // Near black
        }
    }

    pub fn text_secondary(dark: bool) -> Color32 {
        if dark {
            Color32::from_rgb(160, 160, 160)
        } else {
            Color32::from_rgb(100, 100, 100)
        }
    }

    pub fn text_muted(dark: bool) -> Color32 {
        if dark {
            Color32::from_rgb(110, 110, 110)
        } else {
            Color32::from_rgb(160, 158, 154)
        }
    }

    // -- Accent (Claude's terracotta/coral for star icon and interactive bits) --

    pub fn accent(dark: bool) -> Color32 {
        if dark {
            Color32::from_rgb(210, 130, 80)
        } else {
            Color32::from_rgb(191, 87, 0)          // Claude's warm orange-brown
        }
    }

    pub fn accent_subtle(dark: bool) -> Color32 {
        if dark {
            Color32::from_rgb(60, 45, 35)
        } else {
            Color32::from_rgb(252, 243, 234)
        }
    }

    // -- Status --
    pub fn status_green() -> Color32 { Color32::from_rgb(80, 180, 100) }
    pub fn status_yellow() -> Color32 { Color32::from_rgb(210, 170, 50) }
    pub fn status_red() -> Color32 { Color32::from_rgb(210, 60, 60) }
    pub fn status_gray() -> Color32 { Color32::from_rgb(160, 160, 160) }

    // -- Input --

    pub fn input_bg(dark: bool) -> Color32 {
        if dark {
            Color32::from_rgb(42, 42, 42)
        } else {
            Color32::WHITE
        }
    }

    pub fn input_border(dark: bool) -> Color32 {
        if dark {
            Color32::from_rgb(60, 60, 60)
        } else {
            Color32::from_rgb(218, 216, 212)       // Subtle warm gray border
        }
    }

    // -- Tool cards --

    pub fn tool_card_bg(dark: bool) -> Color32 {
        if dark {
            Color32::from_rgb(38, 38, 38)
        } else {
            Color32::from_rgb(247, 246, 243)
        }
    }

    // -- Layout constants (calibrated from Claude screenshot) --

    pub const MAX_CONTENT_WIDTH: f32 = 640.0;      // Claude constrains content width
    pub const MESSAGE_SPACING: f32 = 20.0;
    pub const BODY_FONT_SIZE: f32 = 14.5;
    pub const SMALL_FONT_SIZE: f32 = 12.0;
    pub const USER_BUBBLE_RADIUS: f32 = 20.0;
    pub const INPUT_RADIUS: f32 = 24.0;
    pub const SIDEBAR_WIDTH: f32 = 240.0;           // Claude sidebar is ~240px

    /// Apply the Claude-like theme to the egui context.
    pub fn apply(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        style.spacing.item_spacing = egui::vec2(8.0, 4.0);
        style.spacing.window_margin = egui::Margin::same(0);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);

        // Rounder widgets throughout.
        style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(8);
        style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(8);
        style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(8);

        // Make widget backgrounds more subtle.
        style.visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
        style.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;

        // Subtle selection.
        style.visuals.selection.bg_fill = Color32::from_rgba_premultiplied(191, 87, 0, 40);

        ctx.set_style(style);
    }
}

pub fn discovery_color(state: conductor_core::state::DiscoveryState) -> Color32 {
    use conductor_core::state::DiscoveryState;
    match state {
        DiscoveryState::Found => Theme::status_green(),
        DiscoveryState::Scanning => Theme::status_yellow(),
        DiscoveryState::NotFound => Theme::status_gray(),
        DiscoveryState::Error => Theme::status_red(),
        DiscoveryState::NotScanned => Theme::status_gray(),
    }
}
