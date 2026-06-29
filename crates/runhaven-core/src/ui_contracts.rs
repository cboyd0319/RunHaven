use serde::{Deserialize, Serialize};

use crate::provider::auth_profiles::{agent_broker, agent_sign_in};
use crate::runtime::plans::AgentRunPlan;
use crate::runtime::plans::default_network_mode;
use crate::runtime::profiles::AgentProfile;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum RunHavenComponentPayload {
    AgentCatalog(AgentCatalogData),
    LaunchPlan(Box<LaunchPlanData>),
    ActiveRunLogSnapshot(Box<ActiveRunLogSnapshotData>),
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
