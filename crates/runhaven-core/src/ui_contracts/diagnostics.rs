use serde::{Deserialize, Serialize};

use crate::doctor::Check;
use crate::provider::auth_broker::sanitize_broker_request_path;

use super::helpers::{count_field, string_field, strip_terminal_controls, u32_field};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunHavenDiagnosticsData {
    pub doctor_checks: Vec<DoctorCheckData>,
    pub auth_status: AuthStatusData,
    pub egress_log: Vec<EgressDecisionData>,
    pub auth_log: Vec<AuthDecisionData>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorCheckData {
    pub name: String,
    pub ok: bool,
    pub detail: String,
    pub remedy: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatusData {
    pub status: String,
    pub runtime: String,
    pub profiles: Vec<AuthProfileStatusData>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthProfileStatusData {
    pub name: String,
    pub status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EgressDecisionData {
    pub timestamp: String,
    pub profile: String,
    pub decision: String,
    pub host: String,
    pub port: u32,
    pub count: u64,
    pub reason: String,
    pub matched_rule: String,
    pub run_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthDecisionData {
    pub timestamp: String,
    pub profile: String,
    pub broker: String,
    pub decision: String,
    pub method: String,
    pub path: String,
    pub upstream_status: Option<u32>,
    pub count: u64,
    pub reason: String,
    pub run_id: String,
}

impl RunHavenDiagnosticsData {
    pub fn from_payloads(
        doctor_checks: impl IntoIterator<Item = Check>,
        auth_status: serde_json::Value,
        egress_log: impl IntoIterator<Item = serde_json::Value>,
        auth_log: impl IntoIterator<Item = serde_json::Value>,
    ) -> Self {
        Self {
            doctor_checks: doctor_checks
                .into_iter()
                .map(DoctorCheckData::from_check)
                .collect(),
            auth_status: AuthStatusData::from_payload(auth_status),
            egress_log: egress_log
                .into_iter()
                .map(EgressDecisionData::from_log_record)
                .collect(),
            auth_log: auth_log
                .into_iter()
                .map(AuthDecisionData::from_log_record)
                .collect(),
        }
    }
}

impl DoctorCheckData {
    pub fn from_check(check: Check) -> Self {
        let name = strip_terminal_controls(&check.name);
        let detail = if name == "Apple container CLI" && check.ok {
            "found on PATH".to_string()
        } else {
            strip_terminal_controls(&check.detail)
        };
        Self {
            name,
            ok: check.ok,
            detail,
            remedy: strip_terminal_controls(&check.remedy),
        }
    }
}

impl AuthStatusData {
    pub fn from_payload(payload: serde_json::Value) -> Self {
        let profiles = payload
            .get("profiles")
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .map(|item| AuthProfileStatusData {
                        name: string_field(item, "name"),
                        status: string_field(item, "status"),
                    })
                    .collect()
            })
            .unwrap_or_default();
        Self {
            status: string_field(&payload, "status"),
            runtime: string_field(&payload, "runtime"),
            profiles,
        }
    }
}

impl EgressDecisionData {
    pub fn from_log_record(record: serde_json::Value) -> Self {
        Self {
            timestamp: string_field(&record, "timestamp"),
            profile: string_field(&record, "profile"),
            decision: string_field(&record, "decision"),
            host: string_field(&record, "host"),
            port: u32_field(&record, "port"),
            count: count_field(&record),
            reason: string_field(&record, "reason"),
            matched_rule: string_field(&record, "matched_rule"),
            run_id: string_field(&record, "run_id"),
        }
    }
}

impl AuthDecisionData {
    pub fn from_log_record(record: serde_json::Value) -> Self {
        Self {
            timestamp: string_field(&record, "timestamp"),
            profile: string_field(&record, "profile"),
            broker: string_field(&record, "broker"),
            decision: string_field(&record, "decision"),
            method: string_field(&record, "method"),
            path: sanitize_broker_request_path(&string_field(&record, "path")),
            upstream_status: record
                .get("upstream_status")
                .and_then(serde_json::Value::as_u64)
                .map(|status| status as u32),
            count: count_field(&record),
            reason: string_field(&record, "reason"),
            run_id: string_field(&record, "run_id"),
        }
    }
}
