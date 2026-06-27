use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

pub const GIT_STATUS_PATH_LIMIT: usize = 100;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitStatusEntry {
    pub path: String,
    pub status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "available")]
pub enum GitSnapshot {
    #[serde(rename = "true")]
    Available {
        repo_root: String,
        head: Option<String>,
        dirty: bool,
        changed_count: usize,
        paths: Vec<String>,
        truncated: bool,
    },
    #[serde(rename = "false")]
    Unavailable {
        reason: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        repo_root: Option<String>,
    },
}

pub fn capture_git_snapshot(workspace: &Path) -> GitSnapshot {
    let (repo_root, unavailable_reason) = git_repo_root(workspace);
    let Some(repo_root) = repo_root else {
        return GitSnapshot::Unavailable {
            reason: unavailable_reason,
            repo_root: None,
        };
    };
    let head = git_head(Path::new(&repo_root));
    let Some(entries) = read_git_status_entries(Path::new(&repo_root), workspace) else {
        return GitSnapshot::Unavailable {
            reason: "git-status-failed".to_string(),
            repo_root: Some(repo_root),
        };
    };
    let paths = entries
        .into_iter()
        .map(|entry| entry.path)
        .collect::<Vec<_>>();
    GitSnapshot::Available {
        repo_root,
        head,
        dirty: !paths.is_empty(),
        changed_count: paths.len(),
        paths: paths.iter().take(GIT_STATUS_PATH_LIMIT).cloned().collect(),
        truncated: paths.len() > GIT_STATUS_PATH_LIMIT,
    }
}

pub fn read_git_status_entries(repo_root: &Path, workspace: &Path) -> Option<Vec<GitStatusEntry>> {
    let resolved_workspace = workspace.canonicalize().ok()?;
    let output = Command::new("git")
        .args([
            "-C",
            repo_root.to_str()?,
            "status",
            "--porcelain=v1",
            "-z",
            "--",
        ])
        .arg(resolved_workspace)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(parse_git_status_entries(&output.stdout))
}

pub fn git_repo_root(workspace: &Path) -> (Option<String>, String) {
    let output = Command::new("git")
        .args(["-C"])
        .arg(workspace)
        .args(["rev-parse", "--show-toplevel"])
        .output();
    let Ok(output) = output else {
        return (None, "git-not-found".to_string());
    };
    if !output.status.success() {
        return (None, "not-a-git-worktree".to_string());
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        return (None, "not-a-git-worktree".to_string());
    }
    let resolved = PathBuf::from(text)
        .canonicalize()
        .unwrap_or_else(|_| workspace.to_path_buf());
    (Some(resolved.display().to_string()), String::new())
}

pub fn git_head(repo_root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["-C"])
        .arg(repo_root)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let head = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!head.is_empty()).then_some(head)
}

pub fn parse_git_status_entries(output: &[u8]) -> Vec<GitStatusEntry> {
    let entries = output
        .split(|b| *b == 0)
        .filter(|entry| !entry.is_empty())
        .collect::<Vec<_>>();
    let mut parsed = BTreeMap::new();
    let mut index = 0;
    while index < entries.len() {
        let entry = entries[index];
        if entry.len() >= 4 {
            let status = String::from_utf8_lossy(&entry[..2]).to_string();
            let path = String::from_utf8_lossy(&entry[3..]).to_string();
            parsed.insert(
                path.clone(),
                GitStatusEntry {
                    path,
                    status: status.clone(),
                },
            );
            if status.contains('R') || status.contains('C') {
                index += 1;
            }
        }
        index += 1;
    }
    parsed.into_values().collect()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "available")]
