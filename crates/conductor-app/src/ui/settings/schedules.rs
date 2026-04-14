use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::state::*;

use crate::bridge::SharedState;

/// Persistent UI state for the schedules tab.
pub struct SchedulesTabState {
    pub show_add_form: bool,
    pub form: JobForm,
    /// Which job's history is expanded (job_id or empty).
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
    pub schedule_type: u8, // 0=once, 1=interval, 2=cron
    pub interval_minutes: u32,
    pub cron_expression: String,
    pub prompt: String,
    pub execution_mode: u8, // 0=isolated, 1=main_session
    pub delivery_mode: u8,  // 0=silent, 1=webhook
    pub webhook_url: String,
}

pub fn show(
    ui: &mut egui::Ui,
    shared: &SharedState,
    tx: &mpsc::UnboundedSender<Action>,
    tab_state: &mut SchedulesTabState,
) {
    let p = egui_swift::colors::palette(ui);

    ui.heading("Scheduled Tasks");
    ui.add_space(8.0);

    let state = shared.read();
    let jobs = state.scheduler.jobs.clone();
    drop(state);

    // Header row.
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{} job{}", jobs.len(), if jobs.len() == 1 { "" } else { "s" }))
                .color(p.text_secondary),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+ Add Job").clicked() {
                tab_state.show_add_form = !tab_state.show_add_form;
            }
        });
    });

    ui.add_space(8.0);

    // -- Add Job form --
    if tab_state.show_add_form {
        show_add_form(ui, tx, tab_state, &p);
        ui.add_space(12.0);
        ui.separator();
        ui.add_space(8.0);
    }

    // -- Job list --
    if jobs.is_empty() && !tab_state.show_add_form {
        ui.add_space(20.0);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("No scheduled tasks yet")
                    .size(15.0)
                    .color(p.text_muted),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Create a job to run AI prompts on a schedule.")
                    .size(12.0)
                    .color(p.text_muted),
            );
        });
        return;
    }

    for job in &jobs {
        show_job_card(ui, job, tx, tab_state, &p);
        ui.add_space(4.0);
    }
}

