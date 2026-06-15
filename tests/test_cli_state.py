from __future__ import annotations

import io
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import Mock, patch

from runhaven.cli import acquire_state_lock, main, state_lock_path


class CliStateTests(unittest.TestCase):
    def test_state_list_filters_runhaven_volumes(self) -> None:
        output = io.StringIO()
        with (
            redirect_stdout(output),
            patch("runhaven.cli.require_container_cli"),
            patch("runhaven.cli.subprocess.run") as run,
        ):
            run.return_value = Mock(
                returncode=0,
                stdout="runhaven-claude-abc-home\nother-volume\nrunhaven-shell-def-home\n",
                stderr="",
            )

            code = main(["state", "list"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("runhaven-claude-abc-home", text)
        self.assertIn("runhaven-shell-def-home", text)
        self.assertNotIn("other-volume", text)

    def test_state_prune_requires_yes(self) -> None:
        output = io.StringIO()
        with (
            redirect_stdout(output),
            patch("runhaven.cli.require_container_cli"),
            patch("runhaven.cli.subprocess.run") as run,
        ):
            run.return_value = Mock(returncode=0, stdout="runhaven-shell-def-home\n", stderr="")

            code = main(["state", "prune"])

        self.assertEqual(code, 2)
        self.assertIn("--yes", output.getvalue())
        run.assert_called_once()

    def test_state_prune_deletes_runhaven_volumes_with_yes(self) -> None:
        with (
            patch("runhaven.cli.require_container_cli"),
            patch("runhaven.cli.subprocess.run") as run,
        ):
            run.side_effect = [
                Mock(returncode=0, stdout="runhaven-shell-def-home\n", stderr=""),
                Mock(returncode=0, stdout="", stderr=""),
            ]

            code = main(["state", "prune", "--yes"])

        self.assertEqual(code, 0)
        self.assertEqual(
            run.call_args_list[1].args[0],
            ("container", "volume", "delete", "runhaven-shell-def-home"),
        )

    def test_state_list_filters_named_session_volumes(self) -> None:
        output = io.StringIO()
        with (
            redirect_stdout(output),
            patch("runhaven.cli.require_container_cli"),
            patch("runhaven.cli.subprocess.run") as run,
        ):
            run.return_value = Mock(
                returncode=0,
                stdout=(
                    "runhaven-shell-aaa-s-review-12345678-home\n"
                    "runhaven-shell-bbb-s-review-12345678-home\n"
                    "runhaven-shell-ccc-s-default-87654321-home\n"
                    "runhaven-shell-ddd-home\n"
                ),
                stderr="",
            )

            code = main(["state", "list", "--session", "review"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("runhaven-shell-aaa-s-review-12345678-home", text)
        self.assertIn("runhaven-shell-bbb-s-review-12345678-home", text)
        self.assertNotIn("runhaven-shell-ccc-s-default-87654321-home", text)
        self.assertNotIn("runhaven-shell-ddd-home", text)

    def test_state_list_rejects_invalid_session_before_container_command(self) -> None:
        with (
            patch("runhaven.cli.require_container_cli"),
            patch("runhaven.cli.subprocess.run") as run,
        ):
            with self.assertRaises(SystemExit) as error:
                main(["state", "list", "--session", "../shared"])

        self.assertEqual(error.exception.code, 2)
        run.assert_not_called()

    def test_state_prune_filters_named_session_volumes_with_yes(self) -> None:
        with (
            patch("runhaven.cli.require_container_cli"),
            patch("runhaven.cli.subprocess.run") as run,
        ):
            run.side_effect = [
                Mock(
                    returncode=0,
                    stdout=(
                        "runhaven-shell-aaa-s-review-12345678-home\n"
                        "runhaven-shell-bbb-s-default-87654321-home\n"
                    ),
                    stderr="",
                ),
                Mock(returncode=0, stdout="", stderr=""),
            ]

            code = main(["state", "prune", "--session", "review", "--yes"])

        self.assertEqual(code, 0)
        self.assertEqual(run.call_count, 2)
        self.assertEqual(
            run.call_args_list[1].args[0],
            ("container", "volume", "delete", "runhaven-shell-aaa-s-review-12345678-home"),
        )

    def test_state_reset_deletes_exact_planned_named_session_volume_with_yes(self) -> None:
        with TemporaryDirectory() as directory:
            workspace = Path(directory) / "workspace"
            workspace.mkdir()
            output = io.StringIO()
            with (
                redirect_stdout(output),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.subprocess.run") as run,
            ):
                run.return_value = Mock(returncode=0, stdout="", stderr="")

                code = main(
                    [
                        "state",
                        "reset",
                        "shell",
                        "--workspace",
                        str(workspace),
                        "--session",
                        "review",
                        "--yes",
                    ]
                )

        self.assertEqual(code, 0)
        delete_command = run.call_args.args[0]
        self.assertEqual(delete_command[:3], ("container", "volume", "delete"))
        self.assertIn("-s-review-", delete_command[3])
        self.assertIn(delete_command[3], output.getvalue())

    def test_state_reset_requires_yes(self) -> None:
        with TemporaryDirectory() as directory:
            workspace = Path(directory) / "workspace"
            workspace.mkdir()
            output = io.StringIO()
            with (
                redirect_stdout(output),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.subprocess.run") as run,
            ):
                run.return_value = Mock(returncode=1, stdout="", stderr="")
                code = main(
                    [
                        "state",
                        "reset",
                        "shell",
                        "--workspace",
                        str(workspace),
                        "--session",
                        "review",
                    ]
                )

        self.assertEqual(code, 2)
        self.assertIn("--yes", output.getvalue())
        self.assertFalse(
            any(
                call.args[0][:3] == ("container", "volume", "delete")
                for call in run.call_args_list
            )
        )

    def test_state_lock_rejects_concurrent_same_volume(self) -> None:
        with TemporaryDirectory() as directory:
            with patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False):
                lock_path = state_lock_path("runhaven-test-home")

                with acquire_state_lock("runhaven-test-home"):
                    self.assertTrue(lock_path.exists())
                    with self.assertRaisesRegex(ValueError, "already in use"):
                        with acquire_state_lock("runhaven-test-home"):
                            pass


if __name__ == "__main__":
    unittest.main()
