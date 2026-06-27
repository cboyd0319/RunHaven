use serde_json::{Value, json};

use crate::runhaven::provider::auth_profiles::{
    AUTH_BROKER_RUNTIME, AUTH_BROKER_STATUS, auth_broker_profiles,
};
use crate::runhaven::records::read_jsonl;
use crate::runhaven::support::paths::{auth_broker_log_path, egress_policy_log_path};

pub fn read_egress_policy_log(limit: usize) -> anyhow::Result<Vec<Value>> {
    read_jsonl(&egress_policy_log_path(), limit)
}

pub fn read_auth_broker_log(limit: usize) -> anyhow::Result<Vec<Value>> {
    read_jsonl(&auth_broker_log_path(), limit)
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