fn show_add_form(
    ui: &mut egui::Ui,
    tx: &mpsc::UnboundedSender<Action>,
    tab_state: &mut SchedulesTabState,
    p: &egui_swift::colors::Palette,
) {
    let form = &mut tab_state.form;

    egui::Frame::NONE
        .fill(p.surface_raised)
        .corner_radius(egui::CornerRadius::same(8))
        .stroke(egui::Stroke::new(0.5, p.border))
        .inner_margin(egui::Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.label(egui::RichText::new("New Scheduled Job").strong().size(14.0));
            ui.add_space(8.0);

            // Name.
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut form.name);
            });

            // Schedule type.
            ui.horizontal(|ui| {
                ui.label("Schedule:");
                ui.radio_value(&mut form.schedule_type, 1, "Interval");
                ui.radio_value(&mut form.schedule_type, 2, "Cron");
                ui.radio_value(&mut form.schedule_type, 0, "One-time");
            });

            match form.schedule_type {
                1 => {
                    ui.horizontal(|ui| {
                        ui.label("Every");
                        ui.add(egui::DragValue::new(&mut form.interval_minutes).range(1..=10080));
                        ui.label("minutes");
                    });
                }
                2 => {
                    ui.horizontal(|ui| {
                        ui.label("Cron:");
                        ui.text_edit_singleline(&mut form.cron_expression);
                    });
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

            // Prompt.
            ui.label("Prompt:");
            ui.add(
                egui::TextEdit::multiline(&mut form.prompt)
                    .desired_rows(3)
                    .hint_text("What should the AI do?"),
            );

            // Execution mode.
            ui.horizontal(|ui| {
                ui.label("Mode:");
                ui.radio_value(&mut form.execution_mode, 0, "Isolated");
                ui.radio_value(&mut form.execution_mode, 1, "Main session");
            });

            // Delivery.
            ui.horizontal(|ui| {
                ui.label("Deliver:");
                ui.radio_value(&mut form.delivery_mode, 0, "Silent (log only)");
                ui.radio_value(&mut form.delivery_mode, 1, "Webhook");
            });
            if form.delivery_mode == 1 {
                ui.horizontal(|ui| {
                    ui.label("URL:");
                    ui.text_edit_singleline(&mut form.webhook_url);
                });
            }

            ui.add_space(8.0);

            // Buttons.
            ui.horizontal(|ui| {
                let can_create = !form.name.trim().is_empty() && !form.prompt.trim().is_empty();
                if ui
                    .add_enabled(can_create, egui::Button::new("Create Job"))
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

                    // Reset form.
                    *form = JobForm::default();
                    tab_state.show_add_form = false;
                }

                if ui.button("Cancel").clicked() {
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
    p: &egui_swift::colors::Palette,
) {
    let status_color = if job.enabled { p.status_green } else { p.text_muted };

    egui::Frame::NONE
        .fill(p.surface_raised)
        .corner_radius(egui::CornerRadius::same(8))
        .stroke(egui::Stroke::new(0.5, p.border_subtle))
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            // Top row: name, schedule, status.
            ui.horizontal(|ui| {
                // Status dot.
                let (dot_rect, _) =
                    ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                ui.painter()
                    .circle_filled(dot_rect.center(), 4.0, status_color);

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

            // Action buttons.
            ui.horizontal(|ui| {
                if ui.small_button("Run Now").clicked() {
                    let _ = tx.send(Action::RunJobNow {
                        job_id: job.job_id.clone(),
                    });
                }

                let toggle_label = if job.enabled { "Disable" } else { "Enable" };
                if ui.small_button(toggle_label).clicked() {
                    let _ = tx.send(Action::ToggleJob {
                        job_id: job.job_id.clone(),
                        enabled: !job.enabled,
                    });
                }

                if ui.small_button("Delete").clicked() {
                    let _ = tx.send(Action::DeleteJob {
                        job_id: job.job_id.clone(),
                    });
                }

                let history_label = if tab_state.expanded_history.as_deref() == Some(&job.job_id) {
                    "Hide History"
                } else {
                    "History"
                };
                if ui.small_button(history_label).clicked() {
                    if tab_state.expanded_history.as_deref() == Some(&job.job_id) {
                        tab_state.expanded_history = None;
                    } else {
                        tab_state.expanded_history = Some(job.job_id.clone());
                    }
                }
            });

            // Execution history (expandable).
            if tab_state.expanded_history.as_deref() == Some(&job.job_id) && !job.history.is_empty()
            {
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                for run in job.history.iter().rev().take(10) {
                    let status_icon = match run.status {
                        JobRunStatus::Success => "\u{2713}",
                        JobRunStatus::Failure => "\u{2717}",
                        JobRunStatus::Running => "\u{25cf}",
                        JobRunStatus::Cancelled => "\u{2014}",
                    };
                    let run_color = match run.status {
                        JobRunStatus::Success => p.status_green,
                        JobRunStatus::Failure => p.status_red,
                        JobRunStatus::Running => p.accent,
                        JobRunStatus::Cancelled => p.text_muted,
                    };

                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(status_icon).color(run_color).size(12.0),
                        );
                        ui.label(
                            egui::RichText::new(run.started_at.format("%m-%d %H:%M").to_string())
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
                                egui::RichText::new(err).size(11.0).color(p.status_red),
                            );
                        }
                        if let Some(ref summary) = run.output_summary {
                            let short = if summary.len() > 50 {
                                format!("{}...", &summary[..47])
                            } else {
                                summary.clone()
                            };
                            ui.label(
                                egui::RichText::new(short).size(11.0).color(p.text_muted),
                            );
                        }
                    });
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
