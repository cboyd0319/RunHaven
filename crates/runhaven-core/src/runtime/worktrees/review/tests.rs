use std::fs;
use std::path::Path;
use std::process::Command;

use super::*;

fn git(repo: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .status()
        .expect("git available");
    assert!(status.success(), "git {args:?} failed");
}

fn seeded_lifecycle(run_id: &str) -> (tempfile::TempDir, WorktreeLifecycle) {
    let temp = tempfile::tempdir().expect("tempdir");
    let source = temp.path().join("source");
    let worktree = temp.path().join(run_id);
    fs::create_dir_all(source.join("project")).expect("source project");
    git(&source, &["init", "-q"]);
    git(&source, &["config", "user.email", "test@example.com"]);
    git(&source, &["config", "user.name", "RunHaven Test"]);
    fs::write(source.join("project/README.md"), "base\n").expect("README");
    fs::write(source.join("project/dirty.txt"), "base\n").expect("dirty seed");
    fs::write(source.join("project/removed.txt"), "base\n").expect("removed seed");
    git(&source, &["add", "."]);
    git(&source, &["commit", "-qm", "seed"]);
    let base_head = String::from_utf8(
        Command::new("git")
            .arg("-C")
            .arg(&source)
            .args(["rev-parse", "HEAD"])
            .output()
            .expect("git head")
            .stdout,
    )
    .expect("UTF-8 head")
    .trim()
    .to_string();
    let branch = format!("runhaven/codex/{run_id}");
    git(
        &source,
        &[
            "worktree",
            "add",
            "-q",
            "-b",
            &branch,
            worktree.to_str().expect("worktree path"),
            &base_head,
        ],
    );
    let lifecycle = WorktreeLifecycle {
        run_id: run_id.to_string(),
        source_repo_root: source.canonicalize().expect("source root"),
        worktree_root: worktree.canonicalize().expect("worktree root"),
        mounted_workspace: worktree.join("project"),
        branch,
        base_head,
    };
    (temp, lifecycle)
}

#[test]
fn worktree_change_parser_reads_nul_status_path_pairs() {
    let changes = parse_name_status(b"A\0new.txt\0M\0src/lib.rs\0D\0old.txt\0").expect("parse");
    assert_eq!(
        changes,
        vec![
            RawChange {
                path: b"new.txt".to_vec(),
                kind: WorktreeChangeKind::Added,
            },
            RawChange {
                path: b"src/lib.rs".to_vec(),
                kind: WorktreeChangeKind::Changed,
            },
            RawChange {
                path: b"old.txt".to_vec(),
                kind: WorktreeChangeKind::Removed,
            },
        ]
    );
}

#[test]
fn worktree_change_parser_rejects_malformed_status_and_structural_paths() {
    for output in [
        b"M\0path".as_slice(),
        b"M\0".as_slice(),
        b"R100\0old\0new\0".as_slice(),
        b"M\0/absolute\0".as_slice(),
        b"M\0.\0".as_slice(),
        b"M\0../escape\0".as_slice(),
    ] {
        assert!(parse_name_status(output).is_err(), "accepted {output:?}");
    }
    let oversized = vec![b'x'; MAX_GIT_REVIEW_OUTPUT_BYTES + 1];
    assert!(parse_name_status(&oversized).is_err());
}

#[test]
fn worktree_change_display_hides_unsafe_paths_and_caps_at_one_hundred() {
    let mut changes = (0..101)
        .map(|index| RawChange {
            path: format!("safe/{index:03}.txt").into_bytes(),
            kind: WorktreeChangeKind::Changed,
        })
        .collect::<Vec<_>>();
    changes.push(RawChange {
        path: b"hidden\xFF.txt".to_vec(),
        kind: WorktreeChangeKind::Added,
    });
    changes.push(RawChange {
        path: "hidden\u{202e}.txt".as_bytes().to_vec(),
        kind: WorktreeChangeKind::Added,
    });
    let (visible, truncated) = visible_changes(&changes);
    assert_eq!(visible.len(), 100);
    assert!(truncated);
    assert!(visible.iter().all(|change| !change.path.contains("hidden")));
}

