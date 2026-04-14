use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::state::*;
use egui_swift::prelude::*;

use crate::bridge::SharedState;

/// Persistent UI state for the schedules tab.
pub struct SchedulesTabState {
    pub show_add_form: bool,
    pub form: JobForm,
    pub expanded_history: Option<String>,
}

impl Default for SchedulesTabState {
    fn default() -> Self {
        Self {
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

pub fn show(
    ui: &mut egui::Ui,
    shared: &SharedState,
    tx: &mpsc::UnboundedSender<Action>,
    tab_state: &mut SchedulesTabState,
) {
    let p = ui.palette();

    let state = shared.read();
    let jobs = state.scheduler.jobs.clone();
    drop(state);

    // Header row.
    HStack::new().show(ui, |ui| {
        Label::heading("Scheduled Tasks").show(ui);
        Spacer::trailing(ui, |ui| {
            if Button::new("+ Add Job")
                .style(ButtonStyle::BorderedProminent)
                .small(true)
                .show(ui)
                .clicked()
            {
                tab_state.show_add_form = !tab_state.show_add_form;
            }
        });
    });

    ui.add_space(4.0);
    Label::new(&format!(
        "{} job{}",
        jobs.len(),
        if jobs.len() == 1 { "" } else { "s" }
    ))
    .font(Font::Subheadline)
    .secondary()
    .show(ui);
    ui.add_space(8.0);

    // -- Add Job form --
    if tab_state.show_add_form {
        show_add_form(ui, tx, tab_state, &p);
        ui.add_space(8.0);
        Divider::new().show(ui);
        ui.add_space(8.0);
    }

    // -- Job list --
    if jobs.is_empty() && !tab_state.show_add_form {
        EmptyState::new("No scheduled tasks yet")
            .icon("\u{1f4c5}")
            .subtitle("Create a job to run AI prompts on a schedule.")
            .show(ui);
        return;
    }

    for job in &jobs {
        show_job_card(ui, job, tx, tab_state, &p);
    }
}

fn show_add_form(
    ui: &mut egui::Ui,
    tx: &mpsc::UnboundedSender<Action>,
    tab_state: &mut SchedulesTabState,
    p: &Palette,
) {
    let form = &mut tab_state.form;

    Card::new().show(ui, |ui| {
        ui.label(
            egui::RichText::new("New Scheduled Job")
                .strong()
                .size(15.0)
                .color(p.text_primary),
        );
        ui.add_space(8.0);

        // Name.
        TextField::new(&mut form.name)
            .label("Name")
            .placeholder("Job name")
            .show(ui);
        ui.add_space(8.0);

        // Schedule type.
        FormSection::new().header("Schedule").show(ui, |ui| {
            let schedule_types: Vec<(u8, &str)> =
                vec![(1, "Interval"), (2, "Cron"), (0, "One-time")];
            RadioGroup::new(&mut form.schedule_type, &schedule_types).show(ui);
        });

        ui.add_space(4.0);

        match form.schedule_type {
            1 => {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Every").size(13.0).color(p.text_primary),
                    );
                    ui.add(
                        egui::DragValue::new(&mut form.interval_minutes).range(1..=10080),
                    );
                    ui.label(
                        egui::RichText::new("minutes")
                            .size(13.0)
                            .color(p.text_primary),
                    );
                });
            }
            2 => {
                TextField::new(&mut form.cron_expression)
                    .label("Cron expression")
                    .placeholder("0 9 * * 1-5")
                    .show(ui);
                ui.label(
                    egui::RichText::new("e.g. \"0 9 * * 1-5\" = 9 AM weekdays")
                        .size(11.0)
                        .color(p.text_muted),
                );
            }
            _ => {
                ui.label(
                    egui::RichText::new("One-time jobs run immediately when created.")
                        .size(11.0)
                        .color(p.text_muted),
                );
            }
        }

        ui.add_space(8.0);

        // Prompt.
        TextField::new(&mut form.prompt)
            .label("Prompt")
            .placeholder("What should the AI do?")
            .multiline(3)
            .show(ui);
        ui.add_space(8.0);

        // Execution mode.
        FormSection::new().header("Execution Mode").show(ui, |ui| {
            let exec_modes: Vec<(u8, &str)> = vec![(0, "Isolated"), (1, "Main session")];
            RadioGroup::new(&mut form.execution_mode, &exec_modes).show(ui);
        });

        ui.add_space(4.0);

        // Delivery.
        FormSection::new().header("Delivery").show(ui, |ui| {
            let delivery_modes: Vec<(u8, &str)> =
                vec![(0, "Silent (log only)"), (1, "Webhook")];
            RadioGroup::new(&mut form.delivery_mode, &delivery_modes).show(ui);
        });

        if form.delivery_mode == 1 {
            ui.add_space(4.0);
            TextField::new(&mut form.webhook_url)
                .label("Webhook URL")
                .placeholder("https://...")
                .show(ui);
        }

        ui.add_space(12.0);

        // Buttons.
        let can_create = !form.name.trim().is_empty() && !form.prompt.trim().is_empty();
        ButtonRow::show(ui, |ui| {
            if Button::new("Cancel")
                .style(ButtonStyle::Bordered)
                .show(ui)
                .clicked()
            {
                tab_state.show_add_form = false;
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
                tab_state.show_add_form = false;
            }
        });
    });
}

