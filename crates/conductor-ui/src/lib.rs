//! `conductor-ui` — Reusable SwiftUI-style egui components.
//!
//! This crate provides styled, composable UI widgets that match the look and
//! feel of a modern macOS chat application (dark mode, rounded elements,
//! subtle hover/active states).
//!
//! All components are pure functions or builder structs that take `&mut egui::Ui`
//! and return `egui::Response` where applicable.

pub mod colors;
pub mod chat_input;
pub mod nav_row;
pub mod search_field;
pub mod section_header;
pub mod segmented_control;
pub mod suggestion_chip;
pub mod conversation_item;
pub mod user_profile;
pub mod sidebar;
pub mod badge;
