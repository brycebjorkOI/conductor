//! Complete example: a macOS-style Settings app built entirely with egui-swift.
//!
//! Run with: `cargo run -p egui-swift --example settings_app`
//!
//! This demonstrates how to compose egui-swift components into a real app:
//! - NavigationSplitView for sidebar + detail
//! - Section / FormSection for grouped settings
//! - Toggle, Picker, TextField, Stepper for controls
//! - List for scrollable item lists
//! - HStack + Spacer for header layouts
//! - Alert for confirmation dialogs
//! - Label with Font presets for consistent typography
//! - Image::system_name() for SF Symbol icons
//! - ui.palette() for themed colors

use egui_swift::prelude::*;

// One line replaces 15 lines of eframe boilerplate.
egui_swift::app!(SettingsApp, "egui-swift Settings Example", 900.0, 600.0);

#[derive(Default)]
struct SettingsApp {
    selected_tab: String,
    // General settings state
    dark_mode: bool,
    notifications: bool,
    auto_save: bool,
    language: String,
    font_size: f64,
    // Account state
    username: String,
    email: String,
    // Alert state
    show_delete_alert: bool,
    // List state
    selected_item: usize,
    items: Vec<ListItem>,
}

struct ListItem {
    name: String,
    icon: &'static str,
    detail: String,
}

impl Default for ListItem {
    fn default() -> Self {
        Self {
            name: String::new(),
            icon: "doc",
            detail: String::new(),
        }
    }
}

impl SettingsApp {
    fn ensure_defaults(&mut self) {
        if self.language.is_empty() {
            self.language = "en".to_string();
        }
        if self.font_size == 0.0 {
            self.font_size = 14.0;
        }
        if self.items.is_empty() {
            self.items = vec![
                ListItem {
                    name: "Getting Started".into(),
                    icon: "book",
                    detail: "Introduction and setup guide".into(),
                },
                ListItem {
                    name: "API Reference".into(),
                    icon: "doc.text",
                    detail: "Complete API documentation".into(),
                },
                ListItem {
                    name: "Examples".into(),
                    icon: "folder",
                    detail: "Code examples and tutorials".into(),
                },
                ListItem {
                    name: "Release Notes".into(),
                    icon: "sparkles",
                    detail: "What's new in each version".into(),
                },
                ListItem {
                    name: "Contributing".into(),
                    icon: "person.2",
                    detail: "How to contribute to the project".into(),
                },
            ];
        }
    }
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ensure_defaults();

        // Alert (rendered above everything when visible).
        let alert_action = Alert::new("Delete Account?", &mut self.show_delete_alert)
            .message("This will permanently delete your account and all data. This action cannot be undone.")
            .destructive_action("Delete Account")
            .cancel()
            .show(ctx);

        if alert_action == AlertAction::Destructive {
            self.username.clear();
            self.email.clear();
        }

        NavigationSplitView::new("settings")
            .sidebar_width(180.0)
            .show(ctx, |sidebar, detail| {
                sidebar.show(|ui| {
                    Label::heading("Settings").show(ui);
                    Spacer::fixed(8.0).show(ui);
                    Divider::new().show(ui);
                    Spacer::fixed(8.0).show(ui);

                    let tabs = [
                        ("general", "General", "gear"),
                        ("account", "Account", "person.circle"),
                        ("docs", "Documents", "doc.text"),
                        ("about", "About", "info.circle"),
                    ];

                    for (id, label, sf) in tabs {
                        let icon = egui_swift::image::sf_symbol(sf);
                        let active = self.selected_tab == id;
                        if NavRow::new(label).icon(icon).active(active).show(ui).clicked() {
                            self.selected_tab = id.to_string();
                        }
                    }
                });

                detail.show(|ui| {
                    match self.selected_tab.as_str() {
                        "account" => self.show_account(ui),
                        "docs" => self.show_documents(ui),
                        "about" => self.show_about(ui),
                        _ => self.show_general(ui),
                    }
                });
            });
    }
}

