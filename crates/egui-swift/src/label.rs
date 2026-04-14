//! SwiftUI-style Label with chainable font/color modifiers.
//!
//! ```ignore
//! // Simple usage — auto-colors from palette
//! Label::new("Settings").font(Font::Title).show(ui);
//!
//! // With explicit color
//! Label::new("Muted hint").font(Font::Caption).secondary().show(ui);
//!
//! // Shorthand for page headings
//! Label::heading("General").show(ui);
//! ```

use crate::colors;
use crate::typography::Font;

/// Text color semantic, resolved against the palette at render time.
#[derive(Debug, Clone, Copy)]
enum TextColor {
    /// Use `text_primary` from palette.
    Primary,
    /// Use `text_secondary` from palette.
    Secondary,
    /// Use `text_muted` from palette.
    Muted,
    /// Use `accent` from palette.
    Accent,
    /// Use `status_red` / destructive from palette.
    Destructive,
    /// Explicit color override.
    Custom(egui::Color32),
}

pub struct Label<'a> {
    text: &'a str,
    font: Font,
    color: TextColor,
    bold: Option<bool>,
    italic: bool,
    monospace: bool,
}

impl<'a> Label<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            font: Font::Body,
            color: TextColor::Primary,
            bold: None,
            italic: false,
            monospace: false,
        }
    }

    /// Shorthand for a page heading: Title font, primary color, bold.
    pub fn heading(text: &'a str) -> Self {
        Self {
            text,
            font: Font::Title,
            color: TextColor::Primary,
            bold: Some(true),
            italic: false,
            monospace: false,
        }
    }

    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    /// Use `text_secondary` color.
    pub fn secondary(mut self) -> Self {
        self.color = TextColor::Secondary;
        self
    }

    /// Use `text_muted` color.
    pub fn muted(mut self) -> Self {
        self.color = TextColor::Muted;
        self
    }

    /// Use `accent` color.
    pub fn accent(mut self) -> Self {
        self.color = TextColor::Accent;
        self
    }

    /// Use `destructive` color.
    pub fn destructive(mut self) -> Self {
        self.color = TextColor::Destructive;
        self
    }

    /// Explicit color override.
    pub fn color(mut self, c: egui::Color32) -> Self {
        self.color = TextColor::Custom(c);
        self
    }

    pub fn bold(mut self, bold: bool) -> Self {
        self.bold = Some(bold);
        self
    }

    pub fn italic(mut self, italic: bool) -> Self {
        self.italic = italic;
        self
    }

    pub fn monospace(mut self, mono: bool) -> Self {
        self.monospace = mono;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let p = colors::palette(ui);

        let resolved_color = match self.color {
            TextColor::Primary => p.text_primary,
            TextColor::Secondary => p.text_secondary,
            TextColor::Muted => p.text_muted,
            TextColor::Accent => p.accent,
            TextColor::Destructive => p.destructive,
            TextColor::Custom(c) => c,
        };

        let is_bold = self.bold.unwrap_or_else(|| self.font.is_bold());

        let mut rt = egui::RichText::new(self.text)
            .size(self.font.size())
            .color(resolved_color);

        if is_bold {
            rt = rt.strong();
        }
        if self.italic {
            rt = rt.italics();
        }
        if self.monospace {
            rt = rt.monospace();
        }

        ui.label(rt)
    }
}
