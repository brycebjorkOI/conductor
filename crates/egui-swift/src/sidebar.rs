//! Sidebar layout helper — provides the outer frame with correct background
//! color, traffic-light padding, and a right-edge separator.
//!
//! ```no_run
//! # let ctx: &egui::Context = todo!();
//! conductor_ui::sidebar::SidebarPanel::new()
//!     .show(ctx, |ui| {
//!         // draw sidebar content here
//!     });
//! ```

use crate::colors;

pub struct SidebarPanel {
    width: f32,
    traffic_light_pad: f32,
}

impl SidebarPanel {
    pub fn new() -> Self {
        Self {
            width: 260.0,
            traffic_light_pad: 38.0,
        }
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    /// Top padding for macOS traffic-light buttons.  Set to 0 on other
    /// platforms or when using a native title bar.
    pub fn traffic_light_pad(mut self, px: f32) -> Self {
        self.traffic_light_pad = px;
        self
    }

    pub fn show(self, ctx: &egui::Context, content: impl FnOnce(&mut egui::Ui)) {
        let dark = ctx.style().visuals.dark_mode;
        let p = if dark { colors::dark() } else { colors::light() };

        egui::SidePanel::left("cui_sidebar")
            .resizable(false)
            .exact_width(self.width)
            .frame(
                egui::Frame::NONE
                    .fill(p.sidebar_bg)
                    .inner_margin(egui::Margin::same(0))
                    .stroke(egui::Stroke::new(0.5, p.border_subtle)),
            )
            .show(ctx, |ui| {
                ui.add_space(self.traffic_light_pad);
                content(ui);
            });
    }
}

impl Default for SidebarPanel {
    fn default() -> Self {
        Self::new()
    }
}