impl SettingsApp {
    fn show_general(&mut self, ui: &mut egui::Ui) {
        Label::heading("General").show(ui);
        Spacer::fixed(16.0).show(ui);

        Section::new().header("Appearance").show(ui, |ui| {
            Toggle::new(&mut self.dark_mode).label("Dark mode").show(ui);
            Spacer::fixed(4.0).show(ui);
            let languages: Vec<(String, &str)> = vec![
                ("en".into(), "English"),
                ("es".into(), "Spanish"),
                ("fr".into(), "French"),
                ("de".into(), "German"),
                ("ja".into(), "Japanese"),
            ];
            Picker::new("Language", &mut self.language, &languages).show(ui);
        });

        Spacer::fixed(12.0).show(ui);

        Section::new().header("Behavior").show(ui, |ui| {
            Toggle::new(&mut self.notifications).label("Enable notifications").show(ui);
            Spacer::fixed(4.0).show(ui);
            Toggle::new(&mut self.auto_save).label("Auto-save documents").show(ui);
        });

        Spacer::fixed(12.0).show(ui);

        Section::new()
            .header("Editor")
            .footer("Font size applies to all documents.")
            .show(ui, |ui| {
                Stepper::new(&mut self.font_size, 10.0..=24.0)
                    .step(1.0)
                    .label("Font size")
                    .show(ui);
            });
    }

    fn show_account(&mut self, ui: &mut egui::Ui) {
        Label::heading("Account").show(ui);
        Spacer::fixed(16.0).show(ui);

        Section::new().header("Profile").show(ui, |ui| {
            TextField::new(&mut self.username)
                .label("Username")
                .placeholder("Enter username")
                .show(ui);
            Spacer::fixed(8.0).show(ui);
            TextField::new(&mut self.email)
                .label("Email")
                .placeholder("user@example.com")
                .show(ui);
        });

        Spacer::fixed(12.0).show(ui);

        Section::new().header("Danger Zone").show(ui, |ui| {
            Label::new("Deleting your account is permanent and cannot be reversed.")
                .font(Font::Caption)
                .muted()
                .show(ui);
            Spacer::fixed(8.0).show(ui);
            if Button::new("Delete Account")
                .style(ButtonStyle::Destructive)
                .show(ui)
                .clicked()
            {
                self.show_delete_alert = true;
            }
        });
    }

    fn show_documents(&mut self, ui: &mut egui::Ui) {
        HStack::new().show(ui, |ui| {
            Label::heading("Documents").show(ui);
            Spacer::trailing(ui, |ui| {
                Label::new(&format!("{} items", self.items.len()))
                    .font(Font::Subheadline)
                    .secondary()
                    .show(ui);
            });
        });
        Spacer::fixed(12.0).show(ui);

        List::new().inset_grouped().divider_inset(16.0).show(ui, |list| {
            for (i, item) in self.items.iter().enumerate() {
                let selected = i == self.selected_item;
                if list
                    .row(selected, |ui| {
                        HStack::new().spacing(8.0).show(ui, |ui| {
                            Image::system_name(item.icon).size(16.0).show(ui);
                            VStack::new().spacing(2.0).show(ui, |ui| {
                                Label::new(&item.name).font(Font::Callout).show(ui);
                                Label::new(&item.detail)
                                    .font(Font::Caption)
                                    .secondary()
                                    .show(ui);
                            });
                        });
                    })
                    .clicked()
                {
                    self.selected_item = i;
                }
            }
        });
    }

    fn show_about(&mut self, ui: &mut egui::Ui) {
        Label::heading("About").show(ui);
        Spacer::fixed(16.0).show(ui);

        Section::new().show(ui, |ui| {
            LabeledContent::new("Version", "1.0.0").show(ui);
            LabeledContent::new("Build", "2024.04.14").show(ui);
            LabeledContent::new("Framework", "egui-swift").show(ui);
        });

        Spacer::fixed(12.0).show(ui);

        Label::new("A demonstration of the egui-swift framework showing how to build native-feeling macOS applications with Rust and egui.")
            .font(Font::Callout)
            .secondary()
            .show(ui);

        Spacer::fixed(16.0).show(ui);
        Divider::new().show(ui);
        Spacer::fixed(8.0).show(ui);

        Label::new("Built with egui-swift — SwiftUI for Rust.")
            .font(Font::Caption)
            .muted()
            .show(ui);
    }
}
