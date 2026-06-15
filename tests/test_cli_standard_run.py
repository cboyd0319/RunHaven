from __future__ import annotations

import json
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

from cli_test_helpers import init_git_repo

from runhaven.cli import main


class CliStandardRunTests(unittest.TestCase):
    def test_standard_run_writes_secret_free_run_record(self) -> None:
        with TemporaryDirectory() as directory:
            with (
                patch.dict(
                    "os.environ",
                    {
                        "RUNHAVEN_CACHE_HOME": directory,
                        "OPENAI_API_KEY": "fake-openai-api-key-value",
                    },
                    clear=True,
                ),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.run_preflight"),
                patch("runhaven.cli.subprocess.call", return_value=7),
            ):
                code = main(
                    [
                        "run",
                        "shell",
                        "--workspace",
                        directory,
                        "--tty",
                        "never",
                        "--",
                        "/bin/true",
                    ]
                )

            self.assertEqual(code, 7)
            records = [
                json.loads(line)
                for line in (Path(directory) / "runs.jsonl").read_text().splitlines()
            ]

        self.assertEqual(len(records), 1)
        record = records[0]
        self.assertEqual(record["profile"], "shell")
        self.assertEqual(record["workspace"], str(Path(directory).resolve()))
        self.assertEqual(record["workspace_scope"], "current")
        self.assertEqual(record["network"], "internet")
        self.assertEqual(record["return_code"], 7)
        self.assertEqual(record["status"], "failed")
        self.assertEqual(record["provider_policy"]["entries"], 0)
        self.assertIsNone(record["auth_broker"]["broker"])
        self.assertEqual(record["cleanup"]["provider_network"], "not-applicable")
        self.assertFalse(record["git"]["available"])
        self.assertEqual(record["git"]["reason"], "not-a-git-worktree")
        self.assertNotIn("fake-openai-api-key-value", json.dumps(records))
        self.assertNotIn("OPENAI_API_KEY", json.dumps(records))
        self.assertNotIn("/bin/true", json.dumps(records))

    def test_standard_run_writes_and_removes_active_run_marker(self) -> None:
        with TemporaryDirectory() as directory:
            cache = Path(directory) / "cache"
            workspace = Path(directory) / "workspace"
            workspace.mkdir()
            active_payloads: list[dict[str, object]] = []

            def fake_container_run(command: tuple[str, ...]) -> int:
                active_files = list((cache / "active-runs").glob("*.json"))
                self.assertEqual(len(active_files), 1)
                payload = json.loads(active_files[0].read_text(encoding="utf-8"))
                active_payloads.append(payload)
                self.assertEqual(payload["profile"], "shell")
                self.assertEqual(payload["workspace"], str(workspace.resolve()))
                self.assertEqual(payload["workspace_scope"], "current")
                self.assertEqual(payload["network"], "internet")
                self.assertEqual(payload["status"], "running")
                self.assertEqual(payload["container_name"], command[command.index("--name") + 1])
                self.assertTrue(str(payload["container_name"]).startswith("runhaven-shell-"))
                serialized = json.dumps(payload)
                self.assertNotIn("/bin/true", serialized)
                self.assertNotIn("OPENAI_API_KEY", serialized)
                self.assertNotIn("fake-openai-api-key-value", serialized)
                return 0

            with (
                patch.dict(
                    "os.environ",
                    {
                        "RUNHAVEN_CACHE_HOME": str(cache),
                        "OPENAI_API_KEY": "fake-openai-api-key-value",
                    },
                    clear=True,
                ),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.run_preflight"),
                patch("runhaven.cli.subprocess.call", side_effect=fake_container_run),
            ):
                code = main(
                    [
                        "run",
                        "shell",
                        "--workspace",
                        str(workspace),
                        "--tty",
                        "never",
                        "--",
                        "/bin/true",
                    ]
                )

            self.assertEqual(code, 0)
            self.assertEqual(len(active_payloads), 1)
            self.assertEqual(list((cache / "active-runs").glob("*.json")), [])

    def test_standard_run_records_stopped_status_when_stop_requested(self) -> None:
        with TemporaryDirectory() as directory:
            cache = Path(directory) / "cache"

            def fake_container_run(command: tuple[str, ...]) -> int:
                active_files = list((cache / "active-runs").glob("*.json"))
                self.assertEqual(len(active_files), 1)
                payload = json.loads(active_files[0].read_text(encoding="utf-8"))
                payload["status"] = "stop-requested"
                payload["stop_requested_at"] = "2026-06-15T00:00:01Z"
                active_files[0].write_text(json.dumps(payload) + "\n", encoding="utf-8")
                return 143

            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.run_preflight"),
                patch("runhaven.cli.subprocess.call", side_effect=fake_container_run),
            ):
                code = main(
                    [
                        "run",
                        "shell",
                        "--workspace",
                        directory,
                        "--tty",
                        "never",
                        "--",
                        "/bin/true",
                    ]
                )

            records = [
                json.loads(line)
                for line in (cache / "runs.jsonl").read_text(encoding="utf-8").splitlines()
            ]

        self.assertEqual(code, 143)
        self.assertEqual(records[0]["status"], "stopped")
        self.assertEqual(records[0]["return_code"], 143)
        self.assertEqual(list((cache / "active-runs").glob("*.json")), [])

    def test_standard_run_records_killed_status_when_kill_requested(self) -> None:
        with TemporaryDirectory() as directory:
            cache = Path(directory) / "cache"

            def fake_container_run(command: tuple[str, ...]) -> int:
                active_files = list((cache / "active-runs").glob("*.json"))
                self.assertEqual(len(active_files), 1)
                payload = json.loads(active_files[0].read_text(encoding="utf-8"))
                payload["status"] = "kill-requested"
                payload["kill_requested_at"] = "2026-06-15T00:00:01Z"
                active_files[0].write_text(json.dumps(payload) + "\n", encoding="utf-8")
                return 137

            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.run_preflight"),
                patch("runhaven.cli.subprocess.call", side_effect=fake_container_run),
            ):
                code = main(
                    [
                        "run",
                        "shell",
                        "--workspace",
                        directory,
                        "--tty",
                        "never",
                        "--",
                        "/bin/true",
                    ]
                )

            records = [
                json.loads(line)
                for line in (cache / "runs.jsonl").read_text(encoding="utf-8").splitlines()
            ]

        self.assertEqual(code, 137)
        self.assertEqual(records[0]["status"], "killed")
        self.assertEqual(records[0]["return_code"], 137)
        self.assertEqual(list((cache / "active-runs").glob("*.json")), [])

    def test_standard_run_records_git_change_metadata_without_file_contents(self) -> None:
        with TemporaryDirectory() as directory:
            workspace = Path(directory) / "workspace"
            cache = Path(directory) / "cache"
            workspace.mkdir()
            head = init_git_repo(workspace)

            def fake_container_run(command: tuple[str, ...]) -> int:
                self.assertIn("/bin/true", command)
                (workspace / "tracked.txt").write_text(
                    "SECRET_FROM_FILE\n",
                    encoding="utf-8",
                )
                (workspace / "created.txt").write_text(
                    "CREATED_SECRET_FROM_FILE\n",
                    encoding="utf-8",
                )
                return 0

            with (
                patch.dict(
                    "os.environ",
                    {
                        "RUNHAVEN_CACHE_HOME": str(cache),
                        "OPENAI_API_KEY": "fake-openai-api-key-value",
                    },
                    clear=True,
                ),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.run_preflight"),
                patch("runhaven.cli.subprocess.call", side_effect=fake_container_run),
            ):
                code = main(
                    [
                        "run",
                        "shell",
                        "--workspace",
                        str(workspace),
                        "--tty",
                        "never",
                        "--",
                        "/bin/true",
                    ]
                )

            self.assertEqual(code, 0)
            records = [json.loads(line) for line in (cache / "runs.jsonl").read_text().splitlines()]

        self.assertEqual(len(records), 1)
        record = records[0]
        git = record["git"]
        self.assertTrue(git["available"])
        self.assertEqual(git["repo_root"], str(workspace.resolve()))
        self.assertTrue(git["changed"])
        self.assertEqual(git["before"]["head"], head)
        self.assertFalse(git["before"]["dirty"])
        self.assertEqual(git["before"]["changed_count"], 0)
        self.assertEqual(git["before"]["paths"], [])
        self.assertEqual(git["after"]["head"], head)
        self.assertTrue(git["after"]["dirty"])
        self.assertEqual(git["after"]["changed_count"], 2)
        self.assertCountEqual(git["after"]["paths"], ["created.txt", "tracked.txt"])
        self.assertFalse(git["after"]["truncated"])
        serialized = json.dumps(records)
        self.assertNotIn("SECRET_FROM_FILE", serialized)
        self.assertNotIn("CREATED_SECRET_FROM_FILE", serialized)
        self.assertNotIn("fake-openai-api-key-value", serialized)
        self.assertNotIn("OPENAI_API_KEY", serialized)
        self.assertNotIn("/bin/true", serialized)

    def test_run_executes_preflight_and_container_command(self) -> None:
        with TemporaryDirectory() as directory:
            workspace = Path(directory) / "workspace"
            cache = Path(directory) / "cache"
            workspace.mkdir()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.run_preflight") as preflight,
                patch("runhaven.cli.subprocess.call", return_value=7) as call,
            ):
                code = main(
                    [
                        "run",
                        "shell",
                        "--workspace",
                        str(workspace),
                        "--tty",
                        "never",
                        "--",
                        "/bin/true",
                    ]
                )

        self.assertEqual(code, 7)
        self.assertEqual(preflight.call_count, 2)
        call.assert_called_once()
        self.assertEqual(call.call_args.args[0][-1], "/bin/true")


if __name__ == "__main__":
    unittest.main()
