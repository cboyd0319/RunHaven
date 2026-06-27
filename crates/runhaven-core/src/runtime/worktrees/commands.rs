use std::collections::BTreeMap;

use anyhow::{Result, bail};
use serde_json::{Value, json};

use super::{
    WorktreeLifecycle, git_status_lines, git_stdout, load_worktree_lifecycle,
    merge::cleanup_worktree, merge::format_merge_recovery, merge::merge_worktree_changes,
    verify_lifecycle,
};
use crate::support::project_checks::{SuggestedCheck, suggest_project_checks};
use crate::support::shell;

pub fn runs_worktree_keep(run_id: &str) -> Result<i32> {
    let lifecycle = load_worktree_lifecycle(run_id)?;
    verify_lifecycle(&lifecycle)?;
    let suggested_checks = suggest_project_checks(&lifecycle.mounted_workspace);
    println!("Worktree kept for run {run_id}");
    println!("Source repo: {}", lifecycle.source_repo_root.display());
    println!("Worktree: {}", lifecycle.worktree_root.display());
    println!(
        "Mounted workspace: {}",
        lifecycle.mounted_workspace.display()
    );
    println!("Branch: {}", lifecycle.branch);
    println!("Review: runhaven runs diff {run_id}");
    println!("Recover: runhaven runs recover {run_id}");
    println!("Merge: runhaven runs merge {run_id}");
    println!("Discard: runhaven runs discard {run_id}");
    print_suggested_checks(&suggested_checks);
    Ok(0)
}

pub fn runs_worktree_recover(run_id: &str, json_output: bool) -> Result<i32> {
    let lifecycle = load_worktree_lifecycle(run_id)?;
    verify_lifecycle(&lifecycle)?;
    let payload = worktree_recovery_payload(&lifecycle)?;
    if json_output {
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(0);
    }
    println!("Manual recovery for worktree run {run_id}");
    println!("Source repo: {}", lifecycle.source_repo_root.display());
    println!("Worktree: {}", lifecycle.worktree_root.display());
    println!(
        "Mounted workspace: {}",
        lifecycle.mounted_workspace.display()
    );
    println!("Branch: {}", lifecycle.branch);
    println!("Base HEAD: {}", lifecycle.base_head);
    println!(
        "Source HEAD: {}",
        payload
            .get("source_head")
            .and_then(Value::as_str)
            .unwrap_or("-")
    );
    println!(
        "Worktree HEAD: {}",
        payload
            .get("worktree_head")
            .and_then(Value::as_str)
            .unwrap_or("-")
    );
    print_status(
        "Source status",
        &git_status_lines(&lifecycle.source_repo_root)?,
    );
    print_status(
        "Worktree status",
        &git_status_lines(&lifecycle.worktree_root)?,
    );
    let commands = worktree_recovery_commands(&lifecycle);
    println!("Manual recovery steps:");
    println!("1. Review recorded changes: {}", commands["diff"]);
    println!(
        "2. Inspect the source checkout: {}",
        commands["source_status"]
    );
    println!("   Commit, stash, or remove source-local changes before retrying.");
    println!("3. Inspect the worktree: {}", commands["worktree_status"]);
    println!("   Resolve conflicts or commit finished work in the worktree if needed.");
    println!("4. Retry guarded merge: {}", commands["merge"]);
    println!("5. Keep for manual review: {}", commands["keep"]);
    println!("6. Discard only after review: {}", commands["discard"]);
    print_suggested_checks(&suggest_project_checks(&lifecycle.mounted_workspace));
    Ok(0)
}

pub fn worktree_recovery_payload(lifecycle: &WorktreeLifecycle) -> Result<Value> {
    Ok(json!({
        "run_id": lifecycle.run_id,
        "source_repo_root": lifecycle.source_repo_root.display().to_string(),
        "worktree_root": lifecycle.worktree_root.display().to_string(),
        "mounted_workspace": lifecycle.mounted_workspace.display().to_string(),
        "branch": lifecycle.branch,
        "base_head": lifecycle.base_head,
        "source_head": git_stdout(&lifecycle.source_repo_root, &["rev-parse", "HEAD"])?,
        "worktree_head": git_stdout(&lifecycle.worktree_root, &["rev-parse", "HEAD"])?,
        "source_status": git_status_lines(&lifecycle.source_repo_root)?,
        "worktree_status": git_status_lines(&lifecycle.worktree_root)?,
        "commands": worktree_recovery_commands(lifecycle),
        "next_steps": ["Review recorded changes", "Inspect the source checkout", "Inspect the worktree", "Retry guarded merge", "Keep for manual review", "Discard only after review"],
        "suggested_checks": suggest_project_checks(&lifecycle.mounted_workspace),
    }))
}

pub fn worktree_recovery_commands(lifecycle: &WorktreeLifecycle) -> BTreeMap<String, String> {
    let source = shell::quote(&lifecycle.source_repo_root.display().to_string());
    let worktree = shell::quote(&lifecycle.worktree_root.display().to_string());
    let run_id = &lifecycle.run_id;
    [
        ("diff", format!("runhaven runs diff {run_id}")),
        ("recover", format!("runhaven runs recover {run_id}")),
        (
            "recover_json",
            format!("runhaven runs recover {run_id} --json"),
        ),
        ("source_status", format!("git -C {source} status --short")),
        (
            "worktree_status",
            format!("git -C {worktree} status --short"),
        ),
        ("merge", format!("runhaven runs merge {run_id}")),
        ("keep", format!("runhaven runs keep {run_id}")),
        ("discard", format!("runhaven runs discard {run_id}")),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_string(), value))
    .collect()
}

pub fn runs_worktree_merge(run_id: &str) -> Result<i32> {
    let lifecycle = load_worktree_lifecycle(run_id)?;
    verify_lifecycle(&lifecycle)?;
    if let Err(error) = merge_worktree_changes(&lifecycle) {
        bail!("{}", format_merge_recovery(&lifecycle, &error.to_string()));
    }
    cleanup_worktree(&lifecycle)?;
    println!("Merged worktree run {run_id}");
    println!("Source repo: {}", lifecycle.source_repo_root.display());
    Ok(0)
}

pub fn runs_worktree_discard(run_id: &str) -> Result<i32> {
    let lifecycle = load_worktree_lifecycle(run_id)?;
    verify_lifecycle(&lifecycle)?;
    cleanup_worktree(&lifecycle)?;
    println!("Discarded worktree run {run_id}");
    println!("Source repo: {}", lifecycle.source_repo_root.display());
    Ok(0)
}

fn print_status(title: &str, lines: &[String]) {
    println!("{title}:");
    if lines.is_empty() {
        println!("  clean");
    } else {
        for line in lines {
            println!("  {line}");
        }
    }
}

fn print_suggested_checks(checks: &[SuggestedCheck]) {
    if checks.is_empty() {
        return;
    }
    println!("Suggested checks:");
    for (index, check) in checks.iter().enumerate() {
        println!("{}. {}: {}", index + 1, check.label, check.command);
        println!("   {}", check.reason);
    }
}
