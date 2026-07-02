use super::*;
use std::fs;
use std::path::PathBuf;

use crate::args::RunArgs;
use runhaven_core::runtime::launch::run_standard_agent;
use runhaven_core::runtime::plans::{
    AuthScope, ClaudeAuthSource, NetworkMode, RunOptions, WorkspaceScope, build_run_plan,
};
use runhaven_core::runtime::profiles::get_profile;
use runhaven_core::support::paths::{
    active_run_path, override_cache_root_for_tests, runs_log_path,
};

/// Minimal `RunArgs` for a plan-build test. Non-interactive with no TTY so plan
/// construction is deterministic off a terminal.
fn run_args(agent: &str, workspace: PathBuf) -> RunArgs {
    RunArgs {
        agent: agent.to_string(),
        workspace,
        workspace_scope: "current".to_string(),
        session: None,
        auth_scope: "agent".to_string(),
        image: None,
        cpus: "4".to_string(),
        memory: "4g".to_string(),
        tty: "never".to_string(),
        no_interactive: true,
        network: None,
        provider_host: Vec::new(),
        api_key_broker_env: None,
        read_only_workspace: false,
        ssh: false,
        env: Vec::new(),
        user: "agent".to_string(),
        allow_sensitive_workspace: false,
        allow_root_user: false,
    }
}

#[test]
fn cli_run_plan_for_claude_with_saved_token_selects_keychain_oauth() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    let workspace = tempfile::tempdir().expect("workspace");
    let args = run_args("claude", workspace.path().to_path_buf());

    // Injected presence stands in for the non-prompting Keychain check; no real
    // Keychain is touched. A present token drives the claude-only OAuth path.
    let plan =
        super::run_plan::make_run_plan_with_presence(&args, Vec::new(), None, None, None, || {
            Ok(true)
        })
        .expect("claude run plan");

    assert_eq!(
        plan.claude_auth_source,
        Some(ClaudeAuthSource::KeychainOAuth)
    );
    assert_eq!(plan.network_mode, NetworkMode::Provider);
}

#[test]
fn cli_run_plan_for_claude_without_token_selects_no_broker() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    let workspace = tempfile::tempdir().expect("workspace");
    let args = run_args("claude", workspace.path().to_path_buf());

    let plan =
        super::run_plan::make_run_plan_with_presence(&args, Vec::new(), None, None, None, || {
            Ok(false)
        })
        .expect("claude run plan");

    assert_eq!(plan.claude_auth_source, None);
}

#[test]
fn cli_run_plan_explicit_env_broker_beats_a_saved_token() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    let workspace = tempfile::tempdir().expect("workspace");
    let mut args = run_args("claude", workspace.path().to_path_buf());
    args.api_key_broker_env = Some("ANTHROPIC_API_KEY".to_string());

    // Even with a present token, the explicit env-var broker wins.
    let plan =
        super::run_plan::make_run_plan_with_presence(&args, Vec::new(), None, None, None, || {
            Ok(true)
        })
        .expect("claude run plan");

    assert_eq!(
        plan.claude_auth_source,
        Some(ClaudeAuthSource::ApiKeyBrokerEnv(
            "ANTHROPIC_API_KEY".to_string()
        ))
    );
}

#[test]
fn split_agent_args_after_separator() {
    let args = ["run", "shell", "--", "echo", "--flag"]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    assert_eq!(
        split_agent_args(&args),
        (
            vec!["run".to_string(), "shell".to_string()],
            vec!["echo".to_string(), "--flag".to_string()]
        )
    );
}

#[test]
fn standard_run_launch_error_removes_active_marker_and_writes_record() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());

    let workspace = tempfile::tempdir().expect("workspace");
    let run_id = "standard-launch-error";
    let mut plan = build_run_plan(RunOptions {
        profile: get_profile("shell").expect("profile"),
        workspace: workspace.path().to_path_buf(),
        agent_args: vec!["/bin/true".to_string()],
        image: None,
        cpus: "4".to_string(),
        memory: "4g".to_string(),
        network: NetworkMode::Internet,
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
        claude_auth_source: None,
        worktree: None,
        run_id: Some(run_id.to_string()),
    })
    .expect("plan");
    plan.command[0] = "__runhaven_missing_container_binary__".to_string();

    let error = run_standard_agent(&plan).expect_err("launch should fail");
    assert!(
        error
            .to_string()
            .contains("__runhaven_missing_container_binary__")
    );
    assert!(!active_run_path(run_id).expect("active path").exists());

    let log = fs::read_to_string(runs_log_path()).expect("run log");
    let record: serde_json::Value =
        serde_json::from_str(log.lines().next().expect("one record")).expect("json record");
    assert_eq!(record["run_id"], run_id);
    assert_eq!(record["status"], "failed");
    assert_eq!(record["return_code"], 1);
}
