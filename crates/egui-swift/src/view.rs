//! SwiftUI-style `View` trait for composable, self-contained UI components.
//!
//! Every screen, panel, or reusable widget can be a struct that implements
//! `View`. This mirrors SwiftUI's `View` protocol and gives Claude a
//! standard pattern for structuring apps.
//!
//! ```ignore
//! use egui_swift::prelude::*;
//!
//! struct ProfileView {
//!     name: String,
//!     email: String,
//! }
//!
//! impl View for ProfileView {
//!     fn body(&mut self, ui: &mut egui::Ui) {
//!         Label::heading("Profile").show(ui);
//!         Section::new().header("Info").show(ui, |ui| {
//!             TextField::new(&mut self.name).label("Name").show(ui);
//!             TextField::new(&mut self.email).label("Email").show(ui);
//!         });
//!     }
//! }
//!
//! // In your App::update():
//! self.profile_view.show(ui);
//! ```
//!
//! Views compose naturally — one view can embed another:
//! ```ignore
//! struct SettingsView {
//!     general: GeneralView,
//!     account: AccountView,
//! }
//!
//! impl View for SettingsView {
//!     fn body(&mut self, ui: &mut egui::Ui) {
//!         self.general.show(ui);
//!         Divider::new().show(ui);
//!         self.account.show(ui);
//!     }
//! }
//! ```

/// A composable UI component with its own state and rendering logic.
///
/// This is the egui-swift equivalent of SwiftUI's `View` protocol.
/// Implement `body()` to define what the view renders.
/// Call `.show(ui)` to render it.
pub trait View {
    /// Define the view's content. This is called every frame (immediate mode).
    fn body(&mut self, ui: &mut egui::Ui);

    /// Render this view into the given `Ui`. Default impl just calls `body()`.
    ///
    /// Override this to wrap content in a frame, scroll area, or other container.
    fn show(&mut self, ui: &mut egui::Ui) {
        self.body(ui);
    }
}
