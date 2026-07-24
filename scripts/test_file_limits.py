#!/usr/bin/env python3
"""Adversarial fixtures for canonical source-limit enforcement."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path

from check_file_limits import check_repository, measure_file


class FileLimitTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        subprocess.run(
            ["git", "init", "-b", "main"],
            cwd=self.root,
            check=True,
            stdout=subprocess.DEVNULL,
        )
        self.policy = {
            "schema_version": 1,
            "source_limits": {
                "counting": "physical_lines_including_blank_and_comment_lines",
                "review_lines": 300,
                "hard_lines": 500,
                "review_bytes": 32768,
                "hard_bytes": 65536,
            },
            "included_extensions": [".json", ".py", ".rs"],
            "included_source_paths": [],
            "startup_context": {
                "files": ["repository-structure-policy.json"],
                "state_reads": [],
                "loading_hops": {"repository-structure-policy.json": 0},
                "warning_percent": 90,
                "hard_lines": 700,
                "hard_bytes": 49152,
            },
            "modularization_review_file": "src/modularization-reviews.json",
            "structure": {
                "allowed_source_roots": ["src"],
                "unitized_source_roots": ["src"],
                "allowed_root_source_files": ["repository-structure-policy.json"],
                "forbidden_root_source_files": True,
                "architecture_check": "scripts/python.sh scripts/check_repo_layout.py",
                "source_limit_check": "scripts/python.sh scripts/check_file_limits.py",
                "units": [
                    {
                        "id": "fixture",
                        "kind": "reusable",
                        "root": "src",
                        "manifest": "src/manifest.json",
                        "public_entrypoints": ["src/lib.rs"],
                        "test_entrypoints": ["src/contract.test.sh"],
                        "documentation": ["src/README.md"],
                        "allowed_dependencies": [],
                    }
                ],
            },
            "non_hand_authored_exclusions": [],
            "hand_authored_inclusions": [],
            "retired_paths": [],
            "exceptions": [],
        }
        (self.root / "src").mkdir()
        (self.root / "src/manifest.json").write_text("{}\n", encoding="utf-8")
        (self.root / "src/lib.rs").write_text("pub fn fixture() {}\n", encoding="utf-8")
        (self.root / "src/contract.test.sh").write_text("#!/bin/sh\n", encoding="utf-8")
        (self.root / "src/README.md").write_text("# Fixture\n", encoding="utf-8")
        self.reviews: list[dict[str, str]] = []
        self.write_reviews()
        self.write_policy()
        self.assertEqual(check_repository(self.root).errors, [], "base fixture policy must parse")

    def tearDown(self) -> None:
        self.temp.cleanup()

    def write_policy(self) -> None:
        (self.root / "repository-structure-policy.json").write_text(
            json.dumps(self.policy, indent=2) + "\n",
            encoding="utf-8",
        )

    def write_reviews(self) -> None:
        (self.root / "src/modularization-reviews.json").write_text(
            json.dumps({"schema_version": 1, "reviews": self.reviews}, indent=2) + "\n",
            encoding="utf-8",
        )

    def retain(self, path: str) -> None:
        self.reviews.append(
            {
                "path": path,
                "owner": "fixture owner",
                "decision": "retain",
                "rationale": "fixture remains one focused responsibility",
                "review_date": "2026-07-14",
                "next_trigger": "split if another responsibility is added",
            }
        )
        self.write_reviews()

    def write_bytes(self, name: str, content: bytes) -> Path:
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
        return path

    def errors(self) -> list[str]:
        return check_repository(self.root).errors

    def warnings(self) -> list[str]:
        return check_repository(self.root).warnings

    def assert_error(self, text: str) -> None:
        errors = self.errors()
        self.assertTrue(any(text in error for error in errors), errors)

    def test_valid_untracked_source_passes(self) -> None:
        self.write_bytes("src/normal.rs", b"fn main() {}\n")
        result = check_repository(self.root)
        self.assertEqual(result.errors, [])
        self.assertEqual(result.warnings, [])

    def test_staged_source_is_scanned(self) -> None:
        self.write_bytes("src/staged.rs", b"x\n" * 501)
        subprocess.run(["git", "add", "src/staged.rs"], cwd=self.root, check=True)
        self.assert_error("hard line limit exceeded: src/staged.rs")

    def test_staged_and_worktree_content_divergence_fails_closed(self) -> None:
        self.write_bytes("src/staged.rs", b"x\n" * 501)
        subprocess.run(["git", "add", "src/staged.rs"], cwd=self.root, check=True)
        self.write_bytes("src/staged.rs", b"fn small() {}\n")
        self.assert_error("staged and working-tree content diverge")

    def test_exact_extensionless_source_path_is_enforced(self) -> None:
        self.policy["included_source_paths"] = ["src/Containerfile"]
        self.write_policy()
        self.write_bytes("src/Containerfile", b"x\n" * 501)
        self.assert_error("hard line limit exceeded: src/Containerfile")

    def test_startup_context_budget_and_measurement_are_enforced(self) -> None:
        self.policy["startup_context"] = {
            "files": ["src/startup.txt"],
            "state_reads": [],
            "loading_hops": {"src/startup.txt": 1},
            "warning_percent": 90,
            "hard_lines": 700,
            "hard_bytes": 49152,
        }
        self.write_policy()
        self.write_bytes("src/startup.txt", b"x\n" * 701)
        result = check_repository(self.root)
        self.assertTrue(any("startup context line budget exceeded" in item for item in result.errors))
        self.assertEqual(result.startup_max_line_bytes, 1)
        self.assertEqual(result.startup_max_hops, 1)

    def test_hard_line_failure_counts_final_unterminated_line(self) -> None:
        path = self.write_bytes("src/lines.py", b"x\n" * 500 + b"x")
        self.assertEqual(measure_file(path).line_count, 501)
        self.assert_error("501 / 500")

    def test_hard_byte_failure(self) -> None:
        self.write_bytes("src/bytes.rs", b"x" * 65_537)
        self.assert_error("65537 / 65536")

    def test_review_thresholds_warn_without_failing(self) -> None:
        self.write_bytes("src/lines.rs", b"x\n" * 300)
        self.write_bytes("src/bytes.py", b"x" * 32_768)
        self.retain("src/lines.rs")
        self.retain("src/bytes.py")
        result = check_repository(self.root)
        self.assertEqual(result.errors, [])
        self.assertTrue(any("300 lines / 300" in item for item in result.warnings))
        self.assertTrue(any("32768 bytes / 32768" in item for item in result.warnings))

    def test_review_threshold_requires_exact_closed_decision(self) -> None:
        self.write_bytes("src/large.rs", b"x\n" * 300)
        self.assert_error("missing modularization review decision: src/large.rs")
        self.reviews = [
            {
                "path": "src/large.rs",
                "owner": "fixture owner",
                "decision": "split",
                "rationale": "fixture has multiple responsibilities",
                "review_date": "2026-07-14",
                "next_trigger": "split before accepting the file",
            }
        ]
        self.write_reviews()
        self.assert_error("open modularization split decision: src/large.rs")

    def test_stale_modularization_review_fails(self) -> None:
        self.retain("src/lib.rs")
        self.assert_error("stale or below-threshold modularization review: src/lib.rs")

    def test_exact_path_exception_ratchets_only_named_rule(self) -> None:
        self.policy["exceptions"] = [
            {
                "path": "src/legacy.rs",
                "affected_rule": "lines",
                "owner": "runtime team",
                "reason": "split is tracked",
                "temporary_ceiling": {"lines": 520, "bytes": 70000},
                "approval_date": "2026-07-14",
                "retirement_condition": "remove after the runtime split",
            }
        ]
        self.write_policy()
        self.write_bytes("src/legacy.rs", b"x\n" * 510)
        self.retain("src/legacy.rs")
        self.assertEqual(self.errors(), [])
        self.write_bytes("src/legacy.rs", b"x" * 65_537)
        self.assert_error("hard byte limit exceeded")

    def test_exception_requires_exact_complete_record(self) -> None:
        self.policy["exceptions"] = [{"path": "src/*.rs"}]
        self.write_policy()
        errors = self.errors()
        self.assertTrue(any("must not contain a glob" in item for item in errors), errors)
        self.assertTrue(any("is missing owner" in item for item in errors), errors)

    def test_stale_exception_fails(self) -> None:
        self.policy["exceptions"] = [
            {
                "path": "src/gone.rs",
                "affected_rule": "lines_and_bytes",
                "owner": "runtime team",
                "reason": "migration",
                "temporary_ceiling": {"lines": 501, "bytes": 65537},
                "approval_date": "2026-07-14",
                "retirement_condition": "delete after split",
            }
        ]
        self.write_policy()
        self.assert_error("stale or non-source")

    def test_non_hand_authored_exclusion_is_exact_and_owned(self) -> None:
        self.policy["non_hand_authored_exclusions"] = [
            {
                "path": "src/generated",
                "category": "generated",
                "owner": "fixture generator",
                "reason": "generated fixture",
            }
        ]
        self.write_policy()
        self.write_bytes("src/generated/large.rs", b"x\n" * 501)
        self.assertEqual(self.errors(), [])

    def test_hand_authored_subtree_inside_exclusion_is_still_enforced(self) -> None:
        self.policy["non_hand_authored_exclusions"] = [
            {
                "path": "src/vendor",
                "category": "vendored",
                "owner": "fixture vendor",
                "reason": "upstream fixture",
            }
        ]
        self.policy["hand_authored_inclusions"] = ["src/vendor/local"]
        self.write_policy()
        self.write_bytes("src/vendor/upstream.rs", b"x\n" * 501)
        self.write_bytes("src/vendor/local/owned.rs", b"x\n" * 501)
        errors = self.errors()
        self.assertTrue(
            any("hard line limit exceeded: src/vendor/local/owned.rs" in item for item in errors),
            errors,
        )
        self.assertFalse(any("src/vendor/upstream.rs" in item for item in errors), errors)

    def test_hand_authored_inclusion_outside_exclusion_fails(self) -> None:
        self.policy["hand_authored_inclusions"] = ["src/local"]
        self.write_policy()
        self.assert_error(
            "hand-authored inclusion must be inside a non-hand-authored exclusion: src/local"
        )

    def test_exclusion_glob_or_missing_metadata_fails(self) -> None:
        self.policy["non_hand_authored_exclusions"] = [
            {"path": "src/generated/*", "category": "generated"}
        ]
        self.write_policy()
        errors = self.errors()
        self.assertTrue(any("must not contain a glob" in item for item in errors), errors)
        self.assertTrue(any("is missing owner" in item for item in errors), errors)

    def test_source_outside_approved_roots_fails(self) -> None:
        self.write_bytes("rogue.rs", b"fn rogue() {}\n")
        self.assert_error("outside approved roots: rogue.rs")

    def test_canonical_limit_drift_fails_closed(self) -> None:
        self.policy["source_limits"]["hard_lines"] = 600
        self.write_policy()
        self.assert_error("source_limits.hard_lines must be 500")

    def test_symlink_is_rejected_without_following_it(self) -> None:
        target = self.write_bytes("src/target.rs", b"safe\n")
        (self.root / "src/link.rs").symlink_to(target)
        self.assert_error("repository scan rejects symlink: src/link.rs")

    def test_ignored_source_is_not_scanned(self) -> None:
        self.write_bytes(".gitignore", b"src/ignored.rs\n")
        self.write_bytes("src/ignored.rs", b"x\n" * 501)
        self.assertEqual(self.errors(), [])

    def test_crlf_is_counted_without_normalization(self) -> None:
        path = self.write_bytes("src/crlf.rs", b"a\r\nb\r\n")
        measurement = measure_file(path)
        self.assertEqual(measurement.line_count, 2)
        self.assertEqual(measurement.byte_count, 6)


if __name__ == "__main__":
    unittest.main(verbosity=2)
