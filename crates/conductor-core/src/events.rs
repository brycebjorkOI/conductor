use std::collections::HashMap;
use std::path::PathBuf;

use crate::state::*;

/// Actions flow from the UI thread to the async runtime.
#[derive(Debug, Clone)]
pub enum Action {
    // -- Conversation --
    SendMessage {
        session_id: SessionId,
        text: String,
        attachments: Vec<PathBuf>,
    },
    AbortGeneration {
        session_id: SessionId,
    },
    SwitchBackend {
        backend_id: String,
    },
    SwitchModel {
        model_id: String,
    },
    SwitchSession {
        session_id: SessionId,
    },
    NewSession,
    ResetSession {
        session_id: SessionId,
    },
    CompactSession {
        session_id: SessionId,
    },
    SetThinkingLevel {
        level: ThinkingLevel,
    },

    // -- Backend management --
    RescanBackends,
    ConfigureBackend {
        backend_id: String,
        config: BackendConfigUpdate,
    },
    SetFallbackOrder {
        ordered_ids: Vec<String>,
    },

    // -- Channels --
    StartChannel {
        platform_id: String,
    },
    StopChannel {
        platform_id: String,
    },

    // -- Scheduling --
    CreateJob {
        definition: ScheduledJob,
    },
    ToggleJob {
        job_id: JobId,
        enabled: bool,
    },
    RunJobNow {
        job_id: JobId,
    },
    DeleteJob {
        job_id: JobId,
    },

    // -- Execution approval --
    ApproveExecution {
        request_id: String,
        response: ApprovalResponse,
    },

    // -- System --
    TogglePanel,
    OpenSettings {
        tab: Option<SettingsTab>,
    },
    CloseSettings,
    ToggleNotifications,
    ToggleSchedules,
    DismissNotification {
        id: String,
    },
    DismissAllNotifications,
    SaveConfig,
    Quit,
}

#[derive(Debug, Clone)]
pub struct BackendConfigUpdate {
    pub enabled: Option<bool>,
    pub default_model: Option<String>,
    pub custom_args: Option<Vec<String>>,
    pub env_overrides: Option<HashMap<String, String>>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum ApprovalResponse {
    AllowOnce,
    AlwaysAllow { pattern: String },
    Deny,
}

/// Describes what the session command parser recognized.
#[derive(Debug, Clone)]
pub enum SessionCommand {
    New,
    Reset,
    Compact,
    Think(ThinkingLevel),
    Verbose,
    Usage,
    Model(String),
    Cli(String),
    Status,
    Unknown(String),
}
