//! SwiftUI-style button with multiple visual styles and press feedback.

use crate::colors;
use crate::ext::ColorExt;
use crate::typography::Font;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonStyle {
    /// Filled accent background, white text.
    BorderedProminent,
    /// Transparent background, accent border and text.
    Bordered,
    /// No background, no border, accent text.
    Borderless,
    /// Filled red background, white text.
    Destructive,
}

pub struct Button<'a> {
    label: &'a str,
    style: ButtonStyle,
    icon: Option<&'a str>,
    enabled: bool,
    small: bool,
}

impl<'a> Button<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            style: ButtonStyle::Bordered,
            icon: None,
            enabled: true,
            small: false,
        }
    }

    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn small(mut self, small: bool) -> Self {
        self.small = small;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let p = colors::palette(ui);
        let font_size = if self.small {
            Font::Subheadline.size()
        } else {
            Font::Body.size()
        };
        let v_pad = if self.small { 3.0 } else { 6.0 };
        let rounding = egui::CornerRadius::same(8);

        let text = if let Some(icon) = self.icon {
            format!("{icon}  {}", self.label)
        } else {
            self.label.to_string()
        };

        let (fill, stroke, text_color) = match self.style {
            ButtonStyle::BorderedProminent => (p.accent, egui::Stroke::NONE, p.text_on_accent),
            ButtonStyle::Bordered => (
                egui::Color32::TRANSPARENT,
                egui::Stroke::new(1.0, p.accent),
                p.accent,
            ),
            ButtonStyle::Borderless => (
                egui::Color32::TRANSPARENT,
                egui::Stroke::NONE,
                p.accent,
            ),
            ButtonStyle::Destructive => (p.destructive, egui::Stroke::NONE, p.text_on_accent),
        };

        let btn = egui::Button::new(
            egui::RichText::new(text).size(font_size).color(text_color),
        )
        .fill(fill)
        .stroke(stroke)
        .corner_radius(rounding)
        .min_size(egui::vec2(0.0, v_pad * 2.0 + font_size));

        let response = if self.enabled {
            ui.add(btn)
        } else {
            ui.add_enabled(false, btn)
        };

        if self.enabled && ui.is_rect_visible(response.rect) {
            let rect = response.rect;
            let is_pressed = response.is_pointer_button_down_on();

            if is_pressed {
                // Press state: darken filled buttons, stronger tint for outline/text buttons.
                match self.style {
                    ButtonStyle::BorderedProminent | ButtonStyle::Destructive => {
                        ui.painter()
                            .rect_filled(rect, rounding, egui::Color32::BLACK.opacity(0.15));
                    }
                    ButtonStyle::Bordered | ButtonStyle::Borderless => {
                        ui.painter()
                            .rect_filled(rect, rounding, p.accent.opacity(0.15));
                    }
                }
            } else if response.hovered() {
                // Hover state: subtle tint for outline/text buttons.
                match self.style {
                    ButtonStyle::Bordered | ButtonStyle::Borderless => {
                        ui.painter()
                            .rect_filled(rect, rounding, p.accent.opacity(0.06));
                    }
                    _ => {}
                }
            }
        }

        response
    }
}
