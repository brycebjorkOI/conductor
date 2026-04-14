//! SwiftUI-style button with multiple visual styles.
//!
//! ```ignore
//! Button::new("Create").style(ButtonStyle::BorderedProminent).show(ui);
//! Button::new("Delete").style(ButtonStyle::Destructive).show(ui);
//! ```

use crate::colors;

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
            crate::typography::Font::Subheadline.size()
        } else {
            crate::typography::Font::Body.size()
        };
        let v_pad = if self.small { 3.0 } else { 6.0 };
        let rounding = egui::CornerRadius::same(8);

        let text = if let Some(icon) = self.icon {
            format!("{icon}  {}", self.label)
        } else {
            self.label.to_string()
        };

        let (fill, stroke, text_color) = match self.style {
            ButtonStyle::BorderedProminent => (
                p.accent,
                egui::Stroke::NONE,
                p.text_on_accent,
            ),
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
            ButtonStyle::Destructive => (
                p.destructive,
                egui::Stroke::NONE,
                p.text_on_accent,
            ),
        };

        let btn = egui::Button::new(
            egui::RichText::new(text)
                .size(font_size)
                .color(text_color),
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

        // Hover tint for Bordered/Borderless styles.
        if response.hovered() && self.enabled {
            match self.style {
                ButtonStyle::Bordered | ButtonStyle::Borderless => {
                    let hover_rect = response.rect;
                    ui.painter().rect_filled(
                        hover_rect,
                        rounding,
                        crate::ext::ColorExt::opacity(p.accent, 0.06),
                    );
                }
                _ => {}
            }
        }

        response
    }
}
