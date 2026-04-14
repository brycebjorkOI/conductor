//! Spinner and progress bar indicators.
//!
//! ```ignore
//! ProgressIndicator::spinner().size(20.0).show(ui);
//! ProgressIndicator::bar(0.65).show(ui);
//! ```

use crate::colors;

pub enum ProgressStyle {
    Spinner,
    Bar(f32), // 0.0..1.0
}

pub struct ProgressIndicator {
    style: ProgressStyle,
    size: f32,
}

impl ProgressIndicator {
    pub fn spinner() -> Self {
        Self {
            style: ProgressStyle::Spinner,
            size: 20.0,
        }
    }

    pub fn bar(progress: f32) -> Self {
        Self {
            style: ProgressStyle::Bar(progress.clamp(0.0, 1.0)),
            size: 4.0, // bar height
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let p = colors::palette(ui);

        match self.style {
            ProgressStyle::Spinner => {
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(self.size, self.size),
                    egui::Sense::hover(),
                );

                if ui.is_rect_visible(rect) {
                    let time = ui.input(|i| i.time);
                    let painter = ui.painter();
                    let center = rect.center();
                    let radius = self.size / 2.0 - 2.0;
                    let segments = 8;

                    for i in 0..segments {
                        let angle = (i as f64 / segments as f64) * std::f64::consts::TAU + time * 4.0;
                        let alpha = ((segments - i) as f32 / segments as f32 * 200.0) as u8;
                        let color = egui::Color32::from_rgba_premultiplied(
                            p.text_secondary.r(),
                            p.text_secondary.g(),
                            p.text_secondary.b(),
                            alpha,
                        );

                        let pos = egui::pos2(
                            center.x + (angle.cos() as f32) * radius,
                            center.y + (angle.sin() as f32) * radius,
                        );
                        painter.circle_filled(pos, 1.5, color);
                    }

                    ui.ctx().request_repaint();
                }
            }
            ProgressStyle::Bar(progress) => {
                let available_width = ui.available_width();
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(available_width, self.size),
                    egui::Sense::hover(),
                );

                if ui.is_rect_visible(rect) {
                    let painter = ui.painter();
                    let rounding = egui::CornerRadius::same((self.size / 2.0) as u8);

                    // Track.
                    painter.rect_filled(rect, rounding, p.border_subtle);

                    // Fill.
                    let fill_rect = egui::Rect::from_min_size(
                        rect.min,
                        egui::vec2(rect.width() * progress, rect.height()),
                    );
                    painter.rect_filled(fill_rect, rounding, p.accent);
                }
            }
        }
    }
}
