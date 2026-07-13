use std::fs;

use super::*;
use crate::contracts::CleanupManagedSessionRequest;
use runhaven_core::runtime::plans::WorkspaceScope;
use runhaven_core::runtime::worktrees::{create_worktree_for_run, worktree_record};

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
    assert!(unconfirmed.contains("Confirm before deleting this protected copy"));

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
    assert_eq!(
        ineligible,
        "Saved changes are not available for this session."
    );

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
    assert_eq!(
        unsupported,
        "Saved changes are not available for this session."
    );
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

    assert_eq!(
        rejected,
        "Wait for this session to end before you review its changes."
    );
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

    assert_eq!(
        rejected,
        "RunHaven couldn’t verify the protected copy. It will not change or delete any files."
    );
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
