use egui_swift::prelude::*;

pub fn show(ui: &mut egui::Ui) {
    Label::heading("Skills").show(ui);
    ui.add_space(8.0);

    Label::new(
        "Skills are Markdown instruction documents injected into AI system prompts when active.",
    )
    .font(Font::Callout)
    .secondary()
    .show(ui);
    ui.add_space(12.0);

    FormSection::new().header("Bundled").show(ui, |ui| {
        EmptyState::new("No bundled skills installed yet")
            .subtitle("Skills will appear here when added to the resources/skills/ directory.")
            .show(ui);
    });

    ui.add_space(12.0);

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
