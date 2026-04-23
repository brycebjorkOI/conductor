use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type SessionId = String;
pub type MessageId = String;
pub type JobId = String;
pub type RuleId = String;
pub type StepId = String;

// ---------------------------------------------------------------------------
// Top-level application state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub current_view: ViewMode,
    pub panel_visible: bool,
    pub settings_open: bool,
    pub settings_tab: SettingsTab,
    pub notifications_open: bool,

    pub active_session_id: SessionId,
    pub sessions: HashMap<SessionId, Session>,

    pub backend_registry: Vec<BackendStatus>,
    pub default_backend_id: Option<String>,
    pub fallback_order: Vec<String>,

    pub tray_state: TrayState,

    pub voice: VoiceState,
    pub channels: HashMap<String, ChannelState>,
    pub scheduler: SchedulerState,
    pub job_history: Vec<JobHistoryEntry>,
    pub automation_rules: Vec<AutomationRule>,
    pub connectors: HashMap<String, ConnectorState>,
    pub server_link: ServerConnectionState,
    pub preferences: LearnedPreferences,
    pub notifications: Vec<Notification>,
    pub onboarding_step: Option<u32>,
    pub mcp_servers: HashMap<String, Vec<McpServerEntry>>,
    pub media_devices: MediaDeviceState,
    pub slack: SlackConnectionState,

    pub config: crate::config::AppConfig,
}

