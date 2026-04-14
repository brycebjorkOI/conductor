//! Channel manager: persistent connections to messaging platforms.
//!
//! Each platform adapter implements the `ChannelAdapter` trait. The manager
//! spawns one tokio task per active channel.

use std::time::Duration;


/// A normalized inbound message from any platform.
#[derive(Debug, Clone)]
pub struct NormalizedMessage {
    pub sender_id: String,
    pub sender_name: String,
    pub text: String,
    pub attachments: Vec<String>,
    pub reply_context: Option<String>,
    pub platform_id: String,
}

/// Trait that each platform adapter implements.
#[allow(async_fn_in_trait)]
pub trait ChannelAdapter: Send + Sync + 'static {
    fn platform_id(&self) -> &str;
    fn display_name(&self) -> &str;

    /// Connect to the platform and begin receiving messages.
    async fn connect(&mut self) -> Result<(), String>;

    /// Poll for the next inbound message (blocks until one arrives or error).
    async fn recv(&mut self) -> Result<NormalizedMessage, String>;

    /// Send a response back to the platform.
    async fn send(&mut self, chat_id: &str, text: &str) -> Result<(), String>;

    /// Disconnect gracefully.
    async fn disconnect(&mut self);
}

/// Exponential backoff configuration for reconnection.
pub struct Backoff {
    current: Duration,
    max: Duration,
    factor: f64,
    consecutive_failures: u32,
    max_failures: u32,
}

impl Backoff {
    pub fn new() -> Self {
        Self {
            current: Duration::from_secs(1),
            max: Duration::from_secs(60),
            factor: 2.0,
            consecutive_failures: 0,
            max_failures: 10,
        }
    }

    /// Call on each failure. Returns `None` if max failures exceeded.
    pub fn next_delay(&mut self) -> Option<Duration> {
        self.consecutive_failures += 1;
        if self.consecutive_failures > self.max_failures {
            return None; // give up
        }
        let delay = self.current;
        self.current = Duration::from_secs_f64(
            (self.current.as_secs_f64() * self.factor).min(self.max.as_secs_f64()),
        );
        Some(delay)
    }

    /// Reset on successful connection.
    pub fn reset(&mut self) {
        self.current = Duration::from_secs(1);
        self.consecutive_failures = 0;
    }
}

impl Default for Backoff {
    fn default() -> Self {
        Self::new()
    }
}

/// Split a long message at paragraph boundaries for platforms with char limits.
pub fn split_message(text: &str, max_chars: usize) -> Vec<String> {
    if text.len() <= max_chars {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut remaining = text;

    while remaining.len() > max_chars {
        // Find the last paragraph break before the limit.
        let search = &remaining[..max_chars];
        let split_at = search
            .rfind("\n\n")
            .or_else(|| search.rfind('\n'))
            .or_else(|| search.rfind(". "))
            .unwrap_or(max_chars);

        chunks.push(remaining[..split_at].to_string());
        remaining = remaining[split_at..].trim_start();
    }

    if !remaining.is_empty() {
        chunks.push(remaining.to_string());
    }

    chunks
}

/// Strip bot mentions from message text.
pub fn strip_mentions(text: &str, bot_names: &[&str]) -> String {
    let mut result = text.to_string();
    for name in bot_names {
        result = result.replace(&format!("@{name}"), "");
    }
    // Slack-style <@ID> mentions (always strip regardless of bot_names).
    while let Some(start) = result.find("<@") {
        if let Some(end) = result[start..].find('>') {
            result.replace_range(start..start + end + 1, "");
        } else {
            break;
        }
    }
    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_message() {
        let text = "Hello world. This is a test.";
        assert_eq!(split_message(text, 100), vec![text]);

        let long = "A".repeat(5000);
        let chunks = split_message(&long, 2000);
        assert!(chunks.len() >= 3);
        for chunk in &chunks {
            assert!(chunk.len() <= 2000);
        }
    }

    #[test]
    fn test_strip_mentions() {
        assert_eq!(
            strip_mentions("@bot hello there", &["bot"]),
            "hello there"
        );
        assert_eq!(
            strip_mentions("<@U123> help me", &[]),
            "help me"
        );
    }

    #[test]
    fn test_backoff() {
        let mut b = Backoff::new();
        let d1 = b.next_delay().unwrap();
        assert_eq!(d1, Duration::from_secs(1));
        let d2 = b.next_delay().unwrap();
        assert_eq!(d2, Duration::from_secs(2));
        b.reset();
        let d3 = b.next_delay().unwrap();
        assert_eq!(d3, Duration::from_secs(1));
    }
}
