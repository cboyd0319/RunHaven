use std::collections::BTreeMap;

use crate::provider::auth_profiles::agent_sign_in_mode;
use crate::runtime::worktrees::{load_worktree_lifecycle, verify_lifecycle};

use super::helpers::{display_agent_name, i32_field, string_field, u64_pointer_field};
use super::sessions::{
    ManagedSessionCleanupData, ManagedSessionEventData, ManagedSessionNextActionData,
};

pub(super) fn managed_session_lifecycle(status: &str) -> String {
    match status.trim().to_lowercase().as_str() {
        "running" => "running".to_string(),
        "started" | "starting" => "starting".to_string(),
        "finished" | "succeeded" => "finished".to_string(),
        "failed" | "error" | "crashed" => "failed".to_string(),
        "stopped" | "exited" => "stopped".to_string(),
        // An interactive TTY run the user ended by closing it: a terminal,
        // neutral state, never the red "failed" styling.
        "ended" => "ended".to_string(),
        "" | "-" => "unknown".to_string(),
        other => other.to_string(),
    }
}

pub(super) fn managed_session_next_action(
    record: &serde_json::Value,
    agent: &str,
    auth_status: &str,
    lifecycle: &str,
) -> ManagedSessionNextActionData {
    if matches!(lifecycle, "running" | "starting")
        && managed_session_auth_needs_attention(agent, auth_status)
    {
        return ManagedSessionNextActionData {
            kind: "auth_attention".to_string(),
            label: "Finish sign-in".to_string(),
            detail: format!("Sign-in is not ready for {}.", display_agent_name(agent)),
        };
    }

    match lifecycle {
        "running" | "starting" => ManagedSessionNextActionData {
            kind: "monitor".to_string(),
            label: "Session running".to_string(),
            detail: "The assistant is working.".to_string(),
        },
        "finished" | "stopped" | "ended" => ManagedSessionNextActionData {
            kind: "none".to_string(),
            label: "No action needed".to_string(),
            detail: "The session is no longer running.".to_string(),
        },
        "failed" => ManagedSessionNextActionData {
            kind: "review".to_string(),
            label: "Review session".to_string(),
            detail: managed_session_failure_reason(record),
        },
        _ => ManagedSessionNextActionData {
            kind: "review".to_string(),
            label: "Review session".to_string(),
            detail: "RunHaven does not recognize this session state.".to_string(),
        },
    }
}

fn managed_session_failure_reason(record: &serde_json::Value) -> String {
    if u64_pointer_field(record, "/auth_broker/denied") > 0 {
        return "An authentication request was blocked. Review this session's Sign-in details, then try again.".to_string();
    }
    if u64_pointer_field(record, "/provider_policy/denied") > 0 {
        return "A network request was blocked. Review this session's Network details, then try again.".to_string();
    }
    if i32_field(record, "return_code").is_some_and(|code| code != 0) {
        return "The assistant stopped before it finished. Open System to check setup, then try again."
            .to_string();
    }
    "RunHaven couldn’t determine why this session ended. Open System to check setup, then try again."
        .to_string()
}

pub(super) fn managed_session_cleanup(
    record: &serde_json::Value,
    lifecycle: &str,
) -> ManagedSessionCleanupData {
    if matches!(lifecycle, "finished" | "failed" | "stopped" | "ended")
        && managed_session_worktree_cleanup_available(record)
    {
        return ManagedSessionCleanupData {
            available: true,
            target: "worktree".to_string(),
            label: "Delete protected copy".to_string(),
            detail:
                "Permanently delete the assistant’s saved copy. Your original project won’t change."
                    .to_string(),
        };
    }
    ManagedSessionCleanupData::unavailable()
}

fn managed_session_worktree_cleanup_available(record: &serde_json::Value) -> bool {
    if record
        .get("worktree")
        .and_then(serde_json::Value::as_object)
        .is_none()
    {
        return false;
    }
    let run_id = string_field(record, "run_id");
    if run_id.is_empty() {
        return false;
    }
    let Ok(lifecycle) = load_worktree_lifecycle(&run_id) else {
        return false;
    };
    verify_lifecycle(&lifecycle).is_ok()
}

pub(super) fn first_non_empty(primary: String, fallback: String) -> String {
    if primary.trim().is_empty() {
        fallback
    } else {
        primary
    }
}

