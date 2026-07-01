use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::records::find_run_record;
use crate::support::validators::{require_string, validate_run_id};

#[derive(Clone, Debug)]
pub struct WorktreeLifecycle {
    pub run_id: String,
    pub source_repo_root: PathBuf,
    pub worktree_root: PathBuf,
    pub mounted_workspace: PathBuf,
    pub branch: String,
    pub base_head: String,
}

pub fn load_worktree_lifecycle(run_id: &str) -> Result<WorktreeLifecycle> {
    validate_run_id(run_id)?;
    let record = find_run_record(run_id)?;
    let worktree = record
        .get("worktree")
        .ok_or_else(|| anyhow::anyhow!("run {run_id} is not a worktree run"))?;
    Ok(WorktreeLifecycle {
        run_id: run_id.to_string(),
        source_repo_root: record_path(worktree, "source_repo_root", "source repo root")?,
        worktree_root: record_path(worktree, "worktree_root", "worktree")?,
        mounted_workspace: record_path(worktree, "mounted_workspace", "mounted workspace")?,
        branch: require_string(
            worktree.get("branch"),
            "worktree run record is missing branch",
        )?
        .to_string(),
        base_head: require_string(
            worktree.get("base_head"),
            "worktree run record is missing base HEAD",
        )?
        .to_string(),
    })
}

fn record_path(worktree: &serde_json::Value, key: &str, label: &str) -> Result<PathBuf> {
    PathBuf::from(require_string(
        worktree.get(key),
        &format!("worktree run record is missing {label}"),
    )?)
    .canonicalize()
    .map_err(Into::into)
}

pub fn verify_lifecycle(lifecycle: &WorktreeLifecycle) -> Result<()> {
    verify_lifecycle_names(lifecycle)?;
    verify_lifecycle_paths(lifecycle)?;
    verify_lifecycle_git(lifecycle)
}

fn verify_lifecycle_names(lifecycle: &WorktreeLifecycle) -> Result<()> {
    if !lifecycle.branch.starts_with("runhaven/") {
        bail!("recorded branch is not RunHaven-owned; refusing worktree action");
    }
    if !lifecycle
        .branch
        .ends_with(&format!("/{}", lifecycle.run_id))
    {
        bail!("recorded branch does not match the run id; refusing worktree action");
    }
    if lifecycle
        .worktree_root
        .file_name()
        .and_then(|name| name.to_str())
        != Some(&lifecycle.run_id)
    {
        bail!("recorded worktree path does not match the run id; refusing action");
    }
    Ok(())
}

fn verify_lifecycle_paths(lifecycle: &WorktreeLifecycle) -> Result<()> {
    if !lifecycle.source_repo_root.is_dir() {
        bail!("recorded source repository no longer exists");
    }
    if !lifecycle.worktree_root.is_dir() {
        bail!("recorded worktree no longer exists");
    }
    if !lifecycle.mounted_workspace.exists() {
        bail!("recorded mounted workspace no longer exists");
    }
    if lifecycle.source_repo_root == lifecycle.worktree_root {
        bail!("recorded worktree matches source repository; refusing action");
    }
    Ok(())
}

fn verify_lifecycle_git(lifecycle: &WorktreeLifecycle) -> Result<()> {
    let source_root = PathBuf::from(git_stdout(
        &lifecycle.source_repo_root,
        &["rev-parse", "--show-toplevel"],
    )?)
    .canonicalize()?;
    if source_root != lifecycle.source_repo_root {
        bail!("recorded source repository does not match git toplevel");
    }
    let worktree_root = PathBuf::from(git_stdout(
        &lifecycle.worktree_root,
        &["rev-parse", "--show-toplevel"],
    )?)
    .canonicalize()?;
    if worktree_root != lifecycle.worktree_root {
        bail!("recorded worktree does not match git toplevel");
    }
    git_checked(
        &lifecycle.source_repo_root,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("refs/heads/{}", lifecycle.branch),
        ],
        None,
        "verify recorded branch",
    )?;
    let current_branch = git_stdout(&lifecycle.worktree_root, &["branch", "--show-current"])?;
    if current_branch != lifecycle.branch {
        bail!("recorded worktree is not on the recorded branch; refusing action");
    }
    git_checked(
        &lifecycle.source_repo_root,
        &[
            "merge-base",
            "--is-ancestor",
            &lifecycle.base_head,
            &lifecycle.branch,
        ],
        None,
        "verify recorded branch ancestry",
    )
}

pub fn git_stdout(cwd: &Path, args: &[&str]) -> Result<String> {
    let output = git_bytes(cwd, args, None, "run git command")?;
    Ok(String::from_utf8_lossy(&output).trim().to_string())
}

pub(super) fn git_status_lines(cwd: &Path) -> Result<Vec<String>> {
    let text = git_stdout(cwd, &["status", "--short"])?;
    Ok(text
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

pub(super) fn git_checked(
    cwd: &Path,
    args: &[&str],
    input: Option<&[u8]>,
    action: &str,
) -> Result<()> {
    git_bytes(cwd, args, input, action).map(|_| ())
}

pub(super) fn git_bytes(
    cwd: &Path,
    args: &[&str],
    input: Option<&[u8]>,
    action: &str,
) -> Result<Vec<u8>> {
    let mut command = Command::new("git");
    command.args(["-C"]).arg(cwd).args(args);
    if input.is_some() {
        command.stdin(std::process::Stdio::piped());
    }
    command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = command
        .spawn()
        .with_context(|| format!("{action} failed: git is unavailable"))?;
    if let Some(input) = input {
        use std::io::Write;
        child
            .stdin
            .as_mut()
            .context("git stdin missing")?
            .write_all(input)?;
    }
    let output = child.wait_with_output()?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        bail!(
            "{action} failed: {}",
            if detail.is_empty() {
                output.status.to_string()
            } else {
                detail
            }
        );
    }
    Ok(output.stdout)
}
