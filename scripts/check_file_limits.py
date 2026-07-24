#!/usr/bin/env python3
"""Enforce the canonical source limits from repository-structure-policy.json."""

from __future__ import annotations

import argparse
import stat
import sys
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Iterable

from modularization_reviews import load_modularization_reviews
from repository_io import repository_paths
from repository_policy import ExceptionRule, Policy, is_source_path, load_policy


CHUNK_BYTES = 64 * 1024


@dataclass(frozen=True)
class Measurement:
    byte_count: int
    line_count: int
    max_line_bytes: int


@dataclass
class CheckResult:
    errors: list[str]
    warnings: list[str]
    checked_files: int = 0
    startup_files: int = 0
    startup_bytes: int = 0
    startup_lines: int = 0
    startup_max_line_bytes: int = 0
    startup_max_hops: int = 0


def measure_file(path: Path) -> Measurement:
    byte_count = 0
    newline_count = 0
    last_byte: int | None = None
    current_line_bytes = 0
    max_line_bytes = 0
    with path.open("rb") as handle:
        while chunk := handle.read(CHUNK_BYTES):
            byte_count += len(chunk)
            newline_count += chunk.count(b"\n")
            last_byte = chunk[-1]
            lines = chunk.split(b"\n")
            if len(lines) == 1:
                current_line_bytes += len(chunk)
            else:
                max_line_bytes = max(max_line_bytes, current_line_bytes + len(lines[0]))
                if len(lines) > 2:
                    max_line_bytes = max(max_line_bytes, *(len(line) for line in lines[1:-1]))
                current_line_bytes = len(lines[-1])
    line_count = newline_count + (1 if byte_count and last_byte != ord("\n") else 0)
    max_line_bytes = max(max_line_bytes, current_line_bytes)
    return Measurement(byte_count, line_count, max_line_bytes)


def _check_startup_context(root: Path, policy: Policy, result: CheckResult) -> None:
    for relative in policy.startup_files:
        path = _safe_file(root, relative, result)
        if path is None:
            if not any(relative in error for error in result.errors):
                result.errors.append(f"startup context file is missing: {relative}")
            continue
        try:
            measurement = measure_file(path)
        except OSError as exc:
            result.errors.append(f"cannot read startup context file {relative!r}: {exc}")
            continue
        result.startup_files += 1
        result.startup_bytes += measurement.byte_count
        result.startup_lines += measurement.line_count
        result.startup_max_line_bytes = max(
            result.startup_max_line_bytes,
            measurement.max_line_bytes,
        )
    result.startup_max_hops = max(policy.startup_loading_hops.values(), default=0)
    if result.startup_lines > policy.startup_hard_lines:
        result.errors.append(
            f"startup context line budget exceeded: {result.startup_lines} / {policy.startup_hard_lines}"
        )
    if result.startup_bytes > policy.startup_hard_bytes:
        result.errors.append(
            f"startup context byte budget exceeded: {result.startup_bytes} / {policy.startup_hard_bytes}"
        )
    warning_lines = policy.startup_hard_lines * policy.startup_warning_percent // 100
    warning_bytes = policy.startup_hard_bytes * policy.startup_warning_percent // 100
    if result.startup_lines >= warning_lines:
        result.warnings.append(
            f"startup context near line budget: {result.startup_lines} / {policy.startup_hard_lines}"
        )
    if result.startup_bytes >= warning_bytes:
        result.warnings.append(
            f"startup context near byte budget: {result.startup_bytes} / {policy.startup_hard_bytes}"
        )


def _safe_file(root: Path, relative: str, result: CheckResult) -> Path | None:
    candidate = PurePosixPath(relative)
    if candidate.is_absolute() or ".." in candidate.parts:
        result.errors.append(f"repository path escapes root: {relative!r}")
        return None
    path = root.joinpath(*candidate.parts)
    if not path.exists() and not path.is_symlink():
        return None
    try:
        mode = path.lstat().st_mode
    except OSError as exc:
        result.errors.append(f"cannot inspect repository path {relative!r}: {exc}")
        return None
    if stat.S_ISLNK(mode):
        result.errors.append(f"repository scan rejects symlink: {relative}")
        return None
    if not stat.S_ISREG(mode):
        result.errors.append(f"repository scan requires a regular file: {relative}")
        return None
    try:
        path.resolve(strict=True).relative_to(root.resolve(strict=True))
    except (OSError, ValueError):
        result.errors.append(f"repository path resolves outside root: {relative}")
        return None
    return path


