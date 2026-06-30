use std::io::Write;

use anyhow::{Result, bail};
use serde_json::{Value, json};

use crate::provider::auth_broker::BrokerDecision;
use crate::provider::egress::ProxyDecision;
mod diff;
mod log;

pub use diff::{run_diff_text, runs_diff};
pub use log::runs_log;

use crate::runtime::plans::AgentRunPlan;
use crate::runtime::worktrees::worktree_record;
use crate::support::git::{GitChange, git_value_available};
use crate::support::paths::{open_private_append, runs_log_path};

use super::{read_jsonl, read_jsonl_tail_bounded};

pub struct RunRecordInput<'a> {
    pub plan: &'a AgentRunPlan,
    pub run_id: &'a str,
    pub started_at: &'a str,
    pub finished_at: &'a str,
    pub return_code: i32,
    pub status: Option<&'a str>,
    pub provider_decisions: &'a [ProxyDecision],
    pub auth_decisions: Option<&'a [BrokerDecision]>,
    pub cleanup: Value,
    pub git: GitChange,
}

pub fn write_run_record(input: RunRecordInput<'_>) -> Result<()> {
    let path = runs_log_path();
    let mut payload = json!({
        "timestamp": input.finished_at,
        "started_at": input.started_at,
        "finished_at": input.finished_at,
        "run_id": input.run_id,
        "profile": input.plan.profile_name,
        "workspace": input.plan.workspace.display().to_string(),
        "workspace_scope": input.plan.workspace_scope.as_str(),
        "state_volume": input.plan.state_volume,
        "session": input.plan.session,
        "network": input.plan.network_mode.as_str(),
        "status": input.status.unwrap_or(if input.return_code == 0 { "succeeded" } else { "failed" }),
        "return_code": input.return_code,
        "provider_policy": summarize_provider_policy(input.provider_decisions),
        "auth_broker": summarize_auth_broker(input.auth_decisions),
        "cleanup": input.cleanup,
        "git": input.git,
    });
    if let Some(worktree) = &input.plan.worktree {
        payload["worktree"] = worktree_record(worktree);
    }
    let mut file = open_private_append(&path)?;
    writeln!(file, "{}", serde_json::to_string(&payload)?)?;
    Ok(())
}

pub fn summarize_provider_policy(decisions: &[ProxyDecision]) -> Value {
    json!({
        "entries": decisions.len(),
        "allowed": decisions.iter().filter(|decision| decision.decision == "allowed").map(|decision| decision.count).sum::<usize>(),
        "denied": decisions.iter().filter(|decision| decision.decision == "denied").map(|decision| decision.count).sum::<usize>(),
    })
}

pub fn summarize_auth_broker(decisions: Option<&[BrokerDecision]>) -> Value {
    let Some(decisions) = decisions else {
        return json!({"broker": null, "entries": 0, "allowed": 0, "denied": 0, "no_requests": false});
    };
    json!({
        "broker": "codex-api-key",
        "entries": decisions.len(),
        "allowed": decisions.iter().filter(|decision| decision.decision == "allowed").map(|decision| decision.count).sum::<usize>(),
        "denied": decisions.iter().filter(|decision| decision.decision == "denied").map(|decision| decision.count).sum::<usize>(),
        "no_requests": decisions.is_empty(),
    })
}

pub fn runs_list(limit: usize, json_output: bool) -> Result<i32> {
    let records = read_run_records(limit)?;
    if json_output {
        println!("{}", serde_json::to_string_pretty(&records)?);
        return Ok(0);
    }
    if records.is_empty() {
        println!("No RunHaven run records found.");
        return Ok(0);
    }
    for record in records {
        let provider_denied = record
            .pointer("/provider_policy/denied")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let auth_denied = record
            .pointer("/auth_broker/denied")
            .and_then(Value::as_u64)
            .unwrap_or(0);
        let cleanup = record
            .pointer("/cleanup/provider_network")
            .and_then(Value::as_str)
            .unwrap_or("-");
        println!(
            "{}  {}  {}  {}  return={}  provider_denied={}  auth_denied={}  cleanup={}  run={}",
            record
                .get("timestamp")
                .and_then(Value::as_str)
                .unwrap_or("<unknown>"),
            record
                .get("profile")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            record
                .get("network")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            record
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("unknown"),
            record
                .get("return_code")
                .map(Value::to_string)
                .unwrap_or_else(|| "-".to_string()),
            provider_denied,
            auth_denied,
            cleanup,
            record.get("run_id").and_then(Value::as_str).unwrap_or("-"),
        );
    }
    Ok(0)
}

pub fn runs_show(run_id: &str, json_output: bool) -> Result<i32> {
    let record = find_run_record(run_id)?;
    if json_output {
        println!("{}", serde_json::to_string_pretty(&record)?);
        return Ok(0);
    }
    print_run_record(&record);
    Ok(0)
}

