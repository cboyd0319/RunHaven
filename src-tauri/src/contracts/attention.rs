use serde::Deserialize;

/// Dock badge update from the WebView: the count of sessions needing the
/// user, nothing else. The count is validated in Rust before it touches the
/// Dock; see `commands::attention_badge`.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AttentionBadgeRequest {
    pub count: u32,
}
