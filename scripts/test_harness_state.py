#!/usr/bin/env python3
"""Adversarial fixtures for the harness state validator."""

from __future__ import annotations

import copy
import datetime as dt
import json
import subprocess
import tempfile
import unittest
from pathlib import Path

from check_harness_state import check_repository


def feature(feature_id: str, status: str, *, priority: int = 1) -> dict[str, object]:
    item: dict[str, object] = {
        "id": feature_id,
        "priority": priority,
        "area": "test",
        "title": feature_id,
        "user_visible_behavior": f"Observable behavior for {feature_id}.",
        "status": status,
        "dependencies": [],
        "verification": ["python3 scripts/check.py"],
        "evidence": ["evidence.md"] if status == "passing" else [],
    }
    if status == "blocked":
        item["blocker"] = "A real external condition is missing."
        item["next_trigger"] = "The external condition becomes available."
    return item


class HarnessFixture:
    def __init__(self, root: Path) -> None:
        self.root = root
        self.today = dt.date.today().isoformat()
        (root / "scripts").mkdir()
        (root / "AGENTS.md").write_text("# Test repository\n", encoding="utf-8")
        (root / "init.sh").write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
        (root / "scripts/check.py").write_text("print('ok')\n", encoding="utf-8")
        (root / "evidence.md").write_text(
            "\n".join(
                (
                    "# Evidence",
                    "",
                    "## Passing feature evidence index",
                    "",
                    "| Feature ID | Dated evidence scope |",
                    "| --- | --- |",
                    f"| `complete-feature` | {self.today}, Complete fixture verification |",
                    "",
                    "| Date | Scope | Command Or Review | Result | Notes |",
                    "| --- | --- | --- | --- | --- |",
                    f"| {self.today} | Complete fixture verification | `python3 scripts/check.py` | passed | Fixture command exited successfully. |",
                    "",
                )
            ),
            encoding="utf-8",
        )
        self.payload: dict[str, object] = {
            "$schema": "docs/harness/schemas/feature-list.schema.json",
            "schema_version": 1,
            "project": "RunHaven",
            "last_updated": self.today,
            "status_legend": {
                "not_started": "queued",
                "active": "current",
                "blocked": "blocked",
                "passing": "complete",
            },
            "rules": {
                "active_count": 1,
                "passing_is_irreversible": True,
                "evidence_is_external": True,
            },
            "features": [feature("complete-feature", "passing"), feature("active-feature", "active")],
        }
        self.write()

    @property
    def features(self) -> list[dict[str, object]]:
        return self.payload["features"]  # type: ignore[return-value]

    def write(self, *, active: str = "active-feature", status: str = "active") -> None:
        (self.root / "feature_list.json").write_text(
            json.dumps(self.payload, indent=2) + "\n",
            encoding="utf-8",
        )
        (self.root / "PROGRESS.md").write_text(
            "\n".join(
                (
                    "# Progress",
                    "",
                    f"Last updated: {self.today}",
                    "Branch: `main`",
                    f"Active feature: `{active}`",
                    f"Status: `{status}`",
                    "",
                    "## Exact next action",
                    "",
                    "Continue the active fixture.",
                    "",
                )
            ),
            encoding="utf-8",
        )


class HarnessStateTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.fixture = HarnessFixture(self.root)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def errors(self, *, history: bool = False) -> list[str]:
        return check_repository(self.root, compare_history=history)

    def assert_error(self, text: str) -> None:
        self.assertTrue(any(text in error for error in self.errors()), self.errors())

    def test_valid_fixture_passes(self) -> None:
        self.assertEqual(self.errors(), [])

    def test_rejects_retired_state_file(self) -> None:
        (self.root / "session-handoff.md").write_text("stale\n", encoding="utf-8")
        self.assert_error("retired root state file")

    def test_rejects_invalid_status_and_multiple_active_features(self) -> None:
        self.fixture.features[0]["status"] = "done"
        self.fixture.features.append(feature("second-active", "active"))
        self.fixture.write()
        self.assert_error("invalid status")
        self.assert_error("exactly one feature must be active")

    def test_rejects_unresolved_and_cyclic_dependencies(self) -> None:
        self.fixture.features[0]["dependencies"] = ["missing-feature"]
        self.fixture.features[1]["dependencies"] = ["complete-feature"]
        self.fixture.write()
        self.assert_error("unresolved dependency")
        self.fixture.features[0]["dependencies"] = ["active-feature"]
        self.fixture.write()
        self.assert_error("dependency cycle")

    def test_rejects_duplicate_stable_id(self) -> None:
        self.fixture.features.append(copy.deepcopy(self.fixture.features[0]))
        self.fixture.write()
        self.assert_error("duplicate feature id")

    def test_rejects_missing_passing_evidence(self) -> None:
        self.fixture.features[0]["evidence"] = []
        self.fixture.write()
        self.assert_error("requires evidence")

    def test_rejects_missing_or_symlinked_evidence(self) -> None:
        self.fixture.features[0]["evidence"] = ["missing.md"]
        self.fixture.write()
        self.assert_error("does not resolve inside")
        (self.root / "evidence-link.md").symlink_to(self.root / "evidence.md")
        self.fixture.features[0]["evidence"] = ["evidence-link.md"]
        self.fixture.write()
        self.assert_error("must not point to a symlink")

    def test_rejects_passing_evidence_that_does_not_identify_feature(self) -> None:
        (self.root / "evidence.md").write_text("# Unrelated evidence\n", encoding="utf-8")
        self.assert_error("does not identify feature complete-feature")

    def test_rejects_index_without_matching_passing_detail(self) -> None:
        evidence = (self.root / "evidence.md").read_text(encoding="utf-8")
        (self.root / "evidence.md").write_text(
            evidence.replace("| passed |", "| failed |"),
            encoding="utf-8",
        )
        self.assert_error("lacks command, passing result, or notes")

    def test_rejects_startup_state_as_self_referential_evidence(self) -> None:
        self.fixture.features[0]["evidence"] = ["feature_list.json"]
        self.fixture.write()
        self.assert_error("evidence must be outside the startup pack")

    def test_checks_every_evidence_file_before_accepting_a_match(self) -> None:
        (self.root / "oversized-evidence.md").write_text(
            "x" * (256 * 1024 + 1),
            encoding="utf-8",
        )
        self.fixture.features[0]["evidence"] = ["evidence.md", "oversized-evidence.md"]
        self.fixture.write()
        self.assert_error("evidence exceeds bounded read limit")

    def test_rejects_blocked_feature_without_recovery_trigger(self) -> None:
        blocked = feature("blocked-feature", "blocked")
        del blocked["next_trigger"]
        self.fixture.features.append(blocked)
        self.fixture.write()
        self.assert_error("requires next_trigger")

    def test_rejects_prose_instead_of_runnable_verification(self) -> None:
        self.fixture.features[0]["verification"] = ["Review the behavior manually"]
        self.fixture.write()
        self.assert_error("unsupported executable")

    def test_rejects_unknown_cargo_subcommand(self) -> None:
        self.fixture.features[0]["verification"] = ["cargo definitely-not-a-command"]
        self.fixture.write()
        self.assert_error("unsupported Cargo command")

    def test_rejects_shell_composition_in_verification(self) -> None:
        self.fixture.features[0]["verification"] = ["cargo test && echo pass"]
        self.fixture.write()
        self.assert_error("without shell composition")

    def test_rejects_lower_priority_active_feature(self) -> None:
        self.fixture.features.append(feature("higher-priority", "not_started", priority=0))
        self.fixture.write()
        self.assert_error("not the highest-priority unfinished feature")

    def test_rejects_progress_disagreement(self) -> None:
        self.fixture.write(active="complete-feature", status="passing")
        self.assert_error("active feature disagrees")
        self.assert_error("status must be active")

    def test_rejects_date_disagreement_and_unknown_root_key(self) -> None:
        self.fixture.payload["last_updated"] = (dt.date.today() - dt.timedelta(days=1)).isoformat()
        self.fixture.payload["unexpected"] = True
        self.fixture.write()
        self.assert_error("update dates disagree")
        self.assert_error("unknown root keys")

    def test_rejects_machine_local_state(self) -> None:
        progress = (self.root / "PROGRESS.md").read_text(encoding="utf-8")
        fake_home = "/" + "Users/private/work"
        (self.root / "PROGRESS.md").write_text(progress + fake_home + "\n", encoding="utf-8")
        self.assert_error("machine-local absolute path")

    def test_rejects_oversized_state_before_parsing(self) -> None:
        (self.root / "PROGRESS.md").write_text("x" * (256 * 1024 + 1), encoding="utf-8")
        self.assert_error("exceeds bounded read limit")

    def test_rejects_demotion_of_committed_passing_feature(self) -> None:
        subprocess.run(["git", "init", "-b", "main"], cwd=self.root, check=True, stdout=subprocess.DEVNULL)
        subprocess.run(["git", "config", "user.email", "fixture@example.invalid"], cwd=self.root, check=True)
        subprocess.run(["git", "config", "user.name", "Fixture"], cwd=self.root, check=True)
        subprocess.run(["git", "add", "."], cwd=self.root, check=True)
        subprocess.run(["git", "commit", "-m", "fixture"], cwd=self.root, check=True, stdout=subprocess.DEVNULL)
        self.fixture.features[0]["status"] = "not_started"
        self.fixture.write()
        errors = self.errors(history=True)
        self.assertTrue(any("irreversible" in error for error in errors), errors)

    def test_irreversible_history_failure_fails_closed(self) -> None:
        errors = self.errors(history=True)
        self.assertTrue(any("cannot read committed feature state" in error for error in errors), errors)

    def test_state_record_budget_has_warning_and_hard_limit(self) -> None:
        self.fixture.features[1]["title"] = "x" * 1750
        self.fixture.write()
        warnings: list[str] = []
        self.assertEqual(check_repository(self.root, compare_history=False, warnings=warnings), [])
        self.assertTrue(any("record_bytes budget warning" in item for item in warnings), warnings)
        self.fixture.features[1]["title"] = "x" * 2300
        self.fixture.write()
        self.assert_error("state record_bytes budget exceeded")


if __name__ == "__main__":
    unittest.main(verbosity=2)