#[test]
fn worktree_change_probe_reports_a_clean_copy() {
    let (_temp, lifecycle) = seeded_lifecycle("worktree-change-clean");

    assert!(!worktree_has_changes(&lifecycle).expect("clean probe"));
}

#[test]
fn worktree_change_summary_combines_committed_dirty_removed_and_untracked_files() {
    let (_temp, lifecycle) = seeded_lifecycle("worktree-change-summary");
    fs::write(
        lifecycle.worktree_root.join("project/README.md"),
        "committed\n",
    )
    .expect("committed edit");
    git(&lifecycle.worktree_root, &["add", "project/README.md"]);
    git(
        &lifecycle.worktree_root,
        &["commit", "-qm", "assistant edit"],
    );
    fs::write(lifecycle.worktree_root.join("project/dirty.txt"), "dirty\n").expect("dirty edit");
    fs::remove_file(lifecycle.worktree_root.join("project/removed.txt")).expect("remove file");
    fs::create_dir_all(lifecycle.worktree_root.join("project/new/nested")).expect("nested dir");
    fs::write(
        lifecycle.worktree_root.join("project/new/nested/leaf.txt"),
        "new\n",
    )
    .expect("untracked leaf");

    let summary = worktree_change_summary(&lifecycle).expect("change summary");

    assert!(worktree_has_changes(&lifecycle).expect("changed probe"));
    assert_eq!(summary.total_count, 4);
    assert!(!summary.file_changes_truncated);
    assert_eq!(summary.review_id.len(), 64);
    assert_eq!(
        summary.file_changes,
        vec![
            WorktreeFileChange {
                path: "project/README.md".to_string(),
                kind: WorktreeChangeKind::Changed,
            },
            WorktreeFileChange {
                path: "project/dirty.txt".to_string(),
                kind: WorktreeChangeKind::Changed,
            },
            WorktreeFileChange {
                path: "project/new/nested/leaf.txt".to_string(),
                kind: WorktreeChangeKind::Added,
            },
            WorktreeFileChange {
                path: "project/removed.txt".to_string(),
                kind: WorktreeChangeKind::Removed,
            },
        ]
    );
    let encoded = serde_json::to_string(&summary.file_changes).expect("serialize changes");
    assert!(!encoded.contains(lifecycle.worktree_root.to_str().expect("root")));
}

#[test]
fn worktree_change_review_id_changes_when_content_changes_at_the_same_path() {
    let (_temp, lifecycle) = seeded_lifecycle("worktree-change-content-id");
    let path = lifecycle.worktree_root.join("project/README.md");
    fs::write(&path, "first\n").expect("first edit");
    let first = worktree_change_summary(&lifecycle).expect("first summary");
    fs::write(&path, "second\n").expect("second edit");
    let second = worktree_change_summary(&lifecycle).expect("second summary");
    assert_eq!(first.file_changes, second.file_changes);
    assert_ne!(first.review_id, second.review_id);
}

#[test]
fn worktree_change_summary_reports_a_rename_as_removed_and_added() {
    let (_temp, lifecycle) = seeded_lifecycle("worktree-change-rename");
    git(
        &lifecycle.worktree_root,
        &["mv", "project/README.md", "project/RENAMED.md"],
    );

    let summary = worktree_change_summary(&lifecycle).expect("rename summary");

    assert_eq!(summary.total_count, 2);
    assert_eq!(
        summary.file_changes,
        vec![
            WorktreeFileChange {
                path: "project/README.md".to_string(),
                kind: WorktreeChangeKind::Removed,
            },
            WorktreeFileChange {
                path: "project/RENAMED.md".to_string(),
                kind: WorktreeChangeKind::Added,
            },
        ]
    );
}
