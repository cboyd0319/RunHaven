use std::path::PathBuf;

use anyhow::Result;

use crate::validators::validate_run_id;

pub fn runhaven_cache_root() -> PathBuf {
    std::env::var_os("RUNHAVEN_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join("Library").join("Caches").join("runhaven"))
}

pub fn runs_log_path() -> PathBuf {
    runhaven_cache_root().join("runs.jsonl")
}

pub fn egress_policy_log_path() -> PathBuf {
    runhaven_cache_root().join("egress-policy.jsonl")
}

pub fn auth_broker_log_path() -> PathBuf {
    runhaven_cache_root().join("auth-broker.jsonl")
}

pub fn active_runs_dir() -> PathBuf {
    runhaven_cache_root().join("active-runs")
}

pub fn worktrees_dir() -> PathBuf {
    runhaven_cache_root().join("worktrees")
}

pub fn active_run_path(run_id: &str) -> Result<PathBuf> {
    validate_run_id(run_id)?;
    Ok(active_runs_dir().join(format!("{run_id}.json")))
}

pub fn state_lock_path(state_volume: &str) -> PathBuf {
    runhaven_cache_root()
        .join("locks")
        .join(format!("{state_volume}.lock"))
}

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
