from __future__ import annotations

import io
import json
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import Mock, patch

from runhaven.cli import main


class CliActiveStatusTests(unittest.TestCase):
    def test_runs_status_prints_sanitized_active_container_state(self) -> None:
        with TemporaryDirectory() as directory:
            active_dir = Path(directory) / "active-runs"
            active_dir.mkdir()
            (active_dir / "run-active.json").write_text(
                json.dumps(
                    {
                        "timestamp": "2026-06-15T00:00:00Z",
                        "run_id": "run-active",
                        "profile": "shell",
                        "workspace": directory,
                        "network": "internet",
                        "status": "running",
                        "container_name": "runhaven-shell-abc-run",
                        "host_pid": 12345,
                        "command": "do-not-print",
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            inspect_payload = [
                {
                    "id": "runhaven-shell-abc-run",
                    "configuration": {
                        "image": {"reference": "runhaven/base:0.1.0"},
                        "initProcess": {
                            "arguments": ["agent", "--secret-flag"],
                            "environment": ["OPENAI_API_KEY=fake-secret-value"],
                        },
                        "mounts": [{"source": "/host/private", "destination": "/workspace"}],
                    },
                    "status": {
                        "state": "running",
                        "startedDate": "2026-06-15T00:00:10Z",
                        "networks": [
                            {
                                "network": "default",
                                "hostname": "runhaven-shell-abc-run",
                                "ipv4Address": "192.168.64.20/24",
                                "ipv4Gateway": "192.168.64.1",
                            }
                        ],
                    },
                }
            ]
            output = io.StringIO()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.subprocess.run") as run,
                redirect_stdout(output),
            ):
                run.return_value = Mock(
                    returncode=0,
                    stdout=json.dumps(inspect_payload),
                    stderr="",
                )
                code = main(["runs", "status", "run-active"])

        self.assertEqual(code, 0)
        run.assert_called_once_with(
            ("container", "inspect", "runhaven-shell-abc-run"),
            check=False,
            capture_output=True,
            text=True,
        )
        text = output.getvalue()
        self.assertIn("Run id: run-active", text)
        self.assertIn("Marker status: running", text)
        self.assertIn("Container state: running", text)
        self.assertIn("Container started: 2026-06-15T00:00:10Z", text)
        self.assertIn("Container image: runhaven/base:0.1.0", text)
        self.assertIn("default ipv4=192.168.64.20/24", text)
        self.assertNotIn("fake-secret-value", text)
        self.assertNotIn("OPENAI_API_KEY", text)
        self.assertNotIn("secret-flag", text)
        self.assertNotIn("/host/private", text)
        self.assertNotIn("do-not-print", text)

    def test_runs_status_json_is_sanitized(self) -> None:
        with TemporaryDirectory() as directory:
            active_dir = Path(directory) / "active-runs"
            active_dir.mkdir()
            (active_dir / "run-active.json").write_text(
                json.dumps(
                    {
                        "timestamp": "2026-06-15T00:00:00Z",
                        "run_id": "run-active",
                        "profile": "shell",
                        "workspace": directory,
                        "network": "provider",
                        "status": "running",
                        "container_name": "runhaven-shell-abc-run",
                        "state_volume": "runhaven-shell-abc-home",
                        "network_name": "runhaven-provider-abc",
                        "host_pid": 12345,
                        "command": "do-not-print",
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            inspect_payload = [
                {
                    "id": "runhaven-shell-abc-run",
                    "configuration": {
                        "image": {"reference": "runhaven/base:0.1.0"},
                        "resources": {"cpus": 2, "memoryInBytes": 1073741824},
                        "initProcess": {
                            "arguments": ["agent", "--secret-flag"],
                            "environment": ["ANTHROPIC_API_KEY=fake-secret-value"],
                        },
                    },
                    "status": {
                        "state": "running",
                        "startedDate": "2026-06-15T00:00:10Z",
                        "networks": [{"network": "default", "ipv4Address": "192.168.64.20/24"}],
                    },
                }
            ]
            output = io.StringIO()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.subprocess.run") as run,
                redirect_stdout(output),
            ):
                run.return_value = Mock(
                    returncode=0,
                    stdout=json.dumps(inspect_payload),
                    stderr="",
                )
                code = main(["runs", "status", "run-active", "--json"])

        self.assertEqual(code, 0)
        payload = json.loads(output.getvalue())
        self.assertEqual(payload["active_run"]["run_id"], "run-active")
        self.assertEqual(payload["active_run"]["network"], "provider")
        self.assertEqual(payload["container"]["state"], "running")
        self.assertEqual(payload["container"]["image"], "runhaven/base:0.1.0")
        self.assertEqual(payload["container"]["resources"]["cpus"], 2)
        serialized = json.dumps(payload)
        self.assertNotIn("fake-secret-value", serialized)
        self.assertNotIn("ANTHROPIC_API_KEY", serialized)
        self.assertNotIn("secret-flag", serialized)
        self.assertNotIn("do-not-print", serialized)

    def test_runs_status_refuses_unowned_container_name(self) -> None:
        with TemporaryDirectory() as directory:
            active_dir = Path(directory) / "active-runs"
            active_dir.mkdir()
            (active_dir / "run-active.json").write_text(
                json.dumps(
                    {
                        "timestamp": "2026-06-15T00:00:00Z",
                        "run_id": "run-active",
                        "profile": "shell",
                        "workspace": directory,
                        "network": "internet",
                        "status": "running",
                        "container_name": "other-container",
                        "host_pid": 12345,
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            error_output = io.StringIO()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli") as require_container,
                patch("runhaven.cli.subprocess.run") as run,
                redirect_stderr(error_output),
                self.assertRaises(SystemExit) as error,
            ):
                main(["runs", "status", "run-active"])

        self.assertEqual(error.exception.code, 2)
        require_container.assert_not_called()
        run.assert_not_called()
        self.assertIn("not a RunHaven-owned container", error_output.getvalue())

    def test_runs_status_returns_container_inspect_failure(self) -> None:
        with TemporaryDirectory() as directory:
            active_dir = Path(directory) / "active-runs"
            active_dir.mkdir()
            (active_dir / "run-active.json").write_text(
                json.dumps(
                    {
                        "timestamp": "2026-06-15T00:00:00Z",
                        "run_id": "run-active",
                        "profile": "shell",
                        "workspace": directory,
                        "network": "internet",
                        "status": "running",
                        "container_name": "runhaven-shell-abc-run",
                        "host_pid": 12345,
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            error_output = io.StringIO()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.subprocess.run") as run,
                redirect_stderr(error_output),
            ):
                run.return_value = Mock(returncode=7, stdout="", stderr="not found\n")
                code = main(["runs", "status", "run-active"])

        self.assertEqual(code, 7)
        self.assertIn("container inspect failed", error_output.getvalue())


if __name__ == "__main__":
    unittest.main()
