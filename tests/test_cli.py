from __future__ import annotations

import io
import json
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import Mock, patch

from runhaven.cli import (
    acquire_state_lock,
    ensure_internal_network,
    main,
    state_lock_path,
)
from runhaven.doctor import Check


class CliTests(unittest.TestCase):
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

    def test_provider_network_plan_prints_allowlist_summary(self) -> None:
        with TemporaryDirectory() as directory:
            output = io.StringIO()
            with redirect_stdout(output):
                code = main(["plan", "codex", "--workspace", directory, "--network", "provider"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("provider allowlist", text)
        self.assertIn("api.openai.com", text)
        self.assertIn("RunHaven injects proxy environment variables at runtime", text)

    def test_provider_run_injects_proxy_environment_and_cleans_network(self) -> None:
        with TemporaryDirectory() as directory:
            fake_proxy = Mock()
            fake_proxy.server_address = ("0.0.0.0", 49321)
            thread = Mock()
            network_info = Mock(ipv4_gateway="192.168.130.1", ipv4_subnet="192.168.130.0/24")
            with (
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.run_preflight") as preflight,
                patch(
                    "runhaven.cli.inspect_internal_network",
                    return_value=network_info,
                ) as inspect,
                patch("runhaven.cli.create_provider_proxy", return_value=fake_proxy) as proxy,
                patch("runhaven.cli.threading.Thread", return_value=thread),
                patch("runhaven.cli.delete_container_network") as delete_network,
                patch("runhaven.cli.subprocess.call", return_value=9) as call,
            ):
                code = main(
                    [
                        "run",
                        "shell",
                        "--workspace",
                        directory,
                        "--network",
                        "provider",
                        "--provider-host",
                        "example.com",
                        "--env",
                        "HTTPS_PROXY",
                        "--tty",
                        "never",
                        "--",
                        "/bin/true",
                    ]
                )

        self.assertEqual(code, 9)
        self.assertEqual(preflight.call_count, 3)
        provider_network = preflight.call_args_list[-1].args[0][-1]
        inspect.assert_called_once_with(provider_network)
        proxy.assert_called_once()
        self.assertEqual(proxy.call_args.args[0].allowed_hosts, ("example.com",))
        self.assertEqual(proxy.call_args.args[1].ipv4_gateway, "192.168.130.1")
        thread.start.assert_called_once()
        fake_proxy.shutdown.assert_called_once()
        fake_proxy.server_close.assert_called_once()
        thread.join.assert_called_once()
        delete_network.assert_called_once_with(provider_network)
        command = call.call_args.args[0]
        self.assertIn("HTTPS_PROXY=http://192.168.130.1:49321", command)
        self.assertIn("HTTP_PROXY=http://192.168.130.1:49321", command)
        self.assertIn("ALL_PROXY=http://192.168.130.1:49321", command)
        https_proxy_values = [
            value for value in command if value == "HTTPS_PROXY" or value.startswith("HTTPS_PROXY=")
        ]
        self.assertEqual(https_proxy_values[-1], "HTTPS_PROXY=http://192.168.130.1:49321")
        self.assertEqual(command[-1], "/bin/true")

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

    def test_existing_internal_network_is_reused(self) -> None:
        with patch("runhaven.cli.subprocess.run") as run:
            run.return_value = Mock(
                returncode=0,
                stdout=json.dumps([{"configuration": {"mode": "hostOnly"}}]),
                stderr="",
            )

            ensure_internal_network("runhaven-project-internal")

        run.assert_called_once()
        self.assertEqual(
            run.call_args.args[0],
            ("container", "network", "inspect", "runhaven-project-internal"),
        )

    def test_existing_non_internal_network_is_rejected(self) -> None:
        with patch("runhaven.cli.subprocess.run") as run:
            run.return_value = Mock(
                returncode=0,
                stdout=json.dumps([{"configuration": {"mode": "nat"}}]),
                stderr="",
            )

            with self.assertRaisesRegex(ValueError, "not host-only"):
                ensure_internal_network("runhaven-project-internal")

    def test_missing_internal_network_is_created(self) -> None:
        with patch("runhaven.cli.subprocess.run") as run:
            run.side_effect = [Mock(returncode=1, stdout="", stderr=""), Mock(returncode=0)]

            ensure_internal_network("runhaven-project-internal")

        self.assertEqual(run.call_count, 2)
        self.assertEqual(
            run.call_args_list[1].args[0],
            ("container", "network", "create", "--internal", "runhaven-project-internal"),
        )

    def test_plan_tty_always_adds_tty_flag(self) -> None:
        with TemporaryDirectory() as directory:
            output = io.StringIO()
            with redirect_stdout(output):
                code = main(["plan", "shell", "--workspace", directory, "--tty", "always"])

        self.assertEqual(code, 0)
        self.assertIn("--tty", output.getvalue())

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
