use std::io::Write;

use super::*;
use crate::runtime::plans::{AuthScope, NetworkMode, RunOptions, WorkspaceScope, build_run_plan};
use crate::runtime::profiles::get_profile;
use crate::support::git::GitChange;
use crate::support::paths::{ensure_private_parent, override_cache_root_for_tests};

fn record_status_for_run_with_override(
    interactive_tty: bool,
    return_code: i32,
    status: Option<&str>,
) -> String {
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan = build_run_plan(RunOptions {
        profile: get_profile("shell").expect("profile"),
        workspace: workspace.path().to_path_buf(),
        agent_args: vec![
            "/bin/bash".to_string(),
            "-lc".to_string(),
            "true".to_string(),
        ],
        image: None,
        cpus: "4".to_string(),
        memory: "4g".to_string(),
        network: NetworkMode::Internet,
        workspace_scope: WorkspaceScope::Current,
        session: None,
        auth_scope: AuthScope::Agent,
        read_only_workspace: true,
        ssh: false,
        env: Vec::new(),
        user: "agent".to_string(),
        interactive: true,
        tty: false,
        allow_sensitive_workspace: false,
        allow_root_user: false,
        provider_hosts: Vec::new(),
        claude_auth_source: None,
        worktree: None,
        run_id: Some("status-probe".to_string()),
    })
    .expect("plan");
    if interactive_tty {
        plan.make_interactive_tty();
    }

    write_run_record(RunRecordInput {
        plan: &plan,
        run_id: "status-probe",
        started_at: "2026-07-01T00:00:00Z",
        finished_at: "2026-07-01T00:01:00Z",
        return_code,
        status,
        provider_decisions: &[],
        auth_decisions: None,
        cleanup: json!({}),
        git: GitChange::Unavailable {
            reason: "test".to_string(),
            repo_root: None,
        },
    })
    .expect("write run record");

    find_run_record("status-probe")
        .expect("record")
        .get("status")
        .and_then(Value::as_str)
        .expect("status")
        .to_string()
}

fn record_status_for_run(interactive_tty: bool, return_code: i32) -> String {
    record_status_for_run_with_override(interactive_tty, return_code, None)
}

#[test]
fn interactive_tty_natural_nonzero_exit_records_failure() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    ensure_private_parent(&runs_log_path()).expect("runs log parent");
    assert_eq!(record_status_for_run(true, 1), "failed");
}

#[test]
fn interactive_tty_explicit_close_records_ended() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    ensure_private_parent(&runs_log_path()).expect("runs log parent");
    assert_eq!(
        record_status_for_run_with_override(true, 1, Some("ended")),
        "ended"
    );
}

#[test]
fn background_run_keeps_the_exit_code_status_split() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    ensure_private_parent(&runs_log_path()).expect("runs log parent");
    // A background (non-interactive) run still reports failure on non-zero
    // exit and success on zero.
    assert_eq!(record_status_for_run(false, 1), "failed");
    assert_eq!(record_status_for_run(false, 0), "succeeded");
}

#[test]
fn run_records_tail_bounded_keeps_recent_records() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    ensure_private_parent(&runs_log_path()).expect("runs log parent");
    let mut file = std::fs::File::create(runs_log_path()).expect("runs log file");
    for idx in 0..20 {
        writeln!(
            file,
            "{}",
            json!({
                "run_id": format!("run-{idx:03}"),
                "timestamp": format!("2026-06-30T00:{idx:02}:00Z")
            })
        )
        .expect("write run record");
    }

    let records = read_run_records_tail_bounded(3, 2048).expect("tail records");

    let run_ids = records
        .iter()
        .map(|record| {
            record
                .get("run_id")
                .and_then(Value::as_str)
                .expect("run id")
                .to_string()
        })
        .collect::<Vec<_>>();
    assert_eq!(run_ids, vec!["run-017", "run-018", "run-019"]);
}
