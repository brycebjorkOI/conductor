pub fn show(ui: &mut egui::Ui) {
    let p = egui_swift::colors::palette(ui);

    ui.heading("Skills");
    ui.add_space(12.0);

    ui.label("Skills are Markdown instruction documents injected into AI system prompts when active.");
    ui.add_space(8.0);

    // Bundled skills.
    egui_swift::section_header::SectionHeader::new("Bundled").show(ui);
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("No bundled skills installed yet. Skills will appear here when added to the resources/skills/ directory.")
            .size(12.0)
            .color(p.text_muted),
    );

    ui.add_space(16.0);

    // User skills.
    egui_swift::section_header::SectionHeader::new("User Skills").show(ui);
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("Add .md files to ~/.conductor/skills/ to create custom skills.")
            .size(12.0)
            .color(p.text_muted),
    );

    ui.add_space(12.0);
    if ui.button("Open Skills Directory").clicked() {
        let dir = conductor_core::config::config_dir().join("skills");
        let _ = std::fs::create_dir_all(&dir);
        let _ = open::that(&dir);
    }
}
