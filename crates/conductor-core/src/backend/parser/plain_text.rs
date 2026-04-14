use crate::backend::{StreamEvent, StreamParser};

/// Parser for backends that output plain text (Ollama, generic, etc.).
pub struct PlainTextParser;

impl PlainTextParser {
    pub fn new() -> Self {
        Self
    }
}

impl StreamParser for PlainTextParser {
    fn parse_line(&mut self, line: &str) -> Vec<StreamEvent> {
        if line.is_empty() {
            return Vec::new();
        }
        vec![StreamEvent::TextChunk(format!("{line}\n"))]
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
