//! Convenience macros for building egui-swift apps with minimal boilerplate.
//!
//! # `app!` — App entry point
//!
//! Generates `main()`, eframe setup, and applies the macOS style.
//!
//! ```ignore
//! use egui_swift::prelude::*;
//!
//! app!(MyApp, "My App Title", 900.0, 600.0);
//!
//! #[derive(Default)]
//! struct MyApp { /* state */ }
//!
//! impl eframe::App for MyApp {
//!     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//!         egui::CentralPanel::default().show(ctx, |ui| {
//!             Label::heading("Hello!").show(ui);
//!         });
//!     }
//! }
//! ```
//!
//! # `view!` — View struct + impl + Default in one block
//!
//! ```ignore
//! use egui_swift::prelude::*;
//!
//! view! {
//!     pub struct ProfileView {
//!         name: String = String::new(),
//!         email: String = String::new(),
//!     }
//!     fn body(&mut self, ui: &mut egui::Ui) {
//!         Label::heading("Profile").show(ui);
//!         Section::new().header("Info").show(ui, |ui| {
//!             TextField::new(&mut self.name).label("Name").show(ui);
//!             TextField::new(&mut self.email).label("Email").show(ui);
//!         });
//!     }
//! }
//! ```
//!
//! # `hstack!` / `vstack!` — Inline layout shorthand
//!
//! ```ignore
//! hstack!(ui, {
//!     Label::new("Left").show(ui);
//!     Spacer::trailing(ui, |ui| {
//!         Button::new("Right").show(ui);
//!     });
//! });
//!
//! vstack!(ui, spacing: 8.0, {
//!     Label::heading("Title").show(ui);
//!     Label::new("Subtitle").secondary().show(ui);
//! });
//! ```

/// Generate a `main()` function that launches an eframe app with macOS styling.
///
/// ```ignore
/// app!(MyApp, "Window Title", 900.0, 600.0);
/// ```
///
/// Expands to a `main()` that:
/// 1. Creates `NativeOptions` with the given window size and title
/// 2. Calls `apply_macos_style()` on the egui context
/// 3. Creates and runs `MyApp::default()`
///
/// Your `MyApp` struct must implement `Default` and `eframe::App`.
#[macro_export]
macro_rules! app {
    ($app_type:ty, $title:expr, $width:expr, $height:expr) => {
        fn main() -> eframe::Result {
            let options = eframe::NativeOptions {
                viewport: egui::ViewportBuilder::default()
                    .with_inner_size([$width, $height])
                    .with_title($title),
                ..Default::default()
            };
            eframe::run_native($title, options, Box::new(|cc| {
                $crate::theme::apply_macos_style(&cc.egui_ctx);
                Ok(Box::new(<$app_type>::default()))
            }))
        }
    };
    // Minimal variant — just type and title, default 800x600.
    ($app_type:ty, $title:expr) => {
        $crate::app!($app_type, $title, 800.0, 600.0);
    };
}

/// Define a View struct with fields, defaults, and a body implementation in one block.
///
/// ```ignore
/// view! {
///     pub struct SettingsView {
///         dark_mode: bool = false,
///         language: String = "en".to_string(),
///     }
///     fn body(&mut self, ui: &mut egui::Ui) {
///         Label::heading("Settings").show(ui);
///         Toggle::new(&mut self.dark_mode).label("Dark mode").show(ui);
///     }
/// }
/// ```
///
/// Expands to:
/// - The struct definition with all fields
/// - A `Default` implementation using the provided default values
/// - A `View` trait implementation with the body
#[macro_export]
macro_rules! view {
    // Public struct variant.
    (
        pub struct $name:ident {
            $( $field:ident : $ty:ty = $default:expr ),* $(,)?
        }
        fn body(&mut $self_:ident, $ui:ident : &mut egui::Ui) $body:block
    ) => {
        pub struct $name {
            $( pub $field: $ty ),*
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    $( $field: $default ),*
                }
            }
        }

        impl $crate::view::View for $name {
            fn body(&mut $self_, $ui: &mut egui::Ui) $body
        }
    };
    // Private struct variant.
    (
        struct $name:ident {
            $( $field:ident : $ty:ty = $default:expr ),* $(,)?
        }
        fn body(&mut $self_:ident, $ui:ident : &mut egui::Ui) $body:block
    ) => {
        struct $name {
            $( $field: $ty ),*
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    $( $field: $default ),*
                }
            }
        }

        impl $crate::view::View for $name {
            fn body(&mut $self_, $ui: &mut egui::Ui) $body
        }
    };
}

