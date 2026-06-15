from __future__ import annotations

import json
import subprocess
from pathlib import Path


def run_git(repo: Path, *args: str) -> str:
    result = subprocess.run(
        ("git", "-C", str(repo), *args),
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def init_git_repo(repo: Path) -> str:
    subprocess.run(("git", "init"), cwd=repo, check=True, capture_output=True, text=True)
    run_git(repo, "config", "user.email", "runhaven@example.invalid")
    run_git(repo, "config", "user.name", "RunHaven Tests")
    (repo / "tracked.txt").write_text("initial\n", encoding="utf-8")
    run_git(repo, "add", "tracked.txt")
    run_git(repo, "commit", "-m", "initial")
    return run_git(repo, "rev-parse", "HEAD")


def write_run_record_for_git_diff(
    cache: Path,
    *,
    repo: Path,
    run_id: str,
    before_head: str | None,
    after_head: str | None,
    after_dirty: bool,
    after_paths: list[str],
) -> None:
    cache.mkdir(parents=True, exist_ok=True)
    payload = {
        "timestamp": "2026-06-15T00:00:02Z",
        "started_at": "2026-06-15T00:00:02Z",
        "finished_at": "2026-06-15T00:00:03Z",
        "run_id": run_id,
        "profile": "shell",
        "workspace": str(repo),
        "network": "internet",
        "status": "succeeded",
        "return_code": 0,
        "provider_policy": {"entries": 0, "allowed": 0, "denied": 0},
        "auth_broker": {
            "broker": None,
            "entries": 0,
            "allowed": 0,
            "denied": 0,
            "no_requests": False,
        },
        "cleanup": {"provider_network": "not-applicable"},
        "git": {
            "available": True,
            "repo_root": str(repo.resolve()),
            "changed": before_head != after_head or after_dirty,
            "before": {
                "head": before_head,
                "dirty": False,
                "changed_count": 0,
                "paths": [],
                "truncated": False,
            },
            "after": {
                "head": after_head,
                "dirty": after_dirty,
                "changed_count": len(after_paths),
                "paths": after_paths,
                "truncated": False,
            },
        },
    }
    (cache / "runs.jsonl").write_text(json.dumps(payload) + "\n", encoding="utf-8")


def write_active_marker(
    cache: Path,
    *,
    run_id: str,
    timestamp: str,
    container_name: str,
) -> Path:
    active_dir = cache / "active-runs"
    active_dir.mkdir(parents=True, exist_ok=True)
    active_path = active_dir / f"{run_id}.json"
    active_path.write_text(
        json.dumps(
            {
                "timestamp": timestamp,
                "run_id": run_id,
                "profile": "shell",
                "workspace": str(cache),
                "network": "internet",
                "status": "running",
                "container_name": container_name,
                "host_pid": 12345,
            }
        )
        + "\n",
        encoding="utf-8",
    )
    return active_path
