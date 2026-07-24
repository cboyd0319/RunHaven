"""Parse and validate RunHaven's canonical repository structure policy."""

from __future__ import annotations

import json
from dataclasses import dataclass
from datetime import date
from pathlib import Path, PurePosixPath
from typing import Any


POLICY_NAME = "repository-structure-policy.json"
CANONICAL_LIMITS = {
    "counting": "physical_lines_including_blank_and_comment_lines",
    "review_lines": 300,
    "hard_lines": 500,
    "review_bytes": 32_768,
    "hard_bytes": 65_536,
}
GLOB_CHARS = frozenset("*?[]{}")
EXCLUSION_CATEGORIES = {"generated", "lock", "vendored"}
UNIT_KINDS = {"deployable", "image", "reusable"}


@dataclass(frozen=True)
class ExceptionRule:
    path: str
    affected_rule: str
    owner: str
    reason: str
    temporary_lines: int
    temporary_bytes: int
    approval_date: str
    retirement_condition: str


@dataclass(frozen=True)
class Unit:
    identifier: str
    kind: str
    root: str
    manifest: str
    public_entrypoints: tuple[str, ...]
    test_entrypoints: tuple[str, ...]
    documentation: tuple[str, ...]
    allowed_dependencies: tuple[str, ...]
    allowed_vendored_dependencies: tuple[str, ...]


@dataclass(frozen=True)
class Policy:
    included_extensions: frozenset[str]
    included_source_paths: frozenset[str]
    startup_files: tuple[str, ...]
    startup_state_reads: tuple[str, ...]
    startup_loading_hops: dict[str, int]
    startup_warning_percent: int
    startup_hard_lines: int
    startup_hard_bytes: int
    allowed_source_roots: tuple[str, ...]
    unitized_source_roots: tuple[str, ...]
    allowed_root_source_files: frozenset[str]
    modularization_review_file: str
    exclusions: tuple[str, ...]
    hand_authored_inclusions: tuple[str, ...]
    exceptions: dict[str, ExceptionRule]
    units: tuple[Unit, ...]
    retired_paths: tuple[str, ...]


def safe_relative(value: Any, label: str, errors: list[str]) -> str | None:
    if not isinstance(value, str) or not value:
        errors.append(f"{label} must be a nonempty repository-relative path")
        return None
    path = PurePosixPath(value)
    if path.is_absolute() or ".." in path.parts or value != path.as_posix():
        errors.append(f"{label} must be a normalized repository-relative path: {value!r}")
        return None
    if any(character in value for character in GLOB_CHARS):
        errors.append(f"{label} must not contain a glob: {value}")
        return None
    return value.rstrip("/")


def required_text(entry: dict[str, Any], field: str, label: str, errors: list[str]) -> str:
    value = entry.get(field)
    if not isinstance(value, str) or not value.strip():
        errors.append(f"{label} is missing {field}")
        return ""
    return value.strip()


def _load_json(path: Path, errors: list[str]) -> dict[str, Any] | None:
    if path.is_symlink() or not path.is_file():
        errors.append(f"missing regular policy file: {POLICY_NAME}")
        return None
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        errors.append(f"invalid {POLICY_NAME}: {exc}")
        return None
    if not isinstance(payload, dict):
        errors.append(f"{POLICY_NAME} root must be an object")
        return None
    return payload


