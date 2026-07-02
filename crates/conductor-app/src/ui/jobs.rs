use conductor_core::state::*;
use egui_swift::prelude::*;

use crate::bridge::SharedState;

/// Jobs history view — shows all tasks that have been run in the app.
pub struct JobsView {
    shared: SharedState,
    filter: JobFilter,
    expanded_job: Option<String>,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum JobFilter {
    #[default]
    All,
    Running,
    Succeeded,
    Failed,
}

impl JobsView {
    pub fn new(shared: SharedState) -> Self {
        Self {
            shared,
            filter: JobFilter::default(),
            expanded_job: None,
        }
    }
}

impl View for JobsView {
    fn body(&mut self, ui: &mut egui::Ui) {
        let p = ui.palette();

        let state = self.shared.read();
        let mut entries = state.job_history.clone();
        let automation_rules = state.automation_rules.clone();
        drop(state);

        // Sort newest first.
        entries.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        ui.centered_content(Layout::MAX_CONTENT_WIDTH, |ui| {
            // Header.
            Label::heading("Jobs").show(ui);
            egui_swift::spacer!(ui, 4.0);
            Label::new("History of tasks run in the app")
                .font(Font::Subheadline)
                .secondary()
                .show(ui);
            egui_swift::spacer!(ui, 12.0);

            // Filter bar.
            egui_swift::hstack!(ui, {
                let filters = [
                    (JobFilter::All, "All"),
                    (JobFilter::Running, "Running"),
                    (JobFilter::Succeeded, "Succeeded"),
                    (JobFilter::Failed, "Failed"),
                ];
                for (f, label) in filters {
                    let style = if self.filter == f {
                        ButtonStyle::BorderedProminent
                    } else {
                        ButtonStyle::Bordered
                    };
                    if Button::new(label)
                        .style(style)
                        .small(true)
                        .show(ui)
                        .clicked()
                    {
                        self.filter = f;
                    }
                }
            });

            egui_swift::spacer!(ui, 12.0);

            // Apply filter.
            let filtered: Vec<&JobHistoryEntry> = entries
                .iter()
                .filter(|e| match self.filter {
                    JobFilter::All => true,
                    JobFilter::Running => e.status == JobRunStatus::Running,
                    JobFilter::Succeeded => e.status == JobRunStatus::Success,
                    JobFilter::Failed => e.status == JobRunStatus::Failure,
                })
                .collect();

            if filtered.is_empty() {
                egui_swift::spacer!(ui, 24.0);
                EmptyState::new("No jobs yet")
                    .icon("\u{1f4cb}")
                    .subtitle("Jobs will appear here as tasks are run.")
                    .show(ui);
                return;
            }

            // Stats summary.
            let running = entries.iter().filter(|e| e.status == JobRunStatus::Running).count();
            let succeeded = entries.iter().filter(|e| e.status == JobRunStatus::Success).count();
            let failed = entries.iter().filter(|e| e.status == JobRunStatus::Failure).count();

            egui_swift::hstack!(ui, spacing: 16.0, {
                Label::new(&format!("{} total", entries.len()))
                    .font(Font::Caption)
                    .secondary()
                    .show(ui);
                if running > 0 {
                    Label::new(&format!("{running} running"))
                        .font(Font::Caption)
                        .color(p.accent)
                        .show(ui);
                }
                Label::new(&format!("{succeeded} succeeded"))
                    .font(Font::Caption)
                    .color(p.status_green)
                    .show(ui);
                if failed > 0 {
                    Label::new(&format!("{failed} failed"))
                        .font(Font::Caption)
                        .color(p.status_red)
                        .show(ui);
                }
            });

            egui_swift::spacer!(ui, 8.0);

            // Job list.
            ScrollView::vertical().show(ui, |ui| {
                for entry in &filtered {
                    show_job_entry(
                        ui,
                        entry,
                        &automation_rules,
                        &mut self.expanded_job,
                        &p,
                    );
                }
            });
        });
    }
}

fn show_job_entry(
    ui: &mut egui::Ui,
    entry: &JobHistoryEntry,
    automation_rules: &[AutomationRule],
    expanded_job: &mut Option<String>,
    p: &Palette,
) {
    let (status_icon, status_color) = match entry.status {
        JobRunStatus::Running => ("\u{25cf}", p.accent),      // ●
        JobRunStatus::Success => ("\u{2713}", p.status_green), // ✓
        JobRunStatus::Failure => ("\u{2717}", p.status_red),   // ✗
        JobRunStatus::Cancelled => ("\u{2014}", p.text_muted), // —
    };

    Card::new().show(ui, |ui| {
        // Top row: status icon + name + trigger badge.
        egui_swift::hstack!(ui, {
            Label::new(status_icon)
                .font(Font::Body)
                .color(status_color)
                .show(ui);
            Label::new(&entry.job_name)
                .font(Font::Callout)
                .bold(true)
                .show(ui);

            let trigger_label = match entry.trigger {
                JobTrigger::Scheduled => "scheduled",
                JobTrigger::Manual => "manual",
                JobTrigger::Channel => "channel",
                JobTrigger::Automation => "automation",
            };
            Label::new(trigger_label)
                .font(Font::Caption)
                .secondary()
                .show(ui);
        });

        // Timestamp + duration row.
        egui_swift::hstack!(ui, spacing: 12.0, {
            Label::new(&entry.started_at.format("%Y-%m-%d %H:%M:%S").to_string())
                .font(Font::Footnote)
                .monospace(true)
                .color(p.text_secondary)
                .show(ui);
            if let Some(ms) = entry.duration_ms {
                let duration_str = if ms < 1000 {
                    format!("{ms}ms")
                } else if ms < 60_000 {
                    format!("{:.1}s", ms as f64 / 1000.0)
                } else {
                    format!("{:.1}m", ms as f64 / 60_000.0)
                };
                Label::new(&duration_str)
                    .font(Font::Footnote)
                    .color(p.text_muted)
                    .show(ui);
            }
            if let Some(ref backend) = entry.backend_id {
                Label::new(backend)
                    .font(Font::Footnote)
                    .color(p.text_muted)
                    .show(ui);
            }
        });

        // Prompt preview.
        if let Some(ref prompt) = entry.prompt_preview {
            let preview = if prompt.len() > 120 {
                format!("{}...", &prompt[..117])
            } else {
                prompt.clone()
            };
            Label::new(&preview)
                .font(Font::Subheadline)
                .secondary()
                .show(ui);
        }

        // Output summary.
        if let Some(ref summary) = entry.output_summary {
            let short = if summary.len() > 200 {
                format!("{}...", &summary[..197])
            } else {
                summary.clone()
            };
            Label::new(&short)
                .font(Font::Footnote)
                .color(p.text_muted)
                .show(ui);
        }

        // Error message.
        if let Some(ref err) = entry.error {
            Label::new(err)
                .font(Font::Footnote)
                .color(p.status_red)
                .show(ui);
        }

        // Expandable step details for automation runs.
        if entry.trigger == JobTrigger::Automation {
            // Look up the automation rule's run history to find step_results.
            let step_results = entry.job_id.as_ref().and_then(|rule_id| {
                automation_rules
                    .iter()
                    .find(|r| r.rule_id == *rule_id)
                    .and_then(|rule| {
                        rule.history
                            .iter()
                            .find(|h| h.run_id == entry.run_id)
                            .map(|h| h.step_results.clone())
                    })
            });

            if let Some(ref steps) = step_results {
                if !steps.is_empty() {
                    let mut expanded =
                        expanded_job.as_deref() == Some(&entry.run_id);
                    DisclosureGroup::new(
                        &format!("Steps ({})", steps.len()),
                        &mut expanded,
                    )
                    .show(ui, |ui| {
                        for step in steps {
                            show_step_result(ui, step, p);
                        }

                        // Log file link.
                        if let Some(ref rule_id) = entry.job_id {
                            let log_path = conductor_core::automation::run_log_path(
                                rule_id,
                                &entry.run_id,
                            );
                            if log_path.exists() {
                                egui_swift::spacer!(ui, 4.0);
                                if Button::new("View Full Log")
                                    .style(ButtonStyle::Bordered)
                                    .small(true)
                                    .show(ui)
                                    .clicked()
                                {
                                    let _ = open::that(&log_path);
                                }
                                Label::new(&log_path.display().to_string())
                                    .font(Font::Footnote)
                                    .monospace(true)
                                    .muted()
                                    .show(ui);
                            }
                        }
                    });

                    if expanded {
                        *expanded_job = Some(entry.run_id.clone());
                    } else if expanded_job.as_deref() == Some(&entry.run_id) {
                        *expanded_job = None;
                    }
                }
            }
        }
    });
}

fn show_step_result(ui: &mut egui::Ui, step: &StepRunResult, p: &Palette) {
    let (icon, color) = match step.status {
        JobRunStatus::Success => ("\u{2713}", p.status_green),
        JobRunStatus::Failure => ("\u{2717}", p.status_red),
        JobRunStatus::Running => ("\u{25cf}", p.accent),
        JobRunStatus::Cancelled => ("\u{2014}", p.text_muted),
    };

    egui_swift::hstack!(ui, {
        Label::new(icon)
            .font(Font::Footnote)
            .color(color)
            .show(ui);
        Label::new(&step.step_name)
            .font(Font::Footnote)
            .bold(true)
            .show(ui);
        if step.skipped {
            Label::new("skipped")
                .font(Font::Footnote)
                .muted()
                .show(ui);
        }
        if let Some(ms) = step.duration_ms {
            if ms > 0 {
                let dur = if ms < 1000 {
                    format!("{ms}ms")
                } else {
                    format!("{:.1}s", ms as f64 / 1000.0)
                };
                Label::new(&dur)
                    .font(Font::Footnote)
                    .color(p.text_muted)
                    .show(ui);
            }
        }
    });

    // Output preview.
    if let Some(ref output) = step.output {
        let preview = if output.len() > 200 {
            format!("{}...", &output[..197])
        } else {
            output.clone()
        };
        Label::new(&preview)
            .font(Font::Footnote)
            .color(p.text_muted)
            .show(ui);
    }

    // Error.
    if let Some(ref err) = step.error {
        Label::new(err)
            .font(Font::Footnote)
            .color(p.status_red)
            .show(ui);
    }
}
