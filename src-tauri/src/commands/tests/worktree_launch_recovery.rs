use super::*;

#[test]
fn direct_mode_plan_review_still_accepts_a_non_git_folder() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let plan_request = request(workspace.path().to_path_buf());

    let response = build_plan_response(plan_request).expect("direct-mode plan");

    assert!(!response.worktree);
    assert!(
        response
            .warnings
            .iter()
            .any(|warning| warning.code == "direct-workspace")
    );

    let mut read_only_request = request(workspace.path().to_path_buf());
    read_only_request.read_only_workspace = true;
    let read_only_response = build_plan_response(read_only_request).expect("read-only direct plan");
    assert!(
        read_only_response
            .warnings
            .iter()
            .all(|warning| warning.code != "direct-workspace")
    );
}

#[test]
fn worktree_plan_review_rejects_a_dirty_repo_with_beginner_guidance() {
    let (_cache, _cache_home) = isolated_cache();
    let repo = tempfile::tempdir().expect("repo");
    init_git_repo(repo.path());
    fs::write(repo.path().join("project/README.md"), "unsaved change\n")
        .expect("dirty project file");
    let mut plan_request = request(repo.path().join("project"));
    plan_request.worktree = true;

    let error = build_plan_response(plan_request).expect_err("dirty worktree preview");

    assert_eq!(
        error,
        "RunHaven can't make a protected copy while this project has unfinished changes."
    );
}

#[test]
fn worktree_plan_review_rejects_a_non_git_folder_with_beginner_guidance() {
    let (_cache, _cache_home) = isolated_cache();
    let workspace = tempfile::tempdir().expect("workspace");
    let mut plan_request = request(workspace.path().to_path_buf());
    plan_request.worktree = true;

    let error = build_plan_response(plan_request).expect_err("non-git worktree preview");

    assert_eq!(
        error,
        "RunHaven can't make a protected copy of this folder yet."
    );
}

#[test]
fn worktree_plan_review_explains_a_project_without_saved_history() {
    let (_cache, _cache_home) = isolated_cache();
    let repo = tempfile::tempdir().expect("repo");
    git(repo.path(), &["init"]);
    let project = repo.path().join("project");
    fs::create_dir(&project).expect("project folder");
    let mut plan_request = request(project);
    plan_request.worktree = true;

    let error = build_plan_response(plan_request).expect_err("repository has no saved history");

    assert_eq!(
        error,
        "RunHaven can't make a protected copy because this project does not have saved version history yet."
    );
}

#[test]
fn generic_worktree_failures_never_expose_paths_or_version_control_jargon() {
    let error = sanitize_worktree_creation_error_for_test(
        "git worktree failed at /Users/alice/private/project; commit or stash first",
    );

    assert_eq!(
        error,
        "RunHaven can't make a protected copy for this session."
    );
    for forbidden in ["/Users/", "git", "worktree", "commit", "stash"] {
        assert!(
            !error.contains(forbidden),
            "beginner-safe error leaked {forbidden:?}"
        );
    }
}

#[test]
fn post_creation_plan_failure_keeps_a_recoverable_worktree_record() {
    let (_cache, _cache_home) = isolated_cache();
    let repo = tempfile::tempdir().expect("repo");
    init_git_repo(repo.path());
    let run_id = "gui-worktree-plan-failure";
    let mut plan_request = request(repo.path().join("project"));
    plan_request.worktree = true;
    let preview_plan = build_agent_run_plan(&plan_request, Some(run_id.to_string()))
        .expect("validated preview plan");

    let error = build_worktree_run_plan_with_failure_for_test(&plan_request, run_id, &preview_plan)
        .expect_err("injected finalization failure");

    assert_eq!(
        error,
        "RunHaven couldn’t start this session. The protected copy is in History, where you can review or delete it."
    );
    let record = runhaven_core::records::find_run_record(run_id)
        .expect("recoverable failed worktree record");
    assert_eq!(record["status"], "failed");
    assert_eq!(record["cleanup"]["worktree"], "kept-for-recovery");
    let worktree_root = record["worktree"]["worktree_root"]
        .as_str()
        .expect("recorded worktree root");
    let branch = record["worktree"]["branch"]
        .as_str()
        .expect("recorded branch");
    assert!(Path::new(worktree_root).is_dir());
    assert!(git_branch_exists(repo.path(), branch));
}

#[test]
fn late_worktree_state_change_uses_fixed_retry_guidance_without_mutation() {
    let (_cache, _cache_home) = isolated_cache();
    let repo = tempfile::tempdir().expect("repo");
    init_git_repo(repo.path());
    let run_id = "gui-late-worktree-change";
    let project = repo.path().join("project");
    let mut plan_request = request(project.clone());
    plan_request.worktree = true;
    let preview_plan = build_agent_run_plan(&plan_request, Some(run_id.to_string()))
        .expect("validated preview plan");
    fs::write(project.join("README.md"), "changed after revalidation\n")
        .expect("late project change");

    let error = build_worktree_run_plan_with_failure_for_test(&plan_request, run_id, &preview_plan)
        .expect_err("late state change must fail closed");

    assert_eq!(
        error,
        "The project or session settings changed before RunHaven could start. Check your folder and options, then try again."
    );
    assert!(!error.contains(&project.display().to_string()));
    assert!(!git_branch_exists(
        repo.path(),
        "runhaven/codex/gui-late-worktree-change"
    ));
}
