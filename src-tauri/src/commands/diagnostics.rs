use runhaven_core::diagnostics::{
    auth_explain_payload, auth_status_payload, read_auth_broker_log, read_egress_policy_log,
};
use serde_json::Value;

use super::validation::{MAX_AGENT_NAME_LEN, validate_text_len};
use crate::contracts::{
    AuthExplainResponse, AuthLogEntry, AuthLogResponse, AuthProfileStatus, AuthStatusResponse,
    EgressLogEntry, EgressLogResponse,
};

/// Upper bound on log entries returned to the frontend. The records are
/// append-only and read fully into memory by the CLI; the desktop reads only a
/// bounded recent window.
pub(super) const MAX_LOG_ENTRIES: usize = 100;

#[tauri::command]
pub(crate) fn get_egress_log() -> Result<EgressLogResponse, String> {
    let entries = read_egress_policy_log(MAX_LOG_ENTRIES).map_err(|error| error.to_string())?;
    Ok(EgressLogResponse {
        entries: entries.iter().map(egress_entry).collect(),
    })
}

#[tauri::command]
pub(crate) fn get_auth_log() -> Result<AuthLogResponse, String> {
    let entries = read_auth_broker_log(MAX_LOG_ENTRIES).map_err(|error| error.to_string())?;
    Ok(AuthLogResponse {
        entries: entries.iter().map(auth_entry).collect(),
    })
}

#[tauri::command]
pub(crate) fn get_auth_status() -> AuthStatusResponse {
    auth_status_response(&auth_status_payload())
}

#[tauri::command]
pub(crate) fn get_auth_explain(agent: String) -> Result<AuthExplainResponse, String> {
    validate_text_len("agent", &agent, MAX_AGENT_NAME_LEN)?;
    let payload = auth_explain_payload(&agent).map_err(|error| error.to_string())?;
    Ok(auth_explain_response(&payload))
}

pub(super) fn egress_entry(record: &Value) -> EgressLogEntry {
    EgressLogEntry {
        timestamp: string_field(record, "timestamp"),
        profile: string_field(record, "profile"),
        decision: string_field(record, "decision"),
        host: string_field(record, "host"),
        port: u32_field(record, "port"),
        count: count_field(record),
        reason: string_field(record, "reason"),
        matched_rule: string_field(record, "matched_rule"),
        run_id: string_field(record, "run_id"),
    }
}

pub(super) fn auth_entry(record: &Value) -> AuthLogEntry {
    AuthLogEntry {
        timestamp: string_field(record, "timestamp"),
        profile: string_field(record, "profile"),
        broker: string_field(record, "broker"),
        decision: string_field(record, "decision"),
        method: string_field(record, "method"),
        path: string_field(record, "path"),
        upstream_status: record
            .get("upstream_status")
            .and_then(Value::as_u64)
            .map(|status| status as u32),
        count: count_field(record),
        reason: string_field(record, "reason"),
        run_id: string_field(record, "run_id"),
    }
}

pub(super) fn auth_status_response(payload: &Value) -> AuthStatusResponse {
    let profiles = payload
        .get("profiles")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| AuthProfileStatus {
                    name: string_field(item, "name"),
                    status: string_field(item, "status"),
                })
                .collect()
        })
        .unwrap_or_default();
    AuthStatusResponse {
        status: string_field(payload, "status"),
        runtime: string_field(payload, "runtime"),
        profiles,
    }
}

pub(super) fn auth_explain_response(payload: &Value) -> AuthExplainResponse {
    AuthExplainResponse {
        name: string_field(payload, "name"),
        status: string_field(payload, "status"),
        runtime: string_field(payload, "runtime"),
        supported_auth: string_list_field(payload, "supported_auth"),
        host_keeps: string_list_field(payload, "host_keeps"),
        guest_receives: string_list_field(payload, "guest_receives"),
        provider_hosts: string_list_field(payload, "provider_hosts"),
        current_safe_path: string_field(payload, "current_safe_path"),
        notes: string_list_field(payload, "notes"),
    }
}

