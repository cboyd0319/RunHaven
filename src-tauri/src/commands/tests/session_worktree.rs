use std::fs;
use std::process::Command;

use super::*;
use crate::commands::session_worktree::{
    discard_managed_session_worktree_for_test, get_managed_session_worktree_for_test,
    keep_managed_session_worktree_for_test, merge_managed_session_worktree_for_test,
    recover_managed_session_worktree_for_test,
};
use crate::contracts::{WorktreeActionRequest, WorktreeReviewRequest};
use runhaven_core::runtime::plans::WorkspaceScope;
use runhaven_core::runtime::worktrees::{
    WorktreeChangeKind, create_worktree_for_run, worktree_record,
};

/// Build a hermetic finished worktree run and return (repo tempdir, worktree).
pub(super) fn seed_worktree_run(
    run_id: &str,
) -> (
    tempfile::TempDir,
    runhaven_core::runtime::plans::WorktreeRun,
) {
    let repo = tempfile::tempdir().expect("repo tempdir");
    init_git_repo(repo.path());
    let worktree =
        create_worktree_for_run(repo.path(), WorkspaceScope::Current, true, "codex", run_id)
            .expect("worktree");
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
        "session": "worktree-review",
        "state_volume": "runhaven-codex-hidden-home",
        "token": "should-not-leak",
        "worktree": worktree_record(&worktree)
    }));
    (repo, worktree)
}

pub(super) fn action_request(run_id: &str, confirm: bool) -> WorktreeActionRequest {
    let review_id = get_managed_session_worktree_for_test(WorktreeReviewRequest {
        session_id: run_id.to_string(),
    })
    .expect("current worktree review")
    .review_id;
    WorktreeActionRequest {
        session_id: run_id.to_string(),
        confirm,
        review_id: Some(review_id),
    }
}

pub(super) fn assert_no_forbidden(
    encoded: &str,
    repo: &std::path::Path,
    worktree_root: &std::path::Path,
) {
    for forbidden in [
        repo.display().to_string(),
        worktree_root.display().to_string(),
        "runhaven-codex-hidden-home".to_string(),
        "should-not-leak".to_string(),
        "git -C".to_string(),
    ] {
        assert!(
            !encoded.contains(&forbidden),
            "worktree response leaked forbidden value {forbidden:?}"
        );
    }
}

fn git(repo: &std::path::Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .status()
        .expect("git available");
    assert!(status.success(), "git {args:?} failed");
}

#[test]
fn worktree_review_counts_committed_changes_from_the_validated_base() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "worktree-review-committed-change";
    let (_repo, worktree) = seed_worktree_run(run_id);
    fs::write(
        worktree.worktree_root.join("project").join("README.md"),
        "committed assistant change\n",
    )
    .expect("edit worktree");
    git(&worktree.worktree_root, &["add", "project/README.md"]);
    git(
        &worktree.worktree_root,
        &[
            "-c",
            "user.name=RunHaven Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-qm",
            "assistant change",
        ],
    );

    let review = get_managed_session_worktree_for_test(WorktreeReviewRequest {
        session_id: run_id.to_string(),
    })
    .expect("worktree review");

    assert_eq!(review.worktree_changes, 1);
    assert!(!review.clean);
    assert_eq!(review.file_changes.len(), 1);
    assert_eq!(review.file_changes[0].path, "project/README.md");
    assert_eq!(review.file_changes[0].kind, WorktreeChangeKind::Changed);
    assert!(!review.file_changes_truncated);
    assert_eq!(review.review_id.len(), 64);
}

#[test]
fn worktree_review_read_maps_only_safe_fields() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "worktree-review-read";
    let (repo, worktree) = seed_worktree_run(run_id);
    // Introduce a tracked change so the review reports a non-zero change count.
    fs::write(
        worktree.worktree_root.join("project").join("README.md"),
        "changed\n",
    )
    .expect("edit worktree");
    fs::write(
        repo.path().join("project").join("source-local.txt"),
        "local\n",
    )
    .expect("edit source");

    let review = get_managed_session_worktree_for_test(WorktreeReviewRequest {
        session_id: run_id.to_string(),
    })
    .expect("worktree review");

    assert_eq!(review.run_id, run_id);
    assert_eq!(review.branch, worktree.branch);
    assert!(review.branch.starts_with("runhaven/codex/"));
    assert!(review.worktree_changes >= 1);
    assert!(!review.clean);
    assert!(
        review
            .file_changes
            .iter()
            .any(|change| change.path == "project/README.md"
                && change.kind == WorktreeChangeKind::Changed)
    );
    assert_eq!(review.source_changes, 1);
    assert_eq!(review.source_file_changes.len(), 1);
    assert_eq!(
        review.source_file_changes[0].path,
        "project/source-local.txt"
    );
    assert_eq!(
        review.source_file_changes[0].kind,
        WorktreeChangeKind::Added
    );
    assert!(!review.source_file_changes_truncated);
    assert_eq!(
        review.available_actions,
        vec!["merge", "recover", "discard"]
    );

    let encoded = serde_json::to_string(&review).expect("serialize review");
    assert_no_forbidden(&encoded, repo.path(), &worktree.worktree_root);
}

