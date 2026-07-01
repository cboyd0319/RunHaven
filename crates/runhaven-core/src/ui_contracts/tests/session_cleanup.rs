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
