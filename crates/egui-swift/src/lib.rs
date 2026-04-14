//! `egui-swift` — A generic SwiftUI-style UI framework built on top of egui.
//!
//! This crate provides styled, composable UI widgets that match the look and
//! feel of a native macOS / SwiftUI application (dark mode, rounded elements,
//! subtle hover/active states, animations).
//!
//! All components are pure functions or builder structs that take `&mut egui::Ui`
//! and return `egui::Response` where applicable.

// Infrastructure
pub mod colors;
pub mod helpers;
pub mod icons;
pub mod theme;

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
