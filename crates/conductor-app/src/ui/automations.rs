use tokio::sync::mpsc;

use conductor_core::events::Action;
use conductor_core::state::*;
use egui_swift::prelude::*;

use crate::bridge::SharedState;

/// Automations view — create rules that trigger tasks from external events.
pub struct AutomationsView {
    shared: SharedState,
    tx: mpsc::UnboundedSender<Action>,
    show_add_form: bool,
    form: RuleForm,
    expanded_history: Option<String>,
}

impl AutomationsView {
    pub fn new(shared: SharedState, tx: mpsc::UnboundedSender<Action>) -> Self {
        Self {
            shared,
            tx,
            show_add_form: false,
            form: RuleForm::default(),
            expanded_history: None,
        }
    }
}

/// Form state for creating a new automation rule.
#[derive(Default)]
pub struct RuleForm {
    pub name: String,
    pub description: String,
    // Trigger selection: 0=Manual, 1=SlackMessage, 2=Webhook, 3=ChannelMessage, 4=Schedule
    pub trigger_type: u8,
    // Slack trigger fields.
    pub slack_channel: String,
    pub slack_channel_id: Option<String>,
    pub slack_keyword: String,
    // Webhook trigger fields.
    pub webhook_path: String,
    pub webhook_secret: String,
    // Channel trigger fields.
    pub channel_platform: String,
    pub channel_filter: String,
    pub channel_keyword: String,
    // Schedule trigger fields.
    pub schedule_type: u8, // 0=interval, 1=cron
    pub interval_minutes: u32,
    pub cron_expression: String,
    // Steps.
    pub steps: Vec<StepForm>,
}

/// Form state for a single step in the automation flow.
#[derive(Default)]
pub struct StepForm {
    pub name: String,
    // 0=RunPrompt, 1=RunJob, 2=Notify, 3=Filter, 4=Delay, 5=Transform
    pub step_type: u8,
    pub prompt: String,
    pub include_event_context: bool,
    pub include_previous_output: bool,
    pub target_job_id: String,
    pub notify_message: String,
    pub filter_type: u8, // 0=Contains, 1=NotContains, 2=IsEmpty, 3=IsNotEmpty
    pub filter_text: String,
    pub delay_seconds: u64,
    pub transform_prompt: String,
    pub collapsed: bool,
}

