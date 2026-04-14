//! SF Symbols-compatible image component.
//!
//! Maps Apple SF Symbol names to Unicode glyphs so Claude can generate
//! code using the symbol names it already knows.
//!
//! ```ignore
//! Image::system_name("gear").show(ui);
//! Image::system_name("magnifyingglass").show(ui);
//! Image::system_name("trash").tint(p.destructive).show(ui);
//! ```

use crate::colors;

/// Resolve an SF Symbols name to a Unicode glyph.
///
/// Returns the glyph string, or the original name wrapped in brackets
/// if no mapping exists.
pub fn sf_symbol(name: &str) -> &'static str {
    match name {
        // Navigation / Disclosure
        "chevron.right" => "\u{203a}",       // ›
        "chevron.down" => "\u{2304}",        // ⌄
        "chevron.left" => "\u{2039}",        // ‹
        "chevron.up" => "\u{2303}",          // ⌃
        "chevron.left.2" => "\u{00ab}",      // «
        "chevron.right.2" => "\u{00bb}",     // »
        "arrow.left" => "\u{2190}",          // ←
        "arrow.right" => "\u{2192}",         // →
        "arrow.up" => "\u{2191}",            // ↑
        "arrow.down" => "\u{2193}",          // ↓
        "arrow.clockwise" => "\u{21bb}",     // ↻
        "arrow.counterclockwise" => "\u{21ba}", // ↺
        "arrow.up.right" => "\u{2197}",      // ↗
        "arrow.triangle.2.circlepath" => "\u{21bb}", // ↻

        // Actions
        "plus" => "+",
        "minus" => "\u{2212}",              // −
        "plus.circle" => "\u{2295}",        // ⊕
        "minus.circle" => "\u{2296}",       // ⊖
        "xmark" => "\u{2715}",             // ✕
        "xmark.circle" => "\u{2716}",      // ✖
        "xmark.circle.fill" => "\u{2716}", // ✖
        "checkmark" => "\u{2713}",          // ✓
        "checkmark.circle" => "\u{2714}",   // ✔
        "checkmark.circle.fill" => "\u{2714}", // ✔

        // Common objects
        "gear" | "gearshape" => "\u{2699}",           // ⚙
        "gearshape.fill" => "\u{2699}",               // ⚙
        "house" | "house.fill" => "\u{1f3e0}",        // 🏠
        "magnifyingglass" => "\u{1f50d}",              // 🔍
        "bell" | "bell.fill" => "\u{1f514}",           // 🔔
        "bell.slash" | "bell.slash.fill" => "\u{1f515}", // 🔕
        "trash" | "trash.fill" => "\u{1f5d1}",        // 🗑
        "folder" | "folder.fill" => "\u{1f4c1}",      // 📁
        "doc" | "doc.fill" => "\u{1f4c4}",            // 📄
        "doc.text" | "doc.text.fill" => "\u{1f4dd}",  // 📝
        "calendar" => "\u{1f4c5}",                     // 📅
        "clock" | "clock.fill" => "\u{1f551}",        // 🕑
        "star" | "star.fill" => "\u{2b50}",           // ⭐
        "heart" | "heart.fill" => "\u{2764}",         // ❤
        "bookmark" | "bookmark.fill" => "\u{1f516}",  // 🔖
        "tag" | "tag.fill" => "\u{1f3f7}",            // 🏷
        "pin" | "pin.fill" | "mappin" => "\u{1f4cd}", // 📍
        "link" => "\u{1f517}",                         // 🔗
        "paperclip" => "\u{1f4ce}",                    // 📎
        "envelope" | "envelope.fill" => "\u{2709}",   // ✉
        "phone" | "phone.fill" => "\u{1f4de}",        // 📞
        "bubble.left" | "bubble.left.fill" => "\u{1f4ac}", // 💬
        "bubble.right" | "bubble.right.fill" => "\u{1f4ac}", // 💬

        // Media
        "play" | "play.fill" => "\u{25b6}",           // ▶
        "pause" | "pause.fill" => "\u{23f8}",         // ⏸
        "stop" | "stop.fill" => "\u{23f9}",           // ⏹
        "mic" | "mic.fill" => "\u{1f399}",            // 🎙
        "speaker" | "speaker.fill" => "\u{1f508}",    // 🔈
        "speaker.wave.2" | "speaker.wave.2.fill" => "\u{1f50a}", // 🔊

        // People & Communication
        "person" | "person.fill" => "\u{1f464}",      // 👤
        "person.2" | "person.2.fill" => "\u{1f465}",  // 👥
        "person.circle" | "person.circle.fill" => "\u{1f464}", // 👤

        // System / Settings
        "lock" | "lock.fill" => "\u{1f512}",          // 🔒
        "lock.open" | "lock.open.fill" => "\u{1f513}", // 🔓
        "key" | "key.fill" => "\u{1f511}",            // 🔑
        "eye" | "eye.fill" => "\u{1f441}",            // 👁
        "eye.slash" | "eye.slash.fill" => "\u{1f648}", // 🙈
        "wifi" => "\u{1f4f6}",                         // 📶
        "globe" => "\u{1f310}",                        // 🌐
        "network" => "\u{1f310}",                      // 🌐
        "server.rack" => "\u{1f5a5}",                 // 🖥
        "desktopcomputer" => "\u{1f5a5}",             // 🖥
        "terminal" | "terminal.fill" => "\u{1f4bb}",  // 💻
        "cpu" => "\u{1f4bb}",                          // 💻
        "memorychip" => "\u{1f4be}",                   // 💾
        "externaldrive" | "externaldrive.fill" => "\u{1f4be}", // 💾
        "icloud" | "icloud.fill" => "\u{2601}",       // ☁

        // Development
        "hammer" | "hammer.fill" => "\u{1f528}",      // 🔨
        "wrench" | "wrench.fill" => "\u{1f527}",      // 🔧
        "screwdriver" | "screwdriver.fill" => "\u{1fa9b}", // 🪛
        "ant" | "ant.fill" => "\u{1f41b}",            // 🐛
        "ladybug" | "ladybug.fill" => "\u{1f41e}",    // 🐞

        // Shapes / Status
        "circle" => "\u{25cb}",                        // ○
        "circle.fill" => "\u{25cf}",                   // ●
        "square" => "\u{25a1}",                        // □
        "square.fill" => "\u{25a0}",                   // ■
        "triangle" => "\u{25b3}",                      // △
        "triangle.fill" => "\u{25b2}",                 // ▲
        "diamond" => "\u{25c7}",                       // ◇
        "diamond.fill" => "\u{25c6}",                  // ◆
        "exclamationmark.triangle" | "exclamationmark.triangle.fill" => "\u{26a0}", // ⚠
        "info.circle" | "info.circle.fill" => "\u{2139}", // ℹ
        "questionmark.circle" | "questionmark.circle.fill" => "\u{2753}", // ❓

        // Editing
        "pencil" => "\u{270f}",                        // ✏
        "pencil.circle" | "pencil.circle.fill" => "\u{270f}", // ✏
        "square.and.pencil" => "\u{270f}",             // ✏
        "scissors" => "\u{2702}",                      // ✂
        "doc.on.doc" => "\u{1f4cb}",                   // 📋
        "doc.on.clipboard" => "\u{1f4cb}",             // 📋
        "list.bullet" => "\u{2630}",                   // ☰
        "line.3.horizontal" => "\u{2630}",             // ☰
        "slider.horizontal.3" => "\u{2630}",           // ☰
        "text.alignleft" => "\u{2630}",                // ☰

        // Weather / Nature
        "sun.max" | "sun.max.fill" => "\u{2600}",     // ☀
        "moon" | "moon.fill" => "\u{1f319}",          // 🌙
        "cloud" | "cloud.fill" => "\u{2601}",         // ☁
        "bolt" | "bolt.fill" => "\u{26a1}",           // ⚡
        "flame" | "flame.fill" => "\u{1f525}",        // 🔥
        "drop" | "drop.fill" => "\u{1f4a7}",          // 💧
        "leaf" | "leaf.fill" => "\u{1f343}",           // 🍃

        // Misc
        "sparkles" => "\u{2728}",                      // ✨
        "lightbulb" | "lightbulb.fill" => "\u{1f4a1}", // 💡
        "book" | "book.fill" => "\u{1f4d6}",          // 📖
        "books.vertical" | "books.vertical.fill" => "\u{1f4da}", // 📚
        "puzzlepiece" | "puzzlepiece.fill" => "\u{1f9e9}", // 🧩
        "paintbrush" | "paintbrush.fill" => "\u{1f58c}", // 🖌
        "wand.and.stars" => "\u{2728}",                // ✨
        "hand.thumbsup" | "hand.thumbsup.fill" => "\u{1f44d}", // 👍
        "hand.thumbsdown" | "hand.thumbsdown.fill" => "\u{1f44e}", // 👎
        "hand.raised" | "hand.raised.fill" => "\u{270b}", // ✋
        "flag" | "flag.fill" => "\u{1f3f3}",          // 🏳
        "trophy" | "trophy.fill" => "\u{1f3c6}",      // 🏆
        "rosette" => "\u{1f3f5}",                      // 🏵
        "gift" | "gift.fill" => "\u{1f381}",           // 🎁
        "cart" | "cart.fill" => "\u{1f6d2}",           // 🛒
        "creditcard" | "creditcard.fill" => "\u{1f4b3}", // 💳
        "banknote" | "banknote.fill" => "\u{1f4b5}",  // 💵

        // Code / Terminal
        "chevron.left.forwardslash.chevron.right" => "</>",
        "curlybraces" => "{}",
        "number" => "#",
        "at" => "@",

        // Fallback
        _ => "",
    }
}

/// An image rendered from an SF Symbols name or a custom glyph.
pub struct Image {
    glyph: String,
    size: f32,
    tint: Option<egui::Color32>,
}

impl Image {
    /// Create an image from an SF Symbols name.
    ///
    /// ```ignore
    /// Image::system_name("gear").show(ui);
    /// Image::system_name("bell.fill").tint(p.accent).show(ui);
    /// ```
    pub fn system_name(name: &str) -> Self {
        let glyph = sf_symbol(name);
        let resolved = if glyph.is_empty() {
            format!("[{name}]")
        } else {
            glyph.to_string()
        };
        Self {
            glyph: resolved,
            size: 16.0,
            tint: None,
        }
    }

    /// Create an image from a literal glyph / emoji.
    pub fn glyph(glyph: &str) -> Self {
        Self {
            glyph: glyph.to_string(),
            size: 16.0,
            tint: None,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn tint(mut self, color: egui::Color32) -> Self {
        self.tint = Some(color);
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let p = colors::palette(ui);
        let color = self.tint.unwrap_or(p.text_primary);

        ui.label(
            egui::RichText::new(&self.glyph)
                .size(self.size)
                .color(color),
        )
    }
}
