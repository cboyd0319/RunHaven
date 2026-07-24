"""Enforce RunHaven's macOS-only maintained platform projection."""

from __future__ import annotations

import re
from pathlib import Path

from repository_io import regular
from repository_policy import Policy


MAX_TEXT_BYTES = 1_048_576
TEXT_EXTENSIONS = frozenset(
    {".css", ".html", ".md", ".txt"} | {".ps" + "1"}
)
FORK_ROOT = "crates/runhaven-tui/src/tui"
UNSUPPORTED_PATTERNS = (
    ("alternate shell", re.compile("power" + "shell", re.IGNORECASE)),
    ("alternate command interpreter", re.compile(r"\bcmd\.exe\b", re.IGNORECASE)),
    ("alternate subsystem", re.compile(r"\bws" + r"l\b", re.IGNORECASE)),
    (
        "alternate Rust target",
        re.compile(r'target_os\s*=\s*"' + "win" + 'dows"', re.IGNORECASE),
    ),
    (
        "alternate Rust configuration",
        re.compile(r"cfg\(\s*(?:not\(\s*)?" + "win" + r"dows\b", re.IGNORECASE),
    ),
    (
        "alternate sandbox projection",
        re.compile("win" + r"dows[_-]?sandbox", re.IGNORECASE),
    ),
    (
        "alternate terminal projection",
        re.compile("win" + r"dows\s*terminal", re.IGNORECASE),
    ),
    ("alternate installer", re.compile(r"install\.ps" + r"1\b", re.IGNORECASE)),
)


def _matches(relative: str, candidate: str) -> bool:
    return relative == candidate or relative.startswith(candidate + "/")


def _excluded(relative: str, policy: Policy) -> bool:
    if _matches(relative, FORK_ROOT):
        return False
    return any(_matches(relative, candidate) for candidate in policy.exclusions)


def check_platform_projection(
    root: Path,
    policy: Policy,
    paths: list[str],
    errors: list[str],
) -> None:
    """Reject unsupported OS and shell projections from maintained text."""

    allowed_extensions = policy.included_extensions | TEXT_EXTENSIONS
    for relative in paths:
        path = root / relative
        if _excluded(relative, policy) or not regular(path):
            continue
        suffix = path.suffix.lower()
        if suffix not in allowed_extensions:
            continue
        if suffix == ".ps" + "1":
            errors.append(f"unsupported platform script must be absent: {relative}")
            continue
        try:
            payload = path.read_bytes()
        except OSError as exc:
            errors.append(f"cannot inspect platform projection in {relative}: {exc}")
            continue
        if len(payload) > MAX_TEXT_BYTES:
            errors.append(
                f"platform projection input exceeds {MAX_TEXT_BYTES} bytes: {relative}"
            )
            continue
        try:
            text = payload.decode("utf-8")
        except UnicodeError as exc:
            errors.append(f"platform projection input is not UTF-8: {relative}: {exc}")
            continue
        for label, pattern in UNSUPPORTED_PATTERNS:
            match = pattern.search(text)
            if match is None:
                continue
            line = text.count("\n", 0, match.start()) + 1
            errors.append(
                f"unsupported platform projection ({label}): {relative}:{line}"
            )

