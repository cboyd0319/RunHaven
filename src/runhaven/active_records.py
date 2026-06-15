from __future__ import annotations

import json
import os
from datetime import UTC, datetime
from typing import Any

from .cache_paths import active_run_path, active_runs_dir
from .plans import AgentRunPlan
from .validators import validate_run_id, validate_runhaven_container_name
from .worktrees import worktree_record


def write_active_run_record(plan: AgentRunPlan, *, run_id: str, started_at: str) -> None:
    payload: dict[str, object] = {
        "timestamp": started_at,
        "run_id": run_id,
        "profile": plan.profile_name,
        "workspace": str(plan.workspace),
        "workspace_scope": plan.workspace_scope,
        "network": plan.network_mode,
        "status": "running",
        "container_name": plan.container_name,
        "state_volume": plan.state_volume,
        "network_name": plan.network_name,
        "host_pid": os.getpid(),
    }
    if plan.worktree is not None:
        payload["worktree"] = worktree_record(plan.worktree)
    write_active_run_payload(run_id, payload)


def write_active_run_payload(run_id: str, payload: dict[str, object]) -> None:
    path = active_run_path(run_id)
    path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
    temporary_path = path.with_suffix(".tmp")
    temporary_path.write_text(json.dumps(payload, sort_keys=True) + "\n", encoding="utf-8")
    temporary_path.chmod(0o600)
    temporary_path.replace(path)


def find_active_run_record(run_id: str) -> dict[str, Any]:
    path = active_run_path(run_id)
    if not path.exists():
        raise ValueError(f"active run not found: {run_id}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ValueError(f"active run record is invalid: {run_id}") from exc
    if not isinstance(payload, dict):
        raise ValueError(f"active run record is invalid: {run_id}")
    return payload


def mark_active_run_stop_requested(run_id: str, record: dict[str, Any]) -> None:
    updated = dict(record)
    updated["status"] = "stop-requested"
    updated["stop_requested_at"] = utc_timestamp()
    write_active_run_payload(run_id, updated)


def clear_active_run_stop_requested(run_id: str, record: dict[str, Any]) -> None:
    updated = dict(record)
    updated["status"] = "running"
    updated.pop("stop_requested_at", None)
    write_active_run_payload(run_id, updated)


def mark_active_run_kill_requested(run_id: str, record: dict[str, Any]) -> None:
    updated = dict(record)
    updated["status"] = "kill-requested"
    updated["kill_requested_at"] = utc_timestamp()
    write_active_run_payload(run_id, updated)


def clear_active_run_kill_requested(run_id: str, record: dict[str, Any]) -> None:
    updated = dict(record)
    updated["status"] = "running"
    updated.pop("kill_requested_at", None)
    write_active_run_payload(run_id, updated)


def active_run_terminal_status(run_id: str) -> str | None:
    try:
        record = find_active_run_record(run_id)
    except ValueError:
        return None
    if isinstance(record.get("kill_requested_at"), str):
        return "killed"
    if isinstance(record.get("stop_requested_at"), str):
        return "stopped"
    return None


def remove_active_run_record(run_id: str) -> None:
    try:
        active_run_path(run_id).unlink()
    except FileNotFoundError:
        pass


def read_active_run_records() -> list[dict[str, Any]]:
    active_dir = active_runs_dir()
    if not active_dir.exists():
        return []
    records: list[dict[str, Any]] = []
    for path in sorted(active_dir.glob("*.json")):
        try:
            payload = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            continue
        if not isinstance(payload, dict):
            continue
        run_id = payload.get("run_id")
        container_name = payload.get("container_name")
        if not isinstance(run_id, str) or not isinstance(container_name, str):
            continue
        try:
            validate_run_id(run_id)
            validate_runhaven_container_name(container_name)
        except ValueError:
            continue
        records.append(payload)
    return sorted(records, key=active_run_sort_key)


def active_run_sort_key(record: dict[str, Any]) -> tuple[str, str]:
    timestamp = record.get("timestamp")
    run_id = record.get("run_id")
    return (
        timestamp if isinstance(timestamp, str) else "",
        run_id if isinstance(run_id, str) else "",
    )


def utc_timestamp() -> str:
    return datetime.now(UTC).isoformat().replace("+00:00", "Z")
