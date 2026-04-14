//! Collapsible section with animated chevron.
//!
//! ```ignore
//! DisclosureGroup::new("Run History", &mut expanded).show(ui, |ui| { ... });
//! ```

use crate::colors;
use crate::helpers;

const ANIMATION_SECS: f32 = 0.15;

pub struct DisclosureGroup<'a> {
    label: &'a str,
    expanded: &'a mut bool,
    icon: Option<&'a str>,
}

impl<'a> DisclosureGroup<'a> {
    pub fn new(label: &'a str, expanded: &'a mut bool) -> Self {
        Self {
            label,
            expanded,
            icon: None,
        }
    }

    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn show(
        self,
        ui: &mut egui::Ui,
        content: impl FnOnce(&mut egui::Ui),
    ) -> egui::Response {
        let p = colors::palette(ui);
        let id = ui.auto_id_with(self.label);

        // Header row.
        let (header_rect, header_resp) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 28.0), egui::Sense::click());

        if header_resp.clicked() {
            *self.expanded = !*self.expanded;
        }

        let t = helpers::animate_bool(ui, id, *self.expanded, ANIMATION_SECS);

        if ui.is_rect_visible(header_rect) {
            let painter = ui.painter();

            // Hover background.
            if header_resp.hovered() {
                painter.rect_filled(
                    header_rect,
                    egui::CornerRadius::same(4),
                    p.hover_bg,
                );
            }

            // Animated chevron: right (›) when collapsed, down (⌄) when expanded.
            // We interpolate the rotation angle.
            let chevron_center = egui::pos2(header_rect.left() + 10.0, header_rect.center().y);
            let angle = t * std::f32::consts::FRAC_PI_2; // 0 → 90°

            // Draw a simple ">" shape that rotates.
            let size = 4.0;
            let cos = angle.cos();
            let sin = angle.sin();
            let points = [
                egui::vec2(-size * 0.4, -size),
                egui::vec2(size * 0.4, 0.0),
                egui::vec2(-size * 0.4, size),
            ];
            let rotated: Vec<egui::Pos2> = points
                .iter()
                .map(|p| {
                    egui::pos2(
                        chevron_center.x + p.x * cos - p.y * sin,
                        chevron_center.y + p.x * sin + p.y * cos,
                    )
                })
                .collect();
            painter.line(rotated, egui::Stroke::new(1.5, p.text_secondary));

            // Optional icon.
            let mut text_x = header_rect.left() + 22.0;
            if let Some(icon) = self.icon {
                painter.text(
                    egui::pos2(text_x, header_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    icon,
                    egui::FontId::proportional(13.0),
                    p.text_secondary,
                );
                text_x += 20.0;
            }

            // Label.
            painter.text(
                egui::pos2(text_x, header_rect.center().y),
                egui::Align2::LEFT_CENTER,
                self.label,
                egui::FontId::proportional(13.0),
                p.text_primary,
            );
        }

        // Content (shown when expanded, with animation).
        if t > 0.01 {
            // Simple show/hide — full height animation would require measuring
            // content height ahead of time which egui doesn't support natively.
            content(ui);
        }

        header_resp
    }
}
