"""Bounded JSON and TOML readers shared by repository structure checks."""

from __future__ import annotations

import json
import subprocess
import tomllib
from pathlib import Path
from typing import Any


def _git_paths(root: Path, arguments: list[str], label: str, errors: list[str]) -> set[str]:
    result = subprocess.run(
        ["git", *arguments, "-z"],
        cwd=root,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", errors="replace").strip()
        errors.append(f"cannot enumerate {label}: {detail}")
        return set()
    return {
        part.decode("utf-8", errors="surrogateescape")
        for part in result.stdout.split(b"\0")
        if part
    }


def repository_paths(root: Path) -> tuple[list[str], list[str]]:
    errors: list[str] = []
    paths = _git_paths(
        root,
        ["ls-files", "--cached", "--others", "--exclude-standard"],
        "tracked and untracked nonignored files",
        errors,
    )
    staged = _git_paths(
        root,
        ["diff", "--cached", "--name-only", "--diff-filter=ACMR"],
        "staged files",
        errors,
    )
    unstaged = _git_paths(
        root,
        ["diff", "--name-only", "--diff-filter=ACMR"],
        "unstaged files",
        errors,
    )
    for relative in sorted(staged & unstaged):
        errors.append(
            "staged and working-tree content diverge; stage or restore one version "
            f"before verification: {relative}"
        )
    return sorted(paths), errors


def regular(path: Path) -> bool:
    return not path.is_symlink() and path.is_file()


def load_json(path: Path, label: str, errors: list[str]) -> dict[str, Any] | None:
    if not regular(path):
        errors.append(f"missing regular JSON file: {label}")
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        errors.append(f"invalid JSON in {label}: {exc}")
        return None
    if not isinstance(payload, dict):
        errors.append(f"JSON root must be an object: {label}")
        return None
    return payload


def load_toml(path: Path, label: str, errors: list[str]) -> dict[str, Any] | None:
    if not regular(path):
        errors.append(f"missing regular TOML file: {label}")
        return None
    try:
        with path.open("rb") as handle:
            payload = tomllib.load(handle)
    except (OSError, tomllib.TOMLDecodeError) as exc:
        errors.append(f"invalid TOML in {label}: {exc}")
        return None
    return payload


def dependency_names(payload: dict[str, Any]) -> set[str]:
    names: set[str] = set()
    for key in ("dependencies", "dev-dependencies", "build-dependencies"):
        table = payload.get(key, {})
        if isinstance(table, dict):
            for alias, specification in table.items():
                if isinstance(specification, dict) and isinstance(
                    specification.get("package"), str
                ):
                    names.add(specification["package"])
                else:
                    names.add(str(alias))
    targets = payload.get("target", {})
    if isinstance(targets, dict):
        for target in targets.values():
            if isinstance(target, dict):
                names.update(dependency_names(target))
    return names
