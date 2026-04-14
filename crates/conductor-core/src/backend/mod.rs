pub mod definitions;
pub mod discovery;
pub mod orchestrator;
pub mod parser;

use std::collections::HashMap;
use std::path::PathBuf;

use crate::state::{AuthState, CapabilitySet, ModelEntry, ThinkingLevel};

/// Normalized event produced by parsing backend CLI output.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    TextChunk(String),
    ToolStart {
        name: String,
        args: HashMap<String, serde_json::Value>,
    },
    ToolResult {
        name: String,
        result: String,
        success: bool,
    },
    ThinkingChunk(String),
    UsageData(crate::state::UsageMetrics),
    Error(String),
    Done,
}

/// Parameters for building a chat command.
#[derive(Debug, Clone)]
pub struct ChatParams {
    pub message: String,
    pub model: Option<String>,
    pub attachments: Vec<PathBuf>,
    pub thinking_level: Option<ThinkingLevel>,
    pub extra_args: Vec<String>,
    pub system_prompt: Option<String>,
}

/// The resolved CLI command to execute.
#[derive(Debug, Clone)]
pub struct CliCommand {
    pub binary: PathBuf,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: Option<PathBuf>,
}

/// Describes how to interact with an AI CLI backend.
pub trait BackendDefinition: Send + Sync {
    fn backend_id(&self) -> &str;
    fn display_name(&self) -> &str;
    fn binary_name(&self) -> &str;

    fn version_command(&self) -> Vec<String>;
    fn auth_check_command(&self) -> Option<Vec<String>>;
    fn model_list_command(&self) -> Option<Vec<String>>;

    fn parse_version(&self, stdout: &str, exit_code: i32) -> Option<String>;
    fn parse_auth(&self, stdout: &str, stderr: &str, exit_code: i32) -> AuthState;
    fn parse_models(&self, stdout: &str) -> Vec<ModelEntry>;

    fn capabilities(&self) -> CapabilitySet;
    fn static_models(&self) -> Vec<ModelEntry> {
        Vec::new()
    }

    fn build_chat_command(
        &self,
        binary_path: &PathBuf,
        params: &ChatParams,
    ) -> CliCommand;

    fn create_parser(&self) -> Box<dyn StreamParser>;
}

/// Stateful parser for a single response stream.
pub trait StreamParser: Send {
    fn parse_line(&mut self, line: &str) -> Vec<StreamEvent>;
    fn finish(&mut self, exit_code: i32, stderr: &str) -> Vec<StreamEvent>;
}
