use std::fs;

use super::session_worktree::seed_worktree_run_with_status;
use super::*;
use crate::commands::session_worktree::get_managed_session_worktree_for_test;
use crate::contracts::WorktreeReviewRequest;

#[test]
fn ended_interactive_worktree_is_reviewable() {
    let (_cache, _cache_home) = isolated_cache();
    let run_id = "worktree-review-ended-interactive";
    let (_repo, worktree) = seed_worktree_run_with_status(run_id, "ended");
    fs::write(
        worktree.worktree_root.join("project").join("README.md"),
        "interactive assistant change\n",
    )
    .expect("edit worktree");

    let review = get_managed_session_worktree_for_test(WorktreeReviewRequest {
        session_id: run_id.to_string(),
    })
    .expect("ended interactive worktree remains available for review");

    assert_eq!(review.run_id, run_id);
    assert!(review.worktree_changes >= 1);
}
