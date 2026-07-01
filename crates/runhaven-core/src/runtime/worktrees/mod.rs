use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

mod commands;
mod lifecycle;
mod merge;

pub use commands::{
    runs_worktree_discard, runs_worktree_keep, runs_worktree_merge, runs_worktree_recover,
    worktree_recovery_commands, worktree_recovery_payload,
};
pub use lifecycle::{WorktreeLifecycle, load_worktree_lifecycle, verify_lifecycle};
pub use merge::ensure_source_ready_for_merge;

use lifecycle::{git_bytes, git_checked, git_status_lines, git_stdout};

use crate::runtime::plans::{
    WorkspaceScope, WorktreeRun, apply_workspace_scope, validate_workspace,
};
use crate::support::git::{GitSnapshot, capture_git_snapshot, git_head, git_repo_root};
use crate::support::paths::worktrees_dir;
use crate::support::shell;

pub fn preview_worktree(
    workspace: &Path,
    workspace_scope: WorkspaceScope,
    allow_sensitive_workspace: bool,
) -> Result<(PathBuf, PathBuf, String)> {
    let source_workspace =
        resolve_worktree_source(workspace, workspace_scope, allow_sensitive_workspace)?;
    let repo_root = require_git_repo_root(&source_workspace)?;
    let base_head = require_git_head(&repo_root)?;
    ensure_clean_source_repo(&repo_root)?;
    Ok((source_workspace, repo_root, base_head))
}

pub fn create_worktree_for_run(
    workspace: &Path,
    workspace_scope: WorkspaceScope,
    allow_sensitive_workspace: bool,
    profile_name: &str,
    run_id: &str,
) -> Result<WorktreeRun> {
    let (source_workspace, repo_root, base_head) =
        preview_worktree(workspace, workspace_scope, allow_sensitive_workspace)?;
    let worktree_root = worktrees_dir().join(run_id);
    if let Some(parent) = worktree_root.parent() {
        fs::create_dir_all(parent)?;
    }
    let branch = format!("runhaven/{profile_name}/{run_id}");
    let output = Command::new("git")
        .args(["-C"])
        .arg(&repo_root)
        .args(["worktree", "add", "-b", &branch])
        .arg(&worktree_root)
        .arg(&base_head)
        .output()?;
    if !output.status.success() {
        if worktree_root.is_dir() && fs::read_dir(&worktree_root)?.next().is_none() {
            let _ = fs::remove_dir(&worktree_root);
        }
        let detail = String::from_utf8_lossy(if output.stderr.is_empty() {
            &output.stdout
        } else {
            &output.stderr
        })
        .trim()
        .to_string();
        bail!(
            "could not create git worktree: {}",
            if detail.is_empty() {
                output.status.to_string()
            } else {
                detail
            }
        );
    }
    let mounted_workspace = worktree_root.join(
        source_workspace
            .strip_prefix(&repo_root)
            .unwrap_or(Path::new("")),
    );
    fs::create_dir_all(&mounted_workspace)?;
    Ok(WorktreeRun {
        source_workspace,
        source_repo_root: repo_root.clone(),
        worktree_root: worktree_root.canonicalize()?,
        mounted_workspace: mounted_workspace.canonicalize()?,
        branch: branch.clone(),
        base_head: Some(base_head),
        recovery_commands: recovery_commands(&repo_root, &worktree_root, &branch),
    })
}

pub fn resolve_worktree_source(
    workspace: &Path,
    workspace_scope: WorkspaceScope,
    allow_sensitive_workspace: bool,
) -> Result<PathBuf> {
    let resolved = workspace
        .canonicalize()
        .with_context(|| format!("could not resolve workspace path: {}", workspace.display()))?;
    if !resolved.exists() {
        bail!("workspace does not exist: {}", resolved.display());
    }
    if !resolved.is_dir() {
        bail!("workspace is not a directory: {}", resolved.display());
    }
    let (resolved, _) = apply_workspace_scope(&resolved, workspace_scope)?;
    validate_workspace(&resolved, allow_sensitive_workspace)?;
    Ok(resolved)
}