fn show_job_card(
    ui: &mut egui::Ui,
    job: &ScheduledJob,
    tx: &mpsc::UnboundedSender<Action>,
    tab_state: &mut SchedulesTabState,
    p: &Palette,
) {
    let status_color = if job.enabled {
        p.status_green
    } else {
        p.text_muted
    };

    Card::new().show(ui, |ui| {
        // Top row: status + name + schedule.
        ui.horizontal(|ui| {
            StatusDot::new(status_color).show(ui);
            ui.label(egui::RichText::new(&job.name).strong().size(13.0));
            ui.label(
                egui::RichText::new(format_schedule(&job.schedule))
                    .size(12.0)
                    .color(p.text_secondary),
            );
            if let Some(next) = job.next_run {
                ui.label(
                    egui::RichText::new(format!("next: {}", next.format("%H:%M")))
                        .size(11.0)
                        .color(p.text_muted),
                );
            }
        });

        // Prompt preview.
        let preview = if job.payload.prompt.len() > 80 {
            format!("{}...", &job.payload.prompt[..77])
        } else {
            job.payload.prompt.clone()
        };
        ui.label(
            egui::RichText::new(preview)
                .size(12.0)
                .color(p.text_secondary),
        );

        ui.add_space(4.0);

        // Action buttons.
        ui.horizontal(|ui| {
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
                tab_state.expanded_history.as_deref() == Some(&job.job_id);
            DisclosureGroup::new("Run History", &mut expanded).show(ui, |ui| {
                for run in job.history.iter().rev().take(10) {
                    let (status_icon, run_color) = match run.status {
                        JobRunStatus::Success => ("\u{2713}", p.status_green),
                        JobRunStatus::Failure => ("\u{2717}", p.status_red),
                        JobRunStatus::Running => ("\u{25cf}", p.accent),
                        JobRunStatus::Cancelled => ("\u{2014}", p.text_muted),
                    };

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(status_icon)
                                .color(run_color)
                                .size(12.0),
                        );
                        ui.label(
                            egui::RichText::new(
                                run.started_at.format("%m-%d %H:%M").to_string(),
                            )
                            .size(11.0)
                            .monospace()
                            .color(p.text_secondary),
                        );
                        if let Some(ms) = run.duration_ms {
                            ui.label(
                                egui::RichText::new(format!("{:.1}s", ms as f64 / 1000.0))
                                    .size(11.0)
                                    .color(p.text_muted),
                            );
                        }
                        if let Some(ref err) = run.error {
                            ui.label(
                                egui::RichText::new(err)
                                    .size(11.0)
                                    .color(p.status_red),
                            );
                        }
                        if let Some(ref summary) = run.output_summary {
                            let short = if summary.len() > 50 {
                                format!("{}...", &summary[..47])
                            } else {
                                summary.clone()
                            };
                            ui.label(
                                egui::RichText::new(short)
                                    .size(11.0)
                                    .color(p.text_muted),
                            );
                        }
                    });
                }
            });

            // Sync the expanded state back.
            if expanded {
                tab_state.expanded_history = Some(job.job_id.clone());
            } else if tab_state.expanded_history.as_deref() == Some(&job.job_id) {
                tab_state.expanded_history = None;
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
