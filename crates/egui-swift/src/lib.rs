//! `egui-swift` — A generic SwiftUI-style UI framework built on top of egui.
//!
//! This crate provides styled, composable UI widgets that match the look and
//! feel of a native macOS / SwiftUI application. Use SwiftUI naming via the
//! compatibility aliases or the native egui-swift names — both work.
//!
//! # Quick start
//!
//! ```ignore
//! use egui_swift::prelude::*;
//!
//! fn my_view(ui: &mut egui::Ui) {
//!     let p = ui.palette();
//!     Text::new("Settings").font(Font::Title).show(ui);
//!     Section::new().header("Behavior").show(ui, |ui| {
//!         Toggle::new(&mut value).label("Dark mode").show(ui);
//!     });
//!     Image::system_name("gear").show(ui);
//! }
//! ```

// Prelude (single import for everything)
pub mod prelude;

// SwiftUI compatibility aliases
pub mod swiftui_compat;

// Infrastructure
pub mod colors;
pub mod ext;
pub mod helpers;
pub mod icons;
pub mod theme;
pub mod typography;

// Text & images
pub mod image;
pub mod label;

// Leaf controls
pub mod button;
pub mod divider;
pub mod status_dot;
pub mod toggle;

// Containers
pub mod card;
pub mod form_row;
pub mod form_section;

// Input controls
pub mod picker;
pub mod radio_group;
pub mod stepper;
pub mod text_field;

// Composites
pub mod button_row;
pub mod data_table;
pub mod disclosure_group;
pub mod empty_state;

// Navigation
pub mod navigation_split_view;

// Advanced
pub mod context_menu;
pub mod progress_indicator;
pub mod sheet;
pub mod toolbar;

// Existing components
pub mod badge;
pub mod chat_input;
pub mod conversation_item;
pub mod nav_row;
pub mod search_field;
pub mod section_header;
pub mod segmented_control;
pub mod sidebar;
pub mod suggestion_chip;
pub mod user_profile;
