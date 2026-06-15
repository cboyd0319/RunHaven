from __future__ import annotations

import io
import json
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

from cli_test_helpers import init_git_repo, run_git

from runhaven.cli import main


def branch_exists(repo: Path, branch: str) -> bool:
    result = run_git(repo, "branch", "--list", branch)
    return bool(result.strip())


class CliWorktreeLifecycleTests(unittest.TestCase):
    def test_runs_keep_prints_worktree_review_commands_without_cleanup(self) -> None:
        with TemporaryDirectory() as directory:
            repo, cache, record = self.create_dirty_worktree_run(Path(directory))
            worktree_root = Path(record["worktree"]["worktree_root"])
            branch = record["worktree"]["branch"]
            output = io.StringIO()

            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False),
                redirect_stdout(output),
            ):
                code = main(["runs", "keep", record["run_id"]])

            self.assertEqual(code, 0)
            text = output.getvalue()
            self.assertIn(f"Worktree kept for run {record['run_id']}", text)
            self.assertIn(str(worktree_root), text)
            self.assertIn(branch, text)
            self.assertIn(f"runhaven runs merge {record['run_id']}", text)
            self.assertIn(f"runhaven runs discard {record['run_id']}", text)
            self.assertTrue(worktree_root.exists())
            self.assertTrue(branch_exists(repo, branch))

    def test_runs_merge_applies_dirty_worktree_changes_and_cleans_up(self) -> None:
        with TemporaryDirectory() as directory:
            repo, cache, record = self.create_dirty_worktree_run(Path(directory))
            worktree_root = Path(record["worktree"]["worktree_root"])
            branch = record["worktree"]["branch"]
            output = io.StringIO()

            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False),
                redirect_stdout(output),
            ):
                code = main(["runs", "merge", record["run_id"]])

            status = run_git(repo, "status", "--short")

            self.assertEqual(code, 0)
            self.assertIn(f"Merged worktree run {record['run_id']}", output.getvalue())
            self.assertEqual((repo / "tracked.txt").read_text(encoding="utf-8"), "dirty change\n")
            self.assertEqual((repo / "created.txt").read_text(encoding="utf-8"), "created\n")
            self.assertIn("tracked.txt", status)
            self.assertIn("?? created.txt", status)
            self.assertFalse(worktree_root.exists())
            self.assertFalse(branch_exists(repo, branch))

    def test_runs_merge_fast_forwards_committed_worktree_changes(self) -> None:
        with TemporaryDirectory() as directory:
            repo, cache, record = self.create_committed_worktree_run(Path(directory))
            worktree_root = Path(record["worktree"]["worktree_root"])
            branch = record["worktree"]["branch"]
            worktree_head = record["git"]["after"]["head"]
            output = io.StringIO()

            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False),
                redirect_stdout(output),
            ):
                code = main(["runs", "merge", record["run_id"]])

            status = run_git(repo, "status", "--short")
            source_head = run_git(repo, "rev-parse", "HEAD")

            self.assertEqual(code, 0)
            self.assertIn(f"Merged worktree run {record['run_id']}", output.getvalue())
            self.assertEqual(source_head, worktree_head)
            self.assertEqual(status, "")
            self.assertEqual((repo / "tracked.txt").read_text(encoding="utf-8"), "committed\n")
            self.assertFalse(worktree_root.exists())
            self.assertFalse(branch_exists(repo, branch))

    def test_runs_merge_refuses_when_source_head_changed(self) -> None:
        with TemporaryDirectory() as directory:
            repo, cache, record = self.create_dirty_worktree_run(Path(directory))
            worktree_root = Path(record["worktree"]["worktree_root"])
            branch = record["worktree"]["branch"]
            (repo / "source.txt").write_text("source change\n", encoding="utf-8")
            run_git(repo, "add", "source.txt")
            run_git(repo, "commit", "-m", "source moved")
            error_output = io.StringIO()

            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False),
                redirect_stderr(error_output),
                self.assertRaises(SystemExit) as error,
            ):
                main(["runs", "merge", record["run_id"]])

            self.assertEqual(error.exception.code, 2)
            self.assertIn("source repository HEAD changed", error_output.getvalue())
            self.assertTrue(worktree_root.exists())
            self.assertTrue(branch_exists(repo, branch))

    def test_runs_merge_refusal_prints_recovery_commands(self) -> None:
        with TemporaryDirectory() as directory:
            repo, cache, record = self.create_dirty_worktree_run(Path(directory))
            worktree_root = Path(record["worktree"]["worktree_root"])
            branch = record["worktree"]["branch"]
            (repo / "source-local.txt").write_text("local source change\n", encoding="utf-8")
            error_output = io.StringIO()

            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False),
                redirect_stderr(error_output),
                self.assertRaises(SystemExit) as error,
            ):
                main(["runs", "merge", record["run_id"]])

            text = error_output.getvalue()

            self.assertEqual(error.exception.code, 2)
            self.assertIn(f"could not complete merge for run {record['run_id']}", text)
            self.assertIn("source repository has uncommitted changes", text)
            self.assertIn("No cleanup was attempted", text)
            self.assertIn(str(worktree_root), text)
            self.assertIn(branch, text)
            self.assertIn(f"runhaven runs diff {record['run_id']}", text)
            self.assertIn(f"runhaven runs keep {record['run_id']}", text)
            self.assertIn(f"runhaven runs merge {record['run_id']}", text)
            self.assertIn(f"runhaven runs discard {record['run_id']}", text)
            self.assertTrue(worktree_root.exists())
            self.assertTrue(branch_exists(repo, branch))

    def test_runs_discard_removes_worktree_without_touching_source(self) -> None:
        with TemporaryDirectory() as directory:
            repo, cache, record = self.create_dirty_worktree_run(Path(directory))
            worktree_root = Path(record["worktree"]["worktree_root"])
            branch = record["worktree"]["branch"]
            output = io.StringIO()

            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False),
                redirect_stdout(output),
            ):
                code = main(["runs", "discard", record["run_id"]])

            status = run_git(repo, "status", "--short")

            self.assertEqual(code, 0)
            self.assertIn(f"Discarded worktree run {record['run_id']}", output.getvalue())
            self.assertEqual((repo / "tracked.txt").read_text(encoding="utf-8"), "initial\n")
            self.assertFalse((repo / "created.txt").exists())
            self.assertEqual(status, "")
            self.assertFalse(worktree_root.exists())
            self.assertFalse(branch_exists(repo, branch))

    def test_runs_discard_refuses_non_worktree_run_record(self) -> None:
        with TemporaryDirectory() as directory:
            cache = Path(directory)
            (cache / "runs.jsonl").write_text(
                json.dumps(
                    {
                        "timestamp": "2026-06-15T00:00:02Z",
                        "started_at": "2026-06-15T00:00:02Z",
                        "finished_at": "2026-06-15T00:00:03Z",
                        "run_id": "plain-run",
                        "profile": "shell",
                        "workspace": directory,
                        "network": "internet",
                        "status": "succeeded",
                        "return_code": 0,
                        "provider_policy": {"entries": 0, "allowed": 0, "denied": 0},
                        "auth_broker": {
                            "broker": None,
                            "entries": 0,
                            "allowed": 0,
                            "denied": 0,
                            "no_requests": False,
                        },
                        "cleanup": {"provider_network": "not-applicable"},
                        "git": {"available": False, "reason": "not-a-git-worktree"},
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            error_output = io.StringIO()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False),
                redirect_stderr(error_output),
                self.assertRaises(SystemExit) as error,
            ):
                main(["runs", "discard", "plain-run"])

        self.assertEqual(error.exception.code, 2)
        self.assertIn("is not a worktree run", error_output.getvalue())

    def create_dirty_worktree_run(self, directory: Path) -> tuple[Path, Path, dict[str, object]]:
        return self.create_worktree_run(directory, commit_change=False)

    def create_committed_worktree_run(
        self,
        directory: Path,
    ) -> tuple[Path, Path, dict[str, object]]:
        return self.create_worktree_run(directory, commit_change=True)

    def create_worktree_run(
        self,
        directory: Path,
        *,
        commit_change: bool,
    ) -> tuple[Path, Path, dict[str, object]]:
        repo = directory / "repo"
        cache = directory / "cache"
        repo.mkdir()
        init_git_repo(repo)

        def fake_container_run(command: tuple[str, ...]) -> int:
            workspace_mount = next(value for value in command if value.startswith("type=bind,"))
            mount_parts = dict(
                part.split("=", maxsplit=1) for part in workspace_mount.split(",") if "=" in part
            )
            mounted_workspace = Path(mount_parts["source"])
            if commit_change:
                (mounted_workspace / "tracked.txt").write_text("committed\n", encoding="utf-8")
                run_git(mounted_workspace, "add", "tracked.txt")
                run_git(mounted_workspace, "commit", "-m", "agent committed change")
            else:
                (mounted_workspace / "tracked.txt").write_text("dirty change\n", encoding="utf-8")
                (mounted_workspace / "created.txt").write_text("created\n", encoding="utf-8")
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

        self.assertEqual(code, 0)
        records = [json.loads(line) for line in (cache / "runs.jsonl").read_text().splitlines()]
        self.assertEqual(len(records), 1)
        return repo, cache, records[0]


if __name__ == "__main__":
    unittest.main()
