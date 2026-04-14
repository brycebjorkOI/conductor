use std::collections::HashMap;
use std::path::PathBuf;

use crate::backend::parser::{json_stream::JsonStreamParser, plain_text::PlainTextParser};
use crate::backend::*;
use crate::state::*;

// ---------------------------------------------------------------------------
// Registry of all known backends
// ---------------------------------------------------------------------------

pub fn all_known_backends() -> Vec<Box<dyn BackendDefinition>> {
    vec![
        Box::new(AnthropicBackend),
        Box::new(OllamaBackend),
        Box::new(OpenAiBackend),
        Box::new(GeminiBackend),
        Box::new(CodexBackend),
        Box::new(CopilotBackend),
        Box::new(AwsQBackend),
    ]
}

// ---------------------------------------------------------------------------
// Anthropic CLI  (`claude`)
// ---------------------------------------------------------------------------

pub struct AnthropicBackend;

impl BackendDefinition for AnthropicBackend {
    fn backend_id(&self) -> &str {
        "anthropic"
    }
    fn display_name(&self) -> &str {
        "Anthropic Claude CLI"
    }
    fn binary_name(&self) -> &str {
        "claude"
    }
    fn version_command(&self) -> Vec<String> {
        vec!["claude".into(), "--version".into()]
    }
    fn auth_check_command(&self) -> Option<Vec<String>> {
        Some(vec!["claude".into(), "auth".into(), "status".into()])
    }
    fn model_list_command(&self) -> Option<Vec<String>> {
        None
    }
    fn parse_version(&self, stdout: &str, _exit_code: i32) -> Option<String> {
        // "claude v1.2.3" -> "1.2.3"
        stdout
            .split_whitespace()
            .find(|w| w.starts_with('v') || w.chars().next().map_or(false, |c| c.is_ascii_digit()))
            .map(|v| v.trim_start_matches('v').to_string())
    }
    fn parse_auth(&self, stdout: &str, _stderr: &str, exit_code: i32) -> AuthState {
        if exit_code == 0 && stdout.to_lowercase().contains("authenticated") {
            AuthState::Authenticated
        } else if exit_code == 0 {
            AuthState::Authenticated
        } else {
            AuthState::NotAuthenticated
        }
    }
    fn parse_models(&self, _stdout: &str) -> Vec<ModelEntry> {
        Vec::new()
    }
    fn capabilities(&self) -> CapabilitySet {
        CapabilitySet {
            streaming: true,
            interactive_session: true,
            tool_reporting: true,
            vision_input: true,
            thinking_levels: true,
            image_generation: false,
            plan_mode: false,
        }
    }
    fn static_models(&self) -> Vec<ModelEntry> {
        vec![
            ModelEntry {
                model_id: "sonnet".into(),
                display_name: "Claude Sonnet".into(),
                context_window: Some(200_000),
                pricing: Some(PricingInfo {
                    input_per_million: 3.0,
                    output_per_million: 15.0,
                }),
            },
            ModelEntry {
                model_id: "opus".into(),
                display_name: "Claude Opus".into(),
                context_window: Some(200_000),
                pricing: Some(PricingInfo {
                    input_per_million: 15.0,
                    output_per_million: 75.0,
                }),
            },
            ModelEntry {
                model_id: "haiku".into(),
                display_name: "Claude Haiku".into(),
                context_window: Some(200_000),
                pricing: Some(PricingInfo {
                    input_per_million: 0.25,
                    output_per_million: 1.25,
                }),
            },
        ]
    }
    fn build_chat_command(&self, binary_path: &PathBuf, params: &ChatParams) -> CliCommand {
        let mut args = vec![
            "--output-format".into(),
            "stream-json".into(),
            "--verbose".into(),
        ];
        if let Some(ref model) = params.model {
            args.push("--model".into());
            args.push(model.clone());
        }
        if let Some(ref level) = params.thinking_level {
            let level_str = match level {
                ThinkingLevel::Off => "none",
                ThinkingLevel::Minimal => "low",
                ThinkingLevel::Low => "low",
                ThinkingLevel::Medium => "medium",
                ThinkingLevel::High => "high",
                ThinkingLevel::ExtraHigh => "high",
            };
            args.push("--thinking".into());
            args.push(level_str.into());
        }
        for attachment in &params.attachments {
            args.push("--file".into());
            args.push(attachment.to_string_lossy().into_owned());
        }
        args.extend(params.extra_args.clone());
        args.push("-p".into());
        args.push(params.message.clone());

        CliCommand {
            binary: binary_path.clone(),
            args,
            env: HashMap::new(),
            working_dir: None,
        }
    }
    fn create_parser(&self) -> Box<dyn StreamParser> {
        Box::new(JsonStreamParser::new())
    }
}

