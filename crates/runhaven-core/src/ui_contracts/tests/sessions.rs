use super::*;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use crate::runtime::worktrees::{
    create_worktree_for_run, discard_worktree_for_run, worktree_record,
};
use crate::support::paths::{open_private_append, override_cache_root_for_tests, runs_log_path};

fn git(repo: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .status()
        .expect("git available");
    assert!(status.success(), "git {args:?} failed");
}

fn init_clean_repo(repo: &Path) {
    git(repo, &["init", "-q"]);
    git(repo, &["config", "user.email", "test@example.com"]);
    git(repo, &["config", "user.name", "test"]);
    std::fs::write(repo.join("seed.txt"), "base").expect("write seed");
    git(repo, &["add", "seed.txt"]);
    git(repo, &["commit", "-qm", "seed"]);
}

fn append_run_record(record: &serde_json::Value) {
    let mut file = open_private_append(&runs_log_path()).expect("runs log");
    writeln!(
        file,
        "{}",
        serde_json::to_string(record).expect("serialize run record")
    )
    .expect("write run record");
}

#[test]
fn managed_session_list_contract_merges_active_runs_with_auth_status() {
    let data = ManagedSessionListData::from_active_runs_and_auth_status(
        [
            serde_json::json!({
                "timestamp": "2026-06-30T01:00:00Z",
                "run_id": "run-older",
                "profile": "codex",
                "workspace": "/sensitive/workspaces/docs",
                "network": "provider",
                "status": "running",
                "container_name": "runhaven-codex-docs-run",
                "state_volume": "runhaven-codex-docs-home",
                "auth_scope": "project",
                "session": "docs"
            }),
            serde_json::json!({
                "timestamp": "2026-06-30T02:00:00Z",
                "run_id": "run-newer",
                "profile": "claude",
                "workspace": "/sensitive/workspaces/backend",
                "network": "provider",
                "status": "running",
                "container_name": "runhaven-claude-backend-run",
                "state_volume": "runhaven-claude-backend-home",
                "auth_scope": "agent",
                "session": "backend"
            }),
        ],
        serde_json::json!({
            "status": "api-key-broker",
            "runtime": "host-broker",
            "profiles": [
                {"name": "codex", "status": "api-key-broker"},
                {"name": "claude", "status": "api-key-broker"}
            ],
            "token": "should-not-leak"
        }),
    );

    assert_eq!(data.sessions.len(), 2);
    let first = &data.sessions[0];
    assert_eq!(first.id, "run-newer");
    assert_eq!(first.agent, "claude");
    assert_eq!(first.session, "backend");
    assert_eq!(first.project, "backend");
    assert_eq!(first.status, "running");
    assert_eq!(first.status_label, "Running");
    assert_eq!(first.auth_status, "api-key-broker");
    assert_eq!(first.auth_scope, "agent");
    assert_eq!(first.detail_command, "runhaven runs show run-newer");

    let encoded = serde_json::to_string(&data).expect("serialize managed sessions");
    for forbidden in [
        "/sensitive/workspaces",
        "should-not-leak",
        "state_volume",
        "runhaven-codex-docs-home",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "managed session payload leaked forbidden value {forbidden:?}"
        );
    }
}

