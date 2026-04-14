//! Connector registry — metadata for all supported third-party services.

use serde::{Deserialize, Serialize};

/// Definition of a single connector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorDef {
    pub service_id: String,
    pub display_name: String,
    pub category: String,
    pub api_base: String,
    pub auth_type: AuthType,
    pub actions: Vec<ConnectorAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthType {
    OAuth2,
    Basic,
    ApiKey,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorAction {
    pub action_id: String,
    pub display_name: String,
    pub is_write: bool,
}

/// Return the full registry of known connectors (Tier 1 + Tier 2).
pub fn all_connectors() -> Vec<ConnectorDef> {
    vec![
        // -- Tier 1 (Phase 6 initial delivery) --
        connector("google.calendar", "Google Calendar", "Google Workspace", "googleapis.com/calendar/v3", AuthType::OAuth2, &[
            ("list_events", "List Events", false),
            ("get_event", "Get Event", false),
        ]),
        connector("google.gmail", "Gmail", "Google Workspace", "googleapis.com/gmail/v1", AuthType::OAuth2, &[
            ("search_emails", "Search Emails", false),
            ("get_email", "Get Email", false),
        ]),
        connector("github", "GitHub", "Developer Tools", "api.github.com", AuthType::OAuth2, &[
            ("list_issues", "List Issues", false),
            ("list_prs", "List Pull Requests", false),
            ("get_repo", "Get Repository", false),
            ("get_commits", "Get Commits", false),
        ]),
        connector("slack", "Slack", "Communication", "slack.com/api", AuthType::OAuth2, &[
            ("search_messages", "Search Messages", false),
            ("list_channels", "List Channels", false),
        ]),
        connector("linear", "Linear", "Developer Tools", "api.linear.app/graphql", AuthType::OAuth2, &[
            ("list_issues", "List Issues", false),
            ("search", "Search", false),
        ]),
        connector("notion", "Notion", "Developer Tools", "api.notion.com/v1", AuthType::OAuth2, &[
            ("query_database", "Query Database", false),
            ("search", "Search", false),
        ]),

        // -- Tier 2 (incremental additions) --
        connector("google.drive", "Google Drive", "Google Workspace", "googleapis.com/drive/v3", AuthType::OAuth2, &[
            ("list_files", "List Files", false),
            ("search_files", "Search Files", false),
        ]),
        connector("outlook.mail", "Outlook Mail", "Microsoft 365", "graph.microsoft.com/v1.0", AuthType::OAuth2, &[
            ("list_messages", "List Messages", false),
            ("search_messages", "Search Messages", false),
        ]),
        connector("outlook.calendar", "Outlook Calendar", "Microsoft 365", "graph.microsoft.com/v1.0", AuthType::OAuth2, &[
            ("list_events", "List Events", false),
        ]),
        connector("jira", "Jira", "Developer Tools", "atlassian.net/rest/api/3", AuthType::Basic, &[
            ("search_issues", "Search Issues", false),
            ("get_issue", "Get Issue", false),
        ]),
        connector("gitlab", "GitLab", "Developer Tools", "gitlab.com/api/v4", AuthType::OAuth2, &[
            ("list_issues", "List Issues", false),
            ("list_merge_requests", "List Merge Requests", false),
        ]),
        connector("trello", "Trello", "Developer Tools", "api.trello.com/1", AuthType::OAuth2, &[
            ("list_boards", "List Boards", false),
            ("list_cards", "List Cards", false),
        ]),
        connector("hubspot", "HubSpot", "Business", "api.hubapi.com", AuthType::OAuth2, &[
            ("list_contacts", "List Contacts", false),
            ("search", "Search", false),
        ]),
        connector("todoist", "Todoist", "Productivity", "api.todoist.com/rest/v2", AuthType::OAuth2, &[
            ("list_tasks", "List Tasks", false),
        ]),
    ]
}

fn connector(
    id: &str,
    name: &str,
    category: &str,
    api_base: &str,
    auth_type: AuthType,
    actions: &[(&str, &str, bool)],
) -> ConnectorDef {
    ConnectorDef {
        service_id: id.to_string(),
        display_name: name.to_string(),
        category: category.to_string(),
        api_base: api_base.to_string(),
        auth_type,
        actions: actions
            .iter()
            .map(|(id, name, is_write)| ConnectorAction {
                action_id: id.to_string(),
                display_name: name.to_string(),
                is_write: *is_write,
            })
            .collect(),
    }
}
