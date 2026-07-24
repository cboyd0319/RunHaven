"""Shared parsing, evidence, command, and Git helpers for harness state checks."""

from __future__ import annotations

import datetime as dt
import json
import os
import re
import shlex
import subprocess
from pathlib import Path
from typing import Any


VALID_STATUSES = {"not_started", "active", "blocked", "passing"}
REQUIRED_ROOT_FILES = ("AGENTS.md", "PROGRESS.md", "feature_list.json", "init.sh")
RETIRED_ROOT_FILES = (
    "current-state.md",
    "progress.md",
    "session-handoff.md",
    "claude-progress.md",
)
REQUIRED_FEATURE_KEYS = {
    "id",
    "priority",
    "area",
    "title",
    "user_visible_behavior",
    "status",
    "dependencies",
    "verification",
    "evidence",
}
ALLOWED_FEATURE_KEYS = REQUIRED_FEATURE_KEYS | {
    "blocker",
    "next_trigger",
    "superseded_by",
}
ID_PATTERN = re.compile(r"^[a-z0-9]+(?:[.-][a-z0-9]+)*$")
PROGRESS_FIELDS = {
    "last_updated": re.compile(r"^Last updated: (\d{4}-\d{2}-\d{2})$", re.MULTILINE),
    "branch": re.compile(r"^Branch: `([^`]+)`$", re.MULTILINE),
    "active": re.compile(r"^Active feature: `([^`]+)`$", re.MULTILINE),
    "status": re.compile(r"^Status: `([^`]+)`$", re.MULTILINE),
}
RUNNABLE_PREFIXES = {"cargo", "git", "npm", "python3", "test"}
SHELL_META = re.compile(r"(?:&&|\|\||[;|<>`]|\$\(|\n|\r)")
MAX_STATE_BYTES = 256 * 1024
STATE_BUDGETS = {
    "bytes": 32_768,
    "lines": 500,
    "items": 40,
    "record_bytes": 2_048,
}
STATE_WARNING_PERCENT = 90
UNSAFE_STATE = re.compile(
    r"(?:/Users/|/home/|[A-Za-z]:\\|BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY|"
    r"ghp_[A-Za-z0-9]{20,}|sk-[A-Za-z0-9]{20,})"
)


def read_text(path: Path, errors: list[str]) -> str | None:
    if path.is_symlink():
        errors.append(f"state path must not be a symlink: {path.name}")
        return None
    try:
        size = path.stat().st_size
        if size > MAX_STATE_BYTES:
            errors.append(
                f"state file exceeds bounded read limit ({size} > {MAX_STATE_BYTES} bytes): {path.name}"
            )
            return None
        return path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as exc:
        errors.append(f"cannot read {path.name}: {exc}")
        return None


def parse_date(value: Any, label: str, errors: list[str]) -> dt.date | None:
    if not isinstance(value, str):
        errors.append(f"{label} must be an ISO date")
        return None
    try:
        parsed = dt.date.fromisoformat(value)
    except ValueError:
        errors.append(f"{label} must be an ISO date: {value!r}")
        return None
    if parsed > dt.date.today():
        errors.append(f"{label} must not be in the future: {value}")
    return parsed


def safe_relative_file(root: Path, reference: str, label: str, errors: list[str]) -> Path | None:
    path_text = reference.split("#", 1)[0]
    candidate = Path(path_text)
    if not path_text or candidate.is_absolute() or ".." in candidate.parts or "~" in candidate.parts:
        errors.append(f"{label} must be a safe repo-relative path: {reference!r}")
        return None
    path = root / candidate
    if path.is_symlink():
        errors.append(f"{label} must not point to a symlink: {reference!r}")
        return None
    try:
        resolved = path.resolve(strict=True)
        resolved.relative_to(root.resolve(strict=True))
    except (OSError, ValueError):
        errors.append(f"{label} does not resolve inside the repository: {reference!r}")
        return None
    if not resolved.is_file():
        errors.append(f"{label} must point to a file: {reference!r}")
        return None
    return resolved