pub fn require_git_repo_root(workspace: &Path) -> Result<PathBuf> {
    let (repo_root, reason) = git_repo_root(workspace);
    let Some(repo_root) = repo_root else {
        bail!("--worktree requires a git worktree: {reason}");
    };
    Ok(PathBuf::from(repo_root).canonicalize()?)
}

pub fn require_git_head(repo_root: &Path) -> Result<String> {
    git_head(repo_root).ok_or_else(|| {
        anyhow::anyhow!("--worktree requires a git repository with a committed HEAD")
    })
}

pub fn ensure_clean_source_repo(repo_root: &Path) -> Result<()> {
    match capture_git_snapshot(repo_root) {
        GitSnapshot::Unavailable { reason, .. } => bail!(
            "could not inspect source git worktree{}",
            if reason.is_empty() {
                String::new()
            } else {
                format!(": {reason}")
            }
        ),
        GitSnapshot::Available { dirty: true, .. } => bail!(
            "--worktree requires a clean source git worktree.\nOptions:\n1. Commit or stash source changes, then retry with --worktree.\n2. Run without --worktree to use the source checkout directly.\n3. Start from a clean clone or git worktree if you want isolation."
        ),
        GitSnapshot::Available { .. } => Ok(()),
    }
}

pub fn recovery_commands(
    repo_root: &Path,
    worktree_root: &Path,
    branch: &str,
) -> Vec<(String, String)> {
    let repo = shell::quote(&repo_root.display().to_string());
    let worktree = shell::quote(&worktree_root.display().to_string());
    let branch = shell::quote(branch);
    vec![
        (
            "status".to_string(),
            format!("git -C {worktree} status --short"),
        ),
        ("diff".to_string(), format!("git -C {worktree} diff HEAD")),
        ("merge".to_string(), format!("git -C {repo} merge {branch}")),
        (
            "remove_worktree".to_string(),
            format!("git -C {repo} worktree remove {worktree}"),
        ),
        (
            "delete_branch".to_string(),
            format!("git -C {repo} branch -D {branch}"),
        ),
    ]
}

pub fn worktree_record(worktree: &WorktreeRun) -> Value {
    json!({
        "source_workspace": worktree.source_workspace.display().to_string(),
        "source_repo_root": worktree.source_repo_root.display().to_string(),
        "worktree_root": worktree.worktree_root.display().to_string(),
        "mounted_workspace": worktree.mounted_workspace.display().to_string(),
        "branch": worktree.branch,
        "base_head": worktree.base_head,
        "recovery_commands": worktree.recovery_commands.iter().cloned().collect::<std::collections::BTreeMap<_, _>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn git(repo: &Path, args: &[&str]) {
        let status = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .status()
            .expect("git available");
        assert!(status.success(), "git {args:?} failed");
    }

    #[test]
    fn ensure_clean_source_repo_accepts_clean_and_rejects_dirty() {
        let dir = tempfile::tempdir().expect("tempdir");
        let repo = dir.path();
        git(repo, &["init", "-q"]);
        git(repo, &["config", "user.email", "test@example.com"]);
        git(repo, &["config", "user.name", "test"]);
        std::fs::write(repo.join("seed.txt"), "base").expect("write seed");
        git(repo, &["add", "seed.txt"]);
        git(repo, &["commit", "-qm", "seed"]);
        let repo_root = repo.canonicalize().expect("canonicalize");

        ensure_clean_source_repo(&repo_root).expect("clean repo must be accepted");

        std::fs::write(repo_root.join("dirty.txt"), "x").expect("write dirty");
        let error = ensure_clean_source_repo(&repo_root).expect_err("dirty repo must be rejected");
        assert!(error.to_string().contains("clean source git worktree"));
    }
}
