use super::*;
#[allow(unused_imports)]
use crate::contracts::{
    CleanupManagedSessionRequest, KillManagedSessionRequest, KillRunResponse, LaunchStatusRequest,
    LogSnapshotRequest, ManagedSessionDiagnosticsRequest, ManagedSessionLogSnapshotRequest,
    ManagedSessionStatusRequest, RepairManagedSessionRequest, RepairRunResponse,
    StopManagedSessionRequest, StopRunResponse, WriteSessionInputRequest,
};
use crate::launch_status::LaunchStatusRegistry;
use crate::session_input::SessionInputRegistry;
use crate::session_output::SessionOutputRegistry;
use runhaven_core::runtime::active::write_active_run_payload;
use runhaven_core::support::paths::{
    CacheRootOverride, ensure_private_parent, override_cache_root_for_tests, runs_log_path,
};
use serde_json::json;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

mod agent_sign_in;
mod auth;
mod image;
mod image_build;
mod launch_status;
mod live;
mod live_agent;
mod live_support;
mod log_snapshot;
mod network;
mod plan;
mod run_control;
mod run_status;
mod session_cleanup;
mod session_input;
mod session_worktree;
mod sessions;
mod sessions_status;
mod validation;

fn isolated_cache() -> (tempfile::TempDir, CacheRootOverride) {
    let cache = tempfile::tempdir().expect("cache");
    let cache_home = override_cache_root_for_tests(cache.path());
    (cache, cache_home)
}

fn request(workspace: PathBuf) -> RunPlanRequest {
    RunPlanRequest {
        agent: "codex".to_string(),
        workspace_path: workspace.display().to_string(),
        network_mode: "provider".to_string(),
        workspace_scope: "current".to_string(),
        auth_scope: "agent".to_string(),
        session_name: None,
        read_only_workspace: false,
        cpus: "4".to_string(),
        memory: "4g".to_string(),
        provider_hosts: Vec::new(),
        env_names: Vec::new(),
        api_key_broker_env: None,
        image: None,
        allow_sensitive_workspace: false,
        allow_root_user: false,
        user: "agent".to_string(),
        worktree: false,
    }
}

fn shell_request(workspace: PathBuf, session: String) -> RunPlanRequest {
    RunPlanRequest {
        agent: "shell".to_string(),
        workspace_path: workspace.display().to_string(),
        network_mode: "internal".to_string(),
        workspace_scope: "current".to_string(),
        auth_scope: "project".to_string(),
        session_name: Some(session),
        read_only_workspace: true,
        cpus: "1".to_string(),
        memory: "512m".to_string(),
        provider_hosts: Vec::new(),
        env_names: Vec::new(),
        api_key_broker_env: None,
        image: None,
        allow_sensitive_workspace: false,
        allow_root_user: false,
        user: "agent".to_string(),
        worktree: false,
    }
}

/// Append a finished run record to the history log so managed-session lookups
/// resolve the run. Shared by the cleanup and worktree review suites.
fn write_history_run_record(record: serde_json::Value) {
    ensure_private_parent(&runs_log_path()).expect("runs log parent");
    let mut file = if runs_log_path().exists() {
        fs::OpenOptions::new()
            .append(true)
            .open(runs_log_path())
            .expect("append runs log")
    } else {
        File::create(runs_log_path()).expect("create runs log")
    };
    writeln!(file, "{record}").expect("write run record");
}

/// Hermetic git repo with a committed `project/` subdirectory, matching the
/// worktree source layout the core create/verify path expects.
fn init_git_repo(path: &Path) {
    let project = path.join("project");
    fs::create_dir_all(&project).expect("project dir");
    git(path, &["init"]);
    git(path, &["config", "user.name", "RunHaven Test"]);
    git(path, &["config", "user.email", "runhaven-test@example.com"]);
    fs::write(project.join("README.md"), "hello\n").expect("write README");
    git(path, &["add", "."]);
    git(path, &["commit", "-m", "initial"]);
}

fn git_branch_exists(repo_root: &Path, branch: &str) -> bool {
    Command::new("git")
        .args(["-C"])
        .arg(repo_root)
        .args([
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{branch}"),
        ])
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn git(repo_root: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(["-C"])
        .arg(repo_root)
        .args(args)
        .status()
        .expect("run git");
    assert!(status.success(), "git {args:?} failed");
}

fn write_active_run(run_id: &str) {
    write_active_run_payload(
        run_id,
        json!({
            "timestamp": "2026-06-16T00:00:00Z",
            "run_id": run_id,
            "profile": "codex",
            "workspace": "/tmp/runhaven-active",
            "network": "provider",
            "status": "running",
            "container_name": format!("runhaven-codex-{run_id}"),
            "state_volume": "runhaven-codex-active-home",
            "session": "default"
        }),
    )
    .expect("active run payload");
}
