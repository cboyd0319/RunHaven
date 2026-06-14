from __future__ import annotations

import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from runhaven.plans import RunOptions, build_run_plan, validate_env_name
from runhaven.profiles import get_profile


class RunPlanTests(unittest.TestCase):
    def test_default_plan_mounts_only_workspace_and_agent_home(self) -> None:
        with TemporaryDirectory() as directory:
            workspace = Path(directory)
            plan = build_run_plan(
                RunOptions(profile=get_profile("claude"), workspace=workspace)
            )

        command = list(plan.command)
        joined = plan.shell_command()

        self.assertEqual(command[:2], ["container", "run"])
        self.assertIn("--read-only", command)
        self.assertIn("--cap-drop", command)
        self.assertIn("ALL", command)
        self.assertIn("type=volume", joined)
        self.assertIn("target=/home/agent", joined)
        self.assertIn("target=/workspace", joined)
        self.assertIn("PATH=/opt/runhaven-agent/node_modules/.bin", joined)
        self.assertNotIn(str(Path.home()), joined)
        self.assertNotIn("ANTHROPIC_API_KEY", command)
        self.assertEqual(len(plan.preflight), 2)
        self.assertEqual(
            plan.preflight[0],
            ("container", "network", "create", "--internal", "runhaven-volume-prep-internal"),
        )
        self.assertIn("--network", plan.preflight[1])
        self.assertIn("runhaven-volume-prep-internal", plan.preflight[1])
        self.assertIn("chown 1000:1000 /home/agent", plan.shell_preflight()[1])
        self.assertIn("mkdir -p /home/agent/.claude", plan.shell_preflight()[1])
        self.assertIsNone(plan.network_name)

    def test_internal_network_adds_preflight_and_network_flag(self) -> None:
        with TemporaryDirectory() as directory:
            workspace = Path(directory)
            plan = build_run_plan(
                RunOptions(profile=get_profile("codex"), workspace=workspace, network="internal")
            )

        self.assertEqual(len(plan.preflight), 3)
        self.assertEqual(plan.preflight[2][:4], ("container", "network", "create", "--internal"))
        self.assertIsNotNone(plan.network_name)
        self.assertIn("--network", plan.command)
        self.assertIn(plan.network_name, plan.command)

    def test_custom_user_skips_agent_home_chown(self) -> None:
        with TemporaryDirectory() as directory:
            workspace = Path(directory)
            plan = build_run_plan(
                RunOptions(profile=get_profile("shell"), workspace=workspace, user="root")
            )

        self.assertEqual(plan.preflight, ())

    def test_read_only_workspace_marks_bind_mount_readonly(self) -> None:
        with TemporaryDirectory() as directory:
            workspace = Path(directory)
            plan = build_run_plan(
                RunOptions(
                    profile=get_profile("shell"),
                    workspace=workspace,
                    read_only_workspace=True,
                )
            )

        mounts = [
            plan.command[index + 1]
            for index, value in enumerate(plan.command)
            if value == "--mount"
        ]
        workspace_mount = next(mount for mount in mounts if "target=/workspace" in mount)
        self.assertTrue(workspace_mount.endswith(",readonly"))

    def test_explicit_env_inherits_by_name_without_value(self) -> None:
        with TemporaryDirectory() as directory:
            workspace = Path(directory)
            plan = build_run_plan(
                RunOptions(
                    profile=get_profile("claude"),
                    workspace=workspace,
                    env=("ANTHROPIC_API_KEY",),
                )
            )

        command = list(plan.command)
        index = command.index("ANTHROPIC_API_KEY")
        self.assertEqual(command[index - 1], "--env")

    def test_rejects_env_values(self) -> None:
        with self.assertRaisesRegex(ValueError, "variable names"):
            validate_env_name("AWS_SECRET_ACCESS_KEY=value")

    def test_agent_args_override_profile_command(self) -> None:
        with TemporaryDirectory() as directory:
            workspace = Path(directory)
            plan = build_run_plan(
                RunOptions(
                    profile=get_profile("claude"),
                    workspace=workspace,
                    agent_args=("--", "claude", "--version"),
                )
            )

        self.assertEqual(plan.command[-2:], ("claude", "--version"))


if __name__ == "__main__":
    unittest.main()
