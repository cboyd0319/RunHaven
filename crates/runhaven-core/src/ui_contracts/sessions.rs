use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::runtime::worktrees::{load_worktree_lifecycle, verify_lifecycle};

use super::helpers::{
    auth_profiles_by_name, display_agent_name, project_label, status_label, string_field,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedSessionListData {
    pub sessions: Vec<ManagedSessionSummaryData>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedSessionSummaryData {
    pub id: String,
    pub run_id: String,
    pub agent: String,
    pub session: String,
    pub title: String,
    pub project: String,
    pub status: String,
    pub status_label: String,
    pub auth_status: String,
    pub auth_scope: String,
    pub network: String,
    pub started_at: String,
    pub container_name: String,
    pub detail_command: String,
    pub lifecycle: String,
    pub input: ManagedSessionInputData,
    pub cleanup: ManagedSessionCleanupData,
    pub next_action: ManagedSessionNextActionData,
    pub events: Vec<ManagedSessionEventData>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedSessionNextActionData {
    pub kind: String,
    pub label: String,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedSessionInputData {
    pub attached: bool,
    pub status: String,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedSessionCleanupData {
    pub available: bool,
    pub target: String,
    pub label: String,
    pub detail: String,
}

impl ManagedSessionCleanupData {
    fn unavailable() -> Self {
        Self {
            available: false,
            target: String::new(),
            label: String::new(),
            detail: String::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedSessionEventData {
    pub source: String,
    pub label: String,
    pub detail: String,
    pub timestamp: String,
}

impl ManagedSessionListData {
    pub fn from_active_runs_and_auth_status(
        records: impl IntoIterator<Item = serde_json::Value>,
        auth_status: serde_json::Value,
    ) -> Self {
        Self::from_active_and_recent_runs(records, std::iter::empty(), auth_status)
    }

    pub fn from_active_and_recent_runs(
        active_records: impl IntoIterator<Item = serde_json::Value>,
        recent_records: impl IntoIterator<Item = serde_json::Value>,
        auth_status: serde_json::Value,
    ) -> Self {
        let auth_by_agent = auth_profiles_by_name(&auth_status);
        let mut active_run_ids = BTreeSet::new();
        let mut sessions = Vec::new();
        for record in active_records {
            let session = ManagedSessionSummaryData::from_active_run_record(record, &auth_by_agent);
            active_run_ids.insert(session.run_id.clone());
            sessions.push(session);
        }
        for record in recent_records {
            let run_id = string_field(&record, "run_id");
            if run_id.is_empty() || active_run_ids.contains(&run_id) {
                continue;
            }
            sessions.push(ManagedSessionSummaryData::from_run_record(
                record,
                &auth_by_agent,
            ));
        }
        sessions.sort_by(|left, right| {
            right
                .started_at
                .cmp(&left.started_at)
                .then_with(|| right.run_id.cmp(&left.run_id))
        });
        Self { sessions }
    }
}

impl ManagedSessionSummaryData {
    fn from_active_run_record(
        record: serde_json::Value,
        auth_by_agent: &BTreeMap<String, String>,
    ) -> Self {
        let run_id = string_field(&record, "run_id");
        let agent = string_field(&record, "profile");
        let project = project_label(&string_field(&record, "workspace"));
        let status = string_field(&record, "status");
        let status_label = status_label(&status);
        let auth_status = auth_by_agent
            .get(&agent)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let auth_scope = string_field(&record, "auth_scope");
        let network = string_field(&record, "network");
        let started_at = string_field(&record, "timestamp");
        let lifecycle = managed_session_lifecycle(&status);
        let cleanup = ManagedSessionCleanupData::unavailable();
        Self {
            id: run_id.clone(),
            run_id: run_id.clone(),
            agent: agent.clone(),
            session: string_field(&record, "session"),
            title: format!("{} - {}", display_agent_name(&agent), project),
            project,
            status_label: status_label.clone(),
            status,
            auth_status: auth_status.clone(),
            auth_scope,
            network: network.clone(),
            started_at: started_at.clone(),
            container_name: string_field(&record, "container_name"),
            detail_command: format!("runhaven runs show {run_id}"),
            input: ManagedSessionInputData::detached(),
            cleanup,
            next_action: managed_session_next_action(&agent, &auth_status, &lifecycle),
            events: managed_session_events(&status_label, &auth_status, &network, &started_at),
            lifecycle,
        }
    }

    fn from_run_record(
        record: serde_json::Value,
        auth_by_agent: &BTreeMap<String, String>,
    ) -> Self {
        let run_id = string_field(&record, "run_id");
        let agent = string_field(&record, "profile");
        let project = project_label(&string_field(&record, "workspace"));
        let status = string_field(&record, "status");
        let status_label = status_label(&status);
        let auth_status = auth_by_agent
            .get(&agent)
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());
        let auth_scope = string_field(&record, "auth_scope");
        let network = string_field(&record, "network");
        let started_at = first_non_empty(
            string_field(&record, "started_at"),
            string_field(&record, "timestamp"),
        );
        let lifecycle = managed_session_lifecycle(&status);
        let cleanup = managed_session_cleanup(&record, &lifecycle);
        Self {
            id: run_id.clone(),
            run_id: run_id.clone(),
            agent: agent.clone(),
            session: string_field(&record, "session"),
            title: format!("{} - {}", display_agent_name(&agent), project),
            project,
            status_label: status_label.clone(),
            status,
            auth_status: auth_status.clone(),
            auth_scope,
            network: network.clone(),
            started_at: started_at.clone(),
            container_name: string_field(&record, "container_name"),
            detail_command: format!("runhaven runs show {run_id}"),
            input: ManagedSessionInputData::detached(),
            cleanup,
            next_action: managed_session_next_action(&agent, &auth_status, &lifecycle),
            events: managed_session_events(&status_label, &auth_status, &network, &started_at),
            lifecycle,
        }
    }
}

impl ManagedSessionInputData {
    pub fn attached() -> Self {
        Self {
            attached: true,
            status: "attached".to_string(),
            detail: "Session input is ready in this RunHaven window.".to_string(),
        }
    }

    pub fn detached() -> Self {
        Self {
            attached: false,
            status: "detached".to_string(),
            detail: "Session input is not attached in this RunHaven window.".to_string(),
        }
    }
}

fn managed_session_lifecycle(status: &str) -> String {
    match status.trim().to_lowercase().as_str() {
        "running" => "running".to_string(),
        "started" | "starting" => "starting".to_string(),
        "finished" | "succeeded" => "finished".to_string(),
        "failed" | "error" | "crashed" => "failed".to_string(),
        "stopped" | "exited" => "stopped".to_string(),
        "" | "-" => "unknown".to_string(),
        other => other.to_string(),
    }
}

fn managed_session_next_action(
    agent: &str,
    auth_status: &str,
    lifecycle: &str,
) -> ManagedSessionNextActionData {
    if matches!(lifecycle, "running" | "starting")
        && managed_session_auth_needs_attention(agent, auth_status)
    {
        return ManagedSessionNextActionData {
            kind: "auth_attention".to_string(),
            label: "Finish auth setup".to_string(),
            detail: format!("Auth is not ready for {}.", display_agent_name(agent)),
        };
    }

    match lifecycle {
        "running" | "starting" => ManagedSessionNextActionData {
            kind: "monitor".to_string(),
            label: "Monitor session".to_string(),
            detail: "RunHaven is tracking status for this session.".to_string(),
        },
        "finished" | "stopped" => ManagedSessionNextActionData {
            kind: "none".to_string(),
            label: "No action needed".to_string(),
            detail: "The session is no longer running.".to_string(),
        },
        "failed" => ManagedSessionNextActionData {
            kind: "review".to_string(),
            label: "Review session".to_string(),
            detail: "The session reported a failure.".to_string(),
        },
        _ => ManagedSessionNextActionData {
            kind: "review".to_string(),
            label: "Review session".to_string(),
            detail: "RunHaven does not recognize this session state.".to_string(),
        },
    }
}

fn managed_session_cleanup(
    record: &serde_json::Value,
    lifecycle: &str,
) -> ManagedSessionCleanupData {
    if matches!(lifecycle, "finished" | "failed" | "stopped")
        && managed_session_worktree_cleanup_available(record)
    {
        return ManagedSessionCleanupData {
            available: true,
            target: "worktree".to_string(),
            label: "Clean up worktree".to_string(),
            detail: "Delete the recorded worktree and branch for this session.".to_string(),
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

fn first_non_empty(primary: String, fallback: String) -> String {
    if primary.trim().is_empty() {
        fallback
    } else {
        primary
    }
}

fn managed_session_auth_needs_attention(agent: &str, auth_status: &str) -> bool {
    if agent.trim().eq_ignore_ascii_case("shell") {
        return false;
    }

    !matches!(
        auth_status.trim().to_lowercase().as_str(),
        "api-key-broker" | "brokered" | "ready" | "available" | "n/a"
    )
}

fn managed_session_events(
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
            label: "Auth".to_string(),
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
        "provider" => "Provider only".to_string(),
        "internal" => "Local only".to_string(),
        "internet" => "Internet".to_string(),
        "" | "-" => "Unknown".to_string(),
        _ => display_words(network),
    }
}

fn auth_status_label(status: &str) -> String {
    match status.trim().to_lowercase().as_str() {
        "api-key-broker" => "API key broker".to_string(),
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
