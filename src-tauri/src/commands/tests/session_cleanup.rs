use std::fs::{self, File};
use std::io::Write;
use std::process::Command;

use super::*;
use crate::contracts::CleanupManagedSessionRequest;
use runhaven_core::runtime::plans::WorkspaceScope;
use runhaven_core::runtime::worktrees::{create_worktree_for_run, worktree_record};
use runhaven_core::support::paths::{ensure_private_parent, runs_log_path};

#[test]
fn managed_session_cleanup_rejects_unknown_unconfirmed_and_ineligible_targets() {
    let (_cache, _cache_home) = isolated_cache();

    let missing = crate::commands::session_cleanup::cleanup_managed_session_for_test(
        CleanupManagedSessionRequest {
            session_id: "missing-session".to_string(),
            confirm_cleanup: true,
        },
        |_, _| unreachable!("unknown sessions must fail before approval"),
    )
    .expect_err("unknown session rejected");
    assert!(missing.contains("managed session not found"));

    let unconfirmed = crate::commands::session_cleanup::cleanup_managed_session_for_test(
        CleanupManagedSessionRequest {
            session_id: "missing-session".to_string(),
            confirm_cleanup: false,
        },
        |_, _| unreachable!("unconfirmed cleanup must fail before approval"),
    )
    .expect_err("confirmation required");
    assert!(unconfirmed.contains("Confirm the cleanup"));

    write_history_run_record(json!({
        "timestamp": "2026-07-01T06:00:00Z",
        "started_at": "2026-07-01T05:00:00Z",
        "finished_at": "2026-07-01T06:00:00Z",
        "run_id": "managed-session-history-ineligible",
        "profile": "codex",
        "workspace": "/sensitive/workspaces/history-ineligible",
        "network": "provider",
        "status": "queued",
        "auth_scope": "project",
        "session": "history-ineligible"
    }));

    let ineligible = crate::commands::session_cleanup::cleanup_managed_session_for_test(
        CleanupManagedSessionRequest {
            session_id: "managed-session-history-ineligible".to_string(),
            confirm_cleanup: true,
        },
        |_, _| unreachable!("ineligible history must fail before approval"),
    )
    .expect_err("ineligible history rejected");
    assert!(ineligible.contains("history is not eligible"));

    write_history_run_record(json!({
        "timestamp": "2026-07-01T06:30:00Z",
        "started_at": "2026-07-01T05:30:00Z",
        "finished_at": "2026-07-01T06:30:00Z",
        "run_id": "managed-session-history-unsupported",
        "profile": "codex",
        "workspace": "/sensitive/workspaces/history-unsupported",
        "network": "provider",
        "status": "succeeded",
        "auth_scope": "project",
        "session": "history-unsupported"
    }));

    let unsupported = crate::commands::session_cleanup::cleanup_managed_session_for_test(
        CleanupManagedSessionRequest {
            session_id: "managed-session-history-unsupported".to_string(),
            confirm_cleanup: true,
        },
        |_, _| unreachable!("unsupported cleanup must fail before approval"),
    )
    .expect_err("unsupported target rejected");
    assert!(unsupported.contains("only available for RunHaven-owned worktree sessions"));
}

