use serde::{Deserialize, Serialize};

use super::helpers::string_field;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveRunListData {
    pub runs: Vec<ActiveRunSummaryData>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveRunSummaryData {
    pub run_id: String,
    pub profile: String,
    pub network: String,
    pub status: String,
    pub auth_scope: String,
    pub timestamp: String,
    pub state_volume: String,
    pub session: String,
    pub container_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveRunLogSnapshotData {
    pub run_id: String,
    pub captured_at: String,
    pub requested_lines: u32,
    pub text: String,
    pub returned_lines: usize,
    pub truncated: bool,
    pub source: String,
    pub warnings: Vec<String>,
}

impl ActiveRunListData {
    pub fn from_active_run_records(records: impl IntoIterator<Item = serde_json::Value>) -> Self {
        let mut runs = records
            .into_iter()
            .map(ActiveRunSummaryData::from_active_run_record)
            .collect::<Vec<_>>();
        runs.sort_by(|left, right| right.timestamp.cmp(&left.timestamp));
        Self { runs }
    }
}

impl ActiveRunSummaryData {
    pub fn from_active_run_record(record: serde_json::Value) -> Self {
        Self {
            run_id: string_field(&record, "run_id"),
            profile: string_field(&record, "profile"),
            network: string_field(&record, "network"),
            status: string_field(&record, "status"),
            auth_scope: string_field(&record, "auth_scope"),
            timestamp: string_field(&record, "timestamp"),
            state_volume: string_field(&record, "state_volume"),
            session: string_field(&record, "session"),
            container_name: string_field(&record, "container_name"),
        }
    }
}

impl ActiveRunLogSnapshotData {
    pub fn from_active_log_snapshot_payload(value: serde_json::Value) -> serde_json::Result<Self> {
        #[derive(Deserialize)]
        #[serde(rename_all = "snake_case")]
        struct RawActiveRunLogSnapshotData {
            run_id: String,
            captured_at: String,
            requested_lines: u32,
            text: String,
            returned_lines: usize,
            truncated: bool,
            source: String,
            warnings: Vec<String>,
        }

        let raw: RawActiveRunLogSnapshotData = serde_json::from_value(value)?;
        Ok(Self {
            run_id: raw.run_id,
            captured_at: raw.captured_at,
            requested_lines: raw.requested_lines,
            text: raw.text,
            returned_lines: raw.returned_lines,
            truncated: raw.truncated,
            source: raw.source,
            warnings: raw.warnings,
        })
    }
}
