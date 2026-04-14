//! Small count badge (pill with number).
//!
//! ```no_run
//! # let ui: &mut egui::Ui = todo!();
//! conductor_ui::badge::Badge::new(20).show(ui);
//! ```

use crate::colors;

pub struct Badge {
    count: u32,
}

impl Badge {
    pub fn new(count: u32) -> Self {
        Self { count }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        let p = colors::palette(ui);
        let text = format!("{}", self.count);
        let font = egui::FontId::proportional(10.5);

        // Measure text width to size the pill.
        let galley = ui.painter().layout_no_wrap(text.clone(), font.clone(), p.text_muted);
        let text_width = galley.size().x;
        let pill_width = (text_width + 12.0).max(20.0);
        let pill_height = 16.0;

        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(pill_width, pill_height),
            egui::Sense::hover(),
        );

        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same((pill_height / 2.0) as u8),
            p.surface_raised,
        );

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            &text,
            font,
            p.text_muted,
        );
    }
}