#[test]
fn worktree_review_read_rejects_non_worktree_and_active_sessions() {
    let (_cache, _cache_home) = isolated_cache();

    write_history_run_record(json!({
        "timestamp": "2026-07-01T07:05:00Z",
        "started_at": "2026-07-01T06:05:00Z",
        "finished_at": "2026-07-01T07:05:00Z",
        "run_id": "worktree-review-plain",
        "profile": "codex",
        "workspace": "/tmp/plain-run",
        "network": "provider",
        "status": "succeeded",
        "auth_scope": "project",
        "session": "plain"
    }));
    let unsupported = get_managed_session_worktree_for_test(WorktreeReviewRequest {
        session_id: "worktree-review-plain".to_string(),
    })
    .expect_err("plain run has no worktree");
    assert_eq!(
        unsupported,
        "Saved changes are not available for this session."
    );

    let missing = get_managed_session_worktree_for_test(WorktreeReviewRequest {
        session_id: "worktree-review-missing".to_string(),
    })
    .expect_err("unknown session");
    assert!(missing.contains("managed session not found"));
}

#[test]
fn worktree_actions_reject_unconfirmed_before_approval() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "worktree-unconfirmed";
    let (_repo, _worktree) = seed_worktree_run(run_id);

    let error =
        merge_managed_session_worktree_for_test(action_request(run_id, false), |_, _, _| {
            unreachable!("unconfirmed merge must fail before approval")
        })
        .expect_err("confirmation required");
    assert_eq!(
        error,
        "Confirm Apply changes before RunHaven changes your project."
    );

    let error =
        discard_managed_session_worktree_for_test(action_request(run_id, false), |_, _, _| {
            unreachable!("unconfirmed discard must fail before approval")
        })
        .expect_err("confirmation required");
    assert_eq!(
        error,
        "Confirm Discard changes before RunHaven permanently deletes the protected copy."
    );
}

#[test]
fn worktree_mutations_require_the_review_id_before_approval() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "worktree-review-id-required";
    let (_repo, _worktree) = seed_worktree_run(run_id);
    let request = WorktreeActionRequest {
        session_id: run_id.to_string(),
        confirm: true,
        review_id: None,
    };

    let error = merge_managed_session_worktree_for_test(request, |_, _, _| {
        unreachable!("missing review id must fail before approval")
    })
    .expect_err("review id required");

    assert_eq!(
        error,
        "The protected copy changed since you reviewed it. Review the changes again before you continue."
    );
}

#[test]
fn worktree_merge_rejects_active_marker_before_approval() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "worktree-active";
    let (repo, worktree) = seed_worktree_run(run_id);
    write_active_run_payload(
        run_id,
        json!({
            "timestamp": "2026-07-01T06:40:00Z",
            "run_id": run_id,
            "profile": "codex",
            "workspace": repo.path().join("project").display().to_string(),
            "network": "provider",
            "status": "succeeded",
            "container_name": format!("runhaven-codex-{run_id}"),
            "state_volume": "runhaven-codex-hidden-home",
            "session": "worktree-review",
            "token": "should-not-leak"
        }),
    )
    .expect("active run payload");

    let error = merge_managed_session_worktree_for_test(action_request(run_id, true), |_, _, _| {
        unreachable!("active marker must fail before approval")
    })
    .expect_err("active marker rejected");
    assert_eq!(
        error,
        "Wait for this session to end before you review its changes."
    );
    assert!(worktree.worktree_root.exists());
    assert!(git_branch_exists(repo.path(), &worktree.branch));
    assert_no_forbidden(&error, repo.path(), &worktree.worktree_root);
}

#[test]
fn worktree_actions_reject_stale_worktree_before_approval() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "worktree-stale";
    let (repo, worktree) = seed_worktree_run(run_id);
    let request = action_request(run_id, true);
    fs::remove_dir_all(&worktree.worktree_root).expect("remove worktree");

    let error = discard_managed_session_worktree_for_test(request, |_, _, _| {
        unreachable!("stale worktree must fail before approval")
    })
    .expect_err("stale worktree rejected");
    assert_eq!(
        error,
        "RunHaven couldn’t verify the protected copy. It will not change or delete any files."
    );
    assert_no_forbidden(&error, repo.path(), &worktree.worktree_root);
}

