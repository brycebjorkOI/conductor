//! Modal sheet / dialog panel with backdrop and animated fade-in.
//!
//! ```ignore
//! Sheet::new("add_job", &mut open, "New Job").width(480.0).show(ctx, |ui| { ... });
//! ```

use crate::colors;
use crate::ext::ColorExt;
use crate::helpers;

const ANIMATION_SECS: f32 = 0.2;

pub struct Sheet<'a> {
    id: egui::Id,
    open: &'a mut bool,
    title: &'a str,
    width: f32,
}

impl<'a> Sheet<'a> {
    pub fn new(id: impl Into<egui::Id>, open: &'a mut bool, title: &'a str) -> Self {
        Self {
            id: id.into(),
            open,
            title,
            width: 480.0,
        }
    }

    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    pub fn show(self, ctx: &egui::Context, content: impl FnOnce(&mut egui::Ui)) {
        let p = colors::palette_from_ctx(ctx);
        let t = ctx.animate_bool_with_time(self.id, *self.open, ANIMATION_SECS);

        if t <= 0.0 {
            return;
        }

        // Semi-transparent backdrop.
        let screen = ctx.screen_rect();
        let backdrop_alpha = (p.overlay_bg.a() as f32 * t) as u8;

        egui::Area::new(self.id.with("backdrop"))
            .fixed_pos(screen.min)
            .show(ctx, |ui| {
                let (rect, response) =
                    ui.allocate_exact_size(screen.size(), egui::Sense::click());

                if response.clicked() {
                    *self.open = false;
                }

                ui.painter().rect_filled(
                    rect,
                    egui::CornerRadius::ZERO,
                    egui::Color32::BLACK.opacity(backdrop_alpha as f32 / 255.0),
                );
            });

        // Sheet card, centered.
        let sheet_height = screen.height() * 0.7;
        let rounding = egui::CornerRadius::same(16);

        egui::Area::new(self.id.with("sheet"))
            .fixed_pos(egui::pos2(
                (screen.width() - self.width) / 2.0,
                (screen.height() - sheet_height) / 2.0,
            ))
            .show(ctx, |ui| {
                // Shadow.
                let sheet_rect = egui::Rect::from_min_size(
                    ui.cursor().min,
                    egui::vec2(self.width, sheet_height),
                );
                helpers::paint_shadow(ui, sheet_rect, rounding, 8.0, p.shadow);

                egui::Frame::NONE
                    .fill(p.surface)
                    .corner_radius(rounding)
                    .stroke(egui::Stroke::new(0.5, p.border))
                    .inner_margin(egui::Margin::symmetric(20, 16))
                    .show(ui, |ui| {
                        ui.set_width(self.width - 40.0);
                        ui.set_max_height(sheet_height - 32.0);

                        // Title bar.
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(self.title)
                                    .size(15.0)
                                    .strong()
                                    .color(p.text_primary),
                            );

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .add(
                                            egui::Button::new(
                                                egui::RichText::new("\u{2715}")
                                                    .size(14.0)
                                                    .color(p.text_muted),
                                            )
                                            .fill(egui::Color32::TRANSPARENT)
                                            .frame(false),
                                        )
                                        .clicked()
                                    {
                                        *self.open = false;
                                    }
                                },
                            );
                        });

                        ui.add_space(8.0);
                        crate::divider::Divider::new().show(ui);
                        ui.add_space(8.0);

                        // Content area.
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            content(ui);
                        });
                    });
            });
    }
}
