from __future__ import annotations

import io
import json
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

from cli_test_helpers import init_git_repo, run_git, write_run_record_for_git_diff

from runhaven.cli import main


class CliRunsDiffTests(unittest.TestCase):
    def test_runs_diff_prints_live_committed_git_diff(self) -> None:
        with TemporaryDirectory() as directory:
            repo = Path(directory) / "repo"
            cache = Path(directory) / "cache"
            repo.mkdir()
            before_head = init_git_repo(repo)
            (repo / "tracked.txt").write_text("changed\n", encoding="utf-8")
            run_git(repo, "add", "tracked.txt")
            run_git(repo, "commit", "-m", "change tracked")
            after_head = run_git(repo, "rev-parse", "HEAD")
            write_run_record_for_git_diff(
                cache,
                repo=repo,
                run_id="run-diff",
                before_head=before_head,
                after_head=after_head,
                after_dirty=False,
                after_paths=[],
            )
            output = io.StringIO()
            with patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False):
                with redirect_stdout(output):
                    code = main(["runs", "diff", "run-diff"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("diff --git a/tracked.txt b/tracked.txt", text)
        self.assertIn("-initial", text)
        self.assertIn("+changed", text)

    def test_runs_diff_prints_live_dirty_git_diff_with_warning(self) -> None:
        with TemporaryDirectory() as directory:
            repo = Path(directory) / "repo"
            cache = Path(directory) / "cache"
            repo.mkdir()
            head = init_git_repo(repo)
            (repo / "tracked.txt").write_text("dirty change\n", encoding="utf-8")
            write_run_record_for_git_diff(
                cache,
                repo=repo,
                run_id="run-dirty",
                before_head=head,
                after_head=head,
                after_dirty=True,
                after_paths=["tracked.txt"],
            )
            output = io.StringIO()
            error_output = io.StringIO()
            with patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False):
                with redirect_stdout(output), redirect_stderr(error_output):
                    code = main(["runs", "diff", "run-dirty"])

        self.assertEqual(code, 0)
        self.assertIn("+dirty change", output.getvalue())
        self.assertIn("live working tree diff", error_output.getvalue())

    def test_runs_diff_prints_live_untracked_git_diff(self) -> None:
        with TemporaryDirectory() as directory:
            repo = Path(directory) / "repo"
            cache = Path(directory) / "cache"
            repo.mkdir()
            head = init_git_repo(repo)
            (repo / "new.txt").write_text("new file\n", encoding="utf-8")
            write_run_record_for_git_diff(
                cache,
                repo=repo,
                run_id="run-untracked",
                before_head=head,
                after_head=head,
                after_dirty=True,
                after_paths=["new.txt"],
            )
            output = io.StringIO()
            error_output = io.StringIO()
            with patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False):
                with redirect_stdout(output), redirect_stderr(error_output):
                    code = main(["runs", "diff", "run-untracked"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("--- /dev/null", text)
        self.assertIn("+new file", text)
        self.assertIn("live working tree diff", error_output.getvalue())

    def test_runs_diff_includes_committed_and_dirty_changes(self) -> None:
        with TemporaryDirectory() as directory:
            repo = Path(directory) / "repo"
            cache = Path(directory) / "cache"
            repo.mkdir()
            before_head = init_git_repo(repo)
            (repo / "committed.txt").write_text("committed file\n", encoding="utf-8")
            run_git(repo, "add", "committed.txt")
            run_git(repo, "commit", "-m", "add committed file")
            after_head = run_git(repo, "rev-parse", "HEAD")
            (repo / "tracked.txt").write_text("dirty after commit\n", encoding="utf-8")
            write_run_record_for_git_diff(
                cache,
                repo=repo,
                run_id="run-commit-and-dirty",
                before_head=before_head,
                after_head=after_head,
                after_dirty=True,
                after_paths=["tracked.txt"],
            )
            output = io.StringIO()
            error_output = io.StringIO()
            with patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False):
                with redirect_stdout(output), redirect_stderr(error_output):
                    code = main(["runs", "diff", "run-commit-and-dirty"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("diff --git a/committed.txt b/committed.txt", text)
        self.assertIn("+committed file", text)
        self.assertIn("+dirty after commit", text)
        self.assertIn("live working tree diff", error_output.getvalue())

    def test_runs_diff_refuses_unavailable_git_metadata(self) -> None:
        with TemporaryDirectory() as directory:
            Path(directory, "runs.jsonl").write_text(
                json.dumps(
                    {
                        "timestamp": "2026-06-15T00:00:02Z",
                        "started_at": "2026-06-15T00:00:02Z",
                        "finished_at": "2026-06-15T00:00:03Z",
                        "run_id": "run-no-git",
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
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                redirect_stderr(error_output),
                self.assertRaises(SystemExit) as error,
            ):
                main(["runs", "diff", "run-no-git"])

        self.assertEqual(error.exception.code, 2)
        self.assertIn("git metadata is unavailable", error_output.getvalue())

    def test_runs_diff_refuses_when_recorded_head_is_stale(self) -> None:
        with TemporaryDirectory() as directory:
            repo = Path(directory) / "repo"
            cache = Path(directory) / "cache"
            repo.mkdir()
            before_head = init_git_repo(repo)
            write_run_record_for_git_diff(
                cache,
                repo=repo,
                run_id="run-stale",
                before_head=before_head,
                after_head=before_head,
                after_dirty=False,
                after_paths=[],
            )
            (repo / "tracked.txt").write_text("new commit\n", encoding="utf-8")
            run_git(repo, "add", "tracked.txt")
            run_git(repo, "commit", "-m", "new commit")
            error_output = io.StringIO()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False),
                redirect_stderr(error_output),
                self.assertRaises(SystemExit) as error,
            ):
                main(["runs", "diff", "run-stale"])

        self.assertEqual(error.exception.code, 2)
        self.assertIn("git HEAD changed since the recorded run", error_output.getvalue())

    def test_runs_diff_refuses_when_dirty_path_set_changed(self) -> None:
        with TemporaryDirectory() as directory:
            repo = Path(directory) / "repo"
            cache = Path(directory) / "cache"
            repo.mkdir()
            head = init_git_repo(repo)
            (repo / "tracked.txt").write_text("dirty change\n", encoding="utf-8")
            write_run_record_for_git_diff(
                cache,
                repo=repo,
                run_id="run-stale-paths",
                before_head=head,
                after_head=head,
                after_dirty=True,
                after_paths=["tracked.txt"],
            )
            (repo / "extra.txt").write_text("extra\n", encoding="utf-8")
            error_output = io.StringIO()
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False),
                redirect_stderr(error_output),
                self.assertRaises(SystemExit) as error,
            ):
                main(["runs", "diff", "run-stale-paths"])

        self.assertEqual(error.exception.code, 2)
        self.assertIn("git working tree changed since the recorded run", error_output.getvalue())


if __name__ == "__main__":
    unittest.main()
