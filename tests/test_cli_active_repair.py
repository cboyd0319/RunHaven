from __future__ import annotations

import io
import json
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import Mock, patch

from cli_test_helpers import write_active_marker

from runhaven.cli import main


class CliActiveRepairTests(unittest.TestCase):
    def test_runs_repair_removes_marker_when_container_is_missing(self) -> None:
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
                run.return_value = Mock(
                    returncode=1,
                    stdout="",
                    stderr="Error: container not found: runhaven-shell-abc-run\n",
                )
                code = main(["runs", "repair", "run-active"])

            self.assertEqual(code, 0)
            run.assert_called_once_with(
                ("container", "inspect", "runhaven-shell-abc-run"),
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertFalse(active_path.exists())
            self.assertIn("Removed stale active marker", output.getvalue())

    def test_runs_repair_json_reports_removed_marker(self) -> None:
        with TemporaryDirectory() as directory:
            cache = Path(directory)
            active_path = write_active_marker(
                cache,
                run_id="run-active",
                timestamp="2026-06-15T00:00:00Z",
                container_name="runhaven-shell-abc-run",
            )
            output = io.StringIO()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.subprocess.run") as run,
                redirect_stdout(output),
            ):
                run.return_value = Mock(
                    returncode=1,
                    stdout="",
                    stderr="Error: container not found: runhaven-shell-abc-run\n",
                )
                code = main(["runs", "repair", "run-active", "--json"])

            self.assertEqual(code, 0)
            self.assertFalse(active_path.exists())
            payload = json.loads(output.getvalue())
            self.assertEqual(payload["mode"], "single")
            self.assertEqual(payload["summary"], {"kept": 0, "removed": 1, "unverified": 0})
            self.assertEqual(payload["exit_code"], 0)
            self.assertEqual(
                payload["results"],
                [
                    {
                        "container_name": "runhaven-shell-abc-run",
                        "inspect_return_code": 1,
                        "marker_removed": True,
                        "run_id": "run-active",
                        "status": "removed",
                    }
                ],
            )

    def test_runs_repair_refuses_when_container_still_exists(self) -> None:
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
            error_output = io.StringIO()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.subprocess.run") as run,
                redirect_stderr(error_output),
            ):
                run.return_value = Mock(returncode=0, stdout="[]", stderr="")
                code = main(["runs", "repair", "run-active"])

            self.assertEqual(code, 1)
            self.assertTrue(active_path.exists())
            self.assertIn("container still exists", error_output.getvalue())

    def test_runs_repair_leaves_marker_on_unverified_inspect_failure(self) -> None:
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
            error_output = io.StringIO()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.subprocess.run") as run,
                redirect_stderr(error_output),
            ):
                run.return_value = Mock(returncode=7, stdout="", stderr="daemon unavailable\n")
                code = main(["runs", "repair", "run-active"])

            self.assertEqual(code, 7)
            self.assertTrue(active_path.exists())
            self.assertIn("could not confirm", error_output.getvalue())

    def test_runs_repair_all_removes_confirmed_stale_markers(self) -> None:
        with TemporaryDirectory() as directory:
            cache = Path(directory)
            stale_path = write_active_marker(
                cache,
                run_id="run-stale",
                timestamp="2026-06-15T00:00:01Z",
                container_name="runhaven-shell-stale-run",
            )
            live_path = write_active_marker(
                cache,
                run_id="run-live",
                timestamp="2026-06-15T00:00:02Z",
                container_name="runhaven-shell-live-run",
            )

            def fake_inspect(command: tuple[str, ...], **kwargs: object) -> Mock:
                self.assertEqual(
                    kwargs,
                    {"check": False, "capture_output": True, "text": True},
                )
                container_name = command[-1]
                if container_name == "runhaven-shell-stale-run":
                    return Mock(
                        returncode=1,
                        stdout="",
                        stderr="Error: container not found: runhaven-shell-stale-run\n",
                    )
                if container_name == "runhaven-shell-live-run":
                    return Mock(returncode=0, stdout="[]", stderr="")
                raise AssertionError(f"unexpected inspect target: {container_name}")

            output = io.StringIO()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.subprocess.run", side_effect=fake_inspect) as run,
                redirect_stdout(output),
            ):
                code = main(["runs", "repair", "--all"])

            self.assertEqual(code, 0)
            self.assertEqual(
                [call.args[0] for call in run.call_args_list],
                [
                    ("container", "inspect", "runhaven-shell-stale-run"),
                    ("container", "inspect", "runhaven-shell-live-run"),
                ],
            )
            self.assertFalse(stale_path.exists())
            self.assertTrue(live_path.exists())
            text = output.getvalue()
            self.assertIn("Removed stale active marker for run run-stale", text)
            self.assertIn("Kept active marker for run run-live", text)
            self.assertIn("Repair summary: removed=1 kept=1 unverified=0", text)

    def test_runs_repair_all_json_reports_mixed_outcomes(self) -> None:
        with TemporaryDirectory() as directory:
            cache = Path(directory)
            stale_path = write_active_marker(
                cache,
                run_id="run-stale",
                timestamp="2026-06-15T00:00:01Z",
                container_name="runhaven-shell-stale-run",
            )
            live_path = write_active_marker(
                cache,
                run_id="run-live",
                timestamp="2026-06-15T00:00:02Z",
                container_name="runhaven-shell-live-run",
            )
            unknown_path = write_active_marker(
                cache,
                run_id="run-unknown",
                timestamp="2026-06-15T00:00:03Z",
                container_name="runhaven-shell-unknown-run",
            )

            def fake_inspect(command: tuple[str, ...], **kwargs: object) -> Mock:
                self.assertEqual(
                    kwargs,
                    {"check": False, "capture_output": True, "text": True},
                )
                container_name = command[-1]
                if container_name == "runhaven-shell-stale-run":
                    return Mock(
                        returncode=1,
                        stdout="",
                        stderr="Error: container not found: runhaven-shell-stale-run\n",
                    )
                if container_name == "runhaven-shell-live-run":
                    return Mock(returncode=0, stdout="[]", stderr="")
                if container_name == "runhaven-shell-unknown-run":
                    return Mock(returncode=7, stdout="", stderr="daemon unavailable\n")
                raise AssertionError(f"unexpected inspect target: {container_name}")

            output = io.StringIO()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.subprocess.run", side_effect=fake_inspect),
                redirect_stdout(output),
            ):
                code = main(["runs", "repair", "--all", "--json"])

            self.assertEqual(code, 1)
            self.assertFalse(stale_path.exists())
            self.assertTrue(live_path.exists())
            self.assertTrue(unknown_path.exists())
            payload = json.loads(output.getvalue())
            self.assertEqual(payload["mode"], "all")
            self.assertEqual(payload["summary"], {"kept": 1, "removed": 1, "unverified": 1})
            self.assertEqual(payload["exit_code"], 1)
            self.assertEqual(
                payload["results"],
                [
                    {
                        "container_name": "runhaven-shell-stale-run",
                        "inspect_return_code": 1,
                        "marker_removed": True,
                        "run_id": "run-stale",
                        "status": "removed",
                    },
                    {
                        "container_name": "runhaven-shell-live-run",
                        "inspect_return_code": 0,
                        "marker_removed": False,
                        "run_id": "run-live",
                        "status": "kept",
                    },
                    {
                        "container_name": "runhaven-shell-unknown-run",
                        "inspect_return_code": 7,
                        "marker_removed": False,
                        "run_id": "run-unknown",
                        "status": "unverified",
                    },
                ],
            )

    def test_runs_repair_all_json_reports_empty_summary(self) -> None:
        with TemporaryDirectory() as directory:
            output = io.StringIO()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli") as require_container,
                redirect_stdout(output),
            ):
                code = main(["runs", "repair", "--all", "--json"])

        self.assertEqual(code, 0)
        require_container.assert_not_called()
        self.assertEqual(
            json.loads(output.getvalue()),
            {
                "exit_code": 0,
                "mode": "all",
                "results": [],
                "summary": {"kept": 0, "removed": 0, "unverified": 0},
            },
        )

    def test_runs_repair_all_returns_nonzero_when_any_marker_unverified(self) -> None:
        with TemporaryDirectory() as directory:
            cache = Path(directory)
            stale_path = write_active_marker(
                cache,
                run_id="run-stale",
                timestamp="2026-06-15T00:00:01Z",
                container_name="runhaven-shell-stale-run",
            )
            unknown_path = write_active_marker(
                cache,
                run_id="run-unknown",
                timestamp="2026-06-15T00:00:02Z",
                container_name="runhaven-shell-unknown-run",
            )

            def fake_inspect(command: tuple[str, ...], **kwargs: object) -> Mock:
                self.assertEqual(
                    kwargs,
                    {"check": False, "capture_output": True, "text": True},
                )
                container_name = command[-1]
                if container_name == "runhaven-shell-stale-run":
                    return Mock(
                        returncode=1,
                        stdout="",
                        stderr="Error: container not found: runhaven-shell-stale-run\n",
                    )
                if container_name == "runhaven-shell-unknown-run":
                    return Mock(returncode=7, stdout="", stderr="daemon unavailable\n")
                raise AssertionError(f"unexpected inspect target: {container_name}")

            output = io.StringIO()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.subprocess.run", side_effect=fake_inspect),
                redirect_stdout(output),
            ):
                code = main(["runs", "repair", "--all"])

            self.assertEqual(code, 1)
            self.assertFalse(stale_path.exists())
            self.assertTrue(unknown_path.exists())
            text = output.getvalue()
            self.assertIn("Removed stale active marker for run run-stale", text)
            self.assertIn("Could not verify active marker for run run-unknown", text)
            self.assertIn("Repair summary: removed=1 kept=0 unverified=1", text)

    def test_runs_repair_requires_run_id_or_all(self) -> None:
        with TemporaryDirectory() as directory:
            error_output = io.StringIO()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli") as require_container,
                redirect_stderr(error_output),
                self.assertRaises(SystemExit) as error,
            ):
                main(["runs", "repair"])

        self.assertEqual(error.exception.code, 2)
        require_container.assert_not_called()
        self.assertIn("repair requires RUN_ID or --all", error_output.getvalue())

    def test_runs_repair_refuses_run_id_with_all(self) -> None:
        with TemporaryDirectory() as directory:
            error_output = io.StringIO()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli") as require_container,
                redirect_stderr(error_output),
                self.assertRaises(SystemExit) as error,
            ):
                main(["runs", "repair", "run-active", "--all"])

        self.assertEqual(error.exception.code, 2)
        require_container.assert_not_called()
        self.assertIn("--all cannot be used with RUN_ID", error_output.getvalue())

    def test_runs_repair_refuses_unowned_container_name(self) -> None:
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
                main(["runs", "repair", "run-active"])

        self.assertEqual(error.exception.code, 2)
        require_container.assert_not_called()
        run.assert_not_called()
        self.assertIn("not a RunHaven-owned container", error_output.getvalue())


if __name__ == "__main__":
    unittest.main()
