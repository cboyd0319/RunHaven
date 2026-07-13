use super::*;
use crate::runtime::plans::{AuthScope, WorkspaceScope, WorktreeRun};
use crate::support::paths::override_cache_root_for_tests;
use std::path::{Path, PathBuf};

mod network_cleanup;
#[cfg(target_os = "macos")]
mod signals;

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

struct MarkerRemovalFailingRunner;

#[derive(Default)]
struct CountingRunner {
    calls: usize,
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

impl AgentCommandRunner for CountingRunner {
    fn run_agent_command(
        &mut self,
        _command: &[String],
        _env: Option<(&str, &str)>,
        _context: AgentCommandContext<'_>,
    ) -> Result<i32> {
        self.calls += 1;
        Ok(0)
    }
}

impl AgentCommandRunner for MarkerRemovalFailingRunner {
    fn run_agent_command(
        &mut self,
        _command: &[String],
        _env: Option<(&str, &str)>,
        context: AgentCommandContext<'_>,
    ) -> Result<i32> {
        let path = crate::support::paths::active_run_path(context.run_id)?;
        std::fs::remove_file(&path)?;
        std::fs::create_dir(&path)?;
        Ok(0)
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
        claude_auth_source: None,
        security_notices: Vec::new(),
    }
}

#[test]
fn image_not_built_message_points_to_the_build_command_for_bundled_images() {
    let bundled = image_not_built_message("runhaven/codex:0.7.0", "codex");
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
fn successful_launch_completion_runs_before_the_state_lock_is_released() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    let workspace = tempfile::tempdir().expect("workspace");
    let plan = test_plan(workspace.path());
    let volume = plan.state_volume.clone();
    let callback_volume = volume.clone();

    let code = launch_under_state_lock_with_attachment_check(
        &plan,
        |_| Ok(()),
        || Ok(0),
        move || {
            assert!(
                crate::runtime::lock::acquire_state_lock(&callback_volume).is_err(),
                "completion publication must remain inside the launch lock"
            );
            Ok(())
        },
    )
    .expect("locked launch completion");

    assert_eq!(code, 0);
    crate::runtime::lock::acquire_state_lock(&volume)
        .expect("launch releases the state lock after completion publication");
}

#[test]
fn launch_refuses_a_running_container_using_the_same_state_volume() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    let workspace = tempfile::tempdir().expect("workspace");
    let plan = test_plan(workspace.path());
    let container_list = format!(
        r#"[{{
            "configuration": {{
                "id": "foreign-container",
                "mounts": [{{
                    "destination": "/home/agent",
                    "type": {{"volume": {{"name": "{}"}}}}
                }}]
            }},
            "status": {{"state": "running"}}
        }}]"#,
        plan.state_volume
    );
    let launched = std::cell::Cell::new(false);

    let error = launch_under_state_lock_with_attachment_check(
        &plan,
        |state_volume| {
            ensure_state_volume_detached_from_container_list(
                state_volume,
                true,
                container_list.as_bytes(),
            )
        },
        || {
            launched.set(true);
            Ok(0)
        },
        || Ok(()),
    )
    .expect_err("an active shared volume must fail before Apple container starts");

    assert!(error.to_string().contains("saved state is already in use"));
    assert!(
        !error.to_string().contains(&plan.state_volume),
        "the actionable error must not expose the state-volume name"
    );
    assert!(
        !launched.get(),
        "the container launch must not be attempted"
    );
}

#[test]
fn state_volume_attachment_check_fails_closed_without_exposing_container_output() {
    let error = ensure_state_volume_detached_from_container_list(
        "runhaven-private-volume-name",
        true,
        br#"{"configuration":{"environment":["SECRET=value"]}}"#,
    )
    .expect_err("an unexpected list shape must fail closed");

    assert!(error.to_string().contains("could not verify"));
    assert!(!error.to_string().contains("SECRET"));
    assert!(!error.to_string().contains("private-volume"));
}

#[test]
fn standard_launch_refuses_a_plan_without_a_reserved_run_id() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan = test_plan(workspace.path());
    plan.run_id = None;
    let mut runner = CountingRunner::default();

    let error = run_standard_agent_with_runner(&plan, &mut runner)
        .expect_err("a launch without its resource-owning run id must fail closed");

    assert!(error.to_string().contains("run id"));
    assert_eq!(runner.calls, 0, "the container command must not start");
}

#[test]
fn supported_runtime_failure_blocks_launch_before_the_runner_starts() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    let workspace = tempfile::tempdir().expect("workspace");
    let plan = test_plan(workspace.path());
    let mut runner = CountingRunner::default();
    let checks = std::cell::Cell::new(0);

    let error = launch_run_plan_with_runtime_check_and_runner(&plan, &mut runner, || {
        checks.set(checks.get() + 1);
        Err(anyhow::anyhow!("supported runtime is unavailable"))
    })
    .expect_err("unsupported host must fail before launch");

    assert!(error.to_string().contains("supported runtime"));
    assert_eq!(checks.get(), 1, "the runtime check runs exactly once");
    assert_eq!(runner.calls, 0, "the container command must not start");
}

#[test]
fn worktree_runtime_failure_persists_a_recoverable_failed_run() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan = test_plan(workspace.path());
    plan.run_id = Some("worktree-runtime-failure".to_string());
    plan.worktree = Some(WorktreeRun {
        source_workspace: workspace.path().to_path_buf(),
        source_repo_root: workspace.path().to_path_buf(),
        worktree_root: workspace.path().join("worktree-runtime-failure"),
        mounted_workspace: workspace.path().to_path_buf(),
        branch: "runhaven/shell/worktree-runtime-failure".to_string(),
        base_head: Some("base-head".to_string()),
        recovery_commands: Vec::new(),
    });
    let mut runner = CountingRunner::default();

    launch_run_plan_with_runtime_check_and_runner(&plan, &mut runner, || {
        Err(anyhow::anyhow!("supported runtime is unavailable"))
    })
    .expect_err("runtime failure must stop launch");

    assert_eq!(runner.calls, 0, "the agent must not execute");
    let record = crate::records::find_run_record("worktree-runtime-failure")
        .expect("recoverable failed run record");
    assert_eq!(record["status"], "failed");
    assert_eq!(record["cleanup"]["worktree"], "kept-for-recovery");
    assert_eq!(
        record["worktree"]["branch"],
        "runhaven/shell/worktree-runtime-failure"
    );
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

#[test]
fn standard_launch_propagates_active_marker_removal_failure() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan = test_plan(workspace.path());
    plan.run_id = Some("marker-removal-error".to_string());
    let mut runner = MarkerRemovalFailingRunner;

    let error = run_standard_agent_with_runner(&plan, &mut runner)
        .expect_err("marker removal failure must fail the lifecycle");

    assert!(
        error
            .to_string()
            .contains("could not remove active run marker")
    );
    let record = crate::records::find_run_record("marker-removal-error").expect("run record");
    assert_eq!(record["return_code"], 1);
    assert_eq!(record["status"], "failed");
}
