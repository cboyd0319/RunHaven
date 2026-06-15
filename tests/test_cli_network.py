from __future__ import annotations

import io
import unittest
from contextlib import redirect_stdout
from unittest.mock import Mock, patch

from runhaven.cli import main


class CliNetworkTests(unittest.TestCase):
    def test_network_list_filters_runhaven_managed_networks(self) -> None:
        output = io.StringIO()
        with (
            redirect_stdout(output),
            patch("runhaven.cli.require_container_cli"),
            patch("runhaven.cli.subprocess.run") as run,
        ):
            run.return_value = Mock(
                returncode=0,
                stdout=(
                    "default\n"
                    "mca-volume-prep-internal\n"
                    "runhaven-volume-prep-internal\n"
                    "runhaven-1234567890abcdef-internal\n"
                    "runhaven-claude-abcdef1234567890-provider\n"
                    "runhaven-project-internal\n"
                    "runhaven-claude-project-provider\n"
                    "runhaven-unrelated\n"
                    "other-runhaven-project-provider\n"
                ),
                stderr="",
            )

            code = main(["network", "list"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("runhaven-volume-prep-internal", text)
        self.assertIn("runhaven-1234567890abcdef-internal", text)
        self.assertIn("runhaven-claude-abcdef1234567890-provider", text)
        self.assertNotIn("runhaven-project-internal", text)
        self.assertNotIn("runhaven-claude-project-provider", text)
        self.assertNotIn("mca-volume-prep-internal", text)
        self.assertNotIn("runhaven-unrelated", text)
        self.assertNotIn("other-runhaven-project-provider", text)
        run.assert_called_once_with(
            ("container", "network", "list", "--quiet"),
            check=False,
            capture_output=True,
            text=True,
        )

    def test_network_prune_requires_yes(self) -> None:
        output = io.StringIO()
        with (
            redirect_stdout(output),
            patch("runhaven.cli.require_container_cli"),
            patch("runhaven.cli.subprocess.run") as run,
        ):
            run.return_value = Mock(
                returncode=0,
                stdout="runhaven-1234567890abcdef-internal\nrunhaven-unrelated\n",
                stderr="",
            )

            code = main(["network", "prune"])

        self.assertEqual(code, 2)
        text = output.getvalue()
        self.assertIn("runhaven-1234567890abcdef-internal", text)
        self.assertIn("--yes", text)
        self.assertNotIn("runhaven-unrelated", text)
        run.assert_called_once()

    def test_network_prune_deletes_only_runhaven_managed_networks_with_yes(self) -> None:
        with (
            patch("runhaven.cli.require_container_cli"),
            patch("runhaven.cli.subprocess.run") as run,
        ):
            run.side_effect = [
                Mock(
                    returncode=0,
                    stdout=(
                        "runhaven-volume-prep-internal\n"
                        "runhaven-1234567890abcdef-internal\n"
                        "runhaven-claude-abcdef1234567890-provider\n"
                        "runhaven-unrelated\n"
                        "runhaven-claude-project-provider\n"
                    ),
                    stderr="",
                ),
                Mock(returncode=0, stdout="", stderr=""),
                Mock(returncode=0, stdout="", stderr=""),
                Mock(returncode=0, stdout="", stderr=""),
            ]

            code = main(["network", "prune", "--yes"])

        self.assertEqual(code, 0)
        self.assertEqual(run.call_count, 4)
        self.assertEqual(
            [call.args[0] for call in run.call_args_list[1:]],
            [
                ("container", "network", "delete", "runhaven-volume-prep-internal"),
                ("container", "network", "delete", "runhaven-1234567890abcdef-internal"),
                (
                    "container",
                    "network",
                    "delete",
                    "runhaven-claude-abcdef1234567890-provider",
                ),
            ],
        )


if __name__ == "__main__":
    unittest.main()
