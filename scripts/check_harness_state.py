#!/usr/bin/env python3
"""Validate RunHaven's canonical harness state without third-party packages."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

from harness_state_support import (
    ALLOWED_FEATURE_KEYS,
    ID_PATTERN,
    PROGRESS_FIELDS,
    REQUIRED_FEATURE_KEYS,
    REQUIRED_ROOT_FILES,
    RETIRED_ROOT_FILES,
    STATE_BUDGETS,
    STATE_WARNING_PERCENT,
    UNSAFE_STATE,
    VALID_STATUSES,
    check_command,
    check_cycles,
    current_branch,
    evidence_proves_feature,
    head_feature_statuses,
    parse_date,
    read_text,
    safe_relative_file,
)


def check_repository(
    root: Path,
    *,
    compare_history: bool = True,
    warnings: list[str] | None = None,
) -> list[str]:
    root = root.resolve()
    errors: list[str] = []
    try:
        root_names = {entry.name for entry in root.iterdir()}
    except OSError as exc:
        return [f"cannot list repository root: {exc}"]

    for name in REQUIRED_ROOT_FILES:
        path = root / name
        if name not in root_names:
            errors.append(f"missing required root harness file: {name}")
        elif path.is_symlink() or not path.is_file():
            errors.append(f"required root harness path must be a regular file: {name}")
    for name in RETIRED_ROOT_FILES:
        if name in root_names:
            errors.append(f"retired root state file must be absent: {name}")
    if errors:
        return errors

    progress = read_text(root / "PROGRESS.md", errors)
    feature_text = read_text(root / "feature_list.json", errors)
    agents = read_text(root / "AGENTS.md", errors)
    if progress is None or feature_text is None or agents is None:
        return errors
    for name, content in (("AGENTS.md", agents), ("PROGRESS.md", progress), ("feature_list.json", feature_text)):
        if UNSAFE_STATE.search(content):
            errors.append(f"{name} contains a secret marker or machine-local absolute path")

    try:
        payload = json.loads(feature_text)
    except json.JSONDecodeError as exc:
        errors.append(f"feature_list.json is invalid JSON: {exc}")
        return errors
    if not isinstance(payload, dict):
        return errors + ["feature_list.json root must be an object"]

    expected_root_keys = {
        "$schema",
        "schema_version",
        "project",
        "last_updated",
        "status_legend",
        "rules",
        "features",
    }
    if set(payload) != expected_root_keys:
        missing = expected_root_keys - set(payload)
        unknown = set(payload) - expected_root_keys
        if missing:
            errors.append(f"feature_list.json is missing root keys: {', '.join(sorted(missing))}")
        if unknown:
            errors.append(f"feature_list.json has unknown root keys: {', '.join(sorted(unknown))}")
    if payload.get("$schema") != "docs/harness/schemas/feature-list.schema.json":
        errors.append("feature_list.json $schema must point to the canonical repo schema")
    if payload.get("schema_version") != 1:
        errors.append("feature_list.json schema_version must equal 1")
    if payload.get("project") != "RunHaven":
        errors.append("feature_list.json project must equal RunHaven")
    feature_date = parse_date(payload.get("last_updated"), "feature_list.json last_updated", errors)
    legend = payload.get("status_legend")
    if (
        not isinstance(legend, dict)
        or set(legend) != VALID_STATUSES
        or not all(isinstance(value, str) and value.strip() for value in legend.values())
    ):
        errors.append("status_legend must define exactly not_started, active, blocked, and passing")
    rules = payload.get("rules")
    expected_rules = {
        "active_count": 1,
        "passing_is_irreversible": True,
        "evidence_is_external": True,
    }
    if rules != expected_rules:
        errors.append("feature_list.json rules do not match the canonical state contract")

    raw_features = payload.get("features")
    if not isinstance(raw_features, list) or not raw_features:
        return errors + ["feature_list.json features must be a non-empty array"]

    state_measurements = {
        "bytes": len(progress.encode("utf-8")) + len(feature_text.encode("utf-8")),
        "lines": len(progress.splitlines()) + len(feature_text.splitlines()),
        "items": len(raw_features),
        "record_bytes": max(
            len(json.dumps(item, ensure_ascii=False, separators=(",", ":")).encode("utf-8"))
            for item in raw_features
        ),
    }
    for field, observed in state_measurements.items():
        ceiling = STATE_BUDGETS[field]
        if observed > ceiling:
            errors.append(f"state {field} budget exceeded: {observed} > {ceiling}")
        elif warnings is not None and observed * 100 >= ceiling * STATE_WARNING_PERCENT:
            warnings.append(f"state {field} budget warning: {observed} / {ceiling}")

    features: dict[str, dict[str, Any]] = {}
    active_ids: list[str] = []
    for index, item in enumerate(raw_features):
        if not isinstance(item, dict):
            errors.append(f"feature at index {index} must be an object")
            continue
        feature_id = item.get("id")
        label = feature_id if isinstance(feature_id, str) else f"index {index}"
        unknown = set(item) - ALLOWED_FEATURE_KEYS
        missing = REQUIRED_FEATURE_KEYS - set(item)
        if unknown:
            errors.append(f"feature {label} has unknown keys: {', '.join(sorted(unknown))}")
        if missing:
            errors.append(f"feature {label} is missing keys: {', '.join(sorted(missing))}")
        if not isinstance(feature_id, str) or not ID_PATTERN.fullmatch(feature_id):
            errors.append(f"feature id is invalid at index {index}: {feature_id!r}")
            continue
        if feature_id in features:
            errors.append(f"duplicate feature id: {feature_id}")
            continue
        features[feature_id] = item
        status = item.get("status")
        if status not in VALID_STATUSES:
            errors.append(f"feature {feature_id} has invalid status: {status!r}")
        elif status == "active":
            active_ids.append(feature_id)
        priority = item.get("priority")
        if not isinstance(priority, int) or isinstance(priority, bool) or priority < 0:
            errors.append(f"feature {feature_id} priority must be a non-negative integer")
        for key in ("area", "title", "user_visible_behavior"):
            if not isinstance(item.get(key), str) or not item[key].strip():
                errors.append(f"feature {feature_id} {key} must be a non-empty string")
        dependencies = item.get("dependencies")
        if not isinstance(dependencies, list) or not all(isinstance(value, str) for value in dependencies):
            errors.append(f"feature {feature_id} dependencies must be a string array")
        elif len(dependencies) != len(set(dependencies)):
            errors.append(f"feature {feature_id} has duplicate dependencies")
        verification = item.get("verification")
        if not isinstance(verification, list) or not verification:
            errors.append(f"feature {feature_id} verification must be a non-empty command array")
        else:
            for command in verification:
                check_command(command, feature_id, root, errors)
        evidence = item.get("evidence")
        if not isinstance(evidence, list) or not all(isinstance(value, str) for value in evidence):
            errors.append(f"feature {feature_id} evidence must be a string array")
        else:
            if status == "passing" and not evidence:
                errors.append(f"passing feature {feature_id} requires evidence")
            resolved_evidence: list[Path] = []
            for reference in evidence:
                path_text = reference.split("#", 1)[0]
                if Path(path_text).as_posix() in REQUIRED_ROOT_FILES:
                    errors.append(
                        f"feature {feature_id} evidence must be outside the startup pack: "
                        f"{reference!r}"
                    )
                    continue
                resolved = safe_relative_file(
                    root,
                    reference,
                    f"feature {feature_id} evidence",
                    errors,
                )
                if resolved is not None:
                    resolved_evidence.append(resolved)
            if status == "passing" and resolved_evidence:
                evidence_matches = [
                    evidence_proves_feature(path, feature_id, errors)
                    for path in resolved_evidence
                ]
                if not any(evidence_matches):
                    errors.append(f"passing evidence does not identify feature {feature_id}")
        if status == "blocked":
            for key in ("blocker", "next_trigger"):
                if not isinstance(item.get(key), str) or not item[key].strip():
                    errors.append(f"blocked feature {feature_id} requires {key}")
        superseded_by = item.get("superseded_by")
        if superseded_by is not None and not isinstance(superseded_by, str):
            errors.append(f"feature {feature_id} superseded_by must be a feature id")

    if len(active_ids) != 1:
        errors.append(f"exactly one feature must be active; found {len(active_ids)}")

    for feature_id, item in features.items():
        for dependency in item.get("dependencies", []):
            if dependency not in features:
                errors.append(f"feature {feature_id} has unresolved dependency: {dependency}")
            elif dependency == feature_id:
                errors.append(f"feature {feature_id} depends on itself")
        superseded_by = item.get("superseded_by")
        if isinstance(superseded_by, str) and superseded_by not in features:
            errors.append(f"feature {feature_id} has unresolved superseded_by: {superseded_by}")
    check_cycles(features, errors)

    if active_ids:
        active = features[active_ids[0]]
        queued_priorities = [
            item["priority"]
            for item in features.values()
            if item.get("status") == "not_started" and isinstance(item.get("priority"), int)
        ]
        if queued_priorities and active.get("priority", sys.maxsize) > min(queued_priorities):
            errors.append("active feature is not the highest-priority unfinished feature")

    parsed_progress: dict[str, str] = {}
    for key, pattern in PROGRESS_FIELDS.items():
        matches = pattern.findall(progress)
        if len(matches) != 1:
            errors.append(f"PROGRESS.md must contain exactly one canonical {key} field")
        else:
            parsed_progress[key] = matches[0]
    progress_date = parse_date(parsed_progress.get("last_updated"), "PROGRESS.md Last updated", errors)
    if feature_date and progress_date and feature_date != progress_date:
        errors.append("PROGRESS.md and feature_list.json update dates disagree")
    if active_ids and parsed_progress.get("active") != active_ids[0]:
        errors.append("PROGRESS.md active feature disagrees with feature_list.json")
    if parsed_progress.get("status") != "active":
        errors.append("PROGRESS.md status must be active")
    branch = current_branch(root)
    if branch and parsed_progress.get("branch") != branch:
        errors.append(f"PROGRESS.md branch {parsed_progress.get('branch')!r} disagrees with Git branch {branch!r}")

    if compare_history:
        current_statuses = {feature_id: item.get("status") for feature_id, item in features.items()}
        for feature_id, old_status in head_feature_statuses(root, errors).items():
            if old_status == "passing" and current_statuses.get(feature_id) != "passing":
                errors.append(f"passing feature is irreversible and must remain passing: {feature_id}")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    args = parser.parse_args()
    warnings: list[str] = []
    errors = check_repository(args.root, warnings=warnings)
    if errors:
        print("harness state check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    for warning in warnings:
        print(f"warning: {warning}", file=sys.stderr)
    print("harness state check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
