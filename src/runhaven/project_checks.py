from __future__ import annotations

import json
import shlex
from pathlib import Path
from typing import TypedDict


class SuggestedCheck(TypedDict):
    label: str
    command: str
    argv: list[str]
    reason: str


def suggest_project_checks(workspace: Path) -> list[SuggestedCheck]:
    suggestions: list[SuggestedCheck] = []
    package_scripts = package_json_scripts(workspace / "package.json")
    if script_is_real(package_scripts.get("test")):
        suggestions.append(
            runhaven_shell_check(
                workspace,
                label="Node tests",
                tool_args=("npm", "test"),
                reason="package.json defines scripts.test",
            )
        )
    if script_is_real(package_scripts.get("lint")):
        suggestions.append(
            runhaven_shell_check(
                workspace,
                label="Node lint",
                tool_args=("npm", "run", "lint"),
                reason="package.json defines scripts.lint",
            )
        )
    if (workspace / "tests").is_dir():
        suggestions.append(
            runhaven_shell_check(
                workspace,
                label="Python tests",
                tool_args=("python", "-m", "unittest", "discover", "-s", "tests"),
                reason="tests directory exists",
            )
        )
    if ruff_config_exists(workspace):
        suggestions.append(
            runhaven_shell_check(
                workspace,
                label="Python lint",
                tool_args=("python", "-m", "ruff", "check", "."),
                reason="Ruff configuration detected",
            )
        )
    return suggestions


def package_json_scripts(path: Path) -> dict[str, str]:
    if not path.is_file():
        return {}
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return {}
    if not isinstance(data, dict):
        return {}
    scripts = data.get("scripts")
    if not isinstance(scripts, dict):
        return {}
    return {
        key: value
        for key, value in scripts.items()
        if isinstance(key, str) and isinstance(value, str)
    }


def script_is_real(script: str | None) -> bool:
    if script is None:
        return False
    lowered = script.strip().lower()
    if not lowered:
        return False
    return "no test specified" not in lowered


def ruff_config_exists(workspace: Path) -> bool:
    if (workspace / "ruff.toml").is_file() or (workspace / ".ruff.toml").is_file():
        return True
    pyproject = workspace / "pyproject.toml"
    if not pyproject.is_file():
        return False
    try:
        text = pyproject.read_text(encoding="utf-8")
    except OSError:
        return False
    return "[tool.ruff" in text


def runhaven_shell_check(
    workspace: Path,
    *,
    label: str,
    tool_args: tuple[str, ...],
    reason: str,
) -> SuggestedCheck:
    argv = [
        "runhaven",
        "run",
        "shell",
        "--workspace",
        str(workspace),
        "--network",
        "internal",
        "--",
        *tool_args,
    ]
    return {
        "label": label,
        "command": shlex.join(argv),
        "argv": argv,
        "reason": reason,
    }