#[test]
fn managed_session_cleanup_rejects_active_marker_before_approval() {
    let (_cache, _cache_home) = isolated_cache();
    let repo = tempfile::tempdir().expect("repo tempdir");
    init_git_repo(repo.path());

    let run_id = "managed-session-cleanup-active";
    let worktree =
        create_worktree_for_run(repo.path(), WorkspaceScope::Current, true, "codex", run_id)
            .expect("worktree");
    let worktree_root = worktree.worktree_root.clone();
    let branch = worktree.branch.clone();

    write_history_run_record(json!({
        "timestamp": "2026-07-01T06:45:00Z",
        "started_at": "2026-07-01T05:45:00Z",
        "finished_at": "2026-07-01T06:45:00Z",
        "run_id": run_id,
        "profile": "codex",
        "workspace": repo.path().join("project").display().to_string(),
        "network": "provider",
        "status": "succeeded",
        "auth_scope": "project",
        "session": "cleanup-active",
        "worktree": worktree_record(&worktree)
    }));
    write_active_run_payload(
        run_id,
        json!({
            "timestamp": "2026-07-01T06:40:00Z",
            "run_id": run_id,
            "profile": "codex",
            "workspace": repo.path().join("project").display().to_string(),
            "network": "provider",
            "status": "succeeded",
            "auth_scope": "project",
            "container_name": format!("runhaven-codex-{run_id}"),
            "state_volume": "runhaven-codex-active-hidden-home",
            "session": "cleanup-active",
            "token": "active-secret"
        }),
    )
    .expect("active run payload");

    let rejected = crate::commands::session_cleanup::cleanup_managed_session_for_test(
        CleanupManagedSessionRequest {
            session_id: run_id.to_string(),
            confirm_cleanup: true,
        },
        |_, _| unreachable!("active marker must fail before trusted approval"),
    )
    .expect_err("active marker rejected");

    assert!(rejected.contains("only available after this session stops running"));
    assert!(
        worktree_root.exists(),
        "active worktree must not be removed"
    );
    assert!(
        git_branch_exists(repo.path(), &branch),
        "active worktree branch must not be removed"
    );
    for forbidden in [
        repo.path().display().to_string(),
        worktree_root.display().to_string(),
        "runhaven-codex-active-hidden-home".to_string(),
        "active-secret".to_string(),
    ] {
        assert!(
            !rejected.contains(&forbidden),
            "active cleanup rejection leaked forbidden value {forbidden:?}"
        );
    }
}

#[test]
fn managed_session_cleanup_rejects_stale_worktree_before_approval_without_leaking_paths() {
    let (_cache, _cache_home) = isolated_cache();
    let repo = tempfile::tempdir().expect("repo tempdir");
    init_git_repo(repo.path());

    let run_id = "managed-session-cleanup-stale";
    let worktree =
        create_worktree_for_run(repo.path(), WorkspaceScope::Current, true, "codex", run_id)
            .expect("worktree");
    let worktree_root = worktree.worktree_root.clone();

    write_history_run_record(json!({
        "timestamp": "2026-07-01T06:50:00Z",
        "started_at": "2026-07-01T05:50:00Z",
        "finished_at": "2026-07-01T06:50:00Z",
        "run_id": run_id,
        "profile": "codex",
        "workspace": repo.path().join("project").display().to_string(),
        "network": "provider",
        "status": "succeeded",
        "auth_scope": "project",
        "session": "cleanup-stale",
        "state_volume": "runhaven-codex-stale-hidden-home",
        "token": "stale-secret",
        "worktree": worktree_record(&worktree)
    }));
    fs::remove_dir_all(&worktree_root).expect("remove stale worktree");

    let rejected = crate::commands::session_cleanup::cleanup_managed_session_for_test(
        CleanupManagedSessionRequest {
            session_id: run_id.to_string(),
            confirm_cleanup: true,
        },
        |_, _| unreachable!("stale worktree must fail before trusted approval"),
    )
    .expect_err("stale worktree rejected");

    assert!(rejected.contains("could not verify a safe cleanup target"));
    for forbidden in [
        repo.path().display().to_string(),
        worktree_root.display().to_string(),
        "runhaven-codex-stale-hidden-home".to_string(),
        "stale-secret".to_string(),
    ] {
        assert!(
            !rejected.contains(&forbidden),
            "stale cleanup rejection leaked forbidden value {forbidden:?}"
        );
    }
}

