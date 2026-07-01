use serde::{Deserialize, Serialize};

/// Count of prunable RunHaven-managed stale networks. Only a count is exposed;
/// the network names are withheld from the untrusted WebView.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NetworkPrunePreview {
    pub count: usize,
}

/// Request to remove RunHaven-managed stale networks. `confirm` must be true;
/// the Rust command still shows a native confirmation before any deletion.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NetworkPruneRequest {
    #[serde(default)]
    pub confirm: bool,
}

/// Sanitized outcome of a prune: only how many networks were removed versus
/// left in place. No raw names or `container` error text reaches the WebView.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NetworkPruneResponse {
    pub removed: usize,
    pub failed: usize,
}
