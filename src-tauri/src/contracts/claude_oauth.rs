use serde::{Deserialize, Serialize};

/// Request to start the host Claude Keychain sign-in (design C, Slice 4).
/// `confirm` must be true, and the Rust command still shows a native
/// confirmation naming Claude before anything runs. No secret is ever accepted
/// or returned here: the token is captured on the host and stored only in the
/// Keychain.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClaudeOAuthStartRequest {
    #[serde(default)]
    pub confirm: bool,
}

/// Acknowledgement that the background host capture started. The WebView polls
/// `get_claude_oauth_status` for sanitized progress. Carries only an id and
/// fixed presentation strings; never a token or any token-derived value.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClaudeOAuthStartResponse {
    pub sign_in_id: String,
    pub state: String,
    pub label: String,
    pub detail: String,
}

/// Sanitized Claude sign-in status: a plain `status` enum string
/// (`not_signed_in` | `signing` | `signed_in` | `failed`) derived from Keychain
/// item PRESENCE and the in-flight sign-in state, never from the token value.
/// `reason` is a fixed, path-free failure message present only for `failed`.
/// `sign_in_id` correlates the last attempt. No token, no token prefix, no
/// token-derived value ever appears here.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClaudeOAuthStatusResponse {
    pub status: String,
    pub reason: Option<String>,
    pub sign_in_id: Option<String>,
}

/// Request to remove RunHaven's own saved Claude token. `confirm` must be true,
/// and the Rust command still shows a native confirmation before deleting.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClaudeOAuthClearRequest {
    #[serde(default)]
    pub confirm: bool,
}

/// Result of a clear: `cleared` when RunHaven's item was removed, or
/// `already_absent` when there was nothing to remove. No secret is present.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ClaudeOAuthClearResponse {
    pub outcome: String,
    pub detail: String,
}
