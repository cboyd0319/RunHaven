use std::fs;
use std::path::Path;

use anyhow::{Result, bail};

use super::{WorktreeLifecycle, git_bytes, git_checked, git_stdout};
use crate::support::git::{
    GitSnapshot, capture_git_snapshot, parse_git_status_entries, safe_repo_path,
};

pub fn ensure_source_ready_for_merge(lifecycle: &WorktreeLifecycle) -> Result<()> {
    let current_head = git_stdout(&lifecycle.source_repo_root, &["rev-parse", "HEAD"])?;
    if current_head != lifecycle.base_head {
        bail!("source repository HEAD changed since the worktree run; refusing merge");
    }
    match capture_git_snapshot(&lifecycle.source_repo_root) {
        GitSnapshot::Unavailable { .. } => {
            bail!("could not inspect source repository before merge")
        }
        GitSnapshot::Available { dirty: true, .. } => {
            bail!("source repository has uncommitted changes; refusing merge")
        }
        GitSnapshot::Available { .. } => Ok(()),
    }
}

pub(super) fn merge_worktree_changes(lifecycle: &WorktreeLifecycle) -> Result<()> {
    ensure_source_ready_for_merge(lifecycle)?;
    let worktree_head = git_stdout(&lifecycle.worktree_root, &["rev-parse", "HEAD"])?;
    if worktree_head != lifecycle.base_head {
        git_checked(
            &lifecycle.source_repo_root,
            &["merge", "--ff-only", &lifecycle.branch],
            None,
            "fast-forward source repository",
        )?;
    }
    apply_worktree_dirty_changes(lifecycle)
}

pub(super) fn format_merge_recovery(lifecycle: &WorktreeLifecycle, reason: &str) -> String {
    format!(
        "could not complete merge for run {}: {}\nNo cleanup was attempted; review the recorded worktree before retrying.\nSource repo: {}\nWorktree: {}\nBranch: {}\nReview changes: runhaven runs diff {}\nInspect source: git -C {} status --short\nInspect worktree: git -C {} status --short\nManual recovery guide: runhaven runs recover {}\nRetry after fixing the source checkout: runhaven runs merge {}\nKeep for manual review: runhaven runs keep {}\nDiscard after review: runhaven runs discard {}",
        lifecycle.run_id,
        reason,
        lifecycle.source_repo_root.display(),
        lifecycle.worktree_root.display(),
        lifecycle.branch,
        lifecycle.run_id,
        lifecycle.source_repo_root.display(),
        lifecycle.worktree_root.display(),
        lifecycle.run_id,
        lifecycle.run_id,
        lifecycle.run_id,
        lifecycle.run_id
    )
}

fn apply_worktree_dirty_changes(lifecycle: &WorktreeLifecycle) -> Result<()> {
    let paths = untracked_paths(&lifecycle.worktree_root)?;
    ensure_untracked_destinations_available(
        &lifecycle.worktree_root,
        &lifecycle.source_repo_root,
        &paths,
    )?;
    let patch = git_bytes(
        &lifecycle.worktree_root,
        &["diff", "--binary", "HEAD"],
        None,
        "read worktree diff",
    )?;
    if !patch.is_empty() {
        git_checked(
            &lifecycle.source_repo_root,
            &["apply", "--check", "--binary"],
            Some(&patch),
            "verify worktree patch",
        )?;
        git_checked(
            &lifecycle.source_repo_root,
            &["apply", "--binary"],
            Some(&patch),
            "apply worktree patch",
        )?;
    }
    for path in paths {
        copy_untracked_path(&lifecycle.worktree_root, &lifecycle.source_repo_root, &path)?;
    }
    Ok(())
}

fn untracked_paths(worktree_root: &Path) -> Result<Vec<String>> {
    let output = git_bytes(
        worktree_root,
        &[
            "status",
            "--porcelain=v1",
            "-z",
            "--untracked-files=all",
            "--",
        ],
        None,
        "read worktree status",
    )?;
    Ok(parse_git_status_entries(&output)
        .into_iter()
        .filter(|entry| entry.status == "??")
        .map(|entry| entry.path)
        .collect())
}

fn ensure_untracked_destinations_available(
    worktree_root: &Path,
    source_repo_root: &Path,
    paths: &[String],
) -> Result<()> {
    for path in paths {
        safe_repo_path(&worktree_root.display().to_string(), path)?;
        let destination = safe_repo_path(&source_repo_root.display().to_string(), path)?;
        if destination.exists() || destination.is_symlink() {
            bail!("source path already exists while merging untracked file: {path}");
        }
    }
    Ok(())
}

fn copy_untracked_path(worktree_root: &Path, source_repo_root: &Path, path: &str) -> Result<()> {
    let source = safe_repo_path(&worktree_root.display().to_string(), path)?;
    let destination = safe_repo_path(&source_repo_root.display().to_string(), path)?;
    if destination.exists() || destination.is_symlink() {
        bail!("source path already exists while merging untracked file: {path}");
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    if source.is_symlink() {
        let target = fs::read_link(source)?;
        std::os::unix::fs::symlink(target, destination)?;
    } else if source.is_dir() {
        copy_dir_recursive(&source, &destination)?;
    } else {
        fs::copy(source, destination)?;
    }
    Ok(())
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let dest_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir_recursive(&source_path, &dest_path)?;
        } else if source_path.is_symlink() {
            std::os::unix::fs::symlink(fs::read_link(source_path)?, dest_path)?;
        } else {
            fs::copy(source_path, dest_path)?;
        }
    }
    Ok(())
}

pub(super) fn cleanup_worktree(lifecycle: &WorktreeLifecycle) -> Result<()> {
    git_checked(
        &lifecycle.source_repo_root,
        &[
            "worktree",
            "remove",
            "--force",
            &lifecycle.worktree_root.display().to_string(),
        ],
        None,
        "remove recorded worktree",
    )?;
    git_checked(
        &lifecycle.source_repo_root,
        &["branch", "-D", &lifecycle.branch],
        None,
        "delete recorded branch",
    )
}