// ---------------------------------------------------------------------------
// Ollama  (`ollama`)
// ---------------------------------------------------------------------------

pub struct OllamaBackend;

impl BackendDefinition for OllamaBackend {
    fn backend_id(&self) -> &str {
        "ollama"
    }
    fn display_name(&self) -> &str {
        "Ollama (Local)"
    }
    fn binary_name(&self) -> &str {
        "ollama"
    }
    fn version_command(&self) -> Vec<String> {
        vec!["ollama".into(), "--version".into()]
    }
    fn auth_check_command(&self) -> Option<Vec<String>> {
        // Ollama doesn't need auth — checking that the service is running.
        Some(vec!["ollama".into(), "list".into()])
    }
    fn model_list_command(&self) -> Option<Vec<String>> {
        Some(vec!["ollama".into(), "list".into()])
    }
    fn parse_version(&self, stdout: &str, _exit_code: i32) -> Option<String> {
        // "ollama version 0.5.1" -> "0.5.1"
        stdout.split_whitespace().last().map(|s| s.to_string())
    }
    fn parse_auth(&self, _stdout: &str, _stderr: &str, exit_code: i32) -> AuthState {
        if exit_code == 0 {
            AuthState::Authenticated
        } else {
            AuthState::NotAuthenticated
        }
    }
    fn parse_models(&self, stdout: &str) -> Vec<ModelEntry> {
        // Parse the table output from `ollama list`.
        stdout
            .lines()
            .skip(1) // skip header row
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                let name = parts.first()?;
                Some(ModelEntry {
                    model_id: name.to_string(),
                    display_name: name.to_string(),
                    context_window: None,
                    pricing: None,
                })
            })
            .collect()
    }
    fn capabilities(&self) -> CapabilitySet {
        CapabilitySet {
            streaming: true,
            ..Default::default()
        }
    }
    fn build_chat_command(&self, binary_path: &PathBuf, params: &ChatParams) -> CliCommand {
        let model = params
            .model
            .clone()
            .unwrap_or_else(|| "llama3".into());
        CliCommand {
            binary: binary_path.clone(),
            args: vec!["run".into(), model, params.message.clone()],
            env: HashMap::new(),
            working_dir: None,
        }
    }
    fn create_parser(&self) -> Box<dyn StreamParser> {
        Box::new(PlainTextParser::new())
    }
}

// ---------------------------------------------------------------------------
// OpenAI Chat CLI
// ---------------------------------------------------------------------------

pub struct OpenAiBackend;

