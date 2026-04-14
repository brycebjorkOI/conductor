//! Small colored status indicator dot with optional label.
//!
//! ```ignore
//! StatusDot::new(palette.status_green).label("Connected").show(ui);
//! ```

pub struct StatusDot<'a> {
    color: egui::Color32,
    label: Option<&'a str>,
    size: f32,
}

impl<'a> StatusDot<'a> {
    pub fn new(color: egui::Color32) -> Self {
        Self {
            color,
            label: None,
            size: 8.0,
        }
    }

    pub fn label(mut self, text: &'a str) -> Self {
        self.label = Some(text);
        self
    }

    pub fn size(mut self, px: f32) -> Self {
        self.size = px;
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> egui::Response {
        let (dot_rect, response) =
            ui.allocate_exact_size(egui::vec2(self.size, self.size), egui::Sense::hover());

        if ui.is_rect_visible(dot_rect) {
            ui.painter().circle_filled(
                dot_rect.center(),
                self.size / 2.0,
                self.color,
            );
        }

        if let Some(text) = self.label {
            ui.label(
                egui::RichText::new(text)
                    .size(12.0)
                    .color(crate::colors::palette(ui).text_secondary),
            );
        }

        response
    }
}
