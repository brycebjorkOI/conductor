//! Shared layout and painting utilities.

use egui::{Color32, Rect, CornerRadius, Ui};

/// Paint a faked soft drop shadow behind a rect by drawing concentric
/// translucent rects with increasing size.
pub fn paint_shadow(ui: &Ui, rect: Rect, rounding: CornerRadius, spread: f32, color: Color32) {
    let painter = ui.painter();
    let steps = 3u8;
    for i in 1..=steps {
        let expand = spread * (i as f32 / steps as f32);
        let alpha = color.a() / (i as u8 + 1);
        let c = Color32::from_rgba_premultiplied(color.r(), color.g(), color.b(), alpha);
        painter.rect_filled(rect.expand(expand), rounding, c);
    }
}

/// Linearly interpolate between two colors.
pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| -> u8 { (x as f32 + (y as f32 - x as f32) * t) as u8 };
    Color32::from_rgba_premultiplied(
        mix(a.r(), b.r()),
        mix(a.g(), b.g()),
        mix(a.b(), b.b()),
        mix(a.a(), b.a()),
    )
}

/// Smooth 0.0..1.0 animation factor driven by a boolean value.
/// Wraps `ctx.animate_bool_with_time()` with a sensible default speed.
pub fn animate_bool(ui: &Ui, id: egui::Id, value: bool, seconds: f32) -> f32 {
    ui.ctx().animate_bool_with_time(id, value, seconds)
}

/// Truncate a string to `max_chars`, appending "..." if truncated.
pub fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        text.to_string()
    } else {
        format!("{}...", &text[..max_chars.saturating_sub(3)])
    }
}
