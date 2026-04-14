//! Convenience re-exports for common usage.
//!
//! ```ignore
//! use egui_swift::prelude::*;
//! ```
//!
//! This gives you access to all commonly used types — including SwiftUI
//! compatibility aliases — without individual imports.

// Core trait
pub use crate::view::View;

// Extension traits (must be in scope to use .palette(), .centered_content(), .opacity())
pub use crate::ext::{ColorExt, CtxExt, UiExt};

// Typography
pub use crate::typography::Font;
pub use crate::label::Label;

// Images (SF Symbols)
pub use crate::image::Image;

// Colors
pub use crate::colors::{self, Palette};

// Icons
pub use crate::icons;

// Theme / layout
pub use crate::theme::Layout;

// Leaf controls
pub use crate::button::{Button, ButtonStyle};
pub use crate::divider::Divider;
pub use crate::status_dot::StatusDot;
pub use crate::toggle::Toggle;

// Containers
pub use crate::card::Card;
pub use crate::form::Form;
pub use crate::form_row::FormRow;
pub use crate::form_section::FormSection;

// Input controls
pub use crate::picker::Picker;
pub use crate::radio_group::RadioGroup;
pub use crate::stepper::Stepper;
pub use crate::text_field::TextField;

// Composites
pub use crate::button_row::ButtonRow;
pub use crate::data_table::DataTable;
pub use crate::disclosure_group::DisclosureGroup;
pub use crate::empty_state::{EmptyState, EmptyStateResponse};

// Layout primitives
pub use crate::labeled_content::LabeledContent;
pub use crate::scroll_view::ScrollView;
pub use crate::spacer::{FixedSpacer, Spacer};
pub use crate::stacks::{HStack, StackExt, VStack};

// Lists & tables
pub use crate::list::{List, ListStyle};

// Navigation
pub use crate::navigation_split_view::NavigationSplitView;
pub use crate::navigation_stack::{NavController, NavPath, NavigationStack};
pub use crate::tab_view::TabView;

// Advanced
pub use crate::alert::{Alert, AlertAction};
pub use crate::context_menu::MenuItem;
pub use crate::progress_indicator::{ProgressIndicator, ProgressStyle};
pub use crate::sheet::Sheet;
pub use crate::toolbar::Toolbar;

// SwiftUI compatibility aliases
pub use crate::swiftui_compat::{
    ContentUnavailableView, GroupBox, ProgressView, Section, Text,
};

// Existing components
pub use crate::badge::Badge;
pub use crate::chat_input::{ChatInput, ChatInputResponse};
pub use crate::conversation_item::ConversationItem;
pub use crate::nav_row::NavRow;
pub use crate::search_field::SearchField;
pub use crate::section_header::SectionHeader;
pub use crate::segmented_control::SegmentedControl;
pub use crate::sidebar::SidebarPanel;
pub use crate::suggestion_chip::{self, SuggestionChip};
pub use crate::user_profile::{UserProfile, UserProfileResponse};
