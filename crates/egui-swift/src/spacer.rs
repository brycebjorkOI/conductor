//! SwiftUI-style Spacer.
//!
//! In egui's immediate-mode layout, a flexible spacer can't silently push
//! later siblings to the trailing edge (there's no two-pass layout). Instead,
//! `Spacer` provides two modes:
//!
//! ## Fixed spacing (drop-in for `ui.add_space()`)
//! ```ignore
//! Spacer::fixed(16.0).show(ui);
//! ```
//!
//! ## Flexible push with trailing content
//! ```ignore
//! HStack::new().show(ui, |ui| {
//!     Label::new("Left").show(ui);
//!     Spacer::trailing(ui, |ui| {
//!         Label::new("Right").show(ui);
//!     });
//! });
//! ```

pub struct Spacer;

impl Spacer {
    /// Fixed-size spacer — syntactic sugar for `ui.add_space(points)`.
    pub fn fixed(points: f32) -> FixedSpacer {
        FixedSpacer(points)
    }

    /// Push remaining content to the trailing edge (right in LTR layouts).
    ///
    /// This is the egui equivalent of SwiftUI's `Spacer()` — place it
    /// between leading and trailing content inside an HStack.
    ///
    /// ```ignore
    /// HStack::new().show(ui, |ui| {
    ///     Label::new("Title").show(ui);
    ///     Spacer::trailing(ui, |ui| {
    ///         Button::new("Action").show(ui);
    ///     });
    /// });
    /// ```
    pub fn trailing(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), content);
    }

    /// Push remaining content to the bottom.
    ///
    /// Useful inside a VStack to push footer content to the bottom edge.
    pub fn bottom(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) {
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), content);
    }
}

/// A fixed-size spacer.
pub struct FixedSpacer(f32);

impl FixedSpacer {
    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let (_, response) = ui.allocate_exact_size(
            egui::vec2(0.0, self.0),
            egui::Sense::hover(),
        );
        ui.add_space(self.0);
        response
    }
}
