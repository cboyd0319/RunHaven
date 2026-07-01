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