/// Shorthand for `HStack::new().show(ui, |ui| { ... })`.
///
/// ```ignore
/// hstack!(ui, {
///     Label::new("Left").show(ui);
///     Button::new("Right").show(ui);
/// });
///
/// // With spacing:
/// hstack!(ui, spacing: 12.0, {
///     Image::system_name("gear").show(ui);
///     Label::new("Settings").show(ui);
/// });
/// ```
#[macro_export]
macro_rules! hstack {
    ($ui:ident, { $($body:tt)* }) => {
        $crate::stacks::HStack::new().show($ui, |$ui| { $($body)* })
    };
    ($ui:ident, spacing: $spacing:expr, { $($body:tt)* }) => {
        $crate::stacks::HStack::new().spacing($spacing).show($ui, |$ui| { $($body)* })
    };
}

/// Shorthand for `VStack::new().show(ui, |ui| { ... })`.
///
/// ```ignore
/// vstack!(ui, {
///     Label::heading("Title").show(ui);
///     Label::new("Subtitle").secondary().show(ui);
/// });
///
/// // With spacing:
/// vstack!(ui, spacing: 8.0, {
///     Label::new("Item 1").show(ui);
///     Label::new("Item 2").show(ui);
/// });
/// ```
#[macro_export]
macro_rules! vstack {
    ($ui:ident, { $($body:tt)* }) => {
        $crate::stacks::VStack::new().show($ui, |$ui| { $($body)* })
    };
    ($ui:ident, spacing: $spacing:expr, { $($body:tt)* }) => {
        $crate::stacks::VStack::new().spacing($spacing).show($ui, |$ui| { $($body)* })
    };
}

/// SwiftUI-style `Section("Header") { content }`.
///
/// Maps directly to how Claude writes SwiftUI sections.
///
/// ```ignore
/// // SwiftUI:  Section("Appearance") { Toggle("Dark", isOn: $dark) }
/// // Macro:
/// section!(ui, "Appearance", {
///     Toggle::new(&mut self.dark).label("Dark mode").show(ui);
/// });
///
/// // With footer:
/// section!(ui, "Appearance", footer: "Changes apply immediately", {
///     Toggle::new(&mut self.dark).label("Dark mode").show(ui);
/// });
///
/// // No header:
/// section!(ui, {
///     LabeledContent::new("Version", "1.0").show(ui);
/// });
/// ```
#[macro_export]
macro_rules! section {
    // Section("Header") { content }
    ($ui:ident, $header:expr, { $($body:tt)* }) => {
        $crate::form_section::FormSection::new()
            .header($header)
            .show($ui, |$ui| { $($body)* })
    };
    // Section("Header", footer: "Footer") { content }
    ($ui:ident, $header:expr, footer: $footer:expr, { $($body:tt)* }) => {
        $crate::form_section::FormSection::new()
            .header($header)
            .footer($footer)
            .show($ui, |$ui| { $($body)* })
    };
    // Section { content } (no header)
    ($ui:ident, { $($body:tt)* }) => {
        $crate::form_section::FormSection::new()
            .show($ui, |$ui| { $($body)* })
    };
}

/// Quick spacer — `spacer!(ui)` for default or `spacer!(ui, 16.0)` for fixed.
///
/// ```ignore
/// spacer!(ui, 8.0);   // Spacer::fixed(8.0).show(ui)
/// spacer!(ui, 16.0);  // Spacer::fixed(16.0).show(ui)
/// ```
#[macro_export]
macro_rules! spacer {
    ($ui:ident, $size:expr) => {
        $crate::spacer::Spacer::fixed($size).show($ui)
    };
}

