use std::process::Command;

use anyhow::{Result, bail};
use serde_json::json;

use super::lock::acquire_state_lock;
use crate::doctor::find_on_path;
use crate::provider::observability::utc_timestamp;
use crate::provider::runtime as provider_runtime;
use crate::records::{RunRecordInput, write_run_record};
use crate::runtime::active::{
    active_run_terminal_status, remove_active_run_record, write_active_run_record,
};
use crate::runtime::plans::{AgentRunPlan, NetworkMode};
use crate::support::git::{GitChange, capture_git_snapshot, summarize_git_change};

pub struct AgentCommandContext<'a> {
    pub plan: &'a AgentRunPlan,
    pub run_id: &'a str,
    pub started_at: &'a str,
}

pub trait AgentCommandRunner {
    fn run_agent_command(
        &mut self,
        command: &[String],
        env: Option<(&str, &str)>,
        context: AgentCommandContext<'_>,
    ) -> Result<i32>;
}

pub(crate) struct AgentCommandExecution {
    pub(crate) started_at: String,
    pub(crate) finished_at: String,
    pub(crate) return_code: i32,
    pub(crate) terminal_status: Option<String>,
    pub(crate) git: GitChange,
    pub(crate) command_error: Option<anyhow::Error>,
}

pub(crate) struct StatusAgentCommandRunner;

impl AgentCommandRunner for StatusAgentCommandRunner {
    fn run_agent_command(
        &mut self,
        command: &[String],
        env: Option<(&str, &str)>,
        _context: AgentCommandContext<'_>,
    ) -> Result<i32> {
        let mut agent_command = Command::new(&command[0]);
        agent_command.args(&command[1..]);
        if let Some((name, value)) = env {
            agent_command.env(name, value);
        }
        let status = agent_command.status().map_err(|error| {
            anyhow::anyhow!("could not launch agent command {:?}: {error}", command[0])
        })?;
        Ok(status.code().unwrap_or(1))
    }
}

pub fn new_run_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

pub fn launch_run_plan(plan: &AgentRunPlan) -> Result<i32> {
    let mut runner = StatusAgentCommandRunner;
    launch_run_plan_with_runner(plan, &mut runner)
}

pub fn launch_run_plan_with_runner(
    plan: &AgentRunPlan,
    runner: &mut impl AgentCommandRunner,
) -> Result<i32> {
    provider_runtime::validate_runtime_auth_broker_environment(plan)?;
    require_container_cli()?;
    ensure_agent_image_built(plan)?;
    let _state_lock = acquire_state_lock(&plan.state_volume)?;
    if plan.network_mode == NetworkMode::Provider {
        return provider_runtime::run_provider_agent_with_runner(plan, runner);
    }
    for command in &plan.preflight {
        provider_runtime::run_preflight(command)?;
    }
    run_standard_agent_with_runner(plan, runner)
}

/// Fail early with an actionable message when the agent image is not built,
/// instead of letting `container run` try to pull a RunHaven image and return a
/// confusing `registry-1.docker.io 401`.
fn ensure_agent_image_built(plan: &AgentRunPlan) -> Result<()> {
    if crate::image::doctor::image_is_built(&plan.image)? {
        return Ok(());
    }
    bail!(
        "{}",
        image_not_built_message(&plan.image, &plan.profile_name)
    );
}

fn image_not_built_message(image: &str, agent: &str) -> String {
    if image.starts_with("runhaven/") {
        format!(
            "The {agent} image is not built yet, so the sandbox cannot start. Build it once with:\n  runhaven image build {agent}"
        )
    } else {
        format!("The image {image} is not available locally. Build or load it before running.")
    }
}

pub fn run_standard_agent(plan: &AgentRunPlan) -> Result<i32> {
    let mut runner = StatusAgentCommandRunner;
    run_standard_agent_with_runner(plan, &mut runner)
}

