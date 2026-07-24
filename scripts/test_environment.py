#!/usr/bin/env python3
"""Adversarial fixtures for the read-only environment readiness contract."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from check_environment import check_environment


class EnvironmentTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory(prefix="runhaven harness 'ü; ")
        self.root = Path(self.temp.name)
        (self.root / ".python-version").write_text("3.14.6\n", encoding="utf-8")
        (self.root / ".node-version").write_text("26.5.0\n", encoding="utf-8")
        (self.root / "rust-toolchain.toml").write_text(
            '[toolchain]\nchannel = "1.97.0"\n',
            encoding="utf-8",
        )
        (self.root / "package.json").write_text(
            json.dumps(
                {
                    "packageManager": "npm@11.17.0",
                    "engines": {"node": "26.5.0", "npm": "11.17.0"},
                }
            ),
            encoding="utf-8",
        )

    def tearDown(self) -> None:
        self.temp.cleanup()

    @staticmethod
    def command_result(command: list[str], **_kwargs: object) -> subprocess.CompletedProcess[str]:
        if command[-1] == "--version" and command[0].endswith("node"):
            output = "v26.5.0\n"
        elif command[-1] == "--version" and command[0].endswith("npm"):
            output = "11.17.0\n"
        elif command[-1] == "--version" and command[0].endswith("container"):
            output = "container CLI version 1.1.0\n"
        elif "rustc" in command:
            output = "rustc 1.97.0 (fixture)\n"
        else:
            return subprocess.CompletedProcess(command, 1, "unexpected command\n")
        return subprocess.CompletedProcess(command, 0, output)

    def check(self, *, full: bool = False) -> list[str]:
        with (
            patch("check_environment.platform.system", return_value="Darwin"),
            patch("check_environment.platform.mac_ver", return_value=("26.0", ("", "", ""), "")),
            patch("check_environment.platform.machine", return_value="arm64"),
            patch("check_environment.platform.python_version", return_value="3.14.6"),
            patch("check_environment.shutil.which", side_effect=lambda name: f"/usr/bin/{name}"),
            patch("check_environment.subprocess.run", side_effect=self.command_result),
        ):
            return check_environment(self.root, full=full)

    def test_valid_supported_environment_and_path_with_spaces_pass(self) -> None:
        self.assertEqual(self.check(full=True), [])

    def test_rejects_unsupported_os_version_and_architecture(self) -> None:
        with (
            patch("check_environment.platform.system", return_value="Darwin"),
            patch("check_environment.platform.mac_ver", return_value=("25.9", ("", "", ""), "")),
            patch("check_environment.platform.machine", return_value="x86_64"),
            patch("check_environment.platform.python_version", return_value="3.14.6"),
            patch("check_environment.shutil.which", return_value=None),
        ):
            errors = check_environment(self.root)
        self.assertTrue(any("macOS 26" in error for error in errors), errors)
        self.assertTrue(any("Apple silicon" in error for error in errors), errors)

    def test_rejects_runtime_projection_and_command_version_drift(self) -> None:
        package = json.loads((self.root / "package.json").read_text(encoding="utf-8"))
        package["engines"]["node"] = "25.0.0"
        (self.root / "package.json").write_text(json.dumps(package), encoding="utf-8")
        errors = self.check()
        self.assertTrue(any("engines.node must match" in error for error in errors), errors)

    def test_rejects_missing_command_with_exact_name(self) -> None:
        with (
            patch("check_environment.platform.system", return_value="Darwin"),
            patch("check_environment.platform.mac_ver", return_value=("26.0", ("", "", ""), "")),
            patch("check_environment.platform.machine", return_value="arm64"),
            patch("check_environment.platform.python_version", return_value="3.14.6"),
            patch(
                "check_environment.shutil.which",
                side_effect=lambda name: None if name == "container" else f"/usr/bin/{name}",
            ),
            patch("check_environment.subprocess.run", side_effect=self.command_result),
        ):
            errors = check_environment(self.root)
        self.assertIn("required command not found: container", errors)

    def test_rejects_command_timeout_without_hanging(self) -> None:
        def timeout(command: list[str], **_kwargs: object) -> subprocess.CompletedProcess[str]:
            if command[0].endswith("node"):
                raise subprocess.TimeoutExpired(command, 30)
            return self.command_result(command)

        with (
            patch("check_environment.platform.system", return_value="Darwin"),
            patch("check_environment.platform.mac_ver", return_value=("26.0", ("", "", ""), "")),
            patch("check_environment.platform.machine", return_value="arm64"),
            patch("check_environment.platform.python_version", return_value="3.14.6"),
            patch("check_environment.shutil.which", side_effect=lambda name: f"/usr/bin/{name}"),
            patch("check_environment.subprocess.run", side_effect=timeout),
        ):
            errors = check_environment(self.root)
        self.assertTrue(any("cannot inspect Node.js" in error for error in errors), errors)

    def test_rejects_unreadable_runtime_owner_without_traceback(self) -> None:
        with patch.object(Path, "read_text", side_effect=OSError("permission denied")):
            errors = self.check()
        self.assertTrue(any("cannot read runtime owner .python-version" in error for error in errors), errors)

    def test_rejects_symlinked_runtime_owner(self) -> None:
        target = self.root / "python-version-target"
        target.write_text("3.14.6\n", encoding="utf-8")
        (self.root / ".python-version").unlink()
        (self.root / ".python-version").symlink_to(target)
        errors = self.check()
        self.assertTrue(any("missing regular runtime owner: .python-version" in error for error in errors), errors)


if __name__ == "__main__":
    unittest.main(verbosity=2)
