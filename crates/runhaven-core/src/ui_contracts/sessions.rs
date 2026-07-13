use std::collections::{BTreeMap, BTreeSet};

use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};

use super::helpers::{
    auth_profiles_by_name, display_agent_name, project_label, status_label, string_field,
};
use super::session_fields::{
    first_non_empty, managed_session_auth_status, managed_session_cleanup, managed_session_events,
    managed_session_lifecycle, managed_session_next_action,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedSessionListData {
    pub sessions: Vec<ManagedSessionSummaryData>,
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
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

impl Serialize for ManagedSessionSummaryData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Core owns persisted/active metadata but never retains terminal
        // output. The Tauri list response replaces this false value from its
        // bounded process-memory output registry.
        let mut state = serializer.serialize_struct("ManagedSessionSummaryData", 20)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("runId", &self.run_id)?;
        state.serialize_field("agent", &self.agent)?;
        state.serialize_field("session", &self.session)?;
        state.serialize_field("title", &self.title)?;
        state.serialize_field("project", &self.project)?;
        state.serialize_field("status", &self.status)?;
        state.serialize_field("statusLabel", &self.status_label)?;
        state.serialize_field("authStatus", &self.auth_status)?;
        state.serialize_field("authScope", &self.auth_scope)?;
        state.serialize_field("network", &self.network)?;
        state.serialize_field("startedAt", &self.started_at)?;
        state.serialize_field("containerName", &self.container_name)?;
        state.serialize_field("detailCommand", &self.detail_command)?;
        state.serialize_field("lifecycle", &self.lifecycle)?;
        state.serialize_field("input", &self.input)?;
        state.serialize_field("cleanup", &self.cleanup)?;
        state.serialize_field("nextAction", &self.next_action)?;
        state.serialize_field("events", &self.events)?;
        state.serialize_field("outputAvailable", &false)?;
        state.end()
    }
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
    #[serde(default = "default_cleanup_review_state")]
    pub review_state: String,
    pub target: String,
    pub label: String,
    pub detail: String,
}

fn default_cleanup_review_state() -> String {
    "none".to_string()
}

impl ManagedSessionCleanupData {
    pub(super) fn unavailable() -> Self {
        Self {
            available: false,
            review_state: "none".to_string(),
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

/// Which run log a record was read from. Active runs carry no worktree cleanup
/// and start from their launch timestamp; recent history records may expose
/// worktree cleanup and prefer their recorded `started_at`.
enum RecordScope {
    Active,
    Recent,
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
            let session =
                ManagedSessionSummaryData::from_record(record, &auth_by_agent, RecordScope::Active);
            active_run_ids.insert(session.run_id.clone());
            sessions.push(session);
        }
        for record in recent_records {
            let run_id = string_field(&record, "run_id");
            if run_id.is_empty() || active_run_ids.contains(&run_id) {
                continue;
            }
            sessions.push(ManagedSessionSummaryData::from_record(
                record,
                &auth_by_agent,
                RecordScope::Recent,
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
    fn from_record(
        record: serde_json::Value,
        auth_by_agent: &BTreeMap<String, String>,
        scope: RecordScope,
    ) -> Self {
        let run_id = string_field(&record, "run_id");
        let agent = string_field(&record, "profile");
        let project = project_label(&string_field(&record, "workspace"));
        let status = string_field(&record, "status");
        let status_label = status_label(&status);
        let auth_scope = string_field(&record, "auth_scope");
        let auth_status = managed_session_auth_status(&record, &agent, &auth_scope, auth_by_agent);
        let network = string_field(&record, "network");
        let started_at = match scope {
            RecordScope::Active => string_field(&record, "timestamp"),
            RecordScope::Recent => first_non_empty(
                string_field(&record, "started_at"),
                string_field(&record, "timestamp"),
            ),
        };
        let lifecycle = managed_session_lifecycle(&status);
        let cleanup = match scope {
            RecordScope::Active => ManagedSessionCleanupData::unavailable(),
            RecordScope::Recent => managed_session_cleanup(&record, &lifecycle),
        };
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
            next_action: managed_session_next_action(&record, &agent, &auth_status, &lifecycle),
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