def _matches_path(relative: str, candidate: str) -> bool:
    return relative == candidate or relative.startswith(candidate + "/")


def _is_excluded(relative: str, policy: Policy) -> bool:
    if any(_matches_path(relative, candidate) for candidate in policy.hand_authored_inclusions):
        return False
    return any(_matches_path(relative, candidate) for candidate in policy.exclusions)


def _is_allowed_source(relative: str, policy: Policy) -> bool:
    if relative in policy.allowed_root_source_files:
        return True
    return any(relative.startswith(root + "/") for root in policy.allowed_source_roots)


def _is_source(relative: str, policy: Policy) -> bool:
    return is_source_path(relative, policy)


def _evaluate(
    relative: str,
    measurement: Measurement,
    exception: ExceptionRule | None,
) -> tuple[list[str], list[str]]:
    errors: list[str] = []
    warnings: list[str] = []
    raises_lines = exception and exception.affected_rule in {"lines", "lines_and_bytes"}
    raises_bytes = exception and exception.affected_rule in {"bytes", "lines_and_bytes"}
    hard_lines = exception.temporary_lines if raises_lines else 500
    hard_bytes = exception.temporary_bytes if raises_bytes else 65_536
    if measurement.line_count > hard_lines:
        errors.append(f"hard line limit exceeded: {relative}: {measurement.line_count} / {hard_lines}")
    if measurement.byte_count > hard_bytes:
        errors.append(f"hard byte limit exceeded: {relative}: {measurement.byte_count} / {hard_bytes}")
    if measurement.line_count >= 300:
        warnings.append(f"modularization review: {relative}: {measurement.line_count} lines / 300")
    if measurement.byte_count >= 32_768:
        warnings.append(f"modularization review: {relative}: {measurement.byte_count} bytes / 32768")
    return errors, warnings


def check_repository(root: Path) -> CheckResult:
    root = root.resolve()
    policy, policy_errors = load_policy(root)
    result = CheckResult(list(policy_errors), [])
    if policy is None:
        return result
    reviews, review_errors = load_modularization_reviews(root, policy.modularization_review_file)
    result.errors.extend(review_errors)
    paths, enumeration_errors = repository_paths(root)
    result.errors.extend(enumeration_errors)
    _check_startup_context(root, policy, result)
    seen_exceptions: set[str] = set()
    seen_reviews: set[str] = set()
    for relative in paths:
        if _is_excluded(relative, policy) or not _is_source(relative, policy):
            continue
        if not _is_allowed_source(relative, policy):
            result.errors.append(f"source or configuration file is outside approved roots: {relative}")
            continue
        path = _safe_file(root, relative, result)
        if path is None:
            continue
        try:
            measurement = measure_file(path)
        except OSError as exc:
            result.errors.append(f"cannot read repository file {relative!r}: {exc}")
            continue
        exception = policy.exceptions.get(relative)
        if exception:
            seen_exceptions.add(relative)
        errors, warnings = _evaluate(relative, measurement, exception)
        result.errors.extend(errors)
        result.warnings.extend(warnings)
        if warnings:
            review = reviews.get(relative)
            if review is None:
                result.errors.append(f"missing modularization review decision: {relative}")
            else:
                seen_reviews.add(relative)
                if review.decision == "split":
                    result.errors.append(f"open modularization split decision: {relative}")
        result.checked_files += 1
    for relative in sorted(set(policy.exceptions) - seen_exceptions):
        result.errors.append(f"stale or non-source source-limit exception: {relative}")
    for relative in sorted(set(reviews) - seen_reviews):
        result.errors.append(f"stale or below-threshold modularization review: {relative}")
    return result


def main(argv: Iterable[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument("--warnings-as-errors", action="store_true")
    args = parser.parse_args(argv)
    result = check_repository(args.root)
    for warning in result.warnings:
        print(f"warning: {warning}", file=sys.stderr)
    errors = list(result.errors)
    if args.warnings_as_errors:
        errors.extend(result.warnings)
    if errors:
        print("repository source-limit check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print(
        f"repository source-limit check passed ({result.checked_files} source/config files; "
        f"startup={result.startup_files} files/{result.startup_lines} lines/"
        f"{result.startup_bytes} bytes/max-line-{result.startup_max_line_bytes}-bytes/"
        f"max-hops-{result.startup_max_hops})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
