//! Horizontal action button group, right-aligned.
//!
//! ```ignore
//! ButtonRow::show(ui, |ui| {
//!     Button::new("Create").style(ButtonStyle::BorderedProminent).show(ui);
//!     Button::new("Cancel").style(ButtonStyle::Bordered).show(ui);
//! });
//! ```

pub struct ButtonRow;

impl ButtonRow {
    pub fn show(ui: &mut egui::Ui, buttons: impl FnOnce(&mut egui::Ui)) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            buttons(ui);
        });
    }
}