fn string_list_field(record: &Value, name: &str) -> Vec<String> {
    record
        .get(name)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn string_field(record: &Value, name: &str) -> String {
    record
        .get(name)
        .and_then(Value::as_str)
        .unwrap_or("-")
        .to_string()
}

fn u32_field(record: &Value, name: &str) -> u32 {
    record.get(name).and_then(Value::as_u64).unwrap_or(0) as u32
}

fn count_field(record: &Value) -> u64 {
    record.get("count").and_then(Value::as_u64).unwrap_or(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn egress_entry_maps_record_fields_without_paths() {
        let record = serde_json::json!({
            "timestamp": "2026-06-25T00:00:00Z",
            "profile": "codex",
            "decision": "denied",
            "host": "iana.org",
            "port": 443,
            "count": 3,
            "reason": "not-in-allowlist",
            "matched_rule": "",
            "run_id": "abc",
            "workspace": "/Users/secret/project"
        });
        let entry = egress_entry(&record);
        assert_eq!(entry.host, "iana.org");
        assert_eq!(entry.port, 443);
        assert_eq!(entry.count, 3);
        assert_eq!(entry.decision, "denied");
        // workspace path is intentionally not part of EgressLogEntry.
    }

    #[test]
    fn auth_entry_maps_optional_upstream_status() {
        let with_status = auth_entry(&serde_json::json!({
            "decision": "allowed",
            "method": "POST",
            "path": "/v1/responses",
            "upstream_status": 200,
        }));
        assert_eq!(with_status.upstream_status, Some(200));

        let without_status = auth_entry(&serde_json::json!({
            "decision": "no-requests",
            "method": "-",
            "path": "-",
        }));
        assert_eq!(without_status.upstream_status, None);
        assert_eq!(without_status.count, 1);
    }

    #[test]
    fn auth_status_response_maps_payload() {
        let response = auth_status_response(&auth_status_payload());
        assert!(!response.status.is_empty());
        assert!(!response.profiles.is_empty());
    }

    #[test]
    fn auth_explain_response_maps_only_safe_fields() {
        let payload = serde_json::json!({
            "name": "claude",
            "status": "api-key-broker",
            "runtime": "macOS 26+ with Apple container only",
            "supported_auth": ["Anthropic API key through --api-key-broker-env NAME (brokered host-side)"],
            "host_keeps": ["the host environment variable value named by --api-key-broker-env"],
            "guest_receives": ["a placeholder ANTHROPIC_API_KEY value"],
            "current_safe_path": "Use --network provider --api-key-broker-env.",
            "notes": ["note"],
            "provider_hosts": ["api.anthropic.com"],
            // Flags and any stray host path must not appear on the mapped struct.
            "credential_stores_inspected": false,
            "secret_token": "sk-should-never-map",
            "host_path": "/Users/secret/.claude.json",
        });
        let response = auth_explain_response(&payload);
        assert_eq!(response.name, "claude");
        assert_eq!(response.provider_hosts, vec!["api.anthropic.com"]);
        let serialized = serde_json::to_string(&response).expect("serialize");
        assert!(!serialized.contains("sk-should-never-map"));
        assert!(!serialized.contains("/Users/secret"));
        assert!(!serialized.contains("credentialStoresInspected"));
    }

    #[test]
    fn get_auth_explain_returns_known_agent_fields() {
        let response = get_auth_explain("claude".to_string()).expect("known agent");
        assert_eq!(response.name, "claude");
        assert!(!response.supported_auth.is_empty());
    }

    #[test]
    fn get_auth_explain_rejects_unknown_agent() {
        assert!(get_auth_explain("not-a-real-agent".to_string()).is_err());
    }

    #[test]
    fn get_auth_explain_rejects_overlong_agent() {
        let error = get_auth_explain("a".repeat(MAX_AGENT_NAME_LEN + 1))
            .expect_err("overlong agent must fail closed");
        assert!(error.contains("too long"));
    }
}