#[test]
fn managed_session_cleanup_discards_runhaven_owned_worktree_without_leaking_paths() {
    let (_cache, _cache_home) = isolated_cache();
    let repo = tempfile::tempdir().expect("repo tempdir");
    init_git_repo(repo.path());

    let run_id = "managed-session-cleanup";
    let worktree =
        create_worktree_for_run(repo.path(), WorkspaceScope::Current, true, "codex", run_id)
            .expect("worktree");
    let worktree_root = worktree.worktree_root.clone();
    let branch = worktree.branch.clone();

    write_history_run_record(json!({
        "timestamp": "2026-07-01T07:00:00Z",
        "started_at": "2026-07-01T06:00:00Z",
        "finished_at": "2026-07-01T07:00:00Z",
        "run_id": run_id,
        "profile": "codex",
        "workspace": repo.path().join("project").display().to_string(),
        "network": "provider",
        "status": "succeeded",
        "auth_scope": "project",
        "session": "cleanup",
        "state_volume": "runhaven-codex-hidden-home",
        "token": "should-not-leak",
        "worktree": worktree_record(&worktree)
    }));

    let mut cleaned = false;
    let response = crate::commands::session_cleanup::cleanup_managed_session_for_test(
        CleanupManagedSessionRequest {
            session_id: run_id.to_string(),
            confirm_cleanup: true,
        },
        |session, resolved_run_id| {
            assert_eq!(session.id, run_id);
            assert_eq!(resolved_run_id, run_id);
            cleaned = true;
            Ok(true)
        },
    )
    .expect("cleanup managed session");

    assert!(cleaned);
    assert_eq!(response.session.id, run_id);
    assert!(!response.session.cleanup.available);
    assert_eq!(response.session.cleanup.target, "");
    assert_eq!(response.result.run_id, run_id);
    assert_eq!(response.result.target, "worktree");
    assert_eq!(response.result.status, "discarded");
    assert!(
        !worktree_root.exists(),
        "worktree directory should be removed"
    );
    assert!(
        !git_branch_exists(repo.path(), &branch),
        "worktree branch should be removed"
    );

    let encoded = serde_json::to_string(&response).expect("serialize cleanup response");
    for forbidden in [
        repo.path().display().to_string(),
        worktree_root.display().to_string(),
        "runhaven-codex-hidden-home".to_string(),
        "should-not-leak".to_string(),
        "runhaven runs discard".to_string(),
    ] {
        assert!(
            !encoded.contains(&forbidden),
            "managed session cleanup leaked forbidden value {forbidden:?}"
        );
    }
}

#[test]
fn managed_session_cleanup_stops_when_trusted_confirmation_is_denied() {
    let (_cache, _cache_home) = isolated_cache();
    let repo = tempfile::tempdir().expect("repo tempdir");
    init_git_repo(repo.path());

    let run_id = "managed-session-cleanup-denied";
    let worktree =
        create_worktree_for_run(repo.path(), WorkspaceScope::Current, true, "codex", run_id)
            .expect("worktree");
    let worktree_root = worktree.worktree_root.clone();
    let branch = worktree.branch.clone();

    write_history_run_record(json!({
        "timestamp": "2026-07-01T07:30:00Z",
        "started_at": "2026-07-01T06:30:00Z",
        "finished_at": "2026-07-01T07:30:00Z",
        "run_id": run_id,
        "profile": "codex",
        "workspace": repo.path().display().to_string(),
        "network": "provider",
        "status": "succeeded",
        "auth_scope": "project",
        "session": "cleanup-denied",
        "worktree": worktree_record(&worktree)
    }));

    let denied = crate::commands::session_cleanup::cleanup_managed_session_for_test(
        CleanupManagedSessionRequest {
            session_id: run_id.to_string(),
            confirm_cleanup: true,
        },
        |session, resolved_run_id| {
            assert_eq!(session.id, run_id);
            assert_eq!(resolved_run_id, run_id);
            Ok(false)
        },
    )
    .expect_err("trusted cleanup denial rejected");

    assert!(denied.contains("Cleanup canceled"));
    assert!(
        worktree_root.exists(),
        "worktree should stay in place after denial"
    );
    assert!(
        git_branch_exists(repo.path(), &branch),
        "worktree branch should stay in place after denial"
    );
}

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

fn init_git_repo(path: &std::path::Path) {
    let project = path.join("project");
    fs::create_dir_all(&project).expect("project dir");
    git(path, &["init"]);
    git(path, &["config", "user.name", "RunHaven Test"]);
    git(path, &["config", "user.email", "runhaven-test@example.com"]);
    fs::write(project.join("README.md"), "hello\n").expect("write README");
    git(path, &["add", "."]);
    git(path, &["commit", "-m", "initial"]);
}

fn git_branch_exists(repo_root: &std::path::Path, branch: &str) -> bool {
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

fn git(repo_root: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .args(["-C"])
        .arg(repo_root)
        .args(args)
        .status()
        .expect("run git");
    assert!(status.success(), "git {:?} failed", args);
}
