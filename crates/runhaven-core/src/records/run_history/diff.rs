use std::path::Path;

use anyhow::{Result, bail};
use serde_json::Value;

use super::find_run_record;
use crate::support::git::{
    capture_git_snapshot, git_snapshot_paths, git_value_available, run_git_diff,
};
use crate::support::validators::require_string;

pub fn runs_diff(run_id: &str) -> Result<i32> {
    print!("{}", run_diff_text(run_id)?);
    Ok(0)
}

pub fn run_diff_text(run_id: &str) -> Result<String> {
    let record = find_run_record(run_id)?;
    let git = record
        .get("git")
        .ok_or_else(|| anyhow::anyhow!("git metadata is unavailable for run {run_id}"))?;
    if !git_value_available(git) {
        let reason = git.get("reason").and_then(Value::as_str).unwrap_or("");
        bail!(
            "git metadata is unavailable for run {run_id}{}",
            if reason.is_empty() {
                String::new()
            } else {
                format!(": {reason}")
            }
        );
    }
    let before = git
        .get("before")
        .ok_or_else(|| anyhow::anyhow!("run git metadata is missing before snapshot"))?;
    let after = git
        .get("after")
        .ok_or_else(|| anyhow::anyhow!("run git metadata is missing after snapshot"))?;
    let repo_root = require_string(
        git.get("repo_root"),
        "run git metadata is missing repo root",
    )?;
    let workspace = require_string(record.get("workspace"), "run record is missing workspace")?;
    let before_head = require_string(
        before.get("head"),
        "run git metadata is missing a base HEAD; refusing live diff",
    )?;
    let after_head = require_string(
        after.get("head"),
        "run git metadata is missing recorded HEAD; refusing live diff",
    )?;
    let after_dirty = after.get("dirty").and_then(Value::as_bool).unwrap_or(false);
    let after_paths = git_snapshot_paths(after)?;
    if after.get("truncated").and_then(Value::as_bool) == Some(true) {
        bail!("run git path list is truncated; refusing live diff");
    }
    if !Path::new(repo_root).is_dir() {
        bail!("recorded git repo no longer exists; refusing live diff");
    }
    if !Path::new(workspace).exists() {
        bail!("recorded workspace no longer exists; refusing live diff");
    }
    let current = serde_json::to_value(capture_git_snapshot(Path::new(workspace)))?;
    if !git_value_available(&current) {
        bail!("recorded workspace is no longer a git worktree; refusing live diff");
    }
    if current.get("repo_root").and_then(Value::as_str) != Some(repo_root) {
        bail!("workspace git repo no longer matches the recorded run; refusing live diff");
    }
    if current.get("head").and_then(Value::as_str) != Some(after_head) {
        bail!("git HEAD changed since the recorded run; refusing live diff");
    }
    if after_dirty {
        eprintln!(
            "runhaven: showing a live working tree diff; RunHaven verified the recorded HEAD and path set, not file contents since the run."
        );
    }
    if !after_dirty && before_head == after_head {
        return Ok("No git changes recorded for this run.\n".to_string());
    }
    let mut output = String::new();
    let mut printed = false;
    if before_head != after_head {
        let diff = run_git_diff(
            &[
                "git",
                "-C",
                repo_root,
                "diff",
                "--no-ext-diff",
                "--no-color",
                before_head,
                after_head,
            ]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>(),
        )?;
        if !diff.is_empty() {
            output.push_str(&ensure_newline(&diff));
            printed = true;
        }
    }
    if after_dirty && !after_paths.is_empty() {
        let mut command = vec![
            "git".to_string(),
            "-C".to_string(),
            repo_root.to_string(),
            "diff".to_string(),
            "--no-ext-diff".to_string(),
            "--no-color".to_string(),
            after_head.to_string(),
            "--".to_string(),
        ];
        command.extend(after_paths);
        let diff = run_git_diff(&command)?;
        if !diff.is_empty() {
            output.push_str(&ensure_newline(&diff));
            printed = true;
        }
    }
    if !printed {
        output.push_str("No git diff output for recorded changes.\n");
    }
    Ok(output)
}

fn ensure_newline(value: &str) -> String {
    if value.ends_with('\n') {
        value.to_string()
    } else {
        format!("{value}\n")
    }
}
