//! Preference aggregation with time decay and confidence scoring.

use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::state::PreferenceWeight;
use super::signals::{FeedbackSignal, SignalType, base_weight};

/// Default decay rate (half-life ~7 days).
pub const DEFAULT_DECAY_RATE: f64 = 0.099;

/// Smoothing constant for confidence scoring.
const PRIOR: f64 = 3.0;

/// Default minimum confidence for a preference to be injected.
pub const DEFAULT_CONFIDENCE_THRESHOLD: f64 = 0.6;

/// Aggregate raw signals into preference weights.
pub fn aggregate(
    signals: &[FeedbackSignal],
    decay_rate: f64,
    now: DateTime<Utc>,
) -> HashMap<String, PreferenceWeight> {
    let mut categories: HashMap<String, Vec<(f64, String, DateTime<Utc>)>> = HashMap::new();

    for signal in signals {
        let age_days = (now - signal.timestamp).num_seconds() as f64 / 86400.0;
        let weight = base_weight(&signal.signal_type) * (-decay_rate * age_days).exp();

        if let Some(ref detail) = signal.detail {
            let category = classify_category(&signal.signal_type, detail);
            categories
                .entry(category)
                .or_default()
                .push((weight, detail.clone(), signal.timestamp));
        }
    }

    let mut result = HashMap::new();
    for (category, entries) in &categories {
        let total_weight: f64 = entries.iter().map(|(w, _, _)| w.abs()).sum();
        let net_weight: f64 = entries.iter().map(|(w, _, _)| *w).sum();
        let confidence = total_weight / (total_weight + PRIOR);
        let last = entries.iter().map(|(_, _, t)| *t).max().unwrap_or(now);

        // Determine the dominant preference value.
        let value = if net_weight >= 0.0 {
            entries
                .iter()
                .filter(|(w, _, _)| *w > 0.0)
                .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(_, v, _)| v.clone())
                .unwrap_or_default()
        } else {
            entries
                .iter()
                .filter(|(w, _, _)| *w < 0.0)
                .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(_, v, _)| format!("avoids: {v}"))
                .unwrap_or_default()
        };

        if !value.is_empty() {
            result.insert(
                category.clone(),
                PreferenceWeight {
                    category: category.clone(),
                    value,
                    confidence,
                    last_reinforced: last,
                    signal_count: entries.len() as u32,
                },
            );
        }
    }

    result
}

/// Classify a signal's detail text into a preference category.
fn classify_category(signal_type: &SignalType, detail: &str) -> String {
    let lower = detail.to_lowercase();

    if lower.contains("bullet")
        || lower.contains("list")
        || lower.contains("table")
        || lower.contains("code block")
    {
        return "format".to_string();
    }
    if lower.contains("concise")
        || lower.contains("brief")
        || lower.contains("detailed")
        || lower.contains("verbose")
    {
        return "verbosity".to_string();
    }
    if lower.contains("formal")
        || lower.contains("casual")
        || lower.contains("technical")
    {
        return "style".to_string();
    }

    match signal_type {
        SignalType::Directive => "format".to_string(),
        _ => "general".to_string(),
    }
}

/// Serialize preferences above the confidence threshold into a natural-language
/// fragment for system prompt injection.
pub fn serialize_for_prompt(
    preferences: &HashMap<String, PreferenceWeight>,
    threshold: f64,
) -> String {
    let mut lines = Vec::new();

    for pref in preferences.values() {
        if pref.confidence < threshold {
            continue;
        }
        let sentence = match pref.category.as_str() {
            "format" => format!("The user prefers responses formatted with {}.", pref.value),
            "verbosity" => format!("The user prefers {} responses.", pref.value),
            "style" => format!("The user prefers a {} communication style.", pref.value),
            "domain" => format!(
                "The user has domain expertise in {}; assume familiarity.",
                pref.value
            ),
            _ => format!("User preference: {}.", pref.value),
        };
        lines.push(sentence);
    }

    lines.join(" ")
}

/// Update per-backend satisfaction score with EWMA.
pub fn update_satisfaction(
    current: f64,
    signal_type: &SignalType,
    alpha: f64,
) -> f64 {
    let value = match signal_type {
        SignalType::Approval | SignalType::Acceptance => 1.0,
        SignalType::Requery | SignalType::Correction | SignalType::Abandonment => 0.0,
        SignalType::Directive => current, // neutral
    };
    alpha * value + (1.0 - alpha) * current
}
