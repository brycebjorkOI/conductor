//! SwiftUI-compatible type aliases.
//!
//! These aliases let you use SwiftUI naming conventions directly in Rust.
//! Claude (and any developer fluent in SwiftUI) can write code that reads
//! like SwiftUI without learning a new vocabulary.
//!
//! ```ignore
//! use egui_swift::prelude::*;
//!
//! // SwiftUI-style naming "just works":
//! Text::new("Settings").font(Font::Title).show(ui);
//! Section::new().header("Behavior").show(ui, |ui| { ... });
//! GroupBox::new().show(ui, |ui| { ... });
//! ProgressView::spinner().show(ui);
//! ContentUnavailableView::new("No results").icon("🔍").show(ui);
//! ```

/// `Text` — SwiftUI alias for [`Label`](crate::label::Label).
///
/// In SwiftUI: `Text("Hello").font(.title).foregroundColor(.secondary)`
/// In egui-swift: `Text::new("Hello").font(Font::Title).secondary().show(ui)`
pub type Text<'a> = crate::label::Label<'a>;

/// `Section` — SwiftUI alias for [`FormSection`](crate::form_section::FormSection).
///
/// In SwiftUI: `Section("Header") { content }`
/// In egui-swift: `Section::new().header("Header").show(ui, |ui| { content })`
pub type Section<'a> = crate::form_section::FormSection<'a>;

/// `GroupBox` — SwiftUI alias for [`Card`](crate::card::Card).
///
/// In SwiftUI: `GroupBox("Title") { content }`
/// In egui-swift: `GroupBox::new().show(ui, |ui| { content })`
pub type GroupBox = crate::card::Card;

/// `ProgressView` — SwiftUI alias for [`ProgressIndicator`](crate::progress_indicator::ProgressIndicator).
///
/// In SwiftUI: `ProgressView()` or `ProgressView(value: 0.5)`
/// In egui-swift: `ProgressView::spinner().show(ui)` or `ProgressView::bar(0.5).show(ui)`
pub type ProgressView = crate::progress_indicator::ProgressIndicator;

/// `ContentUnavailableView` — SwiftUI alias for [`EmptyState`](crate::empty_state::EmptyState).
///
/// In SwiftUI: `ContentUnavailableView("No results", systemImage: "magnifyingglass")`
/// In egui-swift: `ContentUnavailableView::new("No results").icon("🔍").show(ui)`
pub type ContentUnavailableView<'a> = crate::empty_state::EmptyState<'a>;
