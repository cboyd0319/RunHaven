from __future__ import annotations

import os
from pathlib import Path

from .validators import validate_run_id


def runhaven_cache_root() -> Path:
    override = os.environ.get("RUNHAVEN_CACHE_HOME")
    if override:
        return Path(override)
    return Path.home() / "Library" / "Caches" / "runhaven"


def runs_log_path() -> Path:
    return runhaven_cache_root() / "runs.jsonl"


def egress_policy_log_path() -> Path:
    return runhaven_cache_root() / "egress-policy.jsonl"


def auth_broker_log_path() -> Path:
    return runhaven_cache_root() / "auth-broker.jsonl"


def active_runs_dir() -> Path:
    return runhaven_cache_root() / "active-runs"


def worktrees_dir() -> Path:
    return runhaven_cache_root() / "worktrees"


def active_run_path(run_id: str) -> Path:
    validate_run_id(run_id)
    return active_runs_dir() / f"{run_id}.json"


def state_lock_path(state_volume: str) -> Path:
    return runhaven_cache_root() / "locks" / f"{state_volume}.lock"
