//! Third-party data connectors: OAuth flows, @fetch directive parsing,
//! and connector registry.

pub mod fetch;
pub mod registry;

use serde::{Deserialize, Serialize};

/// OAuth 2.0 token set stored in keychain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSet {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>,
    pub scope: String,
}

/// Initiate an OAuth 2.0 Authorization Code flow with PKCE.
///
/// 1. Generate code_verifier + code_challenge (SHA-256).
/// 2. Start a localhost HTTP server on a random port.
/// 3. Open the browser to the authorization URL.
/// 4. Wait for the callback with the auth code.
/// 5. Exchange the code for tokens.
///
/// This is a placeholder — the actual HTTP server and browser launch
/// require `reqwest` and platform-specific browser-open logic.
pub async fn start_oauth_flow(
    _auth_url: &str,
    _token_url: &str,
    _client_id: &str,
    _client_secret: &str,
    _scopes: &[&str],
) -> Result<TokenSet, String> {
    // TODO: Implement full OAuth flow when reqwest is available.
    Err("OAuth flow not yet implemented".into())
}

/// Refresh an expired access token.
pub async fn refresh_token(
    _token_url: &str,
    _client_id: &str,
    _client_secret: &str,
    _refresh_token: &str,
) -> Result<TokenSet, String> {
    Err("Token refresh not yet implemented".into())
}
