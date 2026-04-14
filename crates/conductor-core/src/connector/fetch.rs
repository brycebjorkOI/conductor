//! `@fetch` directive parsing, anti-loop protection, and result truncation.

/// A parsed @fetch directive.
#[derive(Debug, Clone)]
pub struct FetchDirective {
    pub connector: String,
    pub action: String,
    pub params: Vec<(String, String)>,
}

/// Parse `@fetch(connector.action, key=value, ...)` directives from text.
pub fn parse_fetch_directives(text: &str) -> Vec<FetchDirective> {
    let mut directives = Vec::new();

    for (start, _) in text.match_indices("@fetch(") {
        let rest = &text[start + 7..];
        if let Some(end) = rest.find(')') {
            let inner = rest[..end].trim();
            if let Some(directive) = parse_single(inner) {
                directives.push(directive);
            }
        }
    }

    directives
}

fn parse_single(inner: &str) -> Option<FetchDirective> {
    let mut parts = inner.splitn(2, ',');
    let action_part = parts.next()?.trim();
    let params_str = parts.next().unwrap_or("").trim();

    // connector.action
    let (connector, action) = action_part.split_once('.')?;

    // key=value pairs
    let params: Vec<(String, String)> = if params_str.is_empty() {
        Vec::new()
    } else {
        params_str
            .split(',')
            .filter_map(|p| {
                let (k, v) = p.trim().split_once('=')?;
                Some((k.trim().to_string(), v.trim().to_string()))
            })
            .collect()
    };

    Some(FetchDirective {
        connector: connector.trim().to_string(),
        action: action.trim().to_string(),
        params,
    })
}

/// Anti-loop protection: returns true if the round limit is exceeded.
pub fn exceeds_fetch_limit(round: u32, max_rounds: u32) -> bool {
    round >= max_rounds
}

/// Truncate a fetch result to max_chars, breaking at sentence/paragraph boundaries.
pub fn truncate_result(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        return text.to_string();
    }

    let truncated = &text[..max_chars];
    let break_at = truncated
        .rfind("\n\n")
        .or_else(|| truncated.rfind(". "))
        .or_else(|| truncated.rfind('\n'))
        .unwrap_or(max_chars);

    format!(
        "{}\n\n[Result truncated at {} characters]",
        &text[..break_at],
        max_chars
    )
}

/// Build the prompt header listing all available connectors.
pub fn build_connector_header(connectors: &[(String, Vec<String>)]) -> String {
    if connectors.is_empty() {
        return String::new();
    }

    let mut header = String::from("Available data sources:\n");
    for (service, actions) in connectors {
        let actions_str = actions.join(", ");
        header.push_str(&format!("- {service}: {actions_str}\n"));
    }
    header.push_str("Use @fetch(service.action, param=value) to query.\n");
    header
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fetch() {
        let text = "Check: @fetch(calendar.list_events, range=today)";
        let dirs = parse_fetch_directives(text);
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].connector, "calendar");
        assert_eq!(dirs[0].action, "list_events");
        assert_eq!(dirs[0].params, vec![("range".into(), "today".into())]);
    }

    #[test]
    fn test_truncate() {
        let text = "A".repeat(5000);
        let result = truncate_result(&text, 4000);
        assert!(result.len() < 4100);
        assert!(result.contains("[Result truncated"));
    }

    #[test]
    fn test_anti_loop() {
        assert!(!exceeds_fetch_limit(0, 3));
        assert!(!exceeds_fetch_limit(2, 3));
        assert!(exceeds_fetch_limit(3, 3));
    }
}
