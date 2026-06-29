use serde_json::{Value, json};

use crate::provider::auth_broker::sanitize_broker_request_path;
use crate::provider::auth_profiles::{
    AUTH_BROKER_RUNTIME, AUTH_BROKER_STATUS, auth_broker_profiles,
};
use crate::records::{read_jsonl, read_jsonl_tail_bounded};
use crate::support::paths::{auth_broker_log_path, egress_policy_log_path};

pub fn read_egress_policy_log(limit: usize) -> anyhow::Result<Vec<Value>> {
    read_jsonl(&egress_policy_log_path(), limit)
}

pub fn read_egress_policy_log_tail_bounded(
    limit: usize,
    max_tail_bytes: u64,
) -> anyhow::Result<Vec<Value>> {
    read_jsonl_tail_bounded(&egress_policy_log_path(), limit, max_tail_bytes)
}

pub fn read_auth_broker_log(limit: usize) -> anyhow::Result<Vec<Value>> {
    Ok(sanitize_auth_log_records(read_jsonl(
        &auth_broker_log_path(),
        limit,
    )?))
}

pub fn read_auth_broker_log_tail_bounded(
    limit: usize,
    max_tail_bytes: u64,
) -> anyhow::Result<Vec<Value>> {
    Ok(sanitize_auth_log_records(read_jsonl_tail_bounded(
        &auth_broker_log_path(),
        limit,
        max_tail_bytes,
    )?))
}

fn sanitize_auth_log_records(records: Vec<Value>) -> Vec<Value> {
    records.into_iter().map(sanitize_auth_log_record).collect()
}

fn sanitize_auth_log_record(mut record: Value) -> Value {
    if let Some(path) = record.get("path").and_then(Value::as_str) {
        let path = sanitize_broker_request_path(path);
        if let Some(object) = record.as_object_mut() {
            object.insert("path".to_string(), Value::String(path));
        }
    }
    record
}

/// Secret-free auth broker status payload, shared by the CLI, Tauri, and TUI.
/// Reports broker status, runtime, per-profile broker tiers, and explicit
/// "nothing inspected/printed" flags.
pub fn auth_status_payload() -> Value {
    json!({
        "status": AUTH_BROKER_STATUS,
        "runtime": AUTH_BROKER_RUNTIME,
        "credential_stores_inspected": false,
        "environment_values_inspected": false,
        "secrets_printed": false,
        "profiles": auth_broker_profiles(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::paths::{ensure_private_parent, override_cache_root_for_tests};
    use std::io::Write;

    #[test]
    fn auth_broker_log_reader_sanitizes_legacy_query_paths() {
        let cache = tempfile::tempdir().expect("cache");
        let _cache_home = override_cache_root_for_tests(cache.path());
        ensure_private_parent(&auth_broker_log_path()).expect("auth log parent");
        let mut file = std::fs::File::create(auth_broker_log_path()).expect("auth log file");
        writeln!(
            file,
            "{}",
            json!({
                "timestamp": "2026-06-29T00:00:00Z",
                "profile": "codex",
                "broker": "api-key",
                "decision": "allowed",
                "method": "POST",
                "path": "/v1/responses?token=secret#fragment",
                "upstream_status": 200,
                "count": 1,
                "reason": "-",
                "run_id": "run-123"
            })
        )
        .expect("write auth log");

        let entries = read_auth_broker_log(10).expect("read auth log");

        assert_eq!(
            entries[0].get("path").and_then(Value::as_str),
            Some("/v1/responses")
        );
        let serialized = serde_json::to_string(&entries).expect("serialize");
        assert!(!serialized.contains("token=secret"));
    }
}
