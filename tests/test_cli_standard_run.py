from __future__ import annotations

import io
import json
import unittest
from contextlib import redirect_stderr, redirect_stdout
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

    def test_worktree_dry_run_previews_without_creating_worktree(self) -> None:
        with TemporaryDirectory() as directory:
            repo = Path(directory) / "repo"
            cache = Path(directory) / "cache"
            repo.mkdir()
            head = init_git_repo(repo)

            output = io.StringIO()
            with (
                redirect_stdout(output),
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False),
            ):
                code = main(
                    [
                        "run",
                        "shell",
                        "--workspace",
                        str(repo),
                        "--worktree",
                        "--dry-run",
                    ]
                )

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("Worktree: enabled", text)
        self.assertIn(f"Source workspace: {repo.resolve()}", text)
        self.assertIn(f"Base HEAD: {head}", text)
        self.assertFalse((cache / "worktrees").exists())

    def test_worktree_run_mounts_isolated_worktree_and_records_recovery(self) -> None:
        with TemporaryDirectory() as directory:
            repo = Path(directory) / "repo"
            cache = Path(directory) / "cache"
            repo.mkdir()
            head = init_git_repo(repo)
            mounted_workspaces: list[Path] = []

            def fake_container_run(command: tuple[str, ...]) -> int:
                workspace_mount = next(
                    value for value in command if value.startswith("type=bind,")
                )
                mount_parts = dict(
                    part.split("=", maxsplit=1)
                    for part in workspace_mount.split(",")
                    if "=" in part
                )
                mounted_workspace = Path(mount_parts["source"])
                mounted_workspaces.append(mounted_workspace)
                self.assertNotEqual(mounted_workspace, repo.resolve())
                self.assertTrue(
                    str(mounted_workspace).startswith(str((cache / "worktrees").resolve()))
                )
                (mounted_workspace / "tracked.txt").write_text(
                    "changed in isolated worktree\n",
                    encoding="utf-8",
                )
                return 0

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
                        str(repo),
                        "--worktree",
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
            source_tracked = repo.joinpath("tracked.txt").read_text(encoding="utf-8")

        self.assertEqual(code, 0)
        self.assertEqual(source_tracked, "initial\n")
        self.assertEqual(len(records), 1)
        record = records[0]
        self.assertEqual(record["workspace"], str(mounted_workspaces[0]))
        self.assertEqual(record["workspace_scope"], "current")
        self.assertEqual(record["git"]["repo_root"], str(mounted_workspaces[0]))
        self.assertTrue(record["git"]["changed"])
        self.assertEqual(record["git"]["before"]["head"], head)
        self.assertEqual(record["git"]["after"]["head"], head)
        worktree = record["worktree"]
        self.assertEqual(worktree["source_workspace"], str(repo.resolve()))
        self.assertEqual(worktree["source_repo_root"], str(repo.resolve()))
        self.assertEqual(worktree["mounted_workspace"], str(mounted_workspaces[0]))
        self.assertTrue(worktree["branch"].startswith("runhaven/shell/"))
        self.assertEqual(worktree["base_head"], head)
        self.assertIn("merge", worktree["recovery_commands"])
        self.assertIn("remove_worktree", worktree["recovery_commands"])
        self.assertNotIn("/bin/true", json.dumps(records))

    def test_worktree_run_refuses_dirty_source_before_creating_worktree(self) -> None:
        with TemporaryDirectory() as directory:
            repo = Path(directory) / "repo"
            cache = Path(directory) / "cache"
            repo.mkdir()
            init_git_repo(repo)
            (repo / "dirty.txt").write_text("dirty\n", encoding="utf-8")
            error_output = io.StringIO()

            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                redirect_stderr(error_output),
                self.assertRaises(SystemExit) as error,
            ):
                main(
                    [
                        "run",
                        "shell",
                        "--workspace",
                        str(repo),
                        "--worktree",
                        "--tty",
                        "never",
                        "--",
                        "/bin/true",
                    ]
                )

        self.assertEqual(error.exception.code, 2)
        text = error_output.getvalue()
        self.assertIn("--worktree requires a clean source git worktree", text)
        self.assertIn("Options:", text)
        self.assertIn("Commit or stash source changes, then retry with --worktree", text)
        self.assertIn("Run without --worktree to use the source checkout directly", text)
        self.assertIn("Start from a clean clone or git worktree if you want isolation", text)
        self.assertFalse((cache / "worktrees").exists())


if __name__ == "__main__":
    unittest.main()
