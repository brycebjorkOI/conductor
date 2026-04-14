//! User profile row — avatar circle + name + version, optional settings gear.
//!
//! ```no_run
//! # let ui: &mut egui::Ui = todo!();
//! use conductor_ui::user_profile::UserProfile;
//! let resp = UserProfile::new("José").version("v1.0.0").show(ui);
//! if resp.settings_clicked {
//!     // open settings
//! }
//! ```

use crate::colors;

pub struct UserProfileResponse {
    pub settings_clicked: bool,
}

pub struct UserProfile<'a> {
    name: &'a str,
    version: Option<&'a str>,
    avatar_letter: Option<char>,
}

impl<'a> UserProfile<'a> {
    pub fn new(name: &'a str) -> Self {
        let letter = name.chars().next().unwrap_or('?');
        Self {
            name,
            version: None,
            avatar_letter: Some(letter),
        }
    }

    pub fn version(mut self, version: &'a str) -> Self {
        self.version = Some(version);
        self
    }

    pub fn show(self, ui: &mut egui::Ui) -> UserProfileResponse {
        let p = colors::palette(ui);
        let mut settings_clicked = false;

        ui.horizontal(|ui| {
            ui.add_space(12.0);

            // Avatar circle.
            let avatar_size = 28.0;
            let (avatar_rect, _) = ui.allocate_exact_size(
                egui::vec2(avatar_size, avatar_size),
                egui::Sense::hover(),
            );
            ui.painter().circle_filled(
                avatar_rect.center(),
                avatar_size / 2.0,
                p.accent_bg,
            );
            if let Some(letter) = self.avatar_letter {
                ui.painter().text(
                    avatar_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    letter.to_uppercase().to_string(),
                    egui::FontId::proportional(13.0),
                    p.accent,
                );
            }

            ui.add_space(8.0);

            // Name + version stacked.
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new(self.name)
                        .size(13.0)
                        .color(p.text_primary),
                );
                if let Some(ver) = self.version {
                    ui.label(
                        egui::RichText::new(ver)
                            .size(10.5)
                            .color(p.text_muted),
                    );
                }
            });

            // Settings gear — pushed right.
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(12.0);
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("\u{2699}")
                                .size(16.0)
                                .color(p.text_muted),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .frame(false),
                    )
                    .clicked()
                {
                    settings_clicked = true;
                }
            });
        });

        UserProfileResponse { settings_clicked }
    }
}
