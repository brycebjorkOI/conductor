use std::collections::HashMap;

use crate::backend::{StreamEvent, StreamParser};
use crate::state::UsageMetrics;

/// Parser for NDJSON output (Anthropic CLI, structured-output backends).
///
/// Each line is a JSON object with a `type` field indicating the event kind.
pub struct JsonStreamParser;

impl JsonStreamParser {
    pub fn new() -> Self {
        Self
    }
}

impl StreamParser for JsonStreamParser {
    fn parse_line(&mut self, line: &str) -> Vec<StreamEvent> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Vec::new();
        }

        let obj: serde_json::Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(_) => {
                // Graceful degradation: treat unparseable lines as text.
                return vec![StreamEvent::TextChunk(line.to_string())];
            }
        };

        let event_type = obj
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match event_type {
            "text" | "content_block_delta" | "assistant" => {
                let text = obj
                    .get("text")
                    .or_else(|| obj.get("content"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if text.is_empty() {
                    Vec::new()
                } else {
                    vec![StreamEvent::TextChunk(text)]
                }
            }
            "tool_use" => {
                let name = obj
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let args: HashMap<String, serde_json::Value> = obj
                    .get("input")
                    .or_else(|| obj.get("arguments"))
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();
                vec![StreamEvent::ToolStart { name, args }]
            }
            "tool_result" => {
                let name = obj
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let result = obj
                    .get("output")
                    .or_else(|| obj.get("content"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let success = obj
                    .get("is_error")
                    .and_then(|v| v.as_bool())
                    .map(|e| !e)
                    .unwrap_or(true);
                vec![StreamEvent::ToolResult {
                    name,
                    result,
                    success,
                }]
            }
            "thinking" => {
                let text = obj
                    .get("text")
                    .or_else(|| obj.get("thinking"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if text.is_empty() {
                    Vec::new()
                } else {
                    vec![StreamEvent::ThinkingChunk(text)]
                }
            }
            "usage" | "result" => {
                let mut events = Vec::new();
                if let Some(usage) = obj.get("usage") {
                    let metrics = UsageMetrics {
                        input_tokens: usage
                            .get("input_tokens")
                            .and_then(|v| v.as_u64()),
                        output_tokens: usage
                            .get("output_tokens")
                            .and_then(|v| v.as_u64()),
                        total_tokens: None,
                        estimated_cost: None,
                        cache_read_tokens: usage
                            .get("cache_read_input_tokens")
                            .and_then(|v| v.as_u64()),
                        cache_write_tokens: usage
                            .get("cache_creation_input_tokens")
                            .and_then(|v| v.as_u64()),
                    };
                    events.push(StreamEvent::UsageData(metrics));
                }
                // "result" type may also contain final text
                if event_type == "result" {
                    if let Some(text) = obj.get("result").and_then(|v| v.as_str()) {
                        if !text.is_empty() {
                            events.push(StreamEvent::TextChunk(text.to_string()));
                        }
                    }
                    events.push(StreamEvent::Done);
                }
                events
            }
            "error" => {
                let msg = obj
                    .get("error")
                    .or_else(|| obj.get("message"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown error")
                    .to_string();
                vec![StreamEvent::Error(msg)]
            }
            _ => {
                // Unknown event type — treat as text if it has content.
                if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
                    vec![StreamEvent::TextChunk(text.to_string())]
                } else {
                    Vec::new()
                }
            }
        }
    }

    fn finish(&mut self, exit_code: i32, stderr: &str) -> Vec<StreamEvent> {
        let mut events = Vec::new();
        if exit_code != 0 && !stderr.is_empty() {
            events.push(StreamEvent::Error(stderr.to_string()));
        }
        events.push(StreamEvent::Done);
        events
    }
}