impl BackendDefinition for OpenAiBackend {
    fn backend_id(&self) -> &str { "openai" }
    fn display_name(&self) -> &str { "OpenAI Chat CLI" }
    fn binary_name(&self) -> &str { "chatgpt" }
    fn version_command(&self) -> Vec<String> { vec!["chatgpt".into(), "--version".into()] }
    fn auth_check_command(&self) -> Option<Vec<String>> { None }
    fn model_list_command(&self) -> Option<Vec<String>> { None }
    fn parse_version(&self, stdout: &str, _: i32) -> Option<String> {
        stdout.split_whitespace().last().map(|s| s.to_string())
    }
    fn parse_auth(&self, _: &str, _: &str, exit_code: i32) -> AuthState {
        if exit_code == 0 { AuthState::Authenticated } else { AuthState::Unknown }
    }
    fn parse_models(&self, _: &str) -> Vec<ModelEntry> { Vec::new() }
    fn capabilities(&self) -> CapabilitySet {
        CapabilitySet { streaming: true, vision_input: true, image_generation: true, ..Default::default() }
    }
    fn static_models(&self) -> Vec<ModelEntry> {
        vec![
            ModelEntry { model_id: "gpt-4o".into(), display_name: "GPT-4o".into(), context_window: Some(128_000), pricing: None },
            ModelEntry { model_id: "gpt-4o-mini".into(), display_name: "GPT-4o Mini".into(), context_window: Some(128_000), pricing: None },
            ModelEntry { model_id: "o3-mini".into(), display_name: "o3-mini".into(), context_window: Some(200_000), pricing: None },
        ]
    }
    fn build_chat_command(&self, binary_path: &PathBuf, params: &ChatParams) -> CliCommand {
        let mut args = Vec::new();
        if let Some(ref model) = params.model {
            args.push("-m".into());
            args.push(model.clone());
        }
        args.push(params.message.clone());
        CliCommand { binary: binary_path.clone(), args, env: HashMap::new(), working_dir: None }
    }
    fn create_parser(&self) -> Box<dyn StreamParser> { Box::new(PlainTextParser::new()) }
}

// ---------------------------------------------------------------------------
// Google Gemini CLI
// ---------------------------------------------------------------------------

pub struct GeminiBackend;

impl BackendDefinition for GeminiBackend {
    fn backend_id(&self) -> &str { "gemini" }
    fn display_name(&self) -> &str { "Google Gemini CLI" }
    fn binary_name(&self) -> &str { "gemini" }
    fn version_command(&self) -> Vec<String> { vec!["gemini".into(), "--version".into()] }
    fn auth_check_command(&self) -> Option<Vec<String>> { None }
    fn model_list_command(&self) -> Option<Vec<String>> { None }
    fn parse_version(&self, stdout: &str, _: i32) -> Option<String> {
        stdout.split_whitespace().last().map(|s| s.to_string())
    }
    fn parse_auth(&self, _: &str, _: &str, exit_code: i32) -> AuthState {
        if exit_code == 0 { AuthState::Authenticated } else { AuthState::Unknown }
    }
    fn parse_models(&self, _: &str) -> Vec<ModelEntry> { Vec::new() }
    fn capabilities(&self) -> CapabilitySet {
        CapabilitySet { streaming: true, vision_input: true, ..Default::default() }
    }
    fn static_models(&self) -> Vec<ModelEntry> {
        vec![
            ModelEntry { model_id: "gemini-2.5-pro".into(), display_name: "Gemini 2.5 Pro".into(), context_window: Some(1_000_000), pricing: None },
            ModelEntry { model_id: "gemini-2.5-flash".into(), display_name: "Gemini 2.5 Flash".into(), context_window: Some(1_000_000), pricing: None },
        ]
    }
    fn build_chat_command(&self, binary_path: &PathBuf, params: &ChatParams) -> CliCommand {
        let mut args = Vec::new();
        if let Some(ref model) = params.model {
            args.push("-m".into());
            args.push(model.clone());
        }
        args.push(params.message.clone());
        CliCommand { binary: binary_path.clone(), args, env: HashMap::new(), working_dir: None }
    }
    fn create_parser(&self) -> Box<dyn StreamParser> { Box::new(PlainTextParser::new()) }
}

// ---------------------------------------------------------------------------
// OpenAI Codex CLI
// ---------------------------------------------------------------------------

