use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::state::*;
use egui_swift::prelude::*;

use crate::bridge::SharedState;

/// Persistent UI state for the schedules view.
pub struct SchedulesView {
    shared: SharedState,
    tx: mpsc::UnboundedSender<Action>,
    show_add_form: bool,
    form: JobForm,
    expanded_history: Option<String>,
}

impl SchedulesView {
    pub fn new(shared: SharedState, tx: mpsc::UnboundedSender<Action>) -> Self {
        Self {
            shared,
            tx,
            show_add_form: false,
            form: JobForm::default(),
            expanded_history: None,
        }
    }
}

#[derive(Default)]
pub struct JobForm {
    pub name: String,
    pub schedule_type: u8,
    pub interval_minutes: u32,
    pub cron_expression: String,
    pub prompt: String,
    pub execution_mode: u8,
    pub delivery_mode: u8,
    pub webhook_url: String,
}

impl View for SchedulesView {
    fn body(&mut self, ui: &mut egui::Ui) {
        let p = ui.palette();

        let state = self.shared.read();
        let jobs = state.scheduler.jobs.clone();
        drop(state);

        ui.centered_content(Layout::MAX_CONTENT_WIDTH, |ui| {
            // Header row.
            egui_swift::hstack!(ui, {
                Label::heading("Scheduled Tasks").show(ui);
                Spacer::trailing(ui, |ui| {
                    if Button::new("+ Add Job")
                        .style(ButtonStyle::BorderedProminent)
                        .small(true)
                        .show(ui)
                        .clicked()
                    {
                        self.show_add_form = !self.show_add_form;
                    }
                });
            });

            egui_swift::spacer!(ui, 4.0);
            Label::new(&format!(
                "{} job{}",
                jobs.len(),
                if jobs.len() == 1 { "" } else { "s" }
            ))
            .font(Font::Subheadline)
            .secondary()
            .show(ui);
            egui_swift::spacer!(ui, 8.0);

            // -- Add Job form --
            if self.show_add_form {
                show_add_form(ui, &self.tx, &mut self.form, &mut self.show_add_form, &p);
                egui_swift::spacer!(ui, 8.0);
                Divider::new().show(ui);
                egui_swift::spacer!(ui, 8.0);
            }

            // -- Job list --
            if jobs.is_empty() && !self.show_add_form {
                EmptyState::new("No scheduled tasks yet")
                    .icon("\u{1f4c5}")
                    .subtitle("Create a job to run AI prompts on a schedule.")
                    .show(ui);
                return;
            }

            for job in &jobs {
                show_job_card(ui, job, &self.tx, &mut self.expanded_history, &p);
            }
        });
    }
}