pub(super) fn managed_session_auth_status(
    record: &serde_json::Value,
    agent: &str,
    auth_scope: &str,
    auth_by_agent: &BTreeMap<String, String>,
) -> String {
    if agent.trim().eq_ignore_ascii_case("shell") {
        return "no-auth-needed".to_string();
    }
    if managed_session_has_run_owned_broker(record) {
        // Distinguish the Claude Keychain OAuth broker from the env-var API-key
        // broker so the session view names the real path instead of always
        // saying "API key broker". The broker label is non-secret. A RUNNING run
        // exposes the source on the marker's `claude_auth_source`; a completed run
        // exposes it on `/auth_broker/broker`. Match both so a live keychain-oauth
        // session is not misread as needing sign-in.
        if record
            .pointer("/auth_broker/broker")
            .and_then(serde_json::Value::as_str)
            == Some("keychain-oauth")
            || string_field(record, "claude_auth_source") == "keychain-oauth"
        {
            return "keychain-oauth".to_string();
        }
        return "api-key-broker".to_string();
    }
    if managed_session_uses_login_state(agent, auth_scope) {
        return "login-state".to_string();
    }

    let static_status = auth_by_agent
        .get(agent)
        .map(String::as_str)
        .unwrap_or("unknown");
    match static_status.trim().to_lowercase().as_str() {
        "api-key-broker" | "brokered" | "ready" | "available" | "n/a" => "unknown".to_string(),
        "" | "-" => "unknown".to_string(),
        _ => static_status.to_string(),
    }
}

fn managed_session_has_run_owned_broker(record: &serde_json::Value) -> bool {
    present_value(&string_field(record, "api_key_broker_env"))
        // A running keychain-oauth run has no env-var name; its marker tags the
        // source here so the live session is recognized as brokered, not login.
        || present_value(&string_field(record, "claude_auth_source"))
        || record
            .pointer("/auth_broker/broker")
            .and_then(serde_json::Value::as_str)
            .is_some_and(present_value)
}

fn managed_session_uses_login_state(agent: &str, auth_scope: &str) -> bool {
    agent_sign_in_mode(agent).supports_login()
        && matches!(
            auth_scope.trim().to_lowercase().as_str(),
            "agent" | "project"
        )
}

fn present_value(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty() && trimmed != "-"
}

fn managed_session_auth_needs_attention(agent: &str, auth_status: &str) -> bool {
    if agent.trim().eq_ignore_ascii_case("shell") {
        return false;
    }

    !matches!(
        auth_status.trim().to_lowercase().as_str(),
        "api-key-broker"
            | "keychain-oauth"
            | "login-state"
            | "brokered"
            | "ready"
            | "available"
            | "n/a"
            | "no-auth-needed"
    )
}

pub(super) fn managed_session_events(
    status_label: &str,
    auth_status: &str,
    network: &str,
    timestamp: &str,
) -> Vec<ManagedSessionEventData> {
    vec![
        ManagedSessionEventData {
            source: "run".to_string(),
            label: "Run record".to_string(),
            detail: status_label.to_string(),
            timestamp: timestamp.to_string(),
        },
        ManagedSessionEventData {
            source: "auth".to_string(),
            label: "Sign-in".to_string(),
            detail: auth_status_label(auth_status),
            timestamp: timestamp.to_string(),
        },
        ManagedSessionEventData {
            source: "network".to_string(),
            label: "Network".to_string(),
            detail: network_label(network),
            timestamp: timestamp.to_string(),
        },
    ]
}

fn network_label(network: &str) -> String {
    match network.trim().to_lowercase().as_str() {
        "provider" => "Assistant services only".to_string(),
        "internal" => "Local only".to_string(),
        "internet" => "Internet".to_string(),
        "" | "-" => "Unknown".to_string(),
        _ => display_words(network),
    }
}

fn auth_status_label(status: &str) -> String {
    match status.trim().to_lowercase().as_str() {
        "api-key-broker" => "API key from this Mac".to_string(),
        "keychain-oauth" => "Legacy Claude Keychain sign-in".to_string(),
        "login-state" => "Session sign-in".to_string(),
        "no-auth-needed" => "No sign-in needed".to_string(),
        "n/a" => "N/A".to_string(),
        "" | "-" => "Unknown".to_string(),
        _ => display_words(status),
    }
}

fn display_words(value: &str) -> String {
    let cleaned = value.trim().replace('-', " ");
    if cleaned.is_empty() {
        return "Unknown".to_string();
    }
    cleaned
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            let mut label = first.to_uppercase().to_string();
            label.push_str(chars.as_str());
            label
        })
        .collect::<Vec<_>>()
        .join(" ")
}