pub struct CodexBackend;

impl BackendDefinition for CodexBackend {
    fn backend_id(&self) -> &str { "codex" }
    fn display_name(&self) -> &str { "OpenAI Codex CLI" }
    fn binary_name(&self) -> &str { "codex" }
    fn version_command(&self) -> Vec<String> { vec!["codex".into(), "--version".into()] }
    fn auth_check_command(&self) -> Option<Vec<String>> { None }
    fn model_list_command(&self) -> Option<Vec<String>> { None }
    fn parse_version(&self, stdout: &str, _: i32) -> Option<String> {
        stdout.split_whitespace().last().map(|s| s.to_string())
    }
    fn parse_auth(&self, _: &str, _: &str, exit_code: i32) -> AuthState {
        if exit_code == 0 { AuthState::Authenticated } else { AuthState::Unknown }
    }
    fn parse_models(&self, _: &str) -> Vec<ModelEntry> { Vec::new() }
    fn capabilities(&self) -> CapabilitySet {
        CapabilitySet { streaming: true, tool_reporting: true, plan_mode: true, ..Default::default() }
    }
    fn static_models(&self) -> Vec<ModelEntry> {
        vec![
            ModelEntry { model_id: "codex-mini".into(), display_name: "Codex Mini".into(), context_window: Some(200_000), pricing: None },
        ]
    }
    fn build_chat_command(&self, binary_path: &PathBuf, params: &ChatParams) -> CliCommand {
        let mut args = vec!["--approval-mode".into(), "suggest".into()];
        if let Some(ref model) = params.model {
            args.push("--model".into());
            args.push(model.clone());
        }
        args.push(params.message.clone());
        CliCommand { binary: binary_path.clone(), args, env: HashMap::new(), working_dir: None }
    }
    fn create_parser(&self) -> Box<dyn StreamParser> { Box::new(PlainTextParser::new()) }
}

// ---------------------------------------------------------------------------
// GitHub Copilot CLI
// ---------------------------------------------------------------------------

pub struct CopilotBackend;

impl BackendDefinition for CopilotBackend {
    fn backend_id(&self) -> &str { "copilot" }
    fn display_name(&self) -> &str { "GitHub Copilot CLI" }
    fn binary_name(&self) -> &str { "gh" }
    fn version_command(&self) -> Vec<String> { vec!["gh".into(), "copilot".into(), "--version".into()] }
    fn auth_check_command(&self) -> Option<Vec<String>> { None }
    fn model_list_command(&self) -> Option<Vec<String>> { None }
    fn parse_version(&self, stdout: &str, _: i32) -> Option<String> {
        stdout.split_whitespace().last().map(|s| s.to_string())
    }
    fn parse_auth(&self, _: &str, _: &str, exit_code: i32) -> AuthState {
        if exit_code == 0 { AuthState::Authenticated } else { AuthState::Unknown }
    }
    fn parse_models(&self, _: &str) -> Vec<ModelEntry> { Vec::new() }
    fn capabilities(&self) -> CapabilitySet { CapabilitySet::default() }
    fn build_chat_command(&self, binary_path: &PathBuf, params: &ChatParams) -> CliCommand {
        CliCommand {
            binary: binary_path.clone(),
            args: vec!["copilot".into(), "suggest".into(), params.message.clone()],
            env: HashMap::new(),
            working_dir: None,
        }
    }
    fn create_parser(&self) -> Box<dyn StreamParser> { Box::new(PlainTextParser::new()) }
}

// ---------------------------------------------------------------------------
// AWS Q Developer CLI
// ---------------------------------------------------------------------------

pub struct AwsQBackend;

