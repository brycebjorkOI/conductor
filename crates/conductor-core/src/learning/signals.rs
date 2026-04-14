//! Feedback signal detection from implicit user actions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A raw feedback signal detected from user behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackSignal {
    pub timestamp: DateTime<Utc>,
    pub signal_type: SignalType,
    pub session_id: String,
    pub backend_id: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalType {
    /// User re-asked the same question (negative).
    Requery,
    /// User corrected the AI (negative).
    Correction,
    /// User abandoned without follow-up (negative).
    Abandonment,
    /// User continued on a different topic (implicit positive).
    Acceptance,
    /// User explicitly praised the response (positive).
    Approval,
    /// User gave a style/format instruction (neutral/directive).
    Directive,
}

/// Detect the signal type from the user's next message after an AI response.
pub fn classify_follow_up(previous_user_msg: &str, new_user_msg: &str) -> SignalType {
    let prev_words: std::collections::HashSet<&str> =
        previous_user_msg.split_whitespace().collect();
    let new_words: std::collections::HashSet<&str> =
        new_user_msg.split_whitespace().collect();

    let lower = new_user_msg.to_lowercase();

    // Check for explicit approval.
    let approval_phrases = ["perfect", "thanks", "exactly", "great", "correct", "awesome"];
    if approval_phrases.iter().any(|p| lower.contains(p)) {
        return SignalType::Approval;
    }

    // Check for correction.
    let correction_prefixes = ["no,", "not that", "actually,", "i meant", "wrong", "instead"];
    if correction_prefixes.iter().any(|p| lower.starts_with(p)) {
        return SignalType::Correction;
    }

    // Check for directive.
    let directive_patterns = ["use ", "be more ", "prefer ", "assume i ", "format as "];
    if directive_patterns.iter().any(|p| lower.starts_with(p)) {
        return SignalType::Directive;
    }

    // Check for requery (Jaccard similarity > 0.5).
    if !prev_words.is_empty() && !new_words.is_empty() {
        let intersection = prev_words.intersection(&new_words).count();
        let union = prev_words.union(&new_words).count();
        let similarity = intersection as f64 / union as f64;
        if similarity > 0.5 {
            return SignalType::Requery;
        }
    }

    // Default: acceptance (user moved on).
    SignalType::Acceptance
}

/// Check if a signal is evaluative (positive or negative) vs directive.
pub fn is_evaluative(signal: &SignalType) -> bool {
    matches!(
        signal,
        SignalType::Requery
            | SignalType::Correction
            | SignalType::Abandonment
            | SignalType::Acceptance
            | SignalType::Approval
    )
}

/// The base weight for each signal type.
pub fn base_weight(signal: &SignalType) -> f64 {
    match signal {
        SignalType::Approval | SignalType::Acceptance => 1.0,
        SignalType::Requery | SignalType::Correction | SignalType::Abandonment => -1.0,
        SignalType::Directive => 1.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approval_detection() {
        assert_eq!(
            classify_follow_up("explain Rust", "perfect, thanks!"),
            SignalType::Approval
        );
    }

    #[test]
    fn test_correction_detection() {
        assert_eq!(
            classify_follow_up("explain Rust", "no, I meant Python"),
            SignalType::Correction
        );
    }

    #[test]
    fn test_requery_detection() {
        assert_eq!(
            classify_follow_up("explain async in Rust", "explain async Rust"),
            SignalType::Requery
        );
    }

    #[test]
    fn test_acceptance_detection() {
        assert_eq!(
            classify_follow_up("explain Rust", "now tell me about Go"),
            SignalType::Acceptance
        );
    }
}
