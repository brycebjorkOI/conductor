//! Minimal egui-swift app — demonstrates app!, view!, hstack!, vstack! macros.
//!
//! Run with: `cargo run -p egui-swift --example hello`

use egui_swift::prelude::*;

// One line replaces 15 lines of eframe boilerplate.
egui_swift::app!(HelloApp, "Hello egui-swift");

// view! generates struct + Default + View impl in one block.
egui_swift::view! {
    struct GeneralView {
        dark_mode: bool = false,
        notifications: bool = true,
        language: String = "en".to_string(),
    }
    fn body(&mut self, ui: &mut egui::Ui) {
        Section::new().header("Appearance").show(ui, |ui| {
            Toggle::new(&mut self.dark_mode).label("Dark mode").show(ui);
            Spacer::fixed(4.0).show(ui);
            let langs: Vec<(String, &str)> = vec![
                ("en".into(), "English"),
                ("es".into(), "Spanish"),
                ("ja".into(), "Japanese"),
            ];
            Picker::new("Language", &mut self.language, &langs).show(ui);
        });

        Spacer::fixed(12.0).show(ui);

        Section::new().header("Notifications").show(ui, |ui| {
            Toggle::new(&mut self.notifications).label("Enable notifications").show(ui);
        });
    }
}

#[derive(Default)]
struct HelloApp {
    general: GeneralView,
    count: u32,
}

impl eframe::App for HelloApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let p = ctx.palette();
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(p.surface).inner_margin(egui::Margin::symmetric(24, 24)))
            .show(ctx, |ui| {
                Label::heading("Hello, egui-swift!").show(ui);
                Spacer::fixed(8.0).show(ui);
                Label::new("A minimal app using the app!, view!, hstack!, and vstack! macros.")
                    .font(Font::Callout)
                    .secondary()
                    .show(ui);
                Spacer::fixed(16.0).show(ui);

                // Embed the GeneralView (struct created by view! macro).
                self.general.show(ui);

                Spacer::fixed(16.0).show(ui);

                // hstack! and vstack! macros for inline layouts.
                Section::new().header("Counter").show(ui, |ui| {
                    egui_swift::hstack!(ui, spacing: 12.0, {
                        Label::new(&format!("Count: {}", self.count))
                            .font(Font::Headline)
                            .show(ui);
                        Spacer::trailing(ui, |ui| {
                            if Button::new("+1")
                                .style(ButtonStyle::BorderedProminent)
                                .show(ui)
                                .clicked()
                            {
                                self.count += 1;
                            }
                        });
                    });
                });

                Spacer::fixed(16.0).show(ui);

                // SF Symbols work naturally.
                Section::new().header("Icons").show(ui, |ui| {
                    egui_swift::hstack!(ui, spacing: 16.0, {
                        Image::system_name("gear").size(20.0).show(ui);
                        Image::system_name("bell.fill").size(20.0).tint(p.accent).show(ui);
                        Image::system_name("heart.fill").size(20.0).tint(p.status_red).show(ui);
                        Image::system_name("star.fill").size(20.0).tint(p.status_yellow).show(ui);
                        Image::system_name("checkmark.circle.fill").size(20.0).tint(p.status_green).show(ui);
                    });
                });
            });
    }
}
