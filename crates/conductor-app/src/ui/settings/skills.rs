use egui_swift::prelude::*;

egui_swift::view! {
    pub struct SkillsView {}
    fn body(&mut self, ui: &mut egui::Ui) {
        egui_swift::text!(ui, "Skills", .title);
        egui_swift::spacer!(ui, 8.0);
        egui_swift::text!(ui, "Skills are Markdown instruction documents injected into AI system prompts when active.", .callout, .secondary);
        egui_swift::spacer!(ui, 12.0);

        egui_swift::section!(ui, "Bundled", {
            EmptyState::new("No bundled skills installed yet")
                .subtitle("Skills will appear here when added to the resources/skills/ directory.")
                .show(ui);
        });

        egui_swift::spacer!(ui, 12.0);

        egui_swift::section!(ui, "User Skills", {
            EmptyState::new("No user skills")
                .subtitle("Add .md files to ~/.conductor/skills/ to create custom skills.")
                .show(ui);
        });

        egui_swift::spacer!(ui, 12.0);

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
}
