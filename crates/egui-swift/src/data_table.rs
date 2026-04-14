//! Striped data table with bold header row.
//!
//! ```ignore
//! DataTable::new(&[("Name", 200.0), ("Messages", 80.0)])
//!     .striped(true)
//!     .show(ui, |ui| {
//!         // Use ui.label() and ui.end_row() for each row.
//!     });
//! ```

use crate::colors;

pub struct DataTable<'a> {
    columns: &'a [(&'a str, f32)],
    striped: bool,
}

impl<'a> DataTable<'a> {
    pub fn new(columns: &'a [(&'a str, f32)]) -> Self {
        Self {
            columns,
            striped: true,
        }
    }

    pub fn striped(mut self, s: bool) -> Self {
        self.striped = s;
        self
    }

    pub fn show(self, ui: &mut egui::Ui, rows: impl FnOnce(&mut egui::Ui)) {
        let p = colors::palette(ui);
        let num_cols = self.columns.len();

        egui::Grid::new(ui.auto_id_with("data_table"))
            .num_columns(num_cols)
            .spacing([12.0, 6.0])
            .striped(self.striped)
            .show(ui, |ui| {
                // Header row.
                for (name, _min_width) in self.columns {
                    ui.label(
                        egui::RichText::new(*name)
                            .strong()
                            .size(12.0)
                            .color(p.text_secondary),
                    );
                }
                ui.end_row();

                // Separator after header.
                // (Grid doesn't support per-row separators, so we skip this
                //  and rely on striping for visual separation.)

                // Content rows — caller manages layout.
                rows(ui);
            });
    }
}