impl View for AutomationsView {
    fn body(&mut self, ui: &mut egui::Ui) {
        let p = ui.palette();

        let state = self.shared.read();
        let rules = state.automation_rules.clone();
        drop(state);

        ui.centered_content(Layout::MAX_CONTENT_WIDTH, |ui| {
            // Header row.
            egui_swift::hstack!(ui, {
                Label::heading("Automations").show(ui);
                Spacer::trailing(ui, |ui| {
                    if Button::new("+ New Rule")
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
            Label::new("Trigger tasks from external events")
                .font(Font::Subheadline)
                .secondary()
                .show(ui);
            egui_swift::spacer!(ui, 12.0);

            // -- Add Rule form --
            if self.show_add_form {
                show_add_form(ui, &self.tx, &self.shared, &mut self.form, &mut self.show_add_form, &p);
                egui_swift::spacer!(ui, 8.0);
                Divider::new().show(ui);
                egui_swift::spacer!(ui, 8.0);
            }

            // -- Rules list --
            if rules.is_empty() && !self.show_add_form {
                EmptyState::new("No automations yet")
                    .icon("\u{26a1}")
                    .subtitle("Create a rule to trigger tasks from Slack messages, webhooks, or other events.")
                    .show(ui);
                return;
            }

            ScrollView::vertical().show(ui, |ui| {
                for rule in &rules {
                    show_rule_card(ui, rule, &self.tx, &mut self.expanded_history, &p);
                }
            });
        });
    }
}

fn show_add_form(
    ui: &mut egui::Ui,
    tx: &mpsc::UnboundedSender<Action>,
    shared: &crate::bridge::SharedState,
    form: &mut RuleForm,
    show_add_form: &mut bool,
    p: &Palette,
) {
    Card::new().border_color(p.border).shadow(true).show(ui, |ui| {
        Label::new("New Automation Rule")
            .font(Font::Headline)
            .bold(true)
            .show(ui);
        egui_swift::spacer!(ui, 8.0);

        // Name + description.
        TextField::new(&mut form.name)
            .label("Name")
            .placeholder("e.g. Slack deploy alert summarizer")
            .show(ui);
        egui_swift::spacer!(ui, 4.0);
        TextField::new(&mut form.description)
            .label("Description")
            .placeholder("What does this automation do?")
            .show(ui);
        egui_swift::spacer!(ui, 12.0);

        // -- Trigger --
        FormSection::new().header("When (Trigger)").show(ui, |ui| {
            let trigger_types: Vec<(u8, &str)> = vec![
                (0, "Manual only"),
                (1, "Slack message"),
                (2, "Webhook"),
                (3, "Channel message"),
                (4, "Schedule"),
            ];
            RadioGroup::new(&mut form.trigger_type, &trigger_types).show(ui);
        });

        egui_swift::spacer!(ui, 4.0);

        match form.trigger_type {
            1 => {
                // Slack trigger config with autocomplete channel picker.
                slack_channel_picker(ui, shared, form, p);
                egui_swift::spacer!(ui, 4.0);
                TextField::new(&mut form.slack_keyword)
                    .label("Keyword filter (optional)")
                    .placeholder("e.g. deploy, incident")
                    .show(ui);
                Label::new("Leave empty to trigger on all messages in this channel")
                    .font(Font::Caption)
                    .muted()
                    .show(ui);
            }
            2 => {
                // Webhook config.
                TextField::new(&mut form.webhook_path)
                    .label("Webhook path")
                    .placeholder("/hooks/my-automation")
                    .show(ui);
                egui_swift::spacer!(ui, 4.0);
                TextField::new(&mut form.webhook_secret)
                    .label("Secret (optional)")
                    .placeholder("shared secret for verification")
                    .show(ui);
                Label::new("The webhook will be available at localhost:8090{path}")
                    .font(Font::Caption)
                    .muted()
                    .show(ui);
            }
            3 => {
                // Channel message config.
                TextField::new(&mut form.channel_platform)
                    .label("Platform")
                    .placeholder("e.g. slack, discord, telegram")
                    .show(ui);
                egui_swift::spacer!(ui, 4.0);
                TextField::new(&mut form.channel_filter)
                    .label("Channel filter (optional)")
                    .placeholder("#channel-name")
                    .show(ui);
                egui_swift::spacer!(ui, 4.0);
                TextField::new(&mut form.channel_keyword)
                    .label("Keyword filter (optional)")
                    .placeholder("keyword1, keyword2")
                    .show(ui);
            }
            4 => {
                // Schedule trigger config.
                let sched_types: Vec<(u8, &str)> = vec![(0, "Interval"), (1, "Cron")];
                RadioGroup::new(&mut form.schedule_type, &sched_types).show(ui);
                egui_swift::spacer!(ui, 4.0);

                match form.schedule_type {
                    0 => {
                        egui_swift::hstack!(ui, {
                            Label::new("Every").font(Font::Callout).show(ui);
                            ui.add(
                                egui::DragValue::new(&mut form.interval_minutes).range(1..=10080),
                            );
                            Label::new("minutes").font(Font::Callout).show(ui);
                        });
                    }
                    _ => {
                        TextField::new(&mut form.cron_expression)
                            .label("Cron expression")
                            .placeholder("0 9 * * 1-5")
                            .show(ui);
                        Label::new("e.g. \"0 9 * * 1-5\" = 9 AM weekdays")
                            .font(Font::Caption)
                            .muted()
                            .show(ui);
                    }
                }
            }
            _ => {
                Label::new("This rule will only fire when you click \"Run\" manually.")
                    .font(Font::Caption)
                    .muted()
                    .show(ui);
            }
        }

        egui_swift::spacer!(ui, 12.0);

        // -- Steps --
        FormSection::new().header("Steps").show(ui, |ui| {
            if form.steps.is_empty() {
                Label::new("No steps yet. Add a step to define what happens when the trigger fires.")
                    .font(Font::Caption)
                    .muted()
                    .show(ui);
            }
        });

        let mut remove_idx: Option<usize> = None;
        let mut swap: Option<(usize, usize)> = None;
        let num_steps = form.steps.len();
        for i in 0..num_steps {
            let step_num = i + 1;
            let header = if form.steps[i].name.is_empty() {
                format!("Step {step_num}")
            } else {
                format!("Step {step_num}: {}", form.steps[i].name)
            };

            let step = &mut form.steps[i];
            DisclosureGroup::new(&header, &mut step.collapsed).show(ui, |ui| {
                TextField::new(&mut step.name)
                    .label("Step name")
                    .placeholder("e.g. Summarize message")
                    .show(ui);
                egui_swift::spacer!(ui, 4.0);

                let step_types: Vec<(u8, &str)> = vec![
                    (0, "Run AI prompt"),
                    (1, "Trigger job"),
                    (2, "Send notification"),
                    (3, "Filter"),
                    (4, "Delay"),
                    (5, "Transform"),
                ];
                RadioGroup::new(&mut step.step_type, &step_types).show(ui);
                egui_swift::spacer!(ui, 4.0);

                match step.step_type {
                    0 => {
                        TextField::new(&mut step.prompt)
                            .label("Prompt")
                            .placeholder("What should the AI do?")
                            .multiline(3)
                            .show(ui);
                        egui_swift::spacer!(ui, 4.0);
                        Toggle::new(&mut step.include_event_context)
                            .label("Include trigger event context")
                            .show(ui);
                        Toggle::new(&mut step.include_previous_output)
                            .label("Include previous step output")
                            .show(ui);
                    }
                    1 => {
                        TextField::new(&mut step.target_job_id)
                            .label("Job ID")
                            .placeholder("ID of the job to trigger")
                            .show(ui);
                    }
                    2 => {
                        TextField::new(&mut step.notify_message)
                            .label("Message")
                            .placeholder("Use {previous_output} to include prior step result")
                            .multiline(2)
                            .show(ui);
                    }
                    3 => {
                        let filter_types: Vec<(u8, &str)> = vec![
                            (0, "Contains"),
                            (1, "Does not contain"),
                            (2, "Is empty"),
                            (3, "Is not empty"),
                        ];
                        RadioGroup::new(&mut step.filter_type, &filter_types).show(ui);
                        if step.filter_type <= 1 {
                            egui_swift::spacer!(ui, 4.0);
                            TextField::new(&mut step.filter_text)
                                .label("Text to match")
                                .placeholder("keyword")
                                .show(ui);
                        }
                        Label::new("Stops the pipeline if the condition is not met on the previous step's output")
                            .font(Font::Caption)
                            .muted()
                            .show(ui);
                    }
                    4 => {
                        egui_swift::hstack!(ui, {
                            Label::new("Wait").font(Font::Callout).show(ui);
                            ui.add(egui::DragValue::new(&mut step.delay_seconds).range(1..=3600));
                            Label::new("seconds").font(Font::Callout).show(ui);
                        });
                    }
                    5 => {
                        TextField::new(&mut step.transform_prompt)
                            .label("Transform instruction")
                            .placeholder("e.g. Summarize this in 3 bullet points")
                            .multiline(2)
                            .show(ui);
                        Label::new("The previous step's output will be passed as input to the AI")
                            .font(Font::Caption)
                            .muted()
                            .show(ui);
                    }
                    _ => {}
                }

                egui_swift::spacer!(ui, 4.0);
                egui_swift::hstack!(ui, {
                    if i > 0 {
                        if Button::new("\u{2191}") // ↑
                            .style(ButtonStyle::Bordered)
                            .small(true)
                            .show(ui)
                            .clicked()
                        {
                            swap = Some((i, i - 1));
                        }
                    }
                    if i < num_steps - 1 {
                        if Button::new("\u{2193}") // ↓
                            .style(ButtonStyle::Bordered)
                            .small(true)
                            .show(ui)
                            .clicked()
                        {
                            swap = Some((i, i + 1));
                        }
                    }
                    if Button::new("Remove")
                        .style(ButtonStyle::Destructive)
                        .small(true)
                        .show(ui)
                        .clicked()
                    {
                        remove_idx = Some(i);
                    }
                });
            });
            egui_swift::spacer!(ui, 4.0);
        }

        // Apply deferred mutations.
        if let Some(idx) = remove_idx {
            form.steps.remove(idx);
        }
        if let Some((a, b)) = swap {
            form.steps.swap(a, b);
        }

        if Button::new("+ Add Step")
            .style(ButtonStyle::Bordered)
            .small(true)
            .show(ui)
            .clicked()
        {
            form.steps.push(StepForm::default());
        }

        egui_swift::spacer!(ui, 12.0);

        // Buttons.
        let can_create = !form.name.trim().is_empty() && !form.steps.is_empty();

        ButtonRow::show(ui, |ui| {
            if Button::new("Cancel")
                .style(ButtonStyle::Bordered)
                .show(ui)
                .clicked()
            {
                *show_add_form = false;
            }
            if Button::new("Create Rule")
                .style(ButtonStyle::BorderedProminent)
                .enabled(can_create)
                .show(ui)
                .clicked()
            {
                let trigger = build_trigger(form);
                let steps = build_steps(&form.steps);

                let rule = AutomationRule {
                    rule_id: uuid::Uuid::new_v4().to_string(),
                    name: form.name.clone(),
                    description: form.description.clone(),
                    enabled: true,
                    trigger,
                    action: None,
                    steps,
                    created_at: chrono::Utc::now(),
                    last_triggered: None,
                    trigger_count: 0,
                    history: Vec::new(),
                };

                let _ = tx.send(Action::CreateAutomation { rule });
                *form = RuleForm::default();
                *show_add_form = false;
            }
        });
    });
}

fn show_rule_card(
    ui: &mut egui::Ui,
    rule: &AutomationRule,
    tx: &mpsc::UnboundedSender<Action>,
    expanded_history: &mut Option<String>,
    p: &Palette,
) {
    let status_color = if rule.enabled {
        p.status_green
    } else {
        p.text_muted
    };

    Card::new().show(ui, |ui| {
        // Top row: status + name.
        egui_swift::hstack!(ui, {
            StatusDot::new(status_color).show(ui);
            Label::new(&rule.name).font(Font::Callout).bold(true).show(ui);
            if rule.enabled {
                Label::new("active")
                    .font(Font::Caption)
                    .color(p.status_green)
                    .show(ui);
            } else {
                Label::new("disabled")
                    .font(Font::Caption)
                    .muted()
                    .show(ui);
            }
        });

        // Flow visualization: trigger → steps.
        let trigger_desc = describe_trigger(&rule.trigger);
        show_flow_node(ui, &format!("\u{26a1} {trigger_desc}"), p.accent, p);

        for step in &rule.steps {
            show_flow_arrow(ui, p);
            let (icon, color) = step_icon_color(&step.step_type, p);
            let desc = format!("{icon} {}: {}", step.name, describe_step(&step.step_type));
            show_flow_node(ui, &desc, color, p);
        }

        // Stats row.
        if rule.trigger_count > 0 || rule.last_triggered.is_some() {
            egui_swift::spacer!(ui, 2.0);
            egui_swift::hstack!(ui, spacing: 12.0, {
                if let Some(last) = rule.last_triggered {
                    Label::new(&format!("Last: {}", last.format("%Y-%m-%d %H:%M")))
                        .font(Font::Footnote)
                        .muted()
                        .show(ui);
                }
                Label::new(&format!("{} runs", rule.trigger_count))
                    .font(Font::Footnote)
                    .muted()
                    .show(ui);

                // Count successes/failures.
                let successes = rule.history.iter().filter(|r| r.status == JobRunStatus::Success).count();
                let failures = rule.history.iter().filter(|r| r.status == JobRunStatus::Failure).count();
                if successes > 0 {
                    Label::new(&format!("{successes} ok"))
                        .font(Font::Footnote)
                        .color(p.status_green)
                        .show(ui);
                }
                if failures > 0 {
                    Label::new(&format!("{failures} failed"))
                        .font(Font::Footnote)
                        .color(p.status_red)
                        .show(ui);
                }
            });
        }

        egui_swift::spacer!(ui, 4.0);

        // Action buttons.
        egui_swift::hstack!(ui, {
            if Button::new("Run Now")
                .style(ButtonStyle::Bordered)
                .small(true)
                .show(ui)
                .clicked()
            {
                let _ = tx.send(Action::RunAutomation {
                    rule_id: rule.rule_id.clone(),
                    event_context: Some("Manual trigger from UI".into()),
                });
            }

            let toggle_label = if rule.enabled { "Disable" } else { "Enable" };
            if Button::new(toggle_label)
                .style(ButtonStyle::Bordered)
                .small(true)
                .show(ui)
                .clicked()
            {
                let _ = tx.send(Action::ToggleAutomation {
                    rule_id: rule.rule_id.clone(),
                    enabled: !rule.enabled,
                });
            }

            if Button::new("Delete")
                .style(ButtonStyle::Destructive)
                .small(true)
                .show(ui)
                .clicked()
            {
                let _ = tx.send(Action::DeleteAutomation {
                    rule_id: rule.rule_id.clone(),
                });
            }
        });

        // Expandable history.
        if !rule.history.is_empty() {
            let mut expanded = expanded_history.as_deref() == Some(&rule.rule_id);
            DisclosureGroup::new("Run History", &mut expanded).show(ui, |ui| {
                for run in rule.history.iter().rev().take(10) {
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
                        if let Some(ref event) = run.trigger_event {
                            let short = if event.len() > 40 {
                                format!("{}...", &event[..37])
                            } else {
                                event.clone()
                            };
                            Label::new(&short)
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
                    });
                }
            });

            if expanded {
                *expanded_history = Some(rule.rule_id.clone());
            } else if expanded_history.as_deref() == Some(&rule.rule_id) {
                *expanded_history = None;
            }
        }
    });
}

fn describe_trigger(trigger: &TriggerCondition) -> String {
    match trigger {
        TriggerCondition::SlackMessage { channel_name, keyword_filter } => {
            let base = format!("Slack message in {channel_name}");
            match keyword_filter {
                Some(kw) => format!("{base} containing \"{kw}\""),
                None => base,
            }
        }
        TriggerCondition::Webhook { path, .. } => {
            format!("Webhook POST to {path}")
        }
        TriggerCondition::ChannelMessage { platform_id, channel_filter, keyword_filter } => {
            let mut desc = format!("{platform_id} message");
            if let Some(ch) = channel_filter {
                desc.push_str(&format!(" in {ch}"));
            }
            if let Some(kw) = keyword_filter {
                desc.push_str(&format!(" containing \"{kw}\""));
            }
            desc
        }
        TriggerCondition::Schedule { definition } => {
            match definition {
                ScheduleDefinition::Interval { seconds } => format!("Every {seconds}s"),
                ScheduleDefinition::Cron { expression, .. } => format!("Cron: {expression}"),
                ScheduleDefinition::OneTime { datetime } => {
                    format!("Once at {}", datetime.format("%Y-%m-%d %H:%M"))
                }
            }
        }
        TriggerCondition::Manual => "Manual trigger".into(),
    }
}

fn describe_step(step: &StepAction) -> String {
    match step {
        StepAction::RunPrompt { prompt, .. } => {
            let preview = if prompt.len() > 50 {
                format!("{}...", &prompt[..47])
            } else {
                prompt.clone()
            };
            format!("Run prompt: \"{preview}\"")
        }
        StepAction::RunJob { job_id } => format!("Trigger job {job_id}"),
        StepAction::Notify { message } => {
            let preview = if message.len() > 50 {
                format!("{}...", &message[..47])
            } else {
                message.clone()
            };
            format!("Notify: \"{preview}\"")
        }
        StepAction::Filter { condition } => match condition {
            FilterCondition::Contains { text } => format!("Filter: contains \"{text}\""),
            FilterCondition::NotContains { text } => format!("Filter: not contains \"{text}\""),
            FilterCondition::IsEmpty => "Filter: is empty".into(),
            FilterCondition::IsNotEmpty => "Filter: is not empty".into(),
        },
        StepAction::Delay { seconds } => format!("Delay {seconds}s"),
        StepAction::Transform { prompt } => {
            let preview = if prompt.len() > 50 {
                format!("{}...", &prompt[..47])
            } else {
                prompt.clone()
            };
            format!("Transform: \"{preview}\"")
        }
    }
}

fn step_icon_color<'a>(step: &StepAction, p: &'a Palette) -> (&'static str, egui::Color32) {
    match step {
        StepAction::RunPrompt { .. } => ("\u{1f916}", p.accent),        // 🤖
        StepAction::RunJob { .. } => ("\u{25b6}", p.text_secondary),    // ▶
        StepAction::Notify { .. } => ("\u{1f514}", p.status_yellow),    // 🔔
        StepAction::Filter { .. } => ("\u{1f50d}", p.status_red),       // 🔍
        StepAction::Delay { .. } => ("\u{23f1}", p.text_muted),         // ⏱
        StepAction::Transform { .. } => ("\u{2728}", p.status_green),   // ✨
    }
}

fn show_flow_node(ui: &mut egui::Ui, text: &str, accent: egui::Color32, p: &Palette) {
    egui::Frame::NONE
        .fill(p.surface_raised)
        .corner_radius(egui::CornerRadius::same(6))
        .stroke(egui::Stroke::new(0.5, accent))
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            Label::new(text).font(Font::Caption).show(ui);
        });
}

