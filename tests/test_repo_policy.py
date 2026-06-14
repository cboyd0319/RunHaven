from __future__ import annotations

import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class RepoPolicyTests(unittest.TestCase):
    def test_repo_declares_macos_only_runtime(self) -> None:
        checked_paths = (
            ROOT / "AGENTS.md",
            ROOT / "README.md",
            ROOT / "SECURITY.md",
            ROOT / "CONTRIBUTING.md",
            ROOT / "docs/harness/README.md",
            ROOT / "docs/harness/component-inventory.md",
            ROOT / "docs/harness/manifest.json",
            ROOT / "docs/harness/verification-matrix.md",
            ROOT / "progress.md",
            ROOT / "session-handoff.md",
        )
        text = "\n".join(path.read_text(encoding="utf-8") for path in checked_paths)

        self.assertIn("macOS 26+", text)
        self.assertNotIn("Windows 11", text)
        self.assertNotIn("Ubuntu", text)
        self.assertNotIn("Linux import", text)

    def test_security_policy_uses_reviewed_container_pin(self) -> None:
        text = (ROOT / "SECURITY.md").read_text(encoding="utf-8")

        self.assertIn("`container` 1.0.0", text)
        self.assertNotIn("`container` 1.x", text)

    def test_pin_ledger_declares_only_macos_runner(self) -> None:
        pins = tomllib.loads((ROOT / "pins.toml").read_text(encoding="utf-8"))

        self.assertEqual(set(pins["github_runners"]), {"macos"})

    def test_ci_runs_only_on_macos_26(self) -> None:
        text = (ROOT / ".github/workflows/ci.yml").read_text(encoding="utf-8")

        self.assertIn("macos-26", text)
        self.assertNotIn("ubuntu", text.lower())
        self.assertNotIn("windows", text.lower())

    def test_no_windows_entrypoint_is_published(self) -> None:
        self.assertFalse((ROOT / "init.ps1").exists())


if __name__ == "__main__":
    unittest.main()
