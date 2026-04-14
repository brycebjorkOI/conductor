use std::collections::HashMap;

use crate::backend::{StreamEvent, StreamParser};
use crate::state::UsageMetrics;

/// Parser for the Claude CLI `--output-format stream-json` NDJSON output.
///
/// Actual output format (observed from `claude` CLI):
/// - `{"type":"system","subtype":"init",...}` — init event (ignored)
/// - `{"type":"assistant","message":{"content":[{"type":"text","text":"..."}],"usage":{...}}}` — response
/// - `{"type":"rate_limit_event",...}` — rate limit info (ignored)
/// - `{"type":"result","subtype":"success","result":"...","usage":{...},"total_cost_usd":...}` — final
pub struct JsonStreamParser {
    /// Track whether we've received text from an "assistant" event,
    /// so we don't duplicate it when the "result" event arrives.
    got_assistant_text: bool,
    /// Track whether we've already emitted a Done event from a "result" line.
    got_done: bool,
}

impl JsonStreamParser {
    pub fn new() -> Self {
        Self {
            got_assistant_text: false,
            got_done: false,
        }
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
            // -- Claude CLI: assistant message with nested content --
            "assistant" => {
                let mut events = Vec::new();

                // Extract text from message.content[].text
                if let Some(message) = obj.get("message") {
                    if let Some(content_arr) = message.get("content").and_then(|v| v.as_array()) {
                        for block in content_arr {
                            let block_type = block.get("type").and_then(|v| v.as_str()).unwrap_or("");
                            match block_type {
                                "text" => {
                                    if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                                        if !text.is_empty() {
                                            self.got_assistant_text = true;
                                            events.push(StreamEvent::TextChunk(text.to_string()));
                                        }
                                    }
                                }
                                "tool_use" => {
                                    let name = block.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                                    let args: HashMap<String, serde_json::Value> = block
                                        .get("input")
                                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                                        .unwrap_or_default();
                                    events.push(StreamEvent::ToolStart { name, args });
                                }
                                "thinking" => {
                                    if let Some(text) = block.get("thinking").or_else(|| block.get("text")).and_then(|v| v.as_str()) {
                                        if !text.is_empty() {
                                            events.push(StreamEvent::ThinkingChunk(text.to_string()));
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    // Extract usage from message.usage
                    if let Some(usage) = message.get("usage") {
                        events.push(StreamEvent::UsageData(parse_usage(usage)));
                    }
                }

                events
            }

            // -- Claude CLI: final result --
            "result" => {
                let mut events = Vec::new();

                // Extract final text from "result" field — but only if we
                // didn't already get it from the "assistant" event.
                if !self.got_assistant_text {
                    if let Some(text) = obj.get("result").and_then(|v| v.as_str()) {
                        if !text.is_empty() {
                            events.push(StreamEvent::TextChunk(text.to_string()));
                        }
                    }
                }

                // Extract usage.
                if let Some(usage) = obj.get("usage") {
                    let mut metrics = parse_usage(usage);
                    // Also grab total_cost_usd from the top level.
                    if let Some(cost) = obj.get("total_cost_usd").and_then(|v| v.as_f64()) {
                        metrics.estimated_cost = Some(cost);
                    }
                    events.push(StreamEvent::UsageData(metrics));
                }

                // Check for error — but only if result text is actually empty
                // (a result with text + is_error=false from tool-use-interrupted
                // scenarios should not be treated as an error).
                let is_error = obj.get("is_error").and_then(|v| v.as_bool()).unwrap_or(false);
                if is_error {
                    // Use the "result" field if it's a string, otherwise try "error"
                    let msg = obj.get("result")
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .or_else(|| obj.get("error").and_then(|v| v.as_str()))
                        .unwrap_or("Backend reported an error")
                        .to_string();
                    events.push(StreamEvent::Error(msg));
                }

                self.got_done = true;
                events.push(StreamEvent::Done);
                events
            }

            // -- Standalone text events (generic NDJSON backends) --
            "text" | "content_block_delta" => {
                let text = obj
                    .get("text")
                    .or_else(|| obj.get("delta").and_then(|d| d.get("text")))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if text.is_empty() {
                    Vec::new()
                } else {
                    vec![StreamEvent::TextChunk(text)]
                }
            }

            // -- Tool events --
            "tool_use" => {
                let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                let args: HashMap<String, serde_json::Value> = obj
                    .get("input")
                    .or_else(|| obj.get("arguments"))
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();
                vec![StreamEvent::ToolStart { name, args }]
            }
            "tool_result" => {
                let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                let result = obj
                    .get("output")
                    .or_else(|| obj.get("content"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let success = obj.get("is_error").and_then(|v| v.as_bool()).map(|e| !e).unwrap_or(true);
                vec![StreamEvent::ToolResult { name, result, success }]
            }

            // -- Thinking --
            "thinking" => {
                let text = obj.get("text").or_else(|| obj.get("thinking")).and_then(|v| v.as_str()).unwrap_or("").to_string();
                if text.is_empty() { Vec::new() } else { vec![StreamEvent::ThinkingChunk(text)] }
            }

            // -- Standalone usage --
            "usage" => {
                if let Some(usage) = obj.get("usage") {
                    vec![StreamEvent::UsageData(parse_usage(usage))]
                } else {
                    Vec::new()
                }
            }

            // -- Error --
            "error" => {
                let msg = obj.get("error").or_else(|| obj.get("message")).and_then(|v| v.as_str()).unwrap_or("unknown error").to_string();
                vec![StreamEvent::Error(msg)]
            }

            // -- Events we intentionally skip --
            "system" | "rate_limit_event" => Vec::new(),

            // -- Unknown: try to extract text, otherwise ignore --
            _ => {
                if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
                    vec![StreamEvent::TextChunk(text.to_string())]
                } else {
                    Vec::new()
                }
            }
        }
    }

    fn finish(&mut self, exit_code: i32, stderr: &str) -> Vec<StreamEvent> {
        // If we already got a "result" event with Done, don't duplicate.
        if self.got_done {
            return Vec::new();
        }
        let mut events = Vec::new();
        if exit_code != 0 && !stderr.trim().is_empty() {
            events.push(StreamEvent::Error(stderr.trim().to_string()));
        }
        events.push(StreamEvent::Done);
        events
    }
}

fn parse_usage(usage: &serde_json::Value) -> UsageMetrics {
    UsageMetrics {
        input_tokens: usage.get("input_tokens").and_then(|v| v.as_u64()),
        output_tokens: usage.get("output_tokens").and_then(|v| v.as_u64()),
        total_tokens: None,
        estimated_cost: None,
        cache_read_tokens: usage.get("cache_read_input_tokens").and_then(|v| v.as_u64()),
        cache_write_tokens: usage.get("cache_creation_input_tokens").and_then(|v| v.as_u64()),
    }
}
