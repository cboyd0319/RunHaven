use serde::{Deserialize, Serialize};

/// Rust-owned setup mutations the desktop app supports. The WebView may name
/// one opaque variant; it cannot supply an executable, arguments, or host path.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SetupActionId {
    OpenAppleContainerInstaller,
    StartContainerSystem,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SetupRecoveryRequest {
    pub action_id: SetupActionId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetupAction {
    pub id: SetupActionId,
    pub label: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SetupRecoveryResponse {
    pub action_id: SetupActionId,
    pub status: String,
    pub label: String,
    pub detail: String,
}