def _parse_exceptions(payload: Any, errors: list[str]) -> dict[str, ExceptionRule]:
    if not isinstance(payload, list):
        errors.append("exceptions must be an array")
        return {}
    parsed: dict[str, ExceptionRule] = {}
    for index, raw in enumerate(payload):
        label = f"exceptions[{index}]"
        if not isinstance(raw, dict):
            errors.append(f"{label} must be an object")
            continue
        path = safe_relative(raw.get("path"), f"{label}.path", errors)
        rule = required_text(raw, "affected_rule", label, errors)
        owner = required_text(raw, "owner", label, errors)
        reason = required_text(raw, "reason", label, errors)
        approved = required_text(raw, "approval_date", label, errors)
        retirement = required_text(raw, "retirement_condition", label, errors)
        ceiling = raw.get("temporary_ceiling")
        lines = ceiling.get("lines") if isinstance(ceiling, dict) else None
        byte_count = ceiling.get("bytes") if isinstance(ceiling, dict) else None
        if rule not in {"lines", "bytes", "lines_and_bytes"}:
            errors.append(f"{label}.affected_rule must be lines, bytes, or lines_and_bytes")
        if not isinstance(lines, int) or isinstance(lines, bool) or lines <= 500:
            errors.append(f"{label}.temporary_ceiling.lines must be an integer above 500")
            lines = 500
        if not isinstance(byte_count, int) or isinstance(byte_count, bool) or byte_count <= 65_536:
            errors.append(f"{label}.temporary_ceiling.bytes must be an integer above 65536")
            byte_count = 65_536
        try:
            date.fromisoformat(approved)
        except ValueError:
            errors.append(f"{label}.approval_date must be YYYY-MM-DD")
        if path:
            if path in parsed:
                errors.append(f"duplicate source-limit exception: {path}")
            parsed[path] = ExceptionRule(
                path, rule, owner, reason, lines, byte_count, approved, retirement
            )
    return parsed


def _parse_paths(payload: Any, label: str, errors: list[str]) -> list[str]:
    if not isinstance(payload, list):
        errors.append(f"{label} must be an array")
        return []
    parsed: list[str] = []
    for index, value in enumerate(payload):
        path = safe_relative(value, f"{label}[{index}]", errors)
        if path:
            parsed.append(path)
    if len(parsed) != len(set(parsed)):
        errors.append(f"{label} must not contain duplicates")
    return parsed


def _parse_units(payload: Any, roots: list[str], errors: list[str]) -> list[Unit]:
    if not isinstance(payload, list) or not payload:
        errors.append("structure.units must be a nonempty array")
        return []
    units: list[Unit] = []
    for index, raw in enumerate(payload):
        label = f"structure.units[{index}]"
        if not isinstance(raw, dict):
            errors.append(f"{label} must be an object")
            continue
        identifier = required_text(raw, "id", label, errors)
        kind = required_text(raw, "kind", label, errors)
        root = safe_relative(raw.get("root"), f"{label}.root", errors)
        manifest = safe_relative(raw.get("manifest"), f"{label}.manifest", errors)
        entrypoints = _parse_paths(raw.get("public_entrypoints"), f"{label}.public_entrypoints", errors)
        tests = _parse_paths(raw.get("test_entrypoints"), f"{label}.test_entrypoints", errors)
        documentation = _parse_paths(raw.get("documentation"), f"{label}.documentation", errors)
        if not entrypoints:
            errors.append(f"{label}.public_entrypoints must be nonempty")
        if not tests:
            errors.append(f"{label}.test_entrypoints must be nonempty")
        if not documentation:
            errors.append(f"{label}.documentation must be nonempty")
        dependencies = raw.get("allowed_dependencies")
        if not isinstance(dependencies, list) or any(not isinstance(item, str) for item in dependencies):
            errors.append(f"{label}.allowed_dependencies must be an array of unit ids")
            dependencies = []
        vendored = raw.get("allowed_vendored_dependencies", [])
        if not isinstance(vendored, list) or any(
            not isinstance(item, str) or not item for item in vendored
        ):
            errors.append(f"{label}.allowed_vendored_dependencies must be an array of package ids")
            vendored = []
        elif len(vendored) != len(set(vendored)):
            errors.append(f"{label}.allowed_vendored_dependencies must not contain duplicates")
        if kind not in UNIT_KINDS:
            errors.append(f"{label}.kind must be deployable, image, or reusable")
        if root and not any(root == source or root.startswith(source + "/") for source in roots):
            errors.append(f"{label}.root is outside approved source roots: {root}")
        if root and manifest and not (manifest == root or manifest.startswith(root + "/")):
            errors.append(f"{label}.manifest must be inside its unit root")
        if root:
            for field, paths in (
                ("public entrypoint", entrypoints),
                ("test entrypoint", tests),
                ("documentation", documentation),
            ):
                for path in paths:
                    if not path.startswith(root + "/"):
                        errors.append(f"{label} {field} is outside its unit root: {path}")
        if identifier and root and manifest:
            units.append(
                Unit(
                    identifier,
                    kind,
                    root,
                    manifest,
                    tuple(entrypoints),
                    tuple(tests),
                    tuple(documentation),
                    tuple(dependencies),
                    tuple(vendored),
                )
            )
    identifiers = [unit.identifier for unit in units]
    unit_roots = [unit.root for unit in units]
    if len(identifiers) != len(set(identifiers)):
        errors.append("structure.units contains duplicate ids")
    if len(unit_roots) != len(set(unit_roots)):
        errors.append("structure.units contains duplicate roots")
    known = set(identifiers)
    for unit in units:
        unknown = set(unit.allowed_dependencies) - known
        if unknown:
            errors.append(f"unit {unit.identifier} names unknown dependencies: {', '.join(sorted(unknown))}")
        if unit.identifier in unit.allowed_dependencies:
            errors.append(f"unit {unit.identifier} must not depend on itself")
    return units


