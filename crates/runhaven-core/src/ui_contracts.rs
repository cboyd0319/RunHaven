use serde::{Deserialize, Serialize};

use crate::provider::auth_broker::sanitize_broker_request_path;
use crate::provider::auth_profiles::{agent_broker, agent_sign_in};
use crate::runtime::plans::AgentRunPlan;
use crate::runtime::plans::default_network_mode;
use crate::runtime::profiles::AgentProfile;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum RunHavenComponentPayload {
    AgentCatalog(AgentCatalogData),
    LaunchPlan(Box<LaunchPlanData>),
    ActiveRunList(ActiveRunListData),
    ActiveRunLogSnapshot(Box<ActiveRunLogSnapshotData>),
    Diagnostics(Box<RunHavenDiagnosticsData>),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCatalogData {
    pub agents: Vec<AgentCatalogItemData>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCatalogItemData {
    pub name: String,
    pub description: String,
    pub image: String,
    pub sign_in: String,
    pub broker: String,
    pub default_network: String,
    pub provider_host_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchPlanData {
    pub profile_name: String,
    pub workspace: String,
    pub workspace_scope: String,
    pub workspace_scope_note: Option<String>,
    pub auth_scope: String,
    pub session: String,
    pub state_volume: String,
    pub container_name: String,
    pub image: String,
    pub worktree: Option<LaunchWorktreeData>,
    pub network: LaunchNetworkData,
    pub boundary: LaunchBoundaryData,
    pub preflight_commands: Vec<String>,
    pub command: String,
    pub safety_notes: Vec<String>,
    pub confirm_required: bool,
}

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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunHistoryListData {
    pub runs: Vec<RunHistorySummaryData>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunHistorySummaryData {
    pub run_id: String,
    pub profile: String,
    pub network: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: String,
    pub return_code: Option<i32>,
    pub workspace_scope: String,
    pub session: String,
    pub state_volume: String,
    pub provider_allowed: u64,
    pub provider_denied: u64,
    pub auth_allowed: u64,
    pub auth_denied: u64,
    pub cleanup_provider_network: String,
    pub git_summary: String,
    pub worktree_branch: Option<String>,
    pub review_command: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunHavenDiagnosticsData {
    pub auth_status: AuthStatusData,
    pub egress_log: Vec<EgressDecisionData>,
    pub auth_log: Vec<AuthDecisionData>,
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

impl AgentCatalogData {
    pub fn from_profiles(profiles: impl IntoIterator<Item = AgentProfile>) -> Self {
        Self {
            agents: profiles
                .into_iter()
                .map(|profile| AgentCatalogItemData::from_profile(&profile))
                .collect(),
        }
    }
}

impl AgentCatalogItemData {
    pub fn from_profile(profile: &AgentProfile) -> Self {
        Self {
            name: profile.name.to_string(),
            description: profile.description.to_string(),
            image: profile.image.to_string(),
            sign_in: agent_sign_in(profile.name).to_string(),
            broker: agent_broker(profile.name).to_string(),
            default_network: default_network_mode(profile).as_str().to_string(),
            provider_host_count: profile.provider_hosts.len(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchWorktreeData {
    pub source_workspace: String,
    pub source_repo_root: String,
    pub worktree_root: String,
    pub mounted_workspace: String,
    pub branch: String,
    pub base_head: Option<String>,
    pub recovery_commands: Vec<RecoveryCommandData>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryCommandData {
    pub label: String,
    pub command: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchNetworkData {
    pub mode: String,
    pub name: Option<String>,
    pub summary: String,
    pub provider_allowed_hosts: Vec<String>,
    pub api_key_broker_env: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchBoundaryData {
    pub mounted_workspace: String,
    pub mounted_state_volume: String,
    pub not_shared: Vec<String>,
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
            timestamp: string_field(&record, "timestamp"),
            state_volume: string_field(&record, "state_volume"),
            session: string_field(&record, "session"),
            container_name: string_field(&record, "container_name"),
        }
    }
}

impl RunHistoryListData {
    pub fn from_run_records(records: impl IntoIterator<Item = serde_json::Value>) -> Self {
        let mut runs = records
            .into_iter()
            .map(RunHistorySummaryData::from_run_record)
            .collect::<Vec<_>>();
        runs.sort_by(|left, right| {
            right
                .finished_at
                .cmp(&left.finished_at)
                .then_with(|| right.started_at.cmp(&left.started_at))
                .then_with(|| right.run_id.cmp(&left.run_id))
        });
        Self { runs }
    }
}

impl RunHistorySummaryData {
    pub fn from_run_record(record: serde_json::Value) -> Self {
        let run_id = string_field(&record, "run_id");
        let worktree_branch = record
            .get("worktree")
            .and_then(|worktree| worktree.get("branch"))
            .and_then(serde_json::Value::as_str)
            .map(strip_terminal_controls);
        Self {
            review_command: format!("runhaven runs show {run_id}"),
            run_id,
            profile: string_field(&record, "profile"),
            network: string_field(&record, "network"),
            status: string_field(&record, "status"),
            started_at: string_field(&record, "started_at"),
            finished_at: string_field(&record, "finished_at"),
            return_code: i32_field(&record, "return_code"),
            workspace_scope: string_field(&record, "workspace_scope"),
            session: string_field(&record, "session"),
            state_volume: string_field(&record, "state_volume"),
            provider_allowed: u64_pointer_field(&record, "/provider_policy/allowed"),
            provider_denied: u64_pointer_field(&record, "/provider_policy/denied"),
            auth_allowed: u64_pointer_field(&record, "/auth_broker/allowed"),
            auth_denied: u64_pointer_field(&record, "/auth_broker/denied"),
            cleanup_provider_network: string_pointer_field(&record, "/cleanup/provider_network"),
            git_summary: strip_terminal_controls(&crate::records::format_git_summary(
                record.get("git").unwrap_or(&serde_json::Value::Null),
            )),
            worktree_branch,
        }
    }
}

impl LaunchPlanData {
    pub fn from_plan(plan: &AgentRunPlan) -> Self {
        Self {
            profile_name: plan.profile_name.clone(),
            workspace: plan.workspace.display().to_string(),
            workspace_scope: plan.workspace_scope.as_str().to_string(),
            workspace_scope_note: plan.workspace_scope_note.clone(),
            auth_scope: plan.auth_scope.as_str().to_string(),
            session: plan.session.clone(),
            state_volume: plan.state_volume.clone(),
            container_name: plan.container_name.clone(),
            image: plan.image.clone(),
            worktree: plan.worktree.as_ref().map(|worktree| LaunchWorktreeData {
                source_workspace: worktree.source_workspace.display().to_string(),
                source_repo_root: worktree.source_repo_root.display().to_string(),
                worktree_root: worktree.worktree_root.display().to_string(),
                mounted_workspace: worktree.mounted_workspace.display().to_string(),
                branch: worktree.branch.clone(),
                base_head: worktree.base_head.clone(),
                recovery_commands: worktree
                    .recovery_commands
                    .iter()
                    .map(|(label, command)| RecoveryCommandData {
                        label: label.clone(),
                        command: command.clone(),
                    })
                    .collect(),
            }),
            network: LaunchNetworkData {
                mode: plan.network_mode.as_str().to_string(),
                name: plan.network_name.clone(),
                summary: plan.egress_summary.clone(),
                provider_allowed_hosts: plan.provider_allowed_hosts.clone(),
                api_key_broker_env: plan.api_key_broker_env.clone(),
            },
            boundary: LaunchBoundaryData {
                mounted_workspace: format!("{} -> /workspace", plan.workspace.display()),
                mounted_state_volume: format!("{} -> /home/agent", plan.state_volume),
                not_shared: vec![
                    "host home folder".to_string(),
                    "raw SSH keys".to_string(),
                    "browser profiles".to_string(),
                    "cloud credential folders".to_string(),
                    "arbitrary host environment variables".to_string(),
                ],
            },
            preflight_commands: plan.shell_preflight(),
            command: plan.shell_command(),
            safety_notes: plan.security_notices.clone(),
            confirm_required: !plan.security_notices.is_empty(),
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

impl RunHavenDiagnosticsData {
    pub fn from_payloads(
        auth_status: serde_json::Value,
        egress_log: impl IntoIterator<Item = serde_json::Value>,
        auth_log: impl IntoIterator<Item = serde_json::Value>,
    ) -> Self {
        Self {
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

fn string_field(record: &serde_json::Value, name: &str) -> String {
    strip_terminal_controls(
        record
            .get(name)
            .and_then(serde_json::Value::as_str)
            .unwrap_or("-"),
    )
}

fn u32_field(record: &serde_json::Value, name: &str) -> u32 {
    record
        .get(name)
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as u32
}

fn i32_field(record: &serde_json::Value, name: &str) -> Option<i32> {
    record
        .get(name)
        .and_then(serde_json::Value::as_i64)
        .and_then(|value| i32::try_from(value).ok())
}

fn u64_pointer_field(record: &serde_json::Value, pointer: &str) -> u64 {
    record
        .pointer(pointer)
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
}

fn string_pointer_field(record: &serde_json::Value, pointer: &str) -> String {
    strip_terminal_controls(
        record
            .pointer(pointer)
            .and_then(serde_json::Value::as_str)
            .unwrap_or("-"),
    )
}

fn count_field(record: &serde_json::Value) -> u64 {
    record
        .get("count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(1)
}

fn strip_terminal_controls(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_control()).collect()
}

impl From<&AgentRunPlan> for LaunchPlanData {
    fn from(plan: &AgentRunPlan) -> Self {
        Self::from_plan(plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::plans::{
        AuthScope, NetworkMode, RunOptions, WorkspaceScope, build_run_plan,
    };
    use crate::runtime::profiles::{get_profile, profiles};

    fn test_plan(network: NetworkMode) -> AgentRunPlan {
        let workspace = tempfile::tempdir().expect("workspace");
        build_run_plan(RunOptions {
            profile: get_profile("shell").expect("profile"),
            workspace: workspace.path().to_path_buf(),
            agent_args: vec![
                "/bin/bash".to_string(),
                "-lc".to_string(),
                "echo hello".to_string(),
            ],
            image: None,
            cpus: "4".to_string(),
            memory: "4g".to_string(),
            network,
            workspace_scope: WorkspaceScope::Current,
            session: None,
            auth_scope: AuthScope::Project,
            read_only_workspace: false,
            ssh: false,
            env: Vec::new(),
            user: "agent".to_string(),
            interactive: false,
            tty: false,
            allow_sensitive_workspace: false,
            allow_root_user: false,
            provider_hosts: Vec::new(),
            api_key_broker_env: None,
            worktree: None,
            run_id: None,
        })
        .expect("plan")
    }

    #[test]
    fn launch_plan_contract_maps_safe_plan_fields() {
        let plan = test_plan(NetworkMode::Internal);
        let data = LaunchPlanData::from(&plan);

        assert_eq!(data.profile_name, "shell");
        assert_eq!(data.workspace_scope, "current");
        assert_eq!(data.auth_scope, "project");
        assert_eq!(data.network.mode, "internal");
        assert_eq!(data.state_volume, plan.state_volume);
        assert!(data.boundary.mounted_workspace.ends_with(" -> /workspace"));
        assert!(
            data.boundary
                .not_shared
                .iter()
                .any(|item| item == "host home folder")
        );
        assert!(
            data.command.contains("container run"),
            "command should be copyable CLI text"
        );
        assert!(!data.confirm_required);
    }

    #[test]
    fn launch_plan_contract_marks_lower_security_plan_for_confirm() {
        let plan = test_plan(NetworkMode::Internet);
        let data = LaunchPlanData::from(&plan);

        assert_eq!(data.network.mode, "internet");
        assert!(data.confirm_required);
        assert!(
            data.safety_notes
                .iter()
                .any(|note| note.contains("Unrestricted internet egress"))
        );
    }

    #[test]
    fn launch_plan_contract_round_trips_json() {
        let data = LaunchPlanData::from(&test_plan(NetworkMode::Internal));

        let encoded = serde_json::to_string(&data).expect("serialize");
        let decoded: LaunchPlanData = serde_json::from_str(&encoded).expect("deserialize");

        assert_eq!(decoded, data);
    }

    #[test]
    fn agent_catalog_contract_maps_profiles() {
        let data = AgentCatalogData::from_profiles(profiles());

        let shell = data
            .agents
            .iter()
            .find(|agent| agent.name == "shell")
            .expect("shell agent");
        assert_eq!(shell.sign_in, "n/a");
        assert_eq!(shell.broker, "n/a");
        assert_eq!(shell.default_network, "internet");
        assert_eq!(shell.provider_host_count, 0);

        let codex = data
            .agents
            .iter()
            .find(|agent| agent.name == "codex")
            .expect("codex agent");
        assert_eq!(codex.sign_in, "runhaven login");
        assert_eq!(codex.default_network, "provider");
        assert!(codex.provider_host_count > 0);
    }

    #[test]
    fn component_payload_round_trips_json() {
        let payload = RunHavenComponentPayload::LaunchPlan(Box::new(LaunchPlanData::from(
            &test_plan(NetworkMode::Internal),
        )));

        let encoded = serde_json::to_string(&payload).expect("serialize");
        let decoded: RunHavenComponentPayload =
            serde_json::from_str(&encoded).expect("deserialize");

        assert_eq!(decoded, payload);
    }

    #[test]
    fn active_run_log_snapshot_contract_maps_core_payload_to_ui_shape() {
        let payload = crate::runtime::active::log_snapshot_payload_from_stdout(
            "run-123",
            20,
            b"one\ntwo\n",
            64,
        )
        .expect("active run log payload");

        let data = ActiveRunLogSnapshotData::from_active_log_snapshot_payload(payload)
            .expect("active run log snapshot data");

        assert_eq!(data.run_id, "run-123");
        assert!(!data.captured_at.is_empty());
        assert_eq!(data.requested_lines, 20);
        assert_eq!(data.text, "one\ntwo\n");
        assert_eq!(data.returned_lines, 2);
        assert!(!data.truncated);
        assert_eq!(data.source, "container-stdio");
        assert!(data.warnings.iter().any(|warning| {
            warning.contains("Raw container output") && warning.contains("workspace content")
        }));
    }

    #[test]
    fn active_run_log_snapshot_contract_serializes_camel_case() {
        let payload =
            crate::runtime::active::log_snapshot_payload_from_stdout("run-123", 50, b"line\n", 64)
                .expect("active run log payload");
        let data = ActiveRunLogSnapshotData::from_active_log_snapshot_payload(payload)
            .expect("active run log snapshot data");

        let encoded = serde_json::to_value(&data).expect("serialize snapshot data");

        assert!(encoded.get("runId").is_some());
        assert!(encoded.get("capturedAt").is_some());
        assert!(encoded.get("requestedLines").is_some());
        assert!(encoded.get("returnedLines").is_some());
        assert!(encoded.get("run_id").is_none());
        assert!(encoded.get("captured_at").is_none());
        assert!(encoded.get("requested_lines").is_none());
        assert!(encoded.get("returned_lines").is_none());
    }

    #[test]
    fn active_run_log_snapshot_component_payload_round_trips_json() {
        let payload =
            crate::runtime::active::log_snapshot_payload_from_stdout("run-123", 20, b"one\n", 64)
                .expect("active run log payload");
        let data = ActiveRunLogSnapshotData::from_active_log_snapshot_payload(payload)
            .expect("active run log snapshot data");
        let payload = RunHavenComponentPayload::ActiveRunLogSnapshot(Box::new(data));

        let encoded = serde_json::to_value(&payload).expect("serialize snapshot payload");

        assert_eq!(
            encoded.get("type").and_then(serde_json::Value::as_str),
            Some("activeRunLogSnapshot")
        );
        let data = encoded.get("data").expect("snapshot data");
        assert_eq!(
            data.get("runId").and_then(serde_json::Value::as_str),
            Some("run-123")
        );
        assert!(data.get("requestedLines").is_some());
        assert!(data.get("returnedLines").is_some());
        assert!(data.get("run_id").is_none());
        assert!(data.get("requested_lines").is_none());
        assert!(data.get("returned_lines").is_none());

        let decoded: RunHavenComponentPayload =
            serde_json::from_value(encoded).expect("deserialize snapshot payload");

        assert_eq!(decoded, payload);
    }

    #[test]
    fn active_run_list_contract_omits_workspace_paths() {
        let data = ActiveRunListData::from_active_run_records([serde_json::json!({
            "timestamp": "2026-06-29T00:00:00Z",
            "run_id": "run-123",
            "profile": "codex",
            "workspace": "/Users/c/secret/project",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-project-run",
            "state_volume": "runhaven-codex-shared-home",
            "session": "none"
        })]);

        assert_eq!(data.runs.len(), 1);
        assert_eq!(data.runs[0].run_id, "run-123");
        assert_eq!(data.runs[0].profile, "codex");
        let serialized = serde_json::to_string(&data).expect("serialize");
        assert!(
            !serialized.contains("/Users/c/secret/project"),
            "TUI active-run summaries must not include workspace paths"
        );
    }

    #[test]
    fn diagnostics_contract_maps_secret_free_fields_only() {
        let data = RunHavenDiagnosticsData::from_payloads(
            serde_json::json!({
                "status": "available",
                "runtime": "host-broker",
                "credential_stores_inspected": false,
                "environment_values_inspected": false,
                "secrets_printed": false,
                "profiles": [{"name": "codex", "status": "brokered"}],
                "token": "should-not-leak"
            }),
            [serde_json::json!({
                "timestamp": "2026-06-29T00:00:00Z",
                "profile": "codex",
                "decision": "denied",
                "host": "example.com",
                "port": 443,
                "count": 2,
                "reason": "not-in-allowlist",
                "matched_rule": "",
                "run_id": "run-123",
                "workspace": "/Users/c/secret/project"
            })],
            [serde_json::json!({
                "timestamp": "2026-06-29T00:00:00Z",
                "profile": "codex",
                "broker": "api-key",
                "decision": "allowed",
                "method": "POST",
                "path": "/v1/responses?token=secret#fragment",
                "upstream_status": 200,
                "count": 1,
                "reason": "-",
                "run_id": "run-123",
                "authorization": "Bearer secret"
            })],
        );

        assert_eq!(data.auth_status.status, "available");
        assert_eq!(data.auth_status.profiles[0].name, "codex");
        assert_eq!(data.egress_log[0].host, "example.com");
        assert_eq!(data.auth_log[0].path, "/v1/responses");
        assert_eq!(data.auth_log[0].upstream_status, Some(200));

        let serialized = serde_json::to_string(&data).expect("serialize");
        for forbidden in [
            "/Users/c/secret/project",
            "Bearer secret",
            "token=secret",
            "should-not-leak",
            "credential_stores_inspected",
            "environment_values_inspected",
        ] {
            assert!(
                !serialized.contains(forbidden),
                "diagnostics payload leaked forbidden field/value {forbidden:?}"
            );
        }
    }

    #[test]
    fn run_history_contract_omits_workspace_path_and_sanitizes_display_fields() {
        let data = RunHistoryListData::from_run_records([
            serde_json::json!({
                "timestamp": "2026-06-30T01:00:00Z",
                "started_at": "2026-06-30T00:00:00Z",
                "finished_at": "2026-06-30T01:00:00Z",
                "run_id": "run-123",
                "profile": "codex",
                "workspace": "/Users/c/secret/project",
                "workspace_scope": "current",
                "state_volume": "runhaven-codex-shared-home",
                "session": "none",
                "network": "provider",
                "status": "succeeded",
                "return_code": 0,
                "provider_policy": {"allowed": 3, "denied": 1},
                "auth_broker": {"allowed": 2, "denied": 0},
                "cleanup": {"provider_network": "removed"},
                "git": {"available": "false", "reason": "not-a-git-worktree"},
                "worktree": {"branch": "runhaven/codex/run-\u{1b}123"}
            }),
            serde_json::json!({
                "timestamp": "2026-06-30T02:00:00Z",
                "started_at": "2026-06-30T01:30:00Z",
                "finished_at": "2026-06-30T02:00:00Z",
                "run_id": "run-\u{1b}456",
                "profile": "codex",
                "workspace": "/Users/c/secret/project",
                "workspace_scope": "current",
                "state_volume": "runhaven-codex-shared-home",
                "session": "none",
                "network": "provider",
                "status": "failed",
                "return_code": 1,
                "provider_policy": {"allowed": 0, "denied": 0},
                "auth_broker": {"allowed": 0, "denied": 0},
                "cleanup": {"provider_network": "removed"},
                "git": {"available": "false", "reason": "not-a-git-worktree"}
            }),
        ]);

        assert_eq!(data.runs.len(), 2);
        let run = &data.runs[0];
        assert_eq!(run.run_id, "run-456");
        assert_eq!(run.review_command, "runhaven runs show run-456");
        assert_eq!(run.return_code, Some(1));

        let run = &data.runs[1];
        assert_eq!(run.run_id, "run-123");
        assert_eq!(
            run.worktree_branch.as_deref(),
            Some("runhaven/codex/run-123")
        );
        assert_eq!(run.provider_denied, 1);
        assert!(run.git_summary.contains("not-a-git-worktree"));

        let encoded = serde_json::to_string(&data).expect("serialize history data");
        assert!(!encoded.contains("/Users/c/secret/project"));
        assert!(!encoded.contains("\\u001b"));
    }

    #[test]
    fn component_payload_fixtures_decode() {
        let catalog = include_str!("../tests/fixtures/ui/agent-catalog.json");
        let launch_plan = include_str!("../tests/fixtures/ui/launch-plan-provider.json");

        assert!(matches!(
            serde_json::from_str::<RunHavenComponentPayload>(catalog).expect("agent catalog"),
            RunHavenComponentPayload::AgentCatalog(_)
        ));
        assert!(matches!(
            serde_json::from_str::<RunHavenComponentPayload>(launch_plan).expect("launch plan"),
            RunHavenComponentPayload::LaunchPlan(_)
        ));
    }
}
