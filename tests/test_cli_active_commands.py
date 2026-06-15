from __future__ import annotations

import io
import json
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import Mock, patch

from runhaven.cli import main


class CliActiveCommandTests(unittest.TestCase):
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

    def test_runs_active_prints_active_run_markers(self) -> None:
        with TemporaryDirectory() as directory:
            workspace = Path(directory) / "workspace"
            workspace.mkdir()
            active_dir = Path(directory) / "active-runs"
            active_dir.mkdir()
            (active_dir / "run-new.json").write_text(
                json.dumps(
                    {
                        "timestamp": "2026-06-15T00:00:02Z",
                        "run_id": "run-new",
                        "profile": "codex",
                        "workspace": str(workspace),
                        "network": "provider",
                        "status": "stop-requested",
                        "container_name": "runhaven-codex-new-run",
                        "host_pid": 23456,
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (active_dir / "run-old.json").write_text(
                json.dumps(
                    {
                        "timestamp": "2026-06-15T00:00:01Z",
                        "run_id": "run-old",
                        "profile": "shell",
                        "workspace": str(workspace),
                        "network": "internet",
                        "status": "running",
                        "container_name": "runhaven-shell-old-run",
                        "host_pid": 12345,
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (active_dir / "invalid.json").write_text("{invalid\n", encoding="utf-8")
            output = io.StringIO()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli") as require_container,
                redirect_stdout(output),
            ):
                code = main(["runs", "active"])

        self.assertEqual(code, 0)
        require_container.assert_not_called()
        text = output.getvalue()
        self.assertLess(text.index("run=run-old"), text.index("run=run-new"))
        self.assertIn("shell  internet  running", text)
        self.assertIn("codex  provider  stop-requested", text)
        self.assertIn(f"workspace={workspace}", text)
        self.assertIn("container=runhaven-shell-old-run", text)
        self.assertNotIn("invalid", text)

    def test_runs_active_json_prints_active_run_markers(self) -> None:
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
                        "container_name": "runhaven-shell-active-run",
                        "state_volume": "runhaven-shell-active-home",
                        "network_name": None,
                        "host_pid": 12345,
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            output = io.StringIO()
            with (
                patch.dict(
                    "os.environ",
                    {
                        "RUNHAVEN_CACHE_HOME": directory,
                        "OPENAI_API_KEY": "fake-openai-api-key-value",
                    },
                    clear=True,
                ),
                patch("runhaven.cli.require_container_cli") as require_container,
                redirect_stdout(output),
            ):
                code = main(["runs", "active", "--json"])

        self.assertEqual(code, 0)
        require_container.assert_not_called()
        records = json.loads(output.getvalue())
        self.assertEqual(len(records), 1)
        self.assertEqual(records[0]["run_id"], "run-active")
        self.assertEqual(records[0]["container_name"], "runhaven-shell-active-run")
        serialized = json.dumps(records)
        self.assertNotIn("fake-openai-api-key-value", serialized)
        self.assertNotIn("OPENAI_API_KEY", serialized)

    def test_runs_active_prints_empty_message(self) -> None:
        with TemporaryDirectory() as directory:
            output = io.StringIO()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                redirect_stdout(output),
            ):
                code = main(["runs", "active"])

        self.assertEqual(code, 0)
        self.assertIn("No active RunHaven runs found.", output.getvalue())

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
                        "mounts": [{"source": "/Users/c/private", "destination": "/workspace"}],
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
        self.assertNotIn("/Users/c/private", text)
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