def load_policy(root: Path) -> tuple[Policy | None, list[str]]:
    errors: list[str] = []
    payload = _load_json(root / POLICY_NAME, errors)
    if payload is None:
        return None, errors
    if payload.get("schema_version") != 1:
        errors.append("schema_version must be 1")
    limits = payload.get("source_limits")
    if not isinstance(limits, dict):
        errors.append("source_limits must be an object")
    else:
        for field, expected in CANONICAL_LIMITS.items():
            if limits.get(field) != expected:
                errors.append(f"source_limits.{field} must be {expected!r}")
    extensions = payload.get("included_extensions")
    parsed_extensions: set[str] = set()
    if not isinstance(extensions, list) or not extensions:
        errors.append("included_extensions must be a nonempty array")
    else:
        for value in extensions:
            if not isinstance(value, str) or not value.startswith(".") or value != value.lower():
                errors.append(f"invalid included extension: {value!r}")
            else:
                parsed_extensions.add(value)
    structure = payload.get("structure")
    if not isinstance(structure, dict):
        errors.append("structure must be an object")
        structure = {}
    roots = _parse_paths(structure.get("allowed_source_roots"), "structure.allowed_source_roots", errors)
    if not roots:
        errors.append("structure.allowed_source_roots must be nonempty")
    unitized_roots = _parse_paths(
        structure.get("unitized_source_roots"),
        "structure.unitized_source_roots",
        errors,
    )
    for path in unitized_roots:
        if path not in roots:
            errors.append(f"unitized source root is not an allowed source root: {path}")
    root_files = _parse_paths(
        structure.get("allowed_root_source_files"),
        "structure.allowed_root_source_files",
        errors,
    )
    for path in root_files:
        if "/" in path:
            errors.append(f"allowed root source file must be at repository root: {path}")
    included_source_paths = _parse_paths(
        payload.get("included_source_paths"),
        "included_source_paths",
        errors,
    )
    for path in included_source_paths:
        if not any(path == source or path.startswith(source + "/") for source in roots):
            errors.append(f"included source path is outside approved source roots: {path}")
    startup = payload.get("startup_context")
    if not isinstance(startup, dict):
        errors.append("startup_context must be an object")
        startup = {}
    startup_files = _parse_paths(startup.get("files"), "startup_context.files", errors)
    startup_state_reads = _parse_paths(
        startup.get("state_reads"),
        "startup_context.state_reads",
        errors,
    )
    if not set(startup_state_reads).issubset(startup_files):
        errors.append("startup_context.state_reads must be a subset of startup_context.files")
    loading_hops = startup.get("loading_hops")
    parsed_hops: dict[str, int] = {}
    if not isinstance(loading_hops, dict):
        errors.append("startup_context.loading_hops must be an object")
    else:
        for path, hops in loading_hops.items():
            normalized = safe_relative(path, "startup_context.loading_hops path", errors)
            if not isinstance(hops, int) or isinstance(hops, bool) or hops < 0:
                errors.append(f"startup_context.loading_hops must be nonnegative for {path!r}")
            elif normalized:
                parsed_hops[normalized] = hops
        if set(parsed_hops) != set(startup_files):
            errors.append("startup_context.loading_hops must name every startup file exactly once")
    warning_percent = startup.get("warning_percent")
    if not isinstance(warning_percent, int) or isinstance(warning_percent, bool) or not 1 <= warning_percent < 100:
        errors.append("startup_context.warning_percent must be an integer from 1 to 99")
        warning_percent = 90
    hard_lines = startup.get("hard_lines")
    if not isinstance(hard_lines, int) or isinstance(hard_lines, bool) or hard_lines <= 0:
        errors.append("startup_context.hard_lines must be a positive integer")
        hard_lines = 700
    hard_bytes = startup.get("hard_bytes")
    if not isinstance(hard_bytes, int) or isinstance(hard_bytes, bool) or hard_bytes <= 0:
        errors.append("startup_context.hard_bytes must be a positive integer")
        hard_bytes = 49_152
    if structure.get("forbidden_root_source_files") is not True:
        errors.append("structure.forbidden_root_source_files must be true")
    if structure.get("architecture_check") != "scripts/python.sh scripts/check_repo_layout.py":
        errors.append("structure.architecture_check must name the canonical layout check")
    if structure.get("source_limit_check") != "scripts/python.sh scripts/check_file_limits.py":
        errors.append("structure.source_limit_check must name the canonical source-limit check")
    review_file = safe_relative(
        payload.get("modularization_review_file"),
        "modularization_review_file",
        errors,
    )
    units = _parse_units(structure.get("units"), roots, errors)
    exclusions_raw = payload.get("non_hand_authored_exclusions")
    exclusions: list[str] = []
    if not isinstance(exclusions_raw, list):
        errors.append("non_hand_authored_exclusions must be an array")
    else:
        for index, raw in enumerate(exclusions_raw):
            label = f"non_hand_authored_exclusions[{index}]"
            if not isinstance(raw, dict):
                errors.append(f"{label} must be an object")
                continue
            path = safe_relative(raw.get("path"), f"{label}.path", errors)
            category = required_text(raw, "category", label, errors)
            required_text(raw, "owner", label, errors)
            required_text(raw, "reason", label, errors)
            if category not in EXCLUSION_CATEGORIES:
                errors.append(f"{label}.category must be generated, lock, or vendored")
            if path:
                exclusions.append(path)
    if len(exclusions) != len(set(exclusions)):
        errors.append("non_hand_authored_exclusions must not contain duplicates")
    inclusions = _parse_paths(
        payload.get("hand_authored_inclusions"),
        "hand_authored_inclusions",
        errors,
    )
    for path in inclusions:
        if not any(path == excluded or path.startswith(excluded + "/") for excluded in exclusions):
            errors.append(
                f"hand-authored inclusion must be inside a non-hand-authored exclusion: {path}"
            )
    exceptions = _parse_exceptions(payload.get("exceptions"), errors)
    retired_paths = _parse_paths(payload.get("retired_paths"), "retired_paths", errors)
    for path in sorted(set(exclusions) & set(exceptions)):
        errors.append(f"path cannot be both excluded and excepted: {path}")
    if errors:
        return None, errors
    return (
        Policy(
            frozenset(parsed_extensions),
            frozenset(included_source_paths),
            tuple(startup_files),
            tuple(startup_state_reads),
            parsed_hops,
            warning_percent,
            hard_lines,
            hard_bytes,
            tuple(sorted(roots)),
            tuple(sorted(unitized_roots)),
            frozenset(root_files),
            review_file or "",
            tuple(sorted(exclusions)),
            tuple(sorted(inclusions)),
            exceptions,
            tuple(units),
            tuple(sorted(retired_paths)),
        ),
        [],
    )


def is_source_path(relative: str, policy: Policy) -> bool:
    return (
        relative in policy.included_source_paths
        or PurePosixPath(relative).suffix.lower() in policy.included_extensions
    )
