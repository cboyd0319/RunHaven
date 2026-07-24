"""Parse exact per-file modularization decisions for review-threshold source."""

from __future__ import annotations

import datetime as dt
import json
from dataclasses import dataclass
from pathlib import Path

from repository_policy import safe_relative


MAX_REVIEW_FILE_BYTES = 65_536
REVIEW_KEYS = {"path", "owner", "decision", "rationale", "review_date", "next_trigger"}


@dataclass(frozen=True)
class ModularizationReview:
    path: str
    owner: str
    decision: str
    rationale: str
    review_date: str
    next_trigger: str


def _text(entry: dict[str, object], field: str, label: str, errors: list[str]) -> str:
    value = entry.get(field)
    if not isinstance(value, str) or not value.strip():
        errors.append(f"{label}.{field} must be nonempty text")
        return ""
    return value.strip()


def load_modularization_reviews(
    root: Path, relative: str
) -> tuple[dict[str, ModularizationReview], list[str]]:
    errors: list[str] = []
    path = root / relative
    if path.is_symlink() or not path.is_file():
        return {}, [f"missing regular modularization review file: {relative}"]
    try:
        if path.stat().st_size > MAX_REVIEW_FILE_BYTES:
            return {}, [f"modularization review file exceeds {MAX_REVIEW_FILE_BYTES} bytes"]
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        return {}, [f"invalid modularization review file: {exc}"]
    if not isinstance(payload, dict) or set(payload) != {"schema_version", "reviews"}:
        return {}, ["modularization review file must contain only schema_version and reviews"]
    if payload.get("schema_version") != 1:
        errors.append("modularization review schema_version must equal 1")
    entries = payload.get("reviews")
    if not isinstance(entries, list):
        return {}, errors + ["modularization reviews must be an array"]
    reviews: dict[str, ModularizationReview] = {}
    for index, raw in enumerate(entries):
        label = f"modularization reviews[{index}]"
        if not isinstance(raw, dict):
            errors.append(f"{label} must be an object")
            continue
        if set(raw) != REVIEW_KEYS:
            errors.append(f"{label} must contain exactly {', '.join(sorted(REVIEW_KEYS))}")
        review_path = safe_relative(raw.get("path"), f"{label}.path", errors)
        owner = _text(raw, "owner", label, errors)
        decision = _text(raw, "decision", label, errors)
        rationale = _text(raw, "rationale", label, errors)
        review_date = _text(raw, "review_date", label, errors)
        next_trigger = _text(raw, "next_trigger", label, errors)
        if decision not in {"retain", "split"}:
            errors.append(f"{label}.decision must be retain or split")
        try:
            parsed_date = dt.date.fromisoformat(review_date)
            if parsed_date > dt.date.today():
                errors.append(f"{label}.review_date must not be in the future")
        except ValueError:
            errors.append(f"{label}.review_date must be YYYY-MM-DD")
        if review_path:
            if review_path in reviews:
                errors.append(f"duplicate modularization review path: {review_path}")
            reviews[review_path] = ModularizationReview(
                review_path,
                owner,
                decision,
                rationale,
                review_date,
                next_trigger,
            )
    return reviews, errors