#[test]
fn worktree_keep_leaves_the_worktree_in_place() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "worktree-keep";
    let (repo, worktree) = seed_worktree_run(run_id);

    let response = keep_managed_session_worktree_for_test(
        action_request(run_id, true),
        |session, resolved, branch| {
            assert_eq!(session.id, run_id);
            assert_eq!(resolved, run_id);
            assert_eq!(branch, worktree.branch);
            Ok(true)
        },
    )
    .expect("keep worktree");

    assert_eq!(response.result.action, "keep");
    assert_eq!(response.result.status, "kept");
    assert_eq!(response.result.branch, worktree.branch);
    assert!(response.result.next_steps.is_empty());
    // Keep does not mutate: the worktree, branch, and review availability stay.
    assert!(worktree.worktree_root.exists());
    assert!(git_branch_exists(repo.path(), &worktree.branch));

    let encoded = serde_json::to_string(&response).expect("serialize keep");
    assert_no_forbidden(&encoded, repo.path(), &worktree.worktree_root);
}

#[test]
fn worktree_recover_returns_safe_next_steps_without_paths() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "worktree-recover";
    let (repo, worktree) = seed_worktree_run(run_id);

    let response =
        recover_managed_session_worktree_for_test(action_request(run_id, true), |_, _, _| Ok(true))
            .expect("recover worktree");

    assert_eq!(response.result.action, "recover");
    assert_eq!(response.result.status, "recovered");
    assert!(!response.result.next_steps.is_empty());
    // Recover does not mutate.
    assert!(worktree.worktree_root.exists());
    assert!(git_branch_exists(repo.path(), &worktree.branch));

    let encoded = serde_json::to_string(&response).expect("serialize recover");
    assert_no_forbidden(&encoded, repo.path(), &worktree.worktree_root);
}

#[test]
fn worktree_merge_applies_changes_and_cleans_up_without_leaking_paths() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "worktree-merge";
    let (repo, worktree) = seed_worktree_run(run_id);
    fs::write(
        worktree.worktree_root.join("project").join("README.md"),
        "merged change\n",
    )
    .expect("edit worktree");

    let mut approved = false;
    let response = merge_managed_session_worktree_for_test(
        action_request(run_id, true),
        |session, resolved, _| {
            assert_eq!(session.id, run_id);
            assert_eq!(resolved, run_id);
            approved = true;
            Ok(true)
        },
    )
    .expect("merge worktree");

    assert!(approved);
    assert_eq!(response.result.action, "merge");
    assert_eq!(response.result.status, "merged");
    assert!(!response.session.cleanup.available);
    // The worktree branch change reached the source checkout.
    let merged = fs::read_to_string(repo.path().join("project").join("README.md"))
        .expect("read merged source file");
    assert_eq!(merged, "merged change\n");
    // Merge cleans up the worktree and branch.
    assert!(!worktree.worktree_root.exists());
    assert!(!git_branch_exists(repo.path(), &worktree.branch));

    let encoded = serde_json::to_string(&response).expect("serialize merge");
    assert_no_forbidden(&encoded, repo.path(), &worktree.worktree_root);
}

#[test]
fn worktree_discard_removes_runhaven_owned_worktree_without_leaking_paths() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "worktree-discard";
    let (repo, worktree) = seed_worktree_run(run_id);

    let response =
        discard_managed_session_worktree_for_test(action_request(run_id, true), |_, _, _| Ok(true))
            .expect("discard worktree");

    assert_eq!(response.result.action, "discard");
    assert_eq!(response.result.status, "discarded");
    assert!(!response.session.cleanup.available);
    assert!(!worktree.worktree_root.exists());
    assert!(!git_branch_exists(repo.path(), &worktree.branch));

    let encoded = serde_json::to_string(&response).expect("serialize discard");
    assert_no_forbidden(&encoded, repo.path(), &worktree.worktree_root);
}

#[test]
fn worktree_action_stops_when_confirmation_denied() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "worktree-denied";
    let (repo, worktree) = seed_worktree_run(run_id);

    let error =
        discard_managed_session_worktree_for_test(action_request(run_id, true), |_, _, _| {
            Ok(false)
        })
        .expect_err("denied discard");
    assert_eq!(error, "Discard changes canceled. Nothing changed.");
    assert!(worktree.worktree_root.exists());
    assert!(git_branch_exists(repo.path(), &worktree.branch));
}
