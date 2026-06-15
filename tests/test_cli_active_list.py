from __future__ import annotations

import io
import json
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

from runhaven.cli import main


class CliActiveListTests(unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main()
