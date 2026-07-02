use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenSessionWindowRequest {
    pub session_id: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OpenSessionWindowResponse {
    /// The Rust-minted monitor window label (`session-<n>`, never user data).
    pub window_label: String,
    /// True when an already-open monitor for this session was focused
    /// instead of creating a new window.
    pub focused_existing: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SessionWindowTargetResponse {
    /// The session the calling monitor window was opened for. Resolved from
    /// the calling window's label on the Rust side; the WebView supplies no
    /// input.
    pub session_id: String,
}
