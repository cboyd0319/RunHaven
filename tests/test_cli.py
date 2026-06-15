from __future__ import annotations

import io
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

from cli_test_helpers import init_git_repo

from runhaven.cli import main
from runhaven.doctor import Check


class CliCoreTests(unittest.TestCase):
    def test_agents_lists_known_profiles(self) -> None:
        output = io.StringIO()
        with redirect_stdout(output):
            code = main(["agents"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("claude", text)
        self.assertIn("codex", text)
        self.assertIn("copilot", text)

    def test_plan_prints_dry_run_command(self) -> None:
        with TemporaryDirectory() as directory:
            output = io.StringIO()
            with redirect_stdout(output):
                code = main(
                    ["plan", "shell", "--workspace", directory, "--", "/bin/bash", "-lc", "pwd"]
                )

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("Workspace:", text)
        self.assertIn("State volume:", text)
        self.assertIn("container run", text)
        self.assertIn("/bin/bash -lc pwd", text)
        self.assertIn("Egress: unrestricted internet", text)

    def test_plan_git_root_workspace_scope_expands_and_prints_note(self) -> None:
        with TemporaryDirectory() as directory:
            repo = Path(directory)
            init_git_repo(repo)
            workspace = repo / "package"
            workspace.mkdir()

            output = io.StringIO()
            with redirect_stdout(output):
                code = main(
                    [
                        "plan",
                        "shell",
                        "--workspace",
                        str(workspace),
                        "--workspace-scope",
                        "git-root",
                    ]
                )

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn(f"Workspace: {repo.resolve()}", text)
        self.assertIn("Workspace scope: git-root", text)
        self.assertIn("Workspace note: expanded from", text)

    def test_image_build_dry_run_uses_bundled_containerfile(self) -> None:
        output = io.StringIO()
        with redirect_stdout(output):
            code = main(["image", "build", "shell", "--dry-run"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("container build", text)
        self.assertIn("Containerfile", text)
        self.assertIn("runhaven/base:0.1.0", text)

    def test_missing_workspace_is_user_error(self) -> None:
        with TemporaryDirectory() as directory:
            missing = Path(directory) / "missing"
            with self.assertRaises(SystemExit) as error:
                main(["plan", "shell", "--workspace", str(missing)])

        self.assertEqual(error.exception.code, 2)

    def test_help_does_not_resolve_current_directory(self) -> None:
        output = io.StringIO()
        with (
            redirect_stdout(output),
            patch("runhaven.cli.Path.cwd", side_effect=FileNotFoundError),
            self.assertRaises(SystemExit) as error,
        ):
            main(["--help"])

        self.assertEqual(error.exception.code, 0)
        self.assertIn("Run AI coding agents", output.getvalue())

    def test_run_help_explains_agent_argument_separator(self) -> None:
        output = io.StringIO()
        with redirect_stdout(output), self.assertRaises(SystemExit) as error:
            main(["run", "--help"])

        self.assertEqual(error.exception.code, 0)
        self.assertIn("Use -- before flags meant for the agent", output.getvalue())
        self.assertIn("provider", output.getvalue())
        self.assertIn("runtime allowlist proxy", output.getvalue())
        self.assertIn("--workspace-scope", output.getvalue())

    def test_doctor_prints_remedy_for_failed_checks(self) -> None:
        output = io.StringIO()
        with (
            redirect_stdout(output),
            patch(
                "runhaven.cli.collect_checks",
                return_value=(Check("Apple container CLI", False, "not found", "Install it."),),
            ),
        ):
            code = main(["doctor"])

        self.assertEqual(code, 1)
        text = output.getvalue()
        self.assertIn("fail Apple container CLI", text)
        self.assertIn("fix: Install it.", text)

    def test_setup_prints_remedies_when_prerequisites_fail(self) -> None:
        output = io.StringIO()
        with (
            redirect_stdout(output),
            patch(
                "runhaven.cli.collect_checks",
                return_value=(
                    Check("python", True, "3.14.6"),
                    Check(
                        "container system",
                        False,
                        "not running",
                        "Run `container system start`.",
                    ),
                ),
            ),
        ):
            code = main(["setup"])

        self.assertEqual(code, 1)
        text = output.getvalue()
        self.assertIn("RunHaven setup", text)
        self.assertIn("ok   python: 3.14.6", text)
        self.assertIn("fail container system: not running", text)
        self.assertIn("Next steps", text)
        self.assertIn("- container system: Run `container system start`.", text)
        self.assertIn("runhaven setup", text)
        self.assertNotIn("runhaven image build", text)

    def test_setup_prints_first_run_commands_when_ready(self) -> None:
        output = io.StringIO()
        with (
            redirect_stdout(output),
            patch(
                "runhaven.cli.collect_checks",
                return_value=(
                    Check("python", True, "3.14.6"),
                    Check("container system", True, "running"),
                ),
            ),
        ):
            code = main(["setup"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("RunHaven setup", text)
        self.assertIn("Selected agent: claude", text)
        self.assertIn("runhaven image build claude", text)
        self.assertIn("runhaven plan claude", text)
        self.assertIn("runhaven run claude", text)
        self.assertIn("No host home, raw SSH keys, or cloud credential folders", text)
        self.assertNotIn("Next steps", text)

    def test_setup_accepts_agent_profile(self) -> None:
        output = io.StringIO()
        with (
            redirect_stdout(output),
            patch(
                "runhaven.cli.collect_checks",
                return_value=(Check("container system", True, "running"),),
            ),
        ):
            code = main(["setup", "--agent", "codex"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("Selected agent: codex", text)
        self.assertIn("runhaven image build codex", text)
        self.assertIn("runhaven plan codex", text)
        self.assertIn("runhaven run codex", text)

    def test_setup_prints_goal_based_network_guidance(self) -> None:
        output = io.StringIO()
        with (
            redirect_stdout(output),
            patch(
                "runhaven.cli.collect_checks",
                return_value=(Check("container system", True, "running"),),
            ),
        ):
            code = main(["setup"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("Network choices", text)
        self.assertIn("Local-only", text)
        self.assertIn("runhaven run claude --network internal", text)
        self.assertIn("Provider-only", text)
        self.assertIn("runhaven run claude --network provider", text)
        self.assertIn("Package install", text)
        self.assertIn("Unrestricted internet", text)
        self.assertIn("Use `runhaven plan` before changing network modes.", text)

    def test_setup_prints_workspace_and_credential_guidance(self) -> None:
        output = io.StringIO()
        with (
            redirect_stdout(output),
            patch(
                "runhaven.cli.collect_checks",
                return_value=(Check("container system", True, "running"),),
            ),
        ):
            code = main(["setup"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("Workspace and credentials", text)
        self.assertIn("smallest project directory", text)
        self.assertIn("mounted at /workspace", text)
        self.assertIn("Do not run from your home directory", text)
        self.assertIn("raw SSH keys", text)
        self.assertIn("browser profiles", text)
        self.assertIn("cloud credential folders", text)
        self.assertIn("provider login caches", text)
        self.assertIn("Use `--ssh` for SSH agent forwarding", text)
        self.assertIn("Use `--env NAME` only for a reviewed variable", text)
        self.assertIn("Use `runhaven plan` to confirm the mounted host path.", text)

    def test_plan_tty_always_adds_tty_flag(self) -> None:
        with TemporaryDirectory() as directory:
            output = io.StringIO()
            with redirect_stdout(output):
                code = main(["plan", "shell", "--workspace", directory, "--tty", "always"])

        self.assertEqual(code, 0)
        self.assertIn("--tty", output.getvalue())


if __name__ == "__main__":
    unittest.main()