#[test]
fn managed_session_list_includes_recent_finished_records_without_overriding_active_runs() {
    let data = ManagedSessionListData::from_active_and_recent_runs(
        [serde_json::json!({
            "timestamp": "2026-06-30T02:00:00Z",
            "run_id": "run-still-active",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/active",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-active-run",
            "state_volume": "runhaven-codex-active-home",
            "auth_scope": "project",
            "session": "active"
        })],
        [
            serde_json::json!({
                "timestamp": "2026-06-30T04:00:00Z",
                "started_at": "2026-06-30T02:00:00Z",
                "finished_at": "2026-06-30T04:00:00Z",
                "run_id": "run-still-active",
                "profile": "codex",
                "workspace": "/sensitive/workspaces/active",
                "network": "provider",
                "status": "failed",
                "auth_scope": "project",
                "state_volume": "runhaven-codex-record-home",
                "session": "active"
            }),
            serde_json::json!({
                "timestamp": "2026-06-30T03:00:00Z",
                "started_at": "2026-06-30T01:00:00Z",
                "finished_at": "2026-06-30T03:00:00Z",
                "run_id": "run-finished",
                "profile": "claude",
                "workspace": "/sensitive/workspaces/finished",
                "network": "provider",
                "status": "succeeded",
                "auth_scope": "agent",
                "state_volume": "runhaven-claude-record-home",
                "session": "finished"
            }),
        ],
        serde_json::json!({
            "profiles": [
                {"name": "codex", "status": "api-key-broker"},
                {"name": "claude", "status": "unknown"}
            ]
        }),
    );

    assert_eq!(data.sessions.len(), 2);
    let active = data
        .sessions
        .iter()
        .find(|session| session.run_id == "run-still-active")
        .expect("active session");
    assert_eq!(active.status, "running");
    assert_eq!(active.container_name, "runhaven-codex-active-run");
    assert_eq!(active.next_action.kind, "monitor");

    let finished = data
        .sessions
        .iter()
        .find(|session| session.run_id == "run-finished")
        .expect("finished session");
    assert_eq!(finished.status, "succeeded");
    assert_eq!(finished.lifecycle, "finished");
    assert_eq!(finished.started_at, "2026-06-30T01:00:00Z");
    assert_eq!(finished.container_name, "-");
    assert!(!finished.input.attached);
    assert_eq!(finished.next_action.kind, "none");

    let encoded = serde_json::to_string(&data).expect("serialize managed sessions");
    for forbidden in [
        "/sensitive/workspaces",
        "runhaven-codex-record-home",
        "runhaven-claude-record-home",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "managed session payload leaked forbidden value {forbidden:?}"
        );
    }
}

#[test]
fn managed_session_cleanup_fails_closed_after_recorded_worktree_is_discarded() {
    let cache = tempfile::tempdir().expect("cache");
    let _cache_home = override_cache_root_for_tests(cache.path());
    let repo = tempfile::tempdir().expect("repo");
    init_clean_repo(repo.path());

    let run_id = "cleanup-availability-run";
    let worktree =
        create_worktree_for_run(repo.path(), WorkspaceScope::Current, true, "codex", run_id)
            .expect("create worktree");
    let worktree_root = worktree.worktree_root.clone();
    let record = serde_json::json!({
        "timestamp": "2026-07-01T01:00:00Z",
        "started_at": "2026-07-01T00:00:00Z",
        "finished_at": "2026-07-01T01:00:00Z",
        "run_id": run_id,
        "profile": "codex",
        "workspace": repo.path().join("project").display().to_string(),
        "network": "provider",
        "status": "succeeded",
        "auth_scope": "project",
        "state_volume": "runhaven-codex-cleanup-home",
        "session": "cleanup",
        "worktree": worktree_record(&worktree)
    });
    append_run_record(&record);

    let before = ManagedSessionListData::from_active_and_recent_runs(
        [],
        [record.clone()],
        serde_json::json!({"profiles": [{"name": "codex", "status": "api-key-broker"}]}),
    );

    let session = before
        .sessions
        .first()
        .expect("managed session before cleanup");
    assert_eq!(session.lifecycle, "finished");
    assert!(session.cleanup.available);
    assert_eq!(session.cleanup.target, "worktree");

    let active_stopped = ManagedSessionListData::from_active_and_recent_runs(
        [record.clone()],
        [],
        serde_json::json!({"profiles": [{"name": "codex", "status": "api-key-broker"}]}),
    );
    let session = active_stopped
        .sessions
        .first()
        .expect("active managed session");
    assert_eq!(session.lifecycle, "finished");
    assert!(!session.cleanup.available);

    discard_worktree_for_run(run_id).expect("discard worktree");
    assert!(
        !worktree_root.exists(),
        "worktree directory should be gone after cleanup"
    );

    let after = ManagedSessionListData::from_active_and_recent_runs(
        [],
        [record],
        serde_json::json!({"profiles": [{"name": "codex", "status": "api-key-broker"}]}),
    );

    let session = after
        .sessions
        .first()
        .expect("managed session after cleanup");
    assert_eq!(session.lifecycle, "finished");
    assert!(!session.cleanup.available);
    assert_eq!(session.cleanup.target, "");

    let encoded = serde_json::to_string(&after).expect("serialize managed session");
    for forbidden in [
        worktree_root.display().to_string(),
        repo.path().display().to_string(),
        "runhaven-codex-cleanup-home".to_string(),
    ] {
        assert!(
            !encoded.contains(&forbidden),
            "managed session cleanup contract leaked forbidden value {forbidden:?}"
        );
    }
}