fn show_flow_arrow(ui: &mut egui::Ui, p: &Palette) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 16.0), egui::Sense::hover());
    let center_x = rect.left() + 20.0;
    let top = rect.top() + 2.0;
    let bot = rect.bottom() - 2.0;
    // Vertical line.
    ui.painter().line_segment(
        [egui::pos2(center_x, top), egui::pos2(center_x, bot)],
        egui::Stroke::new(1.0, p.border),
    );
    // Small triangle.
    let tri_size = 3.0;
    ui.painter().add(egui::Shape::convex_polygon(
        vec![
            egui::pos2(center_x, bot),
            egui::pos2(center_x - tri_size, bot - tri_size * 1.5),
            egui::pos2(center_x + tri_size, bot - tri_size * 1.5),
        ],
        p.border,
        egui::Stroke::NONE,
    ));
}

fn build_trigger(form: &RuleForm) -> TriggerCondition {
    match form.trigger_type {
        1 => TriggerCondition::SlackMessage {
            channel_name: form.slack_channel.clone(),
            keyword_filter: if form.slack_keyword.trim().is_empty() {
                None
            } else {
                Some(form.slack_keyword.clone())
            },
        },
        2 => TriggerCondition::Webhook {
            path: form.webhook_path.clone(),
            secret: if form.webhook_secret.trim().is_empty() {
                None
            } else {
                Some(form.webhook_secret.clone())
            },
        },
        3 => TriggerCondition::ChannelMessage {
            platform_id: form.channel_platform.clone(),
            channel_filter: if form.channel_filter.trim().is_empty() {
                None
            } else {
                Some(form.channel_filter.clone())
            },
            keyword_filter: if form.channel_keyword.trim().is_empty() {
                None
            } else {
                Some(form.channel_keyword.clone())
            },
        },
        4 => TriggerCondition::Schedule {
            definition: match form.schedule_type {
                0 => ScheduleDefinition::Interval {
                    seconds: form.interval_minutes as u64 * 60,
                },
                _ => ScheduleDefinition::Cron {
                    expression: form.cron_expression.clone(),
                    timezone: "UTC".into(),
                },
            },
        },
        _ => TriggerCondition::Manual,
    }
}

