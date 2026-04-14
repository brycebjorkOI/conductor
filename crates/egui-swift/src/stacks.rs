//! SwiftUI-style VStack / HStack with chainable modifiers.
//!
//! ```ignore
//! VStack::new()
//!     .spacing(8.0)
//!     .padding(12.0)
//!     .background(p.surface_raised)
//!     .corner_radius(10.0)
//!     .border(0.5, p.border)
//!     .show(ui, |ui| {
//!         Label::heading("Title").show(ui);
//!         Label::new("Subtitle").secondary().show(ui);
//!     });
//!
//! HStack::new().spacing(8.0).show(ui, |ui| {
//!     Image::system_name("gear").show(ui);
//!     Label::new("Settings").show(ui);
//! });
//! ```

/// Shared visual modifiers for stack containers.
#[derive(Default)]
pub struct StackModifiers {
    spacing: Option<f32>,
    padding: Option<egui::Margin>,
    background: Option<egui::Color32>,
    corner_radius: Option<f32>,
    border: Option<(f32, egui::Color32)>,
    frame_width: Option<f32>,
    frame_height: Option<f32>,
}

impl StackModifiers {
    fn has_frame(&self) -> bool {
        self.padding.is_some()
            || self.background.is_some()
            || self.corner_radius.is_some()
            || self.border.is_some()
    }

    fn to_frame(&self) -> egui::Frame {
        let mut frame = egui::Frame::NONE;
        if let Some(bg) = self.background {
            frame = frame.fill(bg);
        }
        if let Some(r) = self.corner_radius {
            frame = frame.corner_radius(egui::CornerRadius::same(r as u8));
        }
        if let Some((w, c)) = self.border {
            frame = frame.stroke(egui::Stroke::new(w, c));
        }
        if let Some(p) = self.padding {
            frame = frame.inner_margin(p);
        }
        frame
    }
}

// ---------------------------------------------------------------------------
// Shared modifier trait — avoids duplicating 7 method impls on each type.
// ---------------------------------------------------------------------------

/// Methods shared by VStack and HStack.
pub trait StackExt: Sized {
    #[doc(hidden)]
    fn modifiers_mut(&mut self) -> &mut StackModifiers;

    /// Set spacing between children (points).
    fn spacing(mut self, s: f32) -> Self {
        self.modifiers_mut().spacing = Some(s);
        self
    }

    /// Uniform padding on all sides.
    fn padding(mut self, p: f32) -> Self {
        self.modifiers_mut().padding = Some(egui::Margin::same(p as i8));
        self
    }

    /// Asymmetric padding (horizontal, vertical).
    fn padding_xy(mut self, h: f32, v: f32) -> Self {
        self.modifiers_mut().padding = Some(egui::Margin::symmetric(h as i8, v as i8));
        self
    }

    /// Background fill color.
    fn background(mut self, c: egui::Color32) -> Self {
        self.modifiers_mut().background = Some(c);
        self
    }

    /// Corner radius for the background.
    fn corner_radius(mut self, r: f32) -> Self {
        self.modifiers_mut().corner_radius = Some(r);
        self
    }

    /// Border stroke.
    fn border(mut self, width: f32, color: egui::Color32) -> Self {
        self.modifiers_mut().border = Some((width, color));
        self
    }

    /// Explicit width constraint.
    fn frame_width(mut self, w: f32) -> Self {
        self.modifiers_mut().frame_width = Some(w);
        self
    }

    /// Explicit height constraint.
    fn frame_height(mut self, h: f32) -> Self {
        self.modifiers_mut().frame_height = Some(h);
        self
    }
}

// ---------------------------------------------------------------------------
// VStack
// ---------------------------------------------------------------------------

/// Vertical stack with optional visual modifiers.
pub struct VStack {
    mods: StackModifiers,
}

impl VStack {
    pub fn new() -> Self {
        Self {
            mods: StackModifiers::default(),
        }
    }

    pub fn show(
        self,
        ui: &mut egui::Ui,
        content: impl FnOnce(&mut egui::Ui),
    ) -> egui::Response {
        let apply = |ui: &mut egui::Ui| {
            if let Some(w) = self.mods.frame_width {
                ui.set_width(w);
            }
            if let Some(h) = self.mods.frame_height {
                ui.set_height(h);
            }
            if let Some(s) = self.mods.spacing {
                ui.spacing_mut().item_spacing.y = s;
            }
            ui.vertical(|ui| content(ui))
        };

        if self.mods.has_frame() {
            self.mods.to_frame().show(ui, |ui| { apply(ui); }).response
        } else {
            apply(ui).response
        }
    }
}

impl Default for VStack {
    fn default() -> Self {
        Self::new()
    }
}

impl StackExt for VStack {
    fn modifiers_mut(&mut self) -> &mut StackModifiers {
        &mut self.mods
    }
}

// ---------------------------------------------------------------------------
// HStack
// ---------------------------------------------------------------------------

/// Horizontal stack with optional visual modifiers.
pub struct HStack {
    mods: StackModifiers,
}

impl HStack {
    pub fn new() -> Self {
        Self {
            mods: StackModifiers::default(),
        }
    }

    pub fn show(
        self,
        ui: &mut egui::Ui,
        content: impl FnOnce(&mut egui::Ui),
    ) -> egui::Response {
        let apply = |ui: &mut egui::Ui| {
            if let Some(w) = self.mods.frame_width {
                ui.set_width(w);
            }
            if let Some(h) = self.mods.frame_height {
                ui.set_height(h);
            }
            if let Some(s) = self.mods.spacing {
                ui.spacing_mut().item_spacing.x = s;
            }
            ui.horizontal(|ui| content(ui))
        };

        if self.mods.has_frame() {
            self.mods.to_frame().show(ui, |ui| { apply(ui); }).response
        } else {
            apply(ui).response
        }
    }
}

impl Default for HStack {
    fn default() -> Self {
        Self::new()
    }
}

impl StackExt for HStack {
    fn modifiers_mut(&mut self) -> &mut StackModifiers {
        &mut self.mods
    }
}
