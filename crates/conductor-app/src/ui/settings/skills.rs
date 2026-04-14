use egui_swift::button::{Button, ButtonStyle};
use egui_swift::colors;
use egui_swift::empty_state::EmptyState;
use egui_swift::form_section::FormSection;

pub fn show(ui: &mut egui::Ui) {
    let p = colors::palette(ui);

    ui.label(
        egui::RichText::new("Skills")
            .size(22.0)
            .strong()
            .color(p.text_primary),
    );
    ui.add_space(8.0);

    ui.label(
        egui::RichText::new(
            "Skills are Markdown instruction documents injected into AI system prompts when active.",
        )
        .size(13.0)
        .color(p.text_secondary),
    );
    ui.add_space(12.0);

    // Bundled skills.
    FormSection::new().header("Bundled").show(ui, |ui| {
        EmptyState::new("No bundled skills installed yet")
            .subtitle("Skills will appear here when added to the resources/skills/ directory.")
            .show(ui);
    });

    ui.add_space(12.0);

    // User skills.
    FormSection::new().header("User Skills").show(ui, |ui| {
        EmptyState::new("No user skills")
            .subtitle("Add .md files to ~/.conductor/skills/ to create custom skills.")
            .show(ui);
    });

    ui.add_space(12.0);

    if Button::new("Open Skills Directory")
        .style(ButtonStyle::Bordered)
        .show(ui)
        .clicked()
    {
        let dir = conductor_core::config::config_dir().join("skills");
        let _ = std::fs::create_dir_all(&dir);
        let _ = open::that(&dir);
    }
}