fn show_add_form(
    ui: &mut egui::Ui,
    tx: &mpsc::UnboundedSender<Action>,
    form: &mut JobForm,
    show_add_form: &mut bool,
    _p: &Palette,
) {
    Card::new().show(ui, |ui| {
        Label::new("New Scheduled Job")
            .font(Font::Headline)
            .bold(true)
            .show(ui);
        egui_swift::spacer!(ui, 8.0);

        // Name.
        TextField::new(&mut form.name)
            .label("Name")
            .placeholder("Job name")
            .show(ui);
        egui_swift::spacer!(ui, 8.0);

        // Schedule type.
        FormSection::new().header("Schedule").show(ui, |ui| {
            let schedule_types: Vec<(u8, &str)> =
                vec![(1, "Interval"), (2, "Cron"), (0, "One-time")];
            RadioGroup::new(&mut form.schedule_type, &schedule_types).show(ui);
        });

        egui_swift::spacer!(ui, 4.0);

        match form.schedule_type {
            1 => {
                egui_swift::hstack!(ui, {
                    Label::new("Every").font(Font::Callout).show(ui);
                    ui.add(
                        egui::DragValue::new(&mut form.interval_minutes).range(1..=10080),
                    );
                    Label::new("minutes").font(Font::Callout).show(ui);
                });
            }
            2 => {
                TextField::new(&mut form.cron_expression)
                    .label("Cron expression")
                    .placeholder("0 9 * * 1-5")
                    .show(ui);
                Label::new("e.g. \"0 9 * * 1-5\" = 9 AM weekdays")
                    .font(Font::Caption)
                    .muted()
                    .show(ui);
            }
            _ => {
                Label::new("One-time jobs run immediately when created.")
                    .font(Font::Caption)
                    .muted()
                    .show(ui);
            }
        }

        egui_swift::spacer!(ui, 8.0);

        // Prompt.
        TextField::new(&mut form.prompt)
            .label("Prompt")
            .placeholder("What should the AI do?")
            .multiline(3)
            .show(ui);
        egui_swift::spacer!(ui, 8.0);

        // Execution mode.
        FormSection::new().header("Execution Mode").show(ui, |ui| {
            let exec_modes: Vec<(u8, &str)> = vec![(0, "Isolated"), (1, "Main session")];
            RadioGroup::new(&mut form.execution_mode, &exec_modes).show(ui);
        });

        egui_swift::spacer!(ui, 4.0);

        // Delivery.
        FormSection::new().header("Delivery").show(ui, |ui| {
            let delivery_modes: Vec<(u8, &str)> =
                vec![(0, "Silent (log only)"), (1, "Webhook")];
            RadioGroup::new(&mut form.delivery_mode, &delivery_modes).show(ui);
        });

        if form.delivery_mode == 1 {
            egui_swift::spacer!(ui, 4.0);
            TextField::new(&mut form.webhook_url)
                .label("Webhook URL")
                .placeholder("https://...")
                .show(ui);
        }

        egui_swift::spacer!(ui, 12.0);

        // Buttons.
        let can_create = !form.name.trim().is_empty() && !form.prompt.trim().is_empty();
        ButtonRow::show(ui, |ui| {
            if Button::new("Cancel")
                .style(ButtonStyle::Bordered)
                .show(ui)
                .clicked()
            {
                *show_add_form = false;
            }
            if Button::new("Create Job")
                .style(ButtonStyle::BorderedProminent)
                .enabled(can_create)
                .show(ui)
                .clicked()
            {
                let schedule = match form.schedule_type {
                    1 => ScheduleDefinition::Interval {
                        seconds: form.interval_minutes as u64 * 60,
                    },
                    2 => ScheduleDefinition::Cron {
                        expression: form.cron_expression.clone(),
                        timezone: "UTC".into(),
                    },
                    _ => ScheduleDefinition::OneTime {
                        datetime: chrono::Utc::now() + chrono::Duration::seconds(5),
                    },
                };
                let delivery = match form.delivery_mode {
                    1 => DeliveryConfig::Webhook {
                        url: form.webhook_url.clone(),
                    },
                    _ => DeliveryConfig::Silent,
                };
                let exec_mode = if form.execution_mode == 1 {
                    ExecutionMode::MainSession
                } else {
                    ExecutionMode::Isolated
                };

                let job = conductor_core::scheduler::create_job(
                    form.name.clone(),
                    schedule,
                    form.prompt.clone(),
                    exec_mode,
                    delivery,
                );
                let _ = tx.send(Action::CreateJob { definition: job });

                *form = JobForm::default();
                *show_add_form = false;
            }
        });
    });
}