pub fn run_standard_agent_with_runner(
    plan: &AgentRunPlan,
    runner: &mut impl AgentCommandRunner,
) -> Result<i32> {
    let run_id = plan.run_id.clone().unwrap_or_else(new_run_id);
    let execution = run_agent_command_lifecycle(plan, &run_id, plan.command.clone(), runner)?;
    let AgentCommandExecution {
        started_at,
        finished_at,
        return_code,
        terminal_status,
        git,
        command_error,
    } = execution;
    write_run_record(RunRecordInput {
        plan,
        run_id: &run_id,
        started_at: &started_at,
        finished_at: &finished_at,
        return_code,
        status: terminal_status.as_deref(),
        provider_decisions: &[],
        auth_decisions: None,
        cleanup: json!({"provider_network": "not-applicable"}),
        git,
    })?;
    if let Some(error) = command_error {
        return Err(error);
    }
    Ok(return_code)
}

pub(crate) fn run_agent_command_lifecycle(
    plan: &AgentRunPlan,
    run_id: &str,
    command: Vec<String>,
    runner: &mut impl AgentCommandRunner,
) -> Result<AgentCommandExecution> {
    let git_before = capture_git_snapshot(&plan.workspace);
    let started_at = utc_timestamp();
    eprintln!("Run id: {run_id}");
    write_active_run_record(plan, run_id, &started_at)?;
    let injection = crate::runtime::login::run_token_injection(plan);
    let command = match &injection {
        Some((env, _)) => crate::runtime::login::with_token_env(&command, &plan.image, env),
        None => command,
    };
    if injection.is_some() {
        eprintln!(
            "Using your stored {} login (injected into the sandbox env).",
            plan.profile_name
        );
    }
    let status_result = runner.run_agent_command(
        &command,
        injection
            .as_ref()
            .map(|(env, value)| (env.as_str(), value.as_str())),
        AgentCommandContext {
            plan,
            run_id,
            started_at: &started_at,
        },
    );
    let terminal_status = active_run_terminal_status(run_id);
    let _ = remove_active_run_record(run_id);
    let finished_at = utc_timestamp();
    let mut command_error = None;
    let return_code = match status_result {
        Ok(code) => code,
        Err(error) => {
            command_error = Some(error);
            1
        }
    };
    let git = summarize_git_change(git_before, capture_git_snapshot(&plan.workspace));
    Ok(AgentCommandExecution {
        started_at,
        finished_at,
        return_code,
        terminal_status,
        git,
        command_error,
    })
}

