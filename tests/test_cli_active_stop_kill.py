from __future__ import annotations

import io
import json
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import Mock, patch

from runhaven.cli import main


class CliActiveStopKillTests(unittest.TestCase):
    def test_runs_stop_stops_active_run_container(self) -> None:
        with TemporaryDirectory() as directory:
            active_dir = Path(directory) / "active-runs"
            active_dir.mkdir()
            active_path = active_dir / "run-active.json"
            active_path.write_text(
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
            output = io.StringIO()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.subprocess.run") as run,
                redirect_stdout(output),
            ):
                run.return_value = Mock(returncode=0)
                code = main(["runs", "stop", "run-active"])

            self.assertEqual(code, 0)
            run.assert_called_once_with(
                ("container", "stop", "runhaven-shell-abc-run"),
                check=False,
            )
            text = output.getvalue()
            self.assertIn("Stop requested", text)
            updated = json.loads(active_path.read_text(encoding="utf-8"))
            self.assertEqual(updated["status"], "stop-requested")
            self.assertIn("stop_requested_at", updated)

    def test_runs_kill_kills_active_run_container(self) -> None:
        with TemporaryDirectory() as directory:
            active_dir = Path(directory) / "active-runs"
            active_dir.mkdir()
            active_path = active_dir / "run-active.json"
            active_path.write_text(
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
            output = io.StringIO()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.subprocess.run") as run,
                redirect_stdout(output),
            ):
                run.return_value = Mock(returncode=0)
                code = main(["runs", "kill", "run-active"])

            self.assertEqual(code, 0)
            run.assert_called_once_with(
                ("container", "kill", "runhaven-shell-abc-run"),
                check=False,
            )
            text = output.getvalue()
            self.assertIn("Kill requested", text)
            updated = json.loads(active_path.read_text(encoding="utf-8"))
            self.assertEqual(updated["status"], "kill-requested")
            self.assertIn("kill_requested_at", updated)

    def test_runs_kill_rolls_back_marker_when_container_kill_fails(self) -> None:
        with TemporaryDirectory() as directory:
            active_dir = Path(directory) / "active-runs"
            active_dir.mkdir()
            active_path = active_dir / "run-active.json"
            active_path.write_text(
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
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.subprocess.run") as run,
            ):
                run.return_value = Mock(returncode=7)
                code = main(["runs", "kill", "run-active"])

            self.assertEqual(code, 7)
            updated = json.loads(active_path.read_text(encoding="utf-8"))
            self.assertEqual(updated["status"], "running")
            self.assertNotIn("kill_requested_at", updated)

    def test_runs_stop_refuses_missing_active_run(self) -> None:
        with TemporaryDirectory() as directory:
            error_output = io.StringIO()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                redirect_stderr(error_output),
                self.assertRaises(SystemExit) as error,
            ):
                main(["runs", "stop", "missing-run"])

        self.assertEqual(error.exception.code, 2)
        self.assertIn("active run not found", error_output.getvalue())

    def test_runs_stop_refuses_unowned_container_name(self) -> None:
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
                redirect_stderr(error_output),
                self.assertRaises(SystemExit) as error,
            ):
                main(["runs", "stop", "run-active"])

        self.assertEqual(error.exception.code, 2)
        require_container.assert_not_called()
        self.assertIn("not a RunHaven-owned container", error_output.getvalue())

    def test_runs_kill_refuses_unowned_container_name(self) -> None:
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
                main(["runs", "kill", "run-active"])

        self.assertEqual(error.exception.code, 2)
        require_container.assert_not_called()
        run.assert_not_called()
        self.assertIn("not a RunHaven-owned container", error_output.getvalue())


if __name__ == "__main__":
    unittest.main()