impl Default for AppState {
    fn default() -> Self {
        let session_id = uuid::Uuid::new_v4().to_string();
        let mut sessions = HashMap::new();
        sessions.insert(
            session_id.clone(),
            Session::new(session_id.clone(), SessionType::Primary, "anthropic"),
        );

        Self {
            current_view: ViewMode::Chat,
            panel_visible: true,
            settings_open: false,
            settings_tab: SettingsTab::About,
            notifications_open: false,
            active_session_id: session_id,
            sessions,
            backend_registry: Vec::new(),
            default_backend_id: None,
            fallback_order: Vec::new(),
            tray_state: TrayState::Idle,
            voice: VoiceState::default(),
            channels: HashMap::new(),
            scheduler: SchedulerState::default(),
            job_history: Vec::new(),
            automation_rules: Vec::new(),
            connectors: HashMap::new(),
            server_link: ServerConnectionState::default(),
            preferences: LearnedPreferences::default(),
            notifications: Vec::new(),
            onboarding_step: None,
            mcp_servers: HashMap::new(),
            media_devices: MediaDeviceState::default(),
            slack: SlackConnectionState::default(),
            config: crate::config::AppConfig::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewMode {
    Chat,
    Settings,
    Onboarding,
    Canvas,
    Jobs,
    Automations,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettingsTab {
    About,
    General,
    Backends,
    Channels,
    Sessions,
    Plugins,
    Skills,
    McpServers,
    Permissions,
    Debug,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrayState {
    Idle,
    Working,
    Paused,
    VoiceActive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThinkingLevel {
    Off,
    Minimal,
    Low,
    Medium,
    High,
    ExtraHigh,
}

// ---------------------------------------------------------------------------
// Backend status
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendStatus {
    pub backend_id: String,
    pub display_name: String,
    pub discovery_state: DiscoveryState,
    pub binary_path: Option<PathBuf>,
    pub version: Option<String>,
    pub auth_state: AuthState,
    pub available_models: Vec<ModelEntry>,
    pub default_model: Option<String>,
    pub capabilities: CapabilitySet,
    pub enabled: bool,
    pub custom_args: Vec<String>,
    pub env_overrides: HashMap<String, String>,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscoveryState {
    NotScanned,
    Scanning,
    Found,
    NotFound,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthState {
    Unknown,
    Authenticated,
    NotAuthenticated,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub model_id: String,
    pub display_name: String,
    pub context_window: Option<u64>,
    pub pricing: Option<PricingInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingInfo {
    pub input_per_million: f64,
    pub output_per_million: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilitySet {
    pub streaming: bool,
    pub interactive_session: bool,
    pub tool_reporting: bool,
    pub vision_input: bool,
    pub thinking_levels: bool,
    pub image_generation: bool,
    pub plan_mode: bool,
}

// ---------------------------------------------------------------------------
// Session
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub session_type: SessionType,
    pub created_at: DateTime<Utc>,
    pub backend_id: String,
    pub model_id: Option<String>,
    pub messages: Vec<Message>,
    pub streaming: Option<StreamingState>,
    pub active_tool_cards: Vec<ToolCard>,
    pub project_binding: Option<ProjectContext>,
    pub usage_totals: UsageTotals,
    pub metadata: HashMap<String, String>,
    pub display_name: Option<String>,
}

impl Session {
    pub fn new(id: SessionId, session_type: SessionType, backend_id: &str) -> Self {
        Self {
            id,
            session_type,
            created_at: Utc::now(),
            backend_id: backend_id.to_string(),
            model_id: None,
            messages: Vec::new(),
            streaming: None,
            active_tool_cards: Vec::new(),
            project_binding: None,
            usage_totals: UsageTotals::default(),
            metadata: HashMap::new(),
            display_name: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionType {
    Primary,
    Channel,
    Scheduled,
    SubAgent,
}

// ---------------------------------------------------------------------------
// Message
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub attachments: Vec<Attachment>,
    pub tool_cards: Vec<ToolCard>,
    pub usage: Option<UsageMetrics>,
    pub backend_id: Option<String>,
    pub model_id: Option<String>,
    pub duration_ms: Option<u64>,
    pub status: MessageStatus,
    pub thinking_content: Option<String>,
}

impl Message {
    pub fn user(content: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            role: MessageRole::User,
            content,
            timestamp: Utc::now(),
            attachments: Vec::new(),
            tool_cards: Vec::new(),
            usage: None,
            backend_id: None,
            model_id: None,
            duration_ms: None,
            status: MessageStatus::Complete,
            thinking_content: None,
        }
    }

    pub fn assistant_streaming() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            role: MessageRole::Assistant,
            content: String::new(),
            timestamp: Utc::now(),
            attachments: Vec::new(),
            tool_cards: Vec::new(),
            usage: None,
            backend_id: None,
            model_id: None,
            duration_ms: None,
            status: MessageStatus::Streaming,
            thinking_content: None,
        }
    }

    pub fn system(content: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            role: MessageRole::System,
            content,
            timestamp: Utc::now(),
            attachments: Vec::new(),
            tool_cards: Vec::new(),
            usage: None,
            backend_id: None,
            model_id: None,
            duration_ms: None,
            status: MessageStatus::Complete,
            thinking_content: None,
        }
    }

    pub fn error(content: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            role: MessageRole::Error,
            content,
            timestamp: Utc::now(),
            attachments: Vec::new(),
            tool_cards: Vec::new(),
            usage: None,
            backend_id: None,
            model_id: None,
            duration_ms: None,
            status: MessageStatus::Error,
            thinking_content: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageStatus {
    Complete,
    Streaming,
    Cancelled,
    Error,
}

// ---------------------------------------------------------------------------
// Sub-structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingState {
    pub accumulated_text: String,
    pub is_active: bool,
    pub can_cancel: bool,
    /// When inside a sub-agent call, this holds the Agent tool's name
    /// so subsequent events can be routed to its sub_steps.
    #[serde(default)]
    pub active_sub_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCard {
    pub tool_name: String,
    pub phase: ToolPhase,
    pub arguments: HashMap<String, serde_json::Value>,
    pub result: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
    pub timestamp: DateTime<Utc>,
    /// Internal steps for sub-agent tool calls (Agent tool_use).
    #[serde(default)]
    pub sub_steps: Vec<SubAgentStep>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolPhase {
    Started,
    Completed,
    Failed,
}

/// A single step inside a sub-agent's execution timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentStep {
    pub kind: SubStepKind,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

/// What kind of step this is inside a sub-agent flow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubStepKind {
    /// Sub-agent called a tool (name).
    ToolUse(String),
    /// Tool returned a result.
    ToolResult,
    /// Sub-agent reasoning/text.
    Reasoning,
    /// Sub-agent completed.
    Done,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub file_path: PathBuf,
    pub file_type: FileType,
    pub display_name: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    Image,
    Pdf,
    Audio,
    Video,
    Other,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageMetrics {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub estimated_cost: Option<f64>,
    pub cache_read_tokens: Option<u64>,
    pub cache_write_tokens: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageTotals {
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost: f64,
    pub message_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub repository_path: PathBuf,
    pub current_branch: Option<String>,
    pub working_directory: PathBuf,
}

// ---------------------------------------------------------------------------
// Voice
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VoiceState {
    pub mode: VoiceMode,
    pub committed_text: String,
    pub volatile_text: String,
    pub audio_level: f32,
    pub session_elapsed_ms: u64,
    pub tts_speaking: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceMode {
    #[default]
    Off,
    WakeWordListening,
    PushToTalkActive,
    TalkMode,
}

// ---------------------------------------------------------------------------
// Channel
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelState {
    pub platform_id: String,
    pub display_name: String,
    pub connection_state: ChannelConnectionState,
    pub error_message: Option<String>,
    pub config: ChannelConfig,
    pub stats: ChannelStats,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Error,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub enabled: bool,
    pub dm_only: bool,
    pub allowed_ids: Vec<String>,
    pub threaded_replies: bool,
    pub strip_mentions: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChannelStats {
    pub messages_received: u64,
    pub messages_sent: u64,
    pub last_activity: Option<DateTime<Utc>>,
    pub uptime_seconds: u64,
}

// ---------------------------------------------------------------------------
// Scheduler
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchedulerState {
    pub jobs: Vec<ScheduledJob>,
    pub next_check: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    pub job_id: JobId,
    pub name: String,
    pub enabled: bool,
    pub schedule: ScheduleDefinition,
    pub payload: JobPayload,
    pub delivery: DeliveryConfig,
    pub retry_policy: RetryPolicy,
    pub history: Vec<JobRun>,
    pub next_run: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleDefinition {
    OneTime {
        datetime: DateTime<Utc>,
    },
    Interval {
        seconds: u64,
    },
    Cron {
        expression: String,
        timezone: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobPayload {
    pub prompt: String,
    pub execution_mode: ExecutionMode,
    pub backend_override: Option<String>,
    pub model_override: Option<String>,
    pub thinking_level: Option<ThinkingLevel>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    MainSession,
    Isolated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryConfig {
    ChannelAnnounce { channel_id: String },
    Webhook { url: String },
    Silent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub backoff_strategy: BackoffStrategy,
    pub initial_delay_seconds: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff_strategy: BackoffStrategy::Exponential,
            initial_delay_seconds: 5,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackoffStrategy {
    None,
    Linear,
    Exponential,
}

/// A top-level record of a task that was run in the app (scheduled or manual).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobHistoryEntry {
    pub run_id: String,
    pub job_name: String,
    pub job_id: Option<JobId>,
    pub trigger: JobTrigger,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub status: JobRunStatus,
    pub error: Option<String>,
    pub output_summary: Option<String>,
    pub backend_id: Option<String>,
    pub prompt_preview: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobTrigger {
    Scheduled,
    Manual,
    Channel,
    Automation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRun {
    pub run_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub status: JobRunStatus,
    pub error: Option<String>,
    pub output_summary: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobRunStatus {
    Running,
    Success,
    Failure,
    Cancelled,
}

// ---------------------------------------------------------------------------
// Automation Rules
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRule {
    pub rule_id: RuleId,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub trigger: TriggerCondition,
    /// Legacy single-action field — kept for backward-compatible deserialization.
    /// Migrated to `steps` on load; never written back.
    #[serde(default, skip_serializing)]
    pub action: Option<AutomationAction>,
    /// Ordered list of steps to execute when the trigger fires.
    #[serde(default)]
    pub steps: Vec<AutomationStep>,
    pub created_at: DateTime<Utc>,
    pub last_triggered: Option<DateTime<Utc>>,
    pub trigger_count: u64,
    pub history: Vec<AutomationRunEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerCondition {
    /// Fires when a message matching the filter arrives on a Slack channel.
    SlackMessage {
        channel_name: String,
        keyword_filter: Option<String>,
    },
    /// Fires when an HTTP request hits a local webhook endpoint.
    Webhook {
        path: String,
        secret: Option<String>,
    },
    /// Fires when a message arrives on any connected channel platform.
    ChannelMessage {
        platform_id: String,
        channel_filter: Option<String>,
        keyword_filter: Option<String>,
    },
    /// Fires on a cron/interval schedule (reuses existing scheduler infra).
    Schedule {
        definition: ScheduleDefinition,
    },
    /// Only fires when the user clicks "Run" manually.
    Manual,
}

/// Legacy action type — kept only for deserializing old automations.json files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutomationAction {
    RunPrompt {
        prompt: String,
        include_event_context: bool,
        backend_override: Option<String>,
        model_override: Option<String>,
    },
    RunJob {
        job_id: JobId,
    },
    Notify {
        message: String,
    },
}

// -- Multi-step automation types --

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationStep {
    pub step_id: StepId,
    pub name: String,
    pub position: u32,
    pub step_type: StepAction,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepAction {
    /// Run an AI prompt.
    RunPrompt {
        prompt: String,
        include_event_context: bool,
        include_previous_output: bool,
        backend_override: Option<String>,
        model_override: Option<String>,
    },
    /// Trigger an existing scheduled job.
    RunJob {
        job_id: JobId,
    },
    /// Send a notification. Use `{previous_output}` in the message to interpolate.
    Notify {
        message: String,
    },
    /// Stop the pipeline if a condition on the previous step's output is not met.
    Filter {
        condition: FilterCondition,
    },
    /// Pause for a duration before continuing.
    Delay {
        seconds: u64,
    },
    /// Transform the previous output via an AI prompt.
    Transform {
        prompt: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterCondition {
    Contains { text: String },
    NotContains { text: String },
    IsEmpty,
    IsNotEmpty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationRunEntry {
    pub run_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub status: JobRunStatus,
    pub trigger_event: Option<String>,
    pub error: Option<String>,
    pub output_summary: Option<String>,
    #[serde(default)]
    pub step_results: Vec<StepRunResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepRunResult {
    pub step_id: StepId,
    pub step_name: String,
    pub status: JobRunStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub output: Option<String>,
    pub error: Option<String>,
    pub skipped: bool,
}

// ---------------------------------------------------------------------------
// Connector
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorState {
    pub service_id: String,
    pub display_name: String,
    pub category: String,
    pub auth_state: ConnectorAuthState,
    pub granted_scopes: Vec<String>,
    pub last_used: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectorAuthState {
    NotConnected,
    Active,
    Expired,
    Error,
}

// ---------------------------------------------------------------------------
// Companion server
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerConnectionState {
    pub connection_mode: ServerConnectionMode,
    pub status: ServerStatus,
    pub server_version: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerConnectionMode {
    #[default]
    Disconnected,
    Local,
    SshTunnel,
    Direct,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error,
}

// ---------------------------------------------------------------------------
// Adaptive learning
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LearnedPreferences {
    pub aggregated: HashMap<String, PreferenceWeight>,
    pub per_backend_satisfaction: HashMap<String, f64>,
    pub last_aggregation: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceWeight {
    pub category: String,
    pub value: String,
    pub confidence: f64,
    pub last_reinforced: DateTime<Utc>,
    pub signal_count: u32,
}

// ---------------------------------------------------------------------------
// Notification
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub body: String,
    pub severity: NotificationSeverity,
    pub timestamp: DateTime<Utc>,
    pub dismissed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationSeverity {
    Info,
    Warning,
    Error,
}

// ---------------------------------------------------------------------------
// MCP
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerEntry {
    pub name: String,
    pub transport: McpTransport,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub url: Option<String>,
    pub env_vars: HashMap<String, String>,
    pub scope: McpScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum McpTransport {
    Stdio,
    Sse,
    StreamableHttp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum McpScope {
    User,
    Project,
}

// ---------------------------------------------------------------------------
// Media
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MediaDeviceState {
    pub cameras: Vec<CameraDevice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraDevice {
    pub device_id: String,
    pub display_name: String,
    pub is_available: bool,
}

// ---------------------------------------------------------------------------
// Slack
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SlackConnectionState {
    pub status: SlackStatus,
    pub workspace_name: Option<String>,
    pub channels: Vec<SlackChannelInfo>,
    pub monitored_channels: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlackStatus {
    #[default]
    Disconnected,
    Checking,
    Connected,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackChannelInfo {
    pub id: String,
    pub name: String,
    pub is_private: bool,
}

// ---------------------------------------------------------------------------
// Approval
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub request_id: String,
    pub tool_name: String,
    pub executable: String,
    pub arguments: Vec<String>,
    pub resolved_path: Option<String>,
    pub host: String,
}
