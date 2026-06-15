from __future__ import annotations

import io
import json
import unittest
from contextlib import redirect_stderr
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

from runhaven.cli import main


class CliActiveAttachLogsTests(unittest.TestCase):
    def test_runs_attach_execs_shell_in_active_container(self) -> None:
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
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.subprocess.call", return_value=0) as call,
                patch("runhaven.cli.sys.stdin.isatty", return_value=True),
                patch("runhaven.cli.sys.stdout.isatty", return_value=True),
            ):
                code = main(["runs", "attach", "run-active"])

        self.assertEqual(code, 0)
        call.assert_called_once_with(
            (
                "container",
                "exec",
                "--interactive",
                "--tty",
                "--user",
                "agent",
                "--workdir",
                "/workspace",
                "runhaven-shell-abc-run",
                "/bin/bash",
            )
        )

    def test_runs_attach_uses_custom_command_without_tty_when_requested(self) -> None:
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
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.subprocess.call", return_value=7) as call,
            ):
                code = main(
                    [
                        "runs",
                        "attach",
                        "run-active",
                        "--tty",
                        "never",
                        "--",
                        "pwd",
                    ]
                )

        self.assertEqual(code, 7)
        command = call.call_args.args[0]
        self.assertNotIn("--tty", command)
        self.assertEqual(
            command,
            (
                "container",
                "exec",
                "--interactive",
                "--user",
                "agent",
                "--workdir",
                "/workspace",
                "runhaven-shell-abc-run",
                "pwd",
            ),
        )

    def test_runs_attach_refuses_unowned_container_name(self) -> None:
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
                patch("runhaven.cli.subprocess.call") as call,
                redirect_stderr(error_output),
                self.assertRaises(SystemExit) as error,
            ):
                main(["runs", "attach", "run-active"])

        self.assertEqual(error.exception.code, 2)
        require_container.assert_not_called()
        call.assert_not_called()
        self.assertIn("not a RunHaven-owned container", error_output.getvalue())

    def test_runs_attach_refuses_root_user_without_override(self) -> None:
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
                patch("runhaven.cli.require_container_cli") as require_container,
                patch("runhaven.cli.subprocess.call") as call,
                redirect_stderr(error_output),
                self.assertRaises(SystemExit) as error,
            ):
                main(["runs", "attach", "run-active", "--user", "root"])

        self.assertEqual(error.exception.code, 2)
        require_container.assert_not_called()
        call.assert_not_called()
        self.assertIn("root user or group requires --allow-root-user", error_output.getvalue())

    def test_runs_attach_allows_root_user_with_override(self) -> None:
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
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.subprocess.call", return_value=0) as call,
                patch("runhaven.cli.sys.stdin.isatty", return_value=True),
                patch("runhaven.cli.sys.stdout.isatty", return_value=True),
            ):
                code = main(
                    [
                        "runs",
                        "attach",
                        "run-active",
                        "--user",
                        "root",
                        "--allow-root-user",
                    ]
                )

        self.assertEqual(code, 0)
        command = call.call_args.args[0]
        self.assertIn("--user", command)
        self.assertEqual(command[command.index("--user") + 1], "root")

    def test_runs_logs_follow_streams_recent_active_container_logs(self) -> None:
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
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.subprocess.call", return_value=0) as call,
            ):
                code = main(["runs", "logs-follow", "run-active"])

        self.assertEqual(code, 0)
        call.assert_called_once_with(
            (
                "container",
                "logs",
                "--follow",
                "-n",
                "200",
                "runhaven-shell-abc-run",
            )
        )

    def test_runs_logs_follow_accepts_line_count_override(self) -> None:
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
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.subprocess.call", return_value=0) as call,
            ):
                code = main(["runs", "logs-follow", "run-active", "--lines", "25"])

        self.assertEqual(code, 0)
        self.assertEqual(call.call_args.args[0][call.call_args.args[0].index("-n") + 1], "25")

    def test_runs_logs_follow_refuses_invalid_line_count(self) -> None:
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
                patch("runhaven.cli.require_container_cli") as require_container,
                patch("runhaven.cli.subprocess.call") as call,
                redirect_stderr(error_output),
                self.assertRaises(SystemExit) as error,
            ):
                main(["runs", "logs-follow", "run-active", "--lines", "0"])

        self.assertEqual(error.exception.code, 2)
        require_container.assert_not_called()
        call.assert_not_called()
        self.assertIn("--lines must be 1 or greater", error_output.getvalue())

    def test_runs_logs_follow_refuses_unowned_container_name(self) -> None:
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
                patch("runhaven.cli.subprocess.call") as call,
                redirect_stderr(error_output),
                self.assertRaises(SystemExit) as error,
            ):
                main(["runs", "logs-follow", "run-active"])

        self.assertEqual(error.exception.code, 2)
        require_container.assert_not_called()
        call.assert_not_called()
        self.assertIn("not a RunHaven-owned container", error_output.getvalue())


if __name__ == "__main__":
    unittest.main()
