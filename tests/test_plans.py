from __future__ import annotations

import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

from runhaven.plans import RunOptions, build_run_plan, validate_env_name
from runhaven.profiles import get_profile
from runhaven.provider_endpoints import BUNDLED_PROVIDER_HOSTS


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
        self.assertIn("--interactive", command)
        self.assertIn("--tty", command)
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
        self.assertEqual(plan.network_mode, "internet")
        self.assertIn("unrestricted internet", plan.egress_summary)

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
        self.assertEqual(plan.network_mode, "internal")
        self.assertIn("host-only", plan.egress_summary)

    def test_provider_network_adds_internal_network_and_allowlist_hosts(self) -> None:
        with TemporaryDirectory() as directory:
            workspace = Path(directory)
            plan = build_run_plan(
                RunOptions(profile=get_profile("codex"), workspace=workspace, network="provider")
            )

        self.assertEqual(plan.network_mode, "provider")
        self.assertIsNotNone(plan.network_name)
        self.assertIn("--network", plan.command)
        self.assertIn(plan.network_name, plan.command)
        self.assertIn("api.openai.com", plan.provider_allowed_hosts)
        self.assertIn("chatgpt.com", plan.provider_allowed_hosts)
        self.assertIn("provider allowlist", plan.egress_summary)
        self.assertNotIn("HTTPS_PROXY", plan.command)

    def test_profile_provider_hosts_match_endpoint_ledger(self) -> None:
        for profile_name, expected_hosts in BUNDLED_PROVIDER_HOSTS.items():
            with self.subTest(profile=profile_name):
                self.assertEqual(get_profile(profile_name).provider_hosts, expected_hosts)

    def test_provider_network_requires_allowed_hosts(self) -> None:
        with TemporaryDirectory() as directory:
            with self.assertRaisesRegex(ValueError, "provider hosts"):
                build_run_plan(
                    RunOptions(
                        profile=get_profile("shell"),
                        workspace=Path(directory),
                        network="provider",
                    )
                )

    def test_root_user_requires_explicit_unsafe_override(self) -> None:
        with TemporaryDirectory() as directory:
            workspace = Path(directory)
            with self.assertRaisesRegex(ValueError, "root user"):
                build_run_plan(
                    RunOptions(
                        profile=get_profile("shell"),
                        workspace=workspace,
                        user="root",
                    )
                )

    def test_root_group_requires_explicit_unsafe_override(self) -> None:
        with TemporaryDirectory() as directory:
            workspace = Path(directory)
            with self.assertRaisesRegex(ValueError, "root user or group"):
                build_run_plan(
                    RunOptions(
                        profile=get_profile("shell"),
                        workspace=workspace,
                        user="agent:0",
                    )
                )

    def test_root_identity_with_leading_zero_requires_explicit_unsafe_override(self) -> None:
        with TemporaryDirectory() as directory:
            workspace = Path(directory)
            for user in ("00", "agent:00"):
                with self.subTest(user=user):
                    with self.assertRaisesRegex(ValueError, "root user or group"):
                        build_run_plan(
                            RunOptions(
                                profile=get_profile("shell"),
                                workspace=workspace,
                                user=user,
                            )
                        )

    def test_allowed_root_user_skips_agent_home_chown(self) -> None:
        with TemporaryDirectory() as directory:
            workspace = Path(directory)
            plan = build_run_plan(
                RunOptions(
                    profile=get_profile("shell"),
                    workspace=workspace,
                    user="root",
                    allow_root_user=True,
                )
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

    def test_rejects_sensitive_workspace_without_override(self) -> None:
        with self.assertRaisesRegex(ValueError, "sensitive workspace"):
            build_run_plan(RunOptions(profile=get_profile("shell"), workspace=Path.home()))

    def test_allows_sensitive_workspace_with_explicit_override(self) -> None:
        plan = build_run_plan(
            RunOptions(
                profile=get_profile("shell"),
                workspace=Path.home(),
                allow_sensitive_workspace=True,
            )
        )

        self.assertEqual(plan.workspace, Path.home().resolve())

    def test_rejects_comma_in_workspace_path(self) -> None:
        with TemporaryDirectory(prefix="runhaven,comma.") as directory:
            with self.assertRaisesRegex(ValueError, "comma"):
                build_run_plan(RunOptions(profile=get_profile("shell"), workspace=Path(directory)))

    def test_rejects_sensitive_system_workspace_without_override(self) -> None:
        for workspace in (Path("/System"), Path("/Library"), Path("/etc")):
            with self.subTest(workspace=workspace):
                with self.assertRaisesRegex(ValueError, "sensitive workspace"):
                    build_run_plan(RunOptions(profile=get_profile("shell"), workspace=workspace))

    def test_rejects_unsafe_image_reference(self) -> None:
        with TemporaryDirectory() as directory:
            with self.assertRaisesRegex(ValueError, "image"):
                build_run_plan(
                    RunOptions(
                        profile=get_profile("shell"),
                        workspace=Path(directory),
                        image="--cap-add",
                    )
                )

    def test_rejects_invalid_resource_values(self) -> None:
        with TemporaryDirectory() as directory:
            workspace = Path(directory)
            with self.assertRaisesRegex(ValueError, "cpus"):
                build_run_plan(
                    RunOptions(profile=get_profile("shell"), workspace=workspace, cpus="many")
                )
            with self.assertRaisesRegex(ValueError, "memory"):
                build_run_plan(
                    RunOptions(profile=get_profile("shell"), workspace=workspace, memory="large")
                )

    def test_rejects_invalid_network_mode(self) -> None:
        with TemporaryDirectory() as directory:
            with self.assertRaisesRegex(ValueError, "network mode"):
                build_run_plan(
                    RunOptions(
                        profile=get_profile("shell"),
                        workspace=Path(directory),
                        network="provider-only",
                    )
                )

    def test_provider_network_rejects_ip_literal_allowed_hosts(self) -> None:
        with TemporaryDirectory() as directory:
            with self.assertRaisesRegex(ValueError, "IP literals"):
                build_run_plan(
                    RunOptions(
                        profile=get_profile("shell"),
                        workspace=Path(directory),
                        network="provider",
                        provider_hosts=("1.1.1.1",),
                    )
                )

    def test_provider_network_rejects_single_label_allowed_hosts(self) -> None:
        with TemporaryDirectory() as directory:
            with self.assertRaisesRegex(ValueError, "fully qualified"):
                build_run_plan(
                    RunOptions(
                        profile=get_profile("shell"),
                        workspace=Path(directory),
                        network="provider",
                        provider_hosts=("com",),
                    )
                )

    def test_workspace_resolution_errors_are_user_errors(self) -> None:
        with (
            patch("runhaven.plans.Path.resolve", side_effect=FileNotFoundError("missing cwd")),
            self.assertRaisesRegex(ValueError, "could not resolve workspace path"),
        ):
            build_run_plan(RunOptions(profile=get_profile("shell"), workspace=Path(".")))

    def test_no_tty_option_disables_tty_flag(self) -> None:
        with TemporaryDirectory() as directory:
            plan = build_run_plan(
                RunOptions(profile=get_profile("shell"), workspace=Path(directory), tty=False)
            )

        self.assertIn("--interactive", plan.command)
        self.assertNotIn("--tty", plan.command)

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