fn show_job_card(
    ui: &mut egui::Ui,
    job: &ScheduledJob,
    tx: &mpsc::UnboundedSender<Action>,
    expanded_history: &mut Option<String>,
    p: &Palette,
) {
    let status_color = if job.enabled {
        p.status_green
    } else {
        p.text_muted
    };

    Card::new().show(ui, |ui| {
        // Top row: status + name + schedule.
        egui_swift::hstack!(ui, {
            StatusDot::new(status_color).show(ui);
            Label::new(&job.name).font(Font::Callout).bold(true).show(ui);
            Label::new(&format_schedule(&job.schedule))
                .font(Font::Subheadline)
                .secondary()
                .show(ui);
            if let Some(next) = job.next_run {
                Label::new(&format!("next: {}", next.format("%H:%M")))
                    .font(Font::Caption)
                    .muted()
                    .show(ui);
            }
        });

        // Prompt preview.
        let preview = if job.payload.prompt.len() > 80 {
            format!("{}...", &job.payload.prompt[..77])
        } else {
            job.payload.prompt.clone()
        };
        Label::new(&preview)
            .font(Font::Subheadline)
            .secondary()
            .show(ui);

        egui_swift::spacer!(ui, 4.0);

        // Action buttons.
        egui_swift::hstack!(ui, {
            if Button::new("Run Now")
                .style(ButtonStyle::Bordered)
                .small(true)
                .show(ui)
                .clicked()
            {
                let _ = tx.send(Action::RunJobNow {
                    job_id: job.job_id.clone(),
                });
            }

            let toggle_label = if job.enabled { "Disable" } else { "Enable" };
            if Button::new(toggle_label)
                .style(ButtonStyle::Bordered)
                .small(true)
                .show(ui)
                .clicked()
            {
                let _ = tx.send(Action::ToggleJob {
                    job_id: job.job_id.clone(),
                    enabled: !job.enabled,
                });
            }

            if Button::new("Delete")
                .style(ButtonStyle::Destructive)
                .small(true)
                .show(ui)
                .clicked()
            {
                let _ = tx.send(Action::DeleteJob {
                    job_id: job.job_id.clone(),
                });
            }
        });

        // Expandable execution history.
        if !job.history.is_empty() {
            let mut expanded =
                expanded_history.as_deref() == Some(&job.job_id);
            DisclosureGroup::new("Run History", &mut expanded).show(ui, |ui| {
                for run in job.history.iter().rev().take(10) {
                    let (status_icon, run_color) = match run.status {
                        JobRunStatus::Success => ("\u{2713}", p.status_green),
                        JobRunStatus::Failure => ("\u{2717}", p.status_red),
                        JobRunStatus::Running => ("\u{25cf}", p.accent),
                        JobRunStatus::Cancelled => ("\u{2014}", p.text_muted),
                    };

                    egui_swift::hstack!(ui, {
                        Label::new(status_icon)
                            .font(Font::Footnote)
                            .color(run_color)
                            .show(ui);
                        Label::new(&run.started_at.format("%m-%d %H:%M").to_string())
                            .font(Font::Footnote)
                            .monospace(true)
                            .color(p.text_secondary)
                            .show(ui);
                        if let Some(ms) = run.duration_ms {
                            Label::new(&format!("{:.1}s", ms as f64 / 1000.0))
                                .font(Font::Footnote)
                                .color(p.text_muted)
                                .show(ui);
                        }
                        if let Some(ref err) = run.error {
                            Label::new(err)
                                .font(Font::Footnote)
                                .color(p.status_red)
                                .show(ui);
                        }
                        if let Some(ref summary) = run.output_summary {
                            let short = if summary.len() > 50 {
                                format!("{}...", &summary[..47])
                            } else {
                                summary.clone()
                            };
                            Label::new(&short)
                                .font(Font::Footnote)
                                .color(p.text_muted)
                                .show(ui);
                        }
                    });
                }
            });

            // Sync the expanded state back.
            if expanded {
                *expanded_history = Some(job.job_id.clone());
            } else if expanded_history.as_deref() == Some(&job.job_id) {
                *expanded_history = None;
            }
        }
    });
}

fn format_schedule(sched: &ScheduleDefinition) -> String {
    match sched {
        ScheduleDefinition::OneTime { datetime } => {
            format!("once at {}", datetime.format("%Y-%m-%d %H:%M"))
        }
        ScheduleDefinition::Interval { seconds } => {
            if *seconds >= 86400 {
                format!("every {}d", seconds / 86400)
            } else if *seconds >= 3600 {
                format!("every {}h", seconds / 3600)
            } else {
                format!("every {}m", seconds / 60)
            }
        }
        ScheduleDefinition::Cron { expression, .. } => {
            format!("cron: {expression}")
        }
    }
}