fn build_steps(forms: &[StepForm]) -> Vec<AutomationStep> {
    forms
        .iter()
        .enumerate()
        .map(|(i, sf)| AutomationStep {
            step_id: uuid::Uuid::new_v4().to_string(),
            name: if sf.name.is_empty() {
                format!("Step {}", i + 1)
            } else {
                sf.name.clone()
            },
            position: i as u32,
            step_type: match sf.step_type {
                0 => StepAction::RunPrompt {
                    prompt: sf.prompt.clone(),
                    include_event_context: sf.include_event_context,
                    include_previous_output: sf.include_previous_output,
                    backend_override: None,
                    model_override: None,
                },
                1 => StepAction::RunJob {
                    job_id: sf.target_job_id.clone(),
                },
                2 => StepAction::Notify {
                    message: sf.notify_message.clone(),
                },
                3 => match sf.filter_type {
                    0 => StepAction::Filter {
                        condition: FilterCondition::Contains {
                            text: sf.filter_text.clone(),
                        },
                    },
                    1 => StepAction::Filter {
                        condition: FilterCondition::NotContains {
                            text: sf.filter_text.clone(),
                        },
                    },
                    2 => StepAction::Filter {
                        condition: FilterCondition::IsEmpty,
                    },
                    _ => StepAction::Filter {
                        condition: FilterCondition::IsNotEmpty,
                    },
                },
                4 => StepAction::Delay {
                    seconds: sf.delay_seconds.max(1),
                },
                _ => StepAction::Transform {
                    prompt: sf.transform_prompt.clone(),
                },
            },
            enabled: true,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Slack channel picker with autocomplete
// ---------------------------------------------------------------------------

/// A text field that shows matching Slack channels as the user types.
/// Validates that the entered channel exists and stores the channel ID.
fn slack_channel_picker(
    ui: &mut egui::Ui,
    shared: &crate::bridge::SharedState,
    form: &mut RuleForm,
    p: &Palette,
) {
    let state = shared.read();
    let slack_channels = state.slack.channels.clone();
    let slack_connected = state.slack.status == SlackStatus::Connected;
    drop(state);

    Label::new("Slack channel").font(Font::Callout).show(ui);
    egui_swift::spacer!(ui, 4.0);

    if !slack_connected {
        Label::new("Slack not connected. Connect in Settings > Channels first.")
            .font(Font::Caption)
            .color(p.status_red)
            .show(ui);
        return;
    }

    // Validation: check if current text matches a known channel.
    let query = form.slack_channel.trim_start_matches('#').to_lowercase();
    let exact_match = slack_channels.iter().find(|c| c.name.to_lowercase() == query);

    // Update the stored channel ID based on match.
    form.slack_channel_id = exact_match.map(|c| c.id.clone());

    // Show the text input with validation border color.
    let border = if form.slack_channel_id.is_some() {
        p.status_green
    } else if query.is_empty() {
        p.border
    } else {
        p.status_red
    };
    let resp = TextField::new(&mut form.slack_channel)
        .placeholder("# search channels...")
        .border_color(border)
        .show(ui);

    // Validation feedback.
    if let Some(ref matched) = exact_match {
        Label::new(&format!("\u{2713} #{} ({})", matched.name, matched.id))
            .font(Font::Caption)
            .color(p.status_green)
            .show(ui);
    } else if !query.is_empty() {
        Label::new("No matching channel found")
            .font(Font::Caption)
            .color(p.status_red)
            .show(ui);
    }

    // Compute matches for the dropdown and tab-complete.
    let matches: Vec<&SlackChannelInfo> = if !query.is_empty() && form.slack_channel_id.is_none() {
        slack_channels
            .iter()
            .filter(|c| c.name.to_lowercase().contains(&query))
            .take(8)
            .collect()
    } else {
        Vec::new()
    };

    // Tab-complete: if the field just lost focus due to Tab and we have matches,
    // select the top match and reclaim focus.
    let tab_pressed = resp.lost_focus()
        && ui.input(|i| i.key_pressed(egui::Key::Tab))
        && !matches.is_empty();
    if tab_pressed {
        form.slack_channel = matches[0].name.clone();
        form.slack_channel_id = Some(matches[0].id.clone());
        // Re-grab focus so the user stays in the field.
        resp.request_focus();
    }

    // Show suggestions dropdown when the field has focus and there are matches.
    if resp.has_focus() && !matches.is_empty() {
        egui::Frame::NONE
                .fill(p.surface_raised)
                .corner_radius(egui::CornerRadius::same(6))
                .stroke(egui::Stroke::new(0.5, p.border))
                .inner_margin(egui::Margin::symmetric(4, 4))
                .show(ui, |ui| {
                    for (i, ch) in matches.iter().enumerate() {
                        let prefix = if ch.is_private { "\u{1f512}" } else { "#" };
                        let label = format!("{prefix} {}", ch.name);
                        let is_top = i == 0;

                        let row_resp = ui.allocate_response(
                            egui::vec2(ui.available_width(), 26.0),
                            egui::Sense::click(),
                        );

                        // Highlight: top item always highlighted, others on hover.
                        if is_top || row_resp.hovered() {
                            let bg = if is_top { p.active_bg } else { p.hover_bg };
                            ui.painter().rect_filled(
                                row_resp.rect,
                                egui::CornerRadius::same(4),
                                bg,
                            );
                        }

                        let text_color = if is_top { p.text_primary } else { p.text_secondary };
                        ui.painter().text(
                            egui::pos2(row_resp.rect.left() + 8.0, row_resp.rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            &label,
                            egui::FontId::proportional(13.0),
                            text_color,
                        );

                        // Show "Tab to select" hint on the top item.
                        if is_top {
                            ui.painter().text(
                                egui::pos2(row_resp.rect.right() - 8.0, row_resp.rect.center().y),
                                egui::Align2::RIGHT_CENTER,
                                "Tab \u{21e5}",
                                egui::FontId::proportional(10.0),
                                p.text_muted,
                            );
                        }

                        if row_resp.clicked() {
                            form.slack_channel = ch.name.clone();
                            form.slack_channel_id = Some(ch.id.clone());
                            ui.memory_mut(|m| m.surrender_focus(resp.id));
                        }
                    }
                });
    }
}