pub fn print_run_record(record: &Value) {
    println!(
        "Run id: {}",
        record.get("run_id").and_then(Value::as_str).unwrap_or("-")
    );
    println!(
        "Started: {}",
        record
            .get("started_at")
            .and_then(Value::as_str)
            .unwrap_or("-")
    );
    println!(
        "Finished: {}",
        record
            .get("finished_at")
            .and_then(Value::as_str)
            .unwrap_or("-")
    );
    println!(
        "Profile: {}",
        record
            .get("profile")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    );
    println!(
        "Workspace: {}",
        record
            .get("workspace")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    );
    if let Some(scope) = record.get("workspace_scope").and_then(Value::as_str) {
        println!("Workspace scope: {scope}");
    }
    if let Some(session) = record.get("session").and_then(Value::as_str) {
        println!("Session: {session}");
    }
    if let Some(volume) = record.get("state_volume").and_then(Value::as_str) {
        println!("State volume: {volume}");
    }
    if let Some(worktree) = record.get("worktree").and_then(Value::as_object) {
        println!(
            "Worktree: {}",
            worktree
                .get("worktree_root")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
        );
        println!(
            "Worktree branch: {}",
            worktree
                .get("branch")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
        );
    }
    println!(
        "Network: {}",
        record
            .get("network")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    );
    println!(
        "Status: {}",
        record
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
    );
    println!(
        "Return code: {}",
        record
            .get("return_code")
            .map(Value::to_string)
            .unwrap_or_else(|| "-".to_string())
    );
    if let Some(git) = record.get("git") {
        println!("{}", format_git_summary(git));
    }
    if let Some(policy) = record.get("provider_policy") {
        println!(
            "Provider policy: entries={} allowed={} denied={}",
            policy.get("entries").and_then(Value::as_u64).unwrap_or(0),
            policy.get("allowed").and_then(Value::as_u64).unwrap_or(0),
            policy.get("denied").and_then(Value::as_u64).unwrap_or(0),
        );
    }
    if let Some(auth) = record.get("auth_broker") {
        println!(
            "Auth broker: broker={} entries={} allowed={} denied={} no_requests={}",
            auth.get("broker").and_then(Value::as_str).unwrap_or("-"),
            auth.get("entries").and_then(Value::as_u64).unwrap_or(0),
            auth.get("allowed").and_then(Value::as_u64).unwrap_or(0),
            auth.get("denied").and_then(Value::as_u64).unwrap_or(0),
            auth.get("no_requests")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        );
    }
    if let Some(cleanup) = record.get("cleanup") {
        println!(
            "Cleanup provider network: {}",
            cleanup
                .get("provider_network")
                .and_then(Value::as_str)
                .unwrap_or("-")
        );
    }
}

pub fn format_git_summary(git: &Value) -> String {
    if !git_value_available(git) {
        let reason = git
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        return format!("Git: unavailable ({reason})");
    }
    let before_head = short_git_head(git.pointer("/before/head").and_then(Value::as_str));
    let after_head = short_git_head(git.pointer("/after/head").and_then(Value::as_str));
    let changed = git.get("changed").and_then(Value::as_bool).unwrap_or(false);
    let files = git
        .pointer("/after/changed_count")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    format!("Git: changed={changed} before={before_head} after={after_head} files={files}")
}

fn short_git_head(head: Option<&str>) -> String {
    head.filter(|value| !value.is_empty())
        .map(|value| value.chars().take(7).collect())
        .unwrap_or_else(|| "-".to_string())
}

pub fn find_run_record(run_id: &str) -> Result<Value> {
    for record in read_run_records(0)?.into_iter().rev() {
        if record.get("run_id").and_then(Value::as_str) == Some(run_id) {
            return Ok(record);
        }
    }
    bail!("run record not found: {run_id}");
}

pub fn read_run_records(limit: usize) -> Result<Vec<Value>> {
    read_jsonl(&runs_log_path(), limit)
}

pub fn read_run_records_tail_bounded(limit: usize, max_tail_bytes: u64) -> Result<Vec<Value>> {
    read_jsonl_tail_bounded(&runs_log_path(), limit, max_tail_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::paths::{ensure_private_parent, override_cache_root_for_tests};

    #[test]
    fn run_records_tail_bounded_keeps_recent_records() {
        let cache = tempfile::tempdir().expect("cache");
        let _cache_home = override_cache_root_for_tests(cache.path());
        ensure_private_parent(&runs_log_path()).expect("runs log parent");
        let mut file = std::fs::File::create(runs_log_path()).expect("runs log file");
        for idx in 0..20 {
            writeln!(
                file,
                "{}",
                json!({
                    "run_id": format!("run-{idx:03}"),
                    "timestamp": format!("2026-06-30T00:{idx:02}:00Z")
                })
            )
            .expect("write run record");
        }

        let records = read_run_records_tail_bounded(3, 2048).expect("tail records");

        let run_ids = records
            .iter()
            .map(|record| {
                record
                    .get("run_id")
                    .and_then(Value::as_str)
                    .expect("run id")
                    .to_string()
            })
            .collect::<Vec<_>>();
        assert_eq!(run_ids, vec!["run-017", "run-018", "run-019"]);
    }
}