def evidence_proves_feature(path: Path, feature_id: str, errors: list[str]) -> bool:
    try:
        size = path.stat().st_size
        if size > MAX_STATE_BYTES:
            errors.append(
                f"feature {feature_id} evidence exceeds bounded read limit "
                f"({size} > {MAX_STATE_BYTES} bytes): {path.name}"
            )
            return False
        content = path.read_text(encoding="utf-8")
    except (OSError, UnicodeError) as exc:
        errors.append(f"cannot read feature {feature_id} evidence {path.name}: {exc}")
        return False
    index_pattern = re.compile(
        rf"^\| `{re.escape(feature_id)}` \| (\d{{4}}-\d{{2}}-\d{{2}}), ([^|]+?) \|$",
        re.MULTILINE,
    )
    index_matches = index_pattern.findall(content)
    if len(index_matches) != 1:
        errors.append(
            f"feature {feature_id} evidence needs one exact passing-feature index row"
        )
        return False
    evidence_date, scope = index_matches[0]
    if parse_date(evidence_date, f"feature {feature_id} evidence date", errors) is None:
        return False
    row_pattern = re.compile(
        rf"^\| {re.escape(evidence_date)} \| {re.escape(scope)} \| ([^|]+) \| ([^|]+) \| ([^|]+) \|$",
        re.MULTILINE,
    )
    rows = row_pattern.findall(content)
    if len(rows) != 1:
        errors.append(
            f"feature {feature_id} evidence index does not resolve to one detailed row"
        )
        return False
    command_or_review, result, notes = (value.strip() for value in rows[0])
    if not command_or_review or not notes or not result.lower().startswith("passed"):
        errors.append(
            f"feature {feature_id} detailed evidence lacks command, passing result, or notes"
        )
        return False
    return True


def check_command(command: Any, feature_id: str, root: Path, errors: list[str]) -> None:
    label = f"feature {feature_id} verification"
    if not isinstance(command, str) or not command.strip():
        errors.append(f"{label} must contain a command string")
        return
    if command != command.strip() or SHELL_META.search(command):
        errors.append(f"{label} must be one exact command without shell composition: {command!r}")
        return
    try:
        parts = shlex.split(command)
    except ValueError as exc:
        errors.append(f"{label} is not shell-parseable: {exc}")
        return
    if not parts:
        errors.append(f"{label} is empty")
        return
    executable = parts[0]
    if executable.startswith("./") or executable.startswith("scripts/"):
        script = root / executable.removeprefix("./")
        if script.is_symlink() or not script.is_file() or not os.access(script, os.X_OK):
            errors.append(
                f"{label} references a missing, non-regular, or non-executable script: {executable!r}"
            )
        return
    if executable not in RUNNABLE_PREFIXES:
        errors.append(f"{label} starts with unsupported executable {executable!r}: {command!r}")
        return
    if executable == "cargo" and (len(parts) < 2 or parts[1] not in {
        "build", "check", "clippy", "fmt", "metadata", "run", "test"
    }):
        errors.append(f"{label} uses an unsupported Cargo command: {command!r}")
    elif executable == "npm" and (len(parts) < 2 or parts[1] not in {"ci", "run", "test"}):
        errors.append(f"{label} uses an unsupported npm command: {command!r}")
    elif executable == "git" and (len(parts) < 2 or parts[1] not in {"diff", "status"}):
        errors.append(f"{label} uses an unsupported read-only Git command: {command!r}")
    elif executable == "python3":
        if len(parts) < 2 or parts[1].startswith("-"):
            errors.append(f"{label} must name one repository Python script: {command!r}")
        else:
            script = root / parts[1]
            if (
                Path(parts[1]).is_absolute()
                or ".." in Path(parts[1]).parts
                or script.is_symlink()
                or not script.is_file()
            ):
                errors.append(f"{label} references an unsafe or missing Python script: {parts[1]!r}")


def check_cycles(features: dict[str, dict[str, Any]], errors: list[str]) -> None:
    visiting: set[str] = set()
    visited: set[str] = set()

    def visit(feature_id: str, chain: list[str]) -> None:
        if feature_id in visited:
            return
        if feature_id in visiting:
            start = chain.index(feature_id)
            errors.append(f"feature dependency cycle: {' -> '.join(chain[start:] + [feature_id])}")
            return
        visiting.add(feature_id)
        chain.append(feature_id)
        for dependency in features[feature_id].get("dependencies", []):
            if dependency in features:
                visit(dependency, chain)
        chain.pop()
        visiting.remove(feature_id)
        visited.add(feature_id)

    for feature_id in features:
        visit(feature_id, [])


def head_feature_statuses(root: Path, errors: list[str]) -> dict[str, str]:
    result = subprocess.run(
        ["git", "show", "HEAD:feature_list.json"],
        cwd=root,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
        encoding="utf-8",
    )
    if result.returncode != 0:
        errors.append("cannot read committed feature state for irreversible-history validation")
        return {}
    try:
        payload = json.loads(result.stdout)
    except json.JSONDecodeError:
        errors.append("committed feature state is invalid JSON; irreversible-history validation failed")
        return {}
    return {
        item.get("id"): item.get("status")
        for item in payload.get("features", [])
        if isinstance(item, dict) and isinstance(item.get("id"), str)
    }


def current_branch(root: Path) -> str | None:
    result = subprocess.run(
        ["git", "branch", "--show-current"],
        cwd=root,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
        encoding="utf-8",
    )
    branch = result.stdout.strip()
    return branch if result.returncode == 0 and branch else None