/// SwiftUI-style `Text("string").font(.preset).foregroundColor(.semantic)`.
///
/// Maps Claude's most common SwiftUI text patterns to one-line calls.
///
/// ```ignore
/// // SwiftUI: Text("Title").font(.title)
/// text!(ui, "Title", .title);
///
/// // SwiftUI: Text("Hint").font(.caption).foregroundColor(.secondary)
/// text!(ui, "Hint", .caption, .secondary);
///
/// // SwiftUI: Text("Error").foregroundColor(.red)
/// text!(ui, "Error", .destructive);
///
/// // Plain body text:
/// text!(ui, "Hello");
/// ```
#[macro_export]
macro_rules! text {
    // text!(ui, "str")
    ($ui:ident, $text:expr) => {
        $crate::label::Label::new($text).show($ui)
    };
    // text!(ui, "str", .title) — font only
    ($ui:ident, $text:expr, .largeTitle) => {
        $crate::label::Label::new($text).font($crate::typography::Font::LargeTitle).show($ui)
    };
    ($ui:ident, $text:expr, .title) => {
        $crate::label::Label::new($text).font($crate::typography::Font::Title).show($ui)
    };
    ($ui:ident, $text:expr, .headline) => {
        $crate::label::Label::new($text).font($crate::typography::Font::Headline).show($ui)
    };
    ($ui:ident, $text:expr, .body) => {
        $crate::label::Label::new($text).font($crate::typography::Font::Body).show($ui)
    };
    ($ui:ident, $text:expr, .callout) => {
        $crate::label::Label::new($text).font($crate::typography::Font::Callout).show($ui)
    };
    ($ui:ident, $text:expr, .subheadline) => {
        $crate::label::Label::new($text).font($crate::typography::Font::Subheadline).show($ui)
    };
    ($ui:ident, $text:expr, .caption) => {
        $crate::label::Label::new($text).font($crate::typography::Font::Caption).show($ui)
    };
    ($ui:ident, $text:expr, .footnote) => {
        $crate::label::Label::new($text).font($crate::typography::Font::Footnote).show($ui)
    };
    // text!(ui, "str", .secondary) — color only
    ($ui:ident, $text:expr, .secondary) => {
        $crate::label::Label::new($text).secondary().show($ui)
    };
    ($ui:ident, $text:expr, .muted) => {
        $crate::label::Label::new($text).muted().show($ui)
    };
    ($ui:ident, $text:expr, .accent) => {
        $crate::label::Label::new($text).accent().show($ui)
    };
    ($ui:ident, $text:expr, .destructive) => {
        $crate::label::Label::new($text).destructive().show($ui)
    };
    // text!(ui, "str", .caption, .secondary) — font + color
    ($ui:ident, $text:expr, .largeTitle, .secondary) => {
        $crate::label::Label::new($text).font($crate::typography::Font::LargeTitle).secondary().show($ui)
    };
    ($ui:ident, $text:expr, .title, .secondary) => {
        $crate::label::Label::new($text).font($crate::typography::Font::Title).secondary().show($ui)
    };
    ($ui:ident, $text:expr, .headline, .secondary) => {
        $crate::label::Label::new($text).font($crate::typography::Font::Headline).secondary().show($ui)
    };
    ($ui:ident, $text:expr, .body, .secondary) => {
        $crate::label::Label::new($text).font($crate::typography::Font::Body).secondary().show($ui)
    };
    ($ui:ident, $text:expr, .callout, .secondary) => {
        $crate::label::Label::new($text).font($crate::typography::Font::Callout).secondary().show($ui)
    };
    ($ui:ident, $text:expr, .subheadline, .secondary) => {
        $crate::label::Label::new($text).font($crate::typography::Font::Subheadline).secondary().show($ui)
    };
    ($ui:ident, $text:expr, .caption, .secondary) => {
        $crate::label::Label::new($text).font($crate::typography::Font::Caption).secondary().show($ui)
    };
    ($ui:ident, $text:expr, .footnote, .secondary) => {
        $crate::label::Label::new($text).font($crate::typography::Font::Footnote).secondary().show($ui)
    };
    ($ui:ident, $text:expr, .caption, .muted) => {
        $crate::label::Label::new($text).font($crate::typography::Font::Caption).muted().show($ui)
    };
    ($ui:ident, $text:expr, .subheadline, .muted) => {
        $crate::label::Label::new($text).font($crate::typography::Font::Subheadline).muted().show($ui)
    };
    ($ui:ident, $text:expr, .callout, .muted) => {
        $crate::label::Label::new($text).font($crate::typography::Font::Callout).muted().show($ui)
    };
    ($ui:ident, $text:expr, .body, .destructive) => {
        $crate::label::Label::new($text).font($crate::typography::Font::Body).destructive().show($ui)
    };
    ($ui:ident, $text:expr, .caption, .destructive) => {
        $crate::label::Label::new($text).font($crate::typography::Font::Caption).destructive().show($ui)
    };
    ($ui:ident, $text:expr, .callout, .accent) => {
        $crate::label::Label::new($text).font($crate::typography::Font::Callout).accent().show($ui)
    };
}
