use std::fs;

use super::*;
use crate::commands::session_worktree::{
    discard_managed_session_worktree_for_test, merge_managed_session_worktree_for_test,
};
use crate::commands::tests::session_worktree::{
    action_request, assert_no_forbidden, seed_worktree_run,
};

const STALE_REVIEW_ERROR: &str = "The protected copy changed since you reviewed it. Review the changes again before you continue.";

#[test]
fn worktree_merge_rejects_a_stale_review_before_native_approval() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "worktree-stale-review-before-approval";
    let (repo, worktree) = seed_worktree_run(run_id);
    let path = worktree.worktree_root.join("project/README.md");
    fs::write(&path, "first reviewed change\n").expect("first edit");
    let request = action_request(run_id, true);
    fs::write(&path, "changed after review\n").expect("second edit");

    let error = merge_managed_session_worktree_for_test(request, |_, _, _| {
        unreachable!("stale review must fail before native approval")
    })
    .expect_err("stale review rejected");

    assert_eq!(error, STALE_REVIEW_ERROR);
    assert!(worktree.worktree_root.exists());
    assert!(git_branch_exists(repo.path(), &worktree.branch));
    assert_eq!(
        fs::read_to_string(repo.path().join("project/README.md")).expect("source README"),
        "hello\n"
    );
    assert_no_forbidden(&error, repo.path(), &worktree.worktree_root);
}

#[test]
fn worktree_merge_rechecks_the_review_after_native_approval() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "worktree-review-changed-during-merge-approval";
    let (repo, worktree) = seed_worktree_run(run_id);
    let path = worktree.worktree_root.join("project/README.md");
    fs::write(&path, "reviewed change\n").expect("reviewed edit");
    let request = action_request(run_id, true);

    let error = merge_managed_session_worktree_for_test(request, |_, _, _| {
        fs::write(&path, "changed during approval\n").map_err(|error| error.to_string())?;
        Ok(true)
    })
    .expect_err("post-approval review change rejected");

    assert_eq!(error, STALE_REVIEW_ERROR);
    assert!(worktree.worktree_root.exists());
    assert!(git_branch_exists(repo.path(), &worktree.branch));
    assert_eq!(
        fs::read_to_string(repo.path().join("project/README.md")).expect("source README"),
        "hello\n"
    );
}

#[test]
fn worktree_discard_rechecks_the_review_after_native_approval() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "worktree-review-changed-during-discard-approval";
    let (repo, worktree) = seed_worktree_run(run_id);
    let path = worktree.worktree_root.join("project/README.md");
    fs::write(&path, "reviewed change\n").expect("reviewed edit");
    let request = action_request(run_id, true);

    let error = discard_managed_session_worktree_for_test(request, |_, _, _| {
        fs::write(&path, "changed during approval\n").map_err(|error| error.to_string())?;
        Ok(true)
    })
    .expect_err("post-approval review change rejected");

    assert_eq!(error, STALE_REVIEW_ERROR);
    assert!(worktree.worktree_root.exists());
    assert!(git_branch_exists(repo.path(), &worktree.branch));
}