#[test]
fn managed_session_component_payload_round_trips_json() {
    let data = ManagedSessionListData::from_active_runs_and_auth_status(
        [serde_json::json!({
            "timestamp": "2026-06-30T02:00:00Z",
            "run_id": "run-123",
            "profile": "codex",
            "workspace": "/sensitive/workspaces/project",
            "network": "provider",
            "status": "running",
            "container_name": "runhaven-codex-project-run",
            "session": "project"
        })],
        serde_json::json!({
            "profiles": [{"name": "codex", "status": "api-key-broker"}]
        }),
    );
    let payload = RunHavenComponentPayload::ManagedSessionList(data);
    let encoded = serde_json::to_value(&payload).expect("serialize managed sessions");

    assert_eq!(
        encoded.get("type").and_then(serde_json::Value::as_str),
        Some("managedSessionList")
    );
    let data = encoded.get("data").expect("session data");
    assert!(data.get("sessions").is_some());
    assert!(data.to_string().contains("runhaven runs show run-123"));

    let decoded: RunHavenComponentPayload =
        serde_json::from_value(encoded).expect("deserialize managed sessions");
    assert_eq!(decoded, payload);
}

#[test]
fn managed_session_contract_derives_lifecycle_action_and_events() {
    let data = ManagedSessionListData::from_active_runs_and_auth_status(
        [
            serde_json::json!({
                "timestamp": "2026-06-30T03:00:00Z",
                "run_id": "run-auth",
                "profile": "claude",
                "workspace": "/sensitive/workspaces/auth",
                "network": "provider",
                "status": "running",
                "container_name": "runhaven-claude-auth-run",
                "state_volume": "runhaven-claude-auth-home",
                "session": "auth",
                "token": "should-not-leak"
            }),
            serde_json::json!({
                "timestamp": "2026-06-30T02:00:00Z",
                "run_id": "run-ready",
                "profile": "codex",
                "workspace": "/sensitive/workspaces/ready",
                "network": "provider",
                "status": "running",
                "container_name": "runhaven-codex-ready-run",
                "state_volume": "runhaven-codex-ready-home",
                "session": "ready"
            }),
        ],
        serde_json::json!({
            "profiles": [
                {"name": "codex", "status": "api-key-broker"},
                {"name": "claude", "status": "unknown"}
            ]
        }),
    );

    let auth = &data.sessions[0];
    assert_eq!(auth.lifecycle, "running");
    assert_eq!(auth.next_action.kind, "auth_attention");
    assert_eq!(auth.next_action.label, "Finish auth setup");
    assert_eq!(auth.next_action.detail, "Auth is not ready for Claude.");

    let ready = &data.sessions[1];
    assert_eq!(ready.lifecycle, "running");
    assert_eq!(ready.next_action.kind, "monitor");
    assert_eq!(ready.next_action.label, "Monitor session");
    assert_eq!(
        ready
            .events
            .iter()
            .map(|event| event.source.as_str())
            .collect::<Vec<_>>(),
        ["run", "auth", "network"]
    );
    assert_eq!(ready.events[0].label, "Run record");
    assert_eq!(ready.events[0].detail, "Running");
    assert_eq!(ready.events[1].detail, "API key broker");
    assert_eq!(ready.events[2].detail, "Provider only");

    let encoded = serde_json::to_string(&data).expect("serialize managed sessions");
    for forbidden in [
        "/sensitive/workspaces",
        "should-not-leak",
        "state_volume",
        "runhaven-claude-auth-home",
        "runhaven-codex-ready-home",
    ] {
        assert!(
            !encoded.contains(forbidden),
            "managed session payload leaked forbidden value {forbidden:?}"
        );
    }
}
