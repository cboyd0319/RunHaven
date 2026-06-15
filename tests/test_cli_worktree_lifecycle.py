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

    def test_runs_keep_prints_project_check_suggestions_without_cleanup(self) -> None:
        with TemporaryDirectory() as directory:
            repo, cache, record = self.create_project_check_worktree_run(Path(directory))
            worktree_root = Path(record["worktree"]["worktree_root"])
            mounted_workspace = record["worktree"]["mounted_workspace"]
            branch = record["worktree"]["branch"]
            output = io.StringIO()

            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False),
                redirect_stdout(output),
            ):
                code = main(["runs", "keep", record["run_id"]])

            text = output.getvalue()

            self.assertEqual(code, 0)
            self.assertIn("Suggested checks:", text)
            self.assertIn(
                f"runhaven run shell --workspace {mounted_workspace} "
                "--network internal -- npm test",
                text,
            )
            self.assertIn(
                f"runhaven run shell --workspace {mounted_workspace} "
                "--network internal -- npm run lint",
                text,
            )
            self.assertIn(
                f"runhaven run shell --workspace {mounted_workspace} "
                "--network internal -- python -m unittest discover -s tests",
                text,
            )
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
            self.assertIn(f"runhaven runs recover {record['run_id']}", text)
            self.assertIn(f"runhaven runs keep {record['run_id']}", text)
            self.assertIn(f"runhaven runs merge {record['run_id']}", text)
            self.assertIn(f"runhaven runs discard {record['run_id']}", text)
            self.assertTrue(worktree_root.exists())
            self.assertTrue(branch_exists(repo, branch))

    def test_runs_recover_prints_manual_steps_without_cleanup(self) -> None:
        with TemporaryDirectory() as directory:
            repo, cache, record = self.create_dirty_worktree_run(Path(directory))
            worktree_root = Path(record["worktree"]["worktree_root"])
            branch = record["worktree"]["branch"]
            (repo / "source-local.txt").write_text("local source change\n", encoding="utf-8")
            output = io.StringIO()

            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False),
                redirect_stdout(output),
            ):
                code = main(["runs", "recover", record["run_id"]])

            text = output.getvalue()

            self.assertEqual(code, 0)
            self.assertIn(f"Manual recovery for worktree run {record['run_id']}", text)
            self.assertIn(str(repo), text)
            self.assertIn(str(worktree_root), text)
            self.assertIn(branch, text)
            self.assertIn("Source status:", text)
            self.assertIn("?? source-local.txt", text)
            self.assertIn("Worktree status:", text)
            self.assertIn(" M tracked.txt", text)
            self.assertIn("?? created.txt", text)
            self.assertIn(f"runhaven runs diff {record['run_id']}", text)
            self.assertIn(f"runhaven runs merge {record['run_id']}", text)
            self.assertIn(f"runhaven runs keep {record['run_id']}", text)
            self.assertIn(f"runhaven runs discard {record['run_id']}", text)
            self.assertTrue(worktree_root.exists())
            self.assertTrue(branch_exists(repo, branch))

    def test_runs_recover_prints_json_without_cleanup(self) -> None:
        with TemporaryDirectory() as directory:
            repo, cache, record = self.create_dirty_worktree_run(Path(directory))
            worktree = record["worktree"]
            worktree_root = Path(worktree["worktree_root"])
            branch = worktree["branch"]
            (repo / "source-local.txt").write_text("local source change\n", encoding="utf-8")
            output = io.StringIO()

            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False),
                redirect_stdout(output),
            ):
                code = main(["runs", "recover", record["run_id"], "--json"])

            payload = json.loads(output.getvalue())
            source_head = run_git(repo, "rev-parse", "HEAD")
            worktree_head = run_git(worktree_root, "rev-parse", "HEAD")

            self.assertEqual(code, 0)
            self.assertEqual(payload["run_id"], record["run_id"])
            self.assertEqual(payload["source_repo_root"], str(repo.resolve()))
            self.assertEqual(payload["worktree_root"], str(worktree_root))
            self.assertEqual(payload["mounted_workspace"], worktree["mounted_workspace"])
            self.assertEqual(payload["branch"], branch)
            self.assertEqual(payload["base_head"], worktree["base_head"])
            self.assertEqual(payload["source_head"], source_head)
            self.assertEqual(payload["worktree_head"], worktree_head)
            self.assertIn("?? source-local.txt", payload["source_status"])
            self.assertIn(" M tracked.txt", payload["worktree_status"])
            self.assertIn("?? created.txt", payload["worktree_status"])
            self.assertEqual(
                payload["commands"]["recover"],
                f"runhaven runs recover {record['run_id']}",
            )
            self.assertEqual(
                payload["commands"]["merge"],
                f"runhaven runs merge {record['run_id']}",
            )
            self.assertIn("Retry guarded merge", " ".join(payload["next_steps"]))
            self.assertTrue(worktree_root.exists())
            self.assertTrue(branch_exists(repo, branch))

    def test_runs_recover_json_includes_project_check_suggestions(self) -> None:
        with TemporaryDirectory() as directory:
            repo, cache, record = self.create_project_check_worktree_run(Path(directory))
            worktree = record["worktree"]
            worktree_root = Path(worktree["worktree_root"])
            output = io.StringIO()

            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": str(cache)}, clear=False),
                redirect_stdout(output),
            ):
                code = main(["runs", "recover", record["run_id"], "--json"])

            payload = json.loads(output.getvalue())
            suggested_checks = payload["suggested_checks"]
            commands = {check["label"]: check["command"] for check in suggested_checks}

            self.assertEqual(code, 0)
            self.assertEqual(commands["Node tests"].split("-- ")[-1], "npm test")
            self.assertEqual(commands["Node lint"].split("-- ")[-1], "npm run lint")
            self.assertEqual(
                commands["Python tests"].split("-- ")[-1],
                "python -m unittest discover -s tests",
            )
            self.assertEqual(
                suggested_checks[0]["argv"][:6],
                [
                    "runhaven",
                    "run",
                    "shell",
                    "--workspace",
                    worktree["mounted_workspace"],
                    "--network",
                ],
            )
            self.assertTrue(worktree_root.exists())
            self.assertTrue(branch_exists(repo, worktree["branch"]))

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

    def create_project_check_worktree_run(
        self,
        directory: Path,
    ) -> tuple[Path, Path, dict[str, object]]:
        return self.create_worktree_run(directory, commit_change=False, project_checks=True)

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
        project_checks: bool = False,
    ) -> tuple[Path, Path, dict[str, object]]:
        repo = directory / "repo"
        cache = directory / "cache"
        repo.mkdir()
        init_git_repo(repo)
        if project_checks:
            (repo / "package.json").write_text(
                json.dumps({"scripts": {"test": "node --test", "lint": "eslint ."}}),
                encoding="utf-8",
            )
            (repo / "tests").mkdir()
            (repo / "tests" / "test_sample.py").write_text(
                "import unittest\n\n"
                "class SampleTests(unittest.TestCase):\n"
                "    def test_sample(self) -> None:\n"
                "        self.assertTrue(True)\n",
                encoding="utf-8",
            )
            run_git(repo, "add", "package.json", "tests/test_sample.py")
            run_git(repo, "commit", "-m", "add project check fixtures")

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
