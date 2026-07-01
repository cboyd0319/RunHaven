use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImageStatusRequest {
    pub agent: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProfileImageStatus {
    pub agent: String,
    pub image: String,
    pub status: String,
    pub ready: bool,
    pub expected_source_digest: String,
    pub local_source_digest: Option<String>,
    pub fix_command: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BuilderStatus {
    pub status: String,
    pub detail: String,
    pub image: Option<String>,
    pub cpus: Option<String>,
    pub memory: Option<String>,
    pub rosetta: Option<bool>,
    pub started_date: Option<String>,
    pub ipv4_address: Option<String>,
    pub warning: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImageStatusResponse {
    pub agent: String,
    pub image: ProfileImageStatus,
    pub builder: BuilderStatus,
}

/// Request to rebuild a bundled agent image. `confirm` must be true; the Rust
/// command still shows a native confirmation before any build starts.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImageBuildRequest {
    pub agent: String,
    #[serde(default)]
    pub confirm: bool,
}

/// Acknowledgement that a background rebuild was accepted. The WebView polls
/// `get_image_build_status` with `build_id` for sanitized progress.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImageBuildStartResponse {
    pub build_id: String,
    pub agent: String,
    pub tag: String,
    pub state: String,
    pub label: String,
    pub detail: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImageBuildStatusRequest {
    pub build_id: String,
}

/// Sanitized background-build progress. `output_tail` is a bounded, secret
/// redacted tail of the builder output; no raw host paths or unbounded logs.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImageBuildStatusResponse {
    pub build_id: String,
    pub agent: String,
    pub tag: String,
    pub state: String,
    pub label: String,
    pub detail: String,
    pub output_tail: String,
    pub updated_at: String,
}