pub fn require_container_cli() -> Result<()> {
    if find_on_path("container").is_some() {
        return Ok(());
    }
    bail!(
        "Apple container CLI was not found. Install it from https://github.com/apple/container/releases and run `container system start`."
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::plans::{AuthScope, WorkspaceScope};
    use crate::support::paths::override_cache_root_for_tests;
    use std::path::{Path, PathBuf};

    struct RecordingRunner {
        expected_run_id: String,
        command: Option<Vec<String>>,
        saw_active_marker: bool,
        active_marker_auth_scope: Option<String>,
    }

    struct FailingRunner {
        expected_run_id: String,
        saw_active_marker: bool,
    }

    impl AgentCommandRunner for RecordingRunner {
        fn run_agent_command(
            &mut self,
            command: &[String],
            env: Option<(&str, &str)>,
            context: AgentCommandContext<'_>,
        ) -> Result<i32> {
            assert_eq!(context.run_id, self.expected_run_id);
            assert_eq!(
                context.plan.run_id.as_deref(),
                Some(self.expected_run_id.as_str())
            );
            assert!(env.is_none());
            self.command = Some(command.to_vec());
            if let Ok(record) = crate::runtime::active::find_active_run_record(context.run_id) {
                self.saw_active_marker = true;
                self.active_marker_auth_scope = record
                    .get("auth_scope")
                    .and_then(serde_json::Value::as_str)
                    .map(str::to_string);
            }
            Ok(0)
        }
    }

    impl AgentCommandRunner for FailingRunner {
        fn run_agent_command(
            &mut self,
            _command: &[String],
            env: Option<(&str, &str)>,
            context: AgentCommandContext<'_>,
        ) -> Result<i32> {
            assert_eq!(context.run_id, self.expected_run_id);
            assert!(env.is_none());
            self.saw_active_marker =
                crate::runtime::active::find_active_run_record(context.run_id).is_ok();
            Err(anyhow::anyhow!("backend runner failed"))
        }
    }

    fn test_plan(workspace: &Path) -> AgentRunPlan {
        AgentRunPlan {
            command: vec![
                "container".to_string(),
                "run".to_string(),
                "image".to_string(),
            ],
            preflight: Vec::new(),
            workspace: PathBuf::from(workspace),
            state_volume: "runhaven-shell-test-home".to_string(),
            session: "default".to_string(),
            container_name: "runhaven-shell-test-runner-seam".to_string(),
            profile_name: "shell".to_string(),
            workspace_scope: WorkspaceScope::Current,
            workspace_scope_note: None,
            auth_scope: AuthScope::Agent,
            worktree: None,
            run_id: Some("runner-seam".to_string()),
            network_name: None,
            network_mode: NetworkMode::Internet,
            egress_summary: "unrestricted internet egress".to_string(),
            image: "image".to_string(),
            provider_allowed_hosts: Vec::new(),
            api_key_broker_env: None,
            security_notices: Vec::new(),
        }
    }

    #[test]
    fn image_not_built_message_points_to_the_build_command_for_bundled_images() {
        let bundled = image_not_built_message("runhaven/codex:0.1.0", "codex");
        assert!(bundled.contains("runhaven image build codex"));
        // A custom --image is the user's to provide, so no build hint.
        let custom = image_not_built_message("my/custom:1.0", "codex");
        assert!(!custom.contains("runhaven image build"));
        assert!(custom.contains("my/custom:1.0"));
    }

    #[test]
    fn standard_launch_can_use_backend_owned_command_runner() {
        let cache = tempfile::tempdir().expect("cache");
        let _cache_home = override_cache_root_for_tests(cache.path());
        let workspace = tempfile::tempdir().expect("workspace");
        let plan = test_plan(workspace.path());
        let mut runner = RecordingRunner {
            expected_run_id: "runner-seam".to_string(),
            command: None,
            saw_active_marker: false,
            active_marker_auth_scope: None,
        };

        let code = run_standard_agent_with_runner(&plan, &mut runner).expect("standard launch");

        assert_eq!(code, 0);
        assert_eq!(runner.command.as_deref(), Some(plan.command.as_slice()));
        assert!(runner.saw_active_marker);
        assert_eq!(runner.active_marker_auth_scope.as_deref(), Some("agent"));
        assert!(crate::runtime::active::find_active_run_record("runner-seam").is_err());
    }

    #[test]
    fn standard_launch_records_and_cleans_up_failed_backend_runner() {
        let cache = tempfile::tempdir().expect("cache");
        let _cache_home = override_cache_root_for_tests(cache.path());
        let workspace = tempfile::tempdir().expect("workspace");
        let mut plan = test_plan(workspace.path());
        plan.run_id = Some("runner-error-seam".to_string());
        let mut runner = FailingRunner {
            expected_run_id: "runner-error-seam".to_string(),
            saw_active_marker: false,
        };

        let error = run_standard_agent_with_runner(&plan, &mut runner).expect_err("runner failure");

        assert!(error.to_string().contains("backend runner failed"));
        assert!(runner.saw_active_marker);
        assert!(crate::runtime::active::find_active_run_record("runner-error-seam").is_err());
        let record = crate::records::find_run_record("runner-error-seam").expect("run record");
        assert_eq!(
            record
                .pointer("/return_code")
                .and_then(serde_json::Value::as_i64),
            Some(1)
        );
        assert_eq!(
            record
                .pointer("/status")
                .and_then(serde_json::Value::as_str),
            Some("failed")
        );
    }
}
