from __future__ import annotations

import unittest
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

from cli_test_helpers import init_git_repo

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
        self.assertIn("--name", command)
        self.assertTrue(plan.container_name.startswith("runhaven-claude-"))
        self.assertTrue(plan.container_name.endswith("-run"))
        self.assertIn(plan.container_name, command)
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

    def test_current_workspace_scope_keeps_subdirectory_inside_git_repo(self) -> None:
        with TemporaryDirectory() as directory:
            repo = Path(directory)
            init_git_repo(repo)
            workspace = repo / "package"
            workspace.mkdir()

            plan = build_run_plan(
                RunOptions(profile=get_profile("shell"), workspace=workspace)
            )

        self.assertEqual(plan.workspace, workspace.resolve())
        self.assertEqual(plan.workspace_scope, "current")
        self.assertIsNotNone(plan.workspace_scope_note)
        self.assertIn("inside git repository root", plan.workspace_scope_note)
        self.assertIn("--workspace-scope git-root", plan.workspace_scope_note)
        workspace_mount = next(mount for mount in plan.command if "target=/workspace" in mount)
        self.assertIn(f"source={workspace.resolve()}", workspace_mount)

    def test_git_root_workspace_scope_expands_subdirectory_explicitly(self) -> None:
        with TemporaryDirectory() as directory:
            repo = Path(directory)
            init_git_repo(repo)
            workspace = repo / "package"
            workspace.mkdir()

            plan = build_run_plan(
                RunOptions(
                    profile=get_profile("shell"),
                    workspace=workspace,
                    workspace_scope="git-root",
                )
            )

        self.assertEqual(plan.workspace, repo.resolve())
        self.assertEqual(plan.workspace_scope, "git-root")
        self.assertIsNotNone(plan.workspace_scope_note)
        self.assertIn("expanded from", plan.workspace_scope_note)
        workspace_mount = next(mount for mount in plan.command if "target=/workspace" in mount)
        self.assertIn(f"source={repo.resolve()}", workspace_mount)

    def test_git_root_workspace_scope_requires_git_worktree(self) -> None:
        with TemporaryDirectory() as directory:
            with self.assertRaisesRegex(ValueError, "requires a git worktree"):
                build_run_plan(
                    RunOptions(
                        profile=get_profile("shell"),
                        workspace=Path(directory),
                        workspace_scope="git-root",
                    )
                )

    def test_internet_network_has_no_provider_allowlist(self) -> None:
        with TemporaryDirectory() as directory:
            plan = build_run_plan(
                RunOptions(profile=get_profile("shell"), workspace=Path(directory))
            )

        self.assertEqual(plan.network_mode, "internet")
        self.assertIsNone(plan.network_name)
        self.assertEqual(plan.provider_allowed_hosts, ())
        self.assertNotIn("--network", plan.command)
        self.assertIn("unrestricted internet", plan.egress_summary)
        self.assertIn("allowlisting is not enforced", plan.egress_summary)

    def test_internal_network_has_no_provider_allowlist_and_disables_internet(self) -> None:
        with TemporaryDirectory() as directory:
            plan = build_run_plan(
                RunOptions(
                    profile=get_profile("shell"),
                    workspace=Path(directory),
                    network="internal",
                )
            )

        self.assertEqual(plan.network_mode, "internal")
        self.assertIsNotNone(plan.network_name)
        self.assertEqual(plan.provider_allowed_hosts, ())
        self.assertIn("--network", plan.command)
        self.assertIn(plan.network_name, plan.command)
        self.assertIn("internet egress disabled", plan.egress_summary)
        self.assertNotIn("provider allowlist", plan.egress_summary)

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
            for profile_name in ("antigravity", "shell"):
                with self.subTest(profile=profile_name):
                    with self.assertRaisesRegex(ValueError, "provider hosts"):
                        build_run_plan(
                            RunOptions(
                                profile=get_profile(profile_name),
                                workspace=Path(directory),
                                network="provider",
                            )
                        )

    def test_provider_network_allows_empty_profile_with_explicit_host(self) -> None:
        with TemporaryDirectory() as directory:
            plan = build_run_plan(
                RunOptions(
                    profile=get_profile("shell"),
                    workspace=Path(directory),
                    network="provider",
                    provider_hosts=("api.example.com",),
                )
            )

        self.assertEqual(plan.network_mode, "provider")
        self.assertEqual(plan.provider_allowed_hosts, ("api.example.com",))
        self.assertIn("provider allowlist", plan.egress_summary)

    def test_codex_api_key_broker_requires_codex_profile(self) -> None:
        with TemporaryDirectory() as directory:
            with self.assertRaisesRegex(ValueError, "Codex API key broker requires codex"):
                build_run_plan(
                    RunOptions(
                        profile=get_profile("shell"),
                        workspace=Path(directory),
                        network="provider",
                        provider_hosts=("api.example.com",),
                        codex_api_key_broker_env="OPENAI_API_KEY",
                    )
                )

    def test_codex_api_key_broker_requires_provider_network(self) -> None:
        with TemporaryDirectory() as directory:
            with self.assertRaisesRegex(ValueError, "--network provider"):
                build_run_plan(
                    RunOptions(
                        profile=get_profile("codex"),
                        workspace=Path(directory),
                        codex_api_key_broker_env="OPENAI_API_KEY",
                    )
                )

    def test_codex_api_key_broker_plan_is_secret_free(self) -> None:
        with TemporaryDirectory() as directory:
            plan = build_run_plan(
                RunOptions(
                    profile=get_profile("codex"),
                    workspace=Path(directory),
                    network="provider",
                    codex_api_key_broker_env="OPENAI_API_KEY",
                )
            )

        self.assertEqual(plan.codex_api_key_broker_env, "OPENAI_API_KEY")
        self.assertIn("Codex API key broker", plan.egress_summary)
        self.assertNotIn("fake-openai-api-key-value", plan.shell_command())
        self.assertNotIn("OPENAI_API_KEY", plan.command)

    def test_codex_api_key_broker_requires_codex_command(self) -> None:
        with TemporaryDirectory() as directory:
            with self.assertRaisesRegex(ValueError, "agent command to start with codex"):
                build_run_plan(
                    RunOptions(
                        profile=get_profile("codex"),
                        workspace=Path(directory),
                        network="provider",
                        agent_args=("--", "python", "-V"),
                        codex_api_key_broker_env="OPENAI_API_KEY",
                    )
                )

    def test_empty_provider_allowlist_behavior_is_explicit_for_each_network_mode(self) -> None:
        with TemporaryDirectory() as directory:
            workspace = Path(directory)
            cases = {
                "internet": "unrestricted internet",
                "internal": "internet egress disabled",
            }
            for network, summary in cases.items():
                with self.subTest(network=network):
                    plan = build_run_plan(
                        RunOptions(
                            profile=get_profile("shell"),
                            workspace=workspace,
                            network=network,
                        )
                    )
                    self.assertEqual(plan.provider_allowed_hosts, ())
                    self.assertIn(summary, plan.egress_summary)

            with self.subTest(network="provider"):
                with self.assertRaisesRegex(ValueError, "provider hosts are required"):
                    build_run_plan(
                        RunOptions(
                            profile=get_profile("shell"),
                            workspace=workspace,
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
