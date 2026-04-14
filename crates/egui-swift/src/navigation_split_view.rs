//! SwiftUI-style NavigationSplitView — sidebar + detail layout.
//!
//! ```ignore
//! NavigationSplitView::new("settings_nav")
//!     .sidebar_width(160.0)
//!     .show(ctx, |sidebar, detail| {
//!         sidebar.show(|ui| {
//!             Label::heading("Settings").show(ui);
//!             NavRow::new("General").icon("⚙").active(true).show(ui);
//!         });
//!         detail.show(|ui| {
//!             general::show(ui);
//!         });
//!     });
//! ```

use crate::colors;

/// A sidebar + detail split layout matching SwiftUI's `NavigationSplitView`.
pub struct NavigationSplitView {
    id: String,
    sidebar_width: f32,
    traffic_light_pad: bool,
}

/// Handle for rendering the sidebar region.
pub struct SidebarBuilder<'a> {
    ctx: &'a egui::Context,
    p: crate::colors::Palette,
    sidebar_width: f32,
    traffic_light_pad: bool,
    id: String,
}

/// Handle for rendering the detail region.
pub struct DetailBuilder<'a> {
    ctx: &'a egui::Context,
    p: crate::colors::Palette,
}

impl NavigationSplitView {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            sidebar_width: 160.0,
            traffic_light_pad: true,
        }
    }

    pub fn sidebar_width(mut self, w: f32) -> Self {
        self.sidebar_width = w;
        self
    }

    /// Whether to add top padding for macOS traffic light buttons (default: true).
    pub fn traffic_light_pad(mut self, pad: bool) -> Self {
        self.traffic_light_pad = pad;
        self
    }

    /// Show the split view. The closure receives sidebar and detail builders.
    pub fn show(
        self,
        ctx: &egui::Context,
        build: impl FnOnce(&mut SidebarBuilder<'_>, &mut DetailBuilder<'_>),
    ) {
        let p = colors::palette_from_ctx(ctx);
        let mut sidebar = SidebarBuilder {
            ctx,
            p,
            sidebar_width: self.sidebar_width,
            traffic_light_pad: self.traffic_light_pad,
            id: self.id,
        };
        let mut detail = DetailBuilder { ctx, p };
        build(&mut sidebar, &mut detail);
    }
}

impl<'a> SidebarBuilder<'a> {
    /// Render the sidebar panel.
    pub fn show(&self, content: impl FnOnce(&mut egui::Ui)) {
        let id = format!("{}_sidebar", self.id);
        egui::SidePanel::left(id)
            .resizable(false)
            .default_width(self.sidebar_width)
            .frame(
                egui::Frame::NONE
                    .fill(self.p.sidebar_bg)
                    .inner_margin(egui::Margin::symmetric(8, 12)),
            )
            .show(self.ctx, |ui| {
                if self.traffic_light_pad {
                    ui.add_space(28.0);
                }
                content(ui);
            });
    }
}

impl<'a> DetailBuilder<'a> {
    /// Render the detail (central) panel with a scroll area.
    pub fn show(&self, content: impl FnOnce(&mut egui::Ui)) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(self.p.surface)
                    .inner_margin(egui::Margin::symmetric(24, 20)),
            )
            .show(self.ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        content(ui);
                    });
            });
    }

    /// Render the detail panel without a scroll area.
    pub fn show_plain(&self, content: impl FnOnce(&mut egui::Ui)) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::NONE
                    .fill(self.p.surface)
                    .inner_margin(egui::Margin::symmetric(24, 20)),
            )
            .show(self.ctx, |ui| {
                content(ui);
            });
    }
}