pub enum GitChange {
    #[serde(rename = "true")]
    Available {
        repo_root: String,
        changed: bool,
        before: GitSnapshotRecord,
        after: GitSnapshotRecord,
    },
    #[serde(rename = "false")]
    Unavailable {
        reason: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        repo_root: Option<String>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GitSnapshotRecord {
    pub head: Option<String>,
    pub dirty: bool,
    pub changed_count: usize,
    pub paths: Vec<String>,
    pub truncated: bool,
}

pub fn summarize_git_change(before: GitSnapshot, after: GitSnapshot) -> GitChange {
    match (before, after) {
        (
            GitSnapshot::Available {
                repo_root: before_root,
                head: before_head,
                dirty: before_dirty,
                changed_count: before_count,
                paths: before_paths,
                truncated: before_truncated,
            },
            GitSnapshot::Available {
                repo_root,
                head,
                dirty,
                changed_count,
                paths,
                truncated,
            },
        ) => {
            let before = GitSnapshotRecord {
                head: before_head,
                dirty: before_dirty,
                changed_count: before_count,
                paths: before_paths,
                truncated: before_truncated,
            };
            let after = GitSnapshotRecord {
                head,
                dirty,
                changed_count,
                paths,
                truncated,
            };
            GitChange::Available {
                repo_root: repo_root.clone(),
                changed: before != after || before_root != repo_root,
                before,
                after,
            }
        }
        (GitSnapshot::Unavailable { reason, repo_root }, _) => {
            GitChange::Unavailable { reason, repo_root }
        }
        (_, GitSnapshot::Unavailable { reason, repo_root }) => {
            GitChange::Unavailable { reason, repo_root }
        }
    }
}

/// True when a serialized `GitSnapshot` or `GitChange` reports availability.
///
/// The `available` field is the serde enum tag, which serializes as the string
/// "true"/"false", not a JSON boolean. Read it as a string, never `as_bool`.
pub fn git_value_available(value: &serde_json::Value) -> bool {
    value.get("available").and_then(serde_json::Value::as_str) == Some("true")
}

pub fn git_snapshot_paths(value: &serde_json::Value) -> Result<Vec<String>> {
    let paths = value
        .get("paths")
        .and_then(serde_json::Value::as_array)
        .context("run git metadata has invalid path list")?;
    let mut values = Vec::with_capacity(paths.len());
    for path in paths {
        let Some(path) = path.as_str() else {
            bail!("run git metadata has invalid path list");
        };
        values.push(path.to_string());
    }
    values.sort();
    Ok(values)
}

pub fn run_git_diff(command: &[String]) -> Result<String> {
    let Some((program, args)) = command.split_first() else {
        bail!("git diff is unavailable");
    };
    let output = Command::new(program)
        .args(args)
        .output()
        .context("git diff is unavailable")?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        bail!(
            "git diff failed: {}",
            if detail.is_empty() {
                output.status.to_string()
            } else {
                detail
            }
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn run_untracked_git_diff(repo_root: &str, path: &str) -> Result<String> {
    let full_path = safe_repo_path(repo_root, path)?;
    let command = vec![
        "git".to_string(),
        "-C".to_string(),
        repo_root.to_string(),
        "diff".to_string(),
        "--no-ext-diff".to_string(),
        "--no-color".to_string(),
        "--no-index".to_string(),
        "--".to_string(),
        "/dev/null".to_string(),
        full_path.display().to_string(),
    ];
    let output = Command::new(&command[0])
        .args(&command[1..])
        .output()
        .context("git diff is unavailable")?;
    if !matches!(output.status.code(), Some(0 | 1)) {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        bail!(
            "git diff failed: {}",
            if detail.is_empty() {
                output.status.to_string()
            } else {
                detail
            }
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn safe_repo_path(repo_root: &str, path: &str) -> Result<PathBuf> {
    let root = Path::new(repo_root).canonicalize()?;
    let full_path = root
        .join(path)
        .canonicalize()
        .unwrap_or_else(|_| root.join(path));
    if !full_path.starts_with(&root) {
        bail!("git path escapes the recorded repository; refusing live diff");
    }
    Ok(full_path)
}
