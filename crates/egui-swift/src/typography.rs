//! SwiftUI-style typography presets.
//!
//! Maps to SwiftUI's Font enum: `.largeTitle`, `.title`, `.headline`,
//! `.body`, `.callout`, `.subheadline`, `.footnote`, `.caption`.

/// Typography preset matching SwiftUI's font styles.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Font {
    /// 34pt, used for large splash/greeting text.
    LargeTitle,
    /// 22pt bold, used for page/section titles.
    Title,
    /// 17pt bold, used for emphasized labels.
    Headline,
    /// 14.5pt, the default body text.
    Body,
    /// 13pt, slightly smaller body text.
    Callout,
    /// 12pt, secondary information.
    Subheadline,
    /// 11pt, captions and timestamps.
    Caption,
    /// 10pt, smallest text.
    Footnote,
}

impl Font {
    /// Font size in points.
    pub fn size(self) -> f32 {
        match self {
            Font::LargeTitle => 34.0,
            Font::Title => 22.0,
            Font::Headline => 17.0,
            Font::Body => 14.5,
            Font::Callout => 13.0,
            Font::Subheadline => 12.0,
            Font::Caption => 11.0,
            Font::Footnote => 10.0,
        }
    }

    /// Whether this font preset is bold by default (like SwiftUI).
    pub fn is_bold(self) -> bool {
        matches!(self, Font::Title | Font::Headline)
    }
}