impl BackendDefinition for AwsQBackend {
    fn backend_id(&self) -> &str { "aws-q" }
    fn display_name(&self) -> &str { "AWS Q Developer" }
    fn binary_name(&self) -> &str { "q" }
    fn version_command(&self) -> Vec<String> { vec!["q".into(), "--version".into()] }
    fn auth_check_command(&self) -> Option<Vec<String>> { None }
    fn model_list_command(&self) -> Option<Vec<String>> { None }
    fn parse_version(&self, stdout: &str, _: i32) -> Option<String> {
        stdout.split_whitespace().last().map(|s| s.to_string())
    }
    fn parse_auth(&self, _: &str, _: &str, exit_code: i32) -> AuthState {
        if exit_code == 0 { AuthState::Authenticated } else { AuthState::Unknown }
    }
    fn parse_models(&self, _: &str) -> Vec<ModelEntry> { Vec::new() }
    fn capabilities(&self) -> CapabilitySet { CapabilitySet { streaming: true, ..Default::default() } }
    fn build_chat_command(&self, binary_path: &PathBuf, params: &ChatParams) -> CliCommand {
        CliCommand {
            binary: binary_path.clone(),
            args: vec!["chat".into(), params.message.clone()],
            env: HashMap::new(),
            working_dir: None,
        }
    }
    fn create_parser(&self) -> Box<dyn StreamParser> { Box::new(PlainTextParser::new()) }
}

// ---------------------------------------------------------------------------
// Generic / User-Defined Backend
// ---------------------------------------------------------------------------

/// A backend configured entirely from user-provided settings.
/// Supports template placeholders: `{message}`, `{model}`, `{extra_args}`.
pub struct GenericBackend {
    pub id: String,
    pub name: String,
    pub binary: String,
    pub version_cmd: Option<Vec<String>>,
    pub auth_cmd: Option<Vec<String>>,
    pub chat_template: Vec<String>,
    pub default_model: Option<String>,
    pub models: Vec<ModelEntry>,
    pub caps: CapabilitySet,
}

impl GenericBackend {
    pub fn from_config(id: &str, name: &str, binary: &str, chat_template: &[String]) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            binary: binary.to_string(),
            version_cmd: None,
            auth_cmd: None,
            chat_template: chat_template.to_vec(),
            default_model: None,
            models: Vec::new(),
            caps: CapabilitySet::default(),
        }
    }
}

impl BackendDefinition for GenericBackend {
    fn backend_id(&self) -> &str { &self.id }
    fn display_name(&self) -> &str { &self.name }
    fn binary_name(&self) -> &str { &self.binary }
    fn version_command(&self) -> Vec<String> {
        self.version_cmd.clone().unwrap_or_else(|| vec![self.binary.clone(), "--version".into()])
    }
    fn auth_check_command(&self) -> Option<Vec<String>> { self.auth_cmd.clone() }
    fn model_list_command(&self) -> Option<Vec<String>> { None }
    fn parse_version(&self, stdout: &str, _: i32) -> Option<String> {
        stdout.split_whitespace().last().map(|s| s.to_string())
    }
    fn parse_auth(&self, _: &str, _: &str, exit_code: i32) -> AuthState {
        if exit_code == 0 { AuthState::Authenticated } else { AuthState::Unknown }
    }
    fn parse_models(&self, _: &str) -> Vec<ModelEntry> { Vec::new() }
    fn capabilities(&self) -> CapabilitySet { self.caps.clone() }
    fn static_models(&self) -> Vec<ModelEntry> { self.models.clone() }
    fn build_chat_command(&self, binary_path: &PathBuf, params: &ChatParams) -> CliCommand {
        let args: Vec<String> = self.chat_template.iter().map(|t| {
            t.replace("{message}", &params.message)
                .replace("{model}", params.model.as_deref().unwrap_or(""))
                .replace("{extra_args}", &params.extra_args.join(" "))
        }).collect();
        CliCommand { binary: binary_path.clone(), args, env: HashMap::new(), working_dir: None }
    }
    fn create_parser(&self) -> Box<dyn StreamParser> { Box::new(PlainTextParser::new()) }
}
