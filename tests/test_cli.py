from __future__ import annotations

import io
import json
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import Mock, patch

from runhaven.auth_broker import (
    CODEX_BROKER_PLACEHOLDER_ENV,
    CODEX_BROKER_PLACEHOLDER_VALUE,
    CODEX_BROKER_PROVIDER_ID,
    BrokerDecision,
)
from runhaven.cli import (
    acquire_state_lock,
    ensure_internal_network,
    main,
    state_lock_path,
)
from runhaven.doctor import Check
from runhaven.egress import ProxyDecision


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
        self.assertIn("chatgpt.com", text)
        self.assertIn("RunHaven injects proxy environment variables at runtime", text)

    def test_provider_run_injects_proxy_environment_and_cleans_network(self) -> None:
        with TemporaryDirectory() as directory:
            fake_proxy = Mock()
            fake_proxy.server_address = ("0.0.0.0", 49321)
            fake_proxy.denied_connect_targets.return_value = ()
            fake_proxy.policy_decisions.return_value = (
                ProxyDecision(
                    host="api.example.com",
                    port=443,
                    decision="allowed",
                    reason="allowed",
                    matched_rule="example.com",
                    count=2,
                ),
            )
            thread = Mock()
            network_info = Mock(ipv4_gateway="192.168.130.1", ipv4_subnet="192.168.130.0/24")
            error_output = io.StringIO()
            with (
                redirect_stderr(error_output),
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

    def test_provider_run_prints_blocked_host_summary(self) -> None:
        with TemporaryDirectory() as directory:
            fake_proxy = Mock()
            fake_proxy.server_address = ("0.0.0.0", 49321)
            fake_proxy.denied_connect_targets.return_value = (
                ("blocked.example.com", 443),
                ("1.1.1.1", 443),
            )
            fake_proxy.policy_decisions.return_value = (
                ProxyDecision(
                    host="blocked.example.com",
                    port=443,
                    decision="denied",
                    reason="not-in-allowlist",
                    matched_rule="",
                    count=3,
                ),
                ProxyDecision(
                    host="1.1.1.1",
                    port=443,
                    decision="denied",
                    reason="ip-literal",
                    matched_rule="",
                    count=1,
                ),
            )
            thread = Mock()
            network_info = Mock(ipv4_gateway="192.168.130.1", ipv4_subnet="192.168.130.0/24")
            error_output = io.StringIO()
            with (
                redirect_stderr(error_output),
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.run_preflight"),
                patch("runhaven.cli.inspect_internal_network", return_value=network_info),
                patch("runhaven.cli.create_provider_proxy", return_value=fake_proxy),
                patch("runhaven.cli.threading.Thread", return_value=thread),
                patch("runhaven.cli.delete_container_network"),
                patch("runhaven.cli.subprocess.call", return_value=0),
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
                        "--tty",
                        "never",
                        "--",
                        "/bin/true",
                    ]
                )

        self.assertEqual(code, 0)
        text = error_output.getvalue()
        self.assertIn("RunHaven provider proxy blocked", text)
        self.assertIn("4 CONNECT requests across 2 target(s)", text)
        self.assertIn("Run id:", text)
        self.assertIn("blocked.example.com:443", text)
        self.assertIn("count=3", text)
        self.assertIn("reason=not-in-allowlist", text)
        self.assertIn("runhaven why host blocked.example.com --agent shell", text)
        self.assertIn("1.1.1.1:443", text)
        self.assertIn("IP literal targets cannot be allowed", text)
        self.assertIn("runhaven egress log --limit 20", text)

    def test_provider_run_writes_policy_log(self) -> None:
        with TemporaryDirectory() as directory:
            fake_proxy = Mock()
            fake_proxy.server_address = ("0.0.0.0", 49321)
            fake_proxy.denied_connect_targets.return_value = ()
            fake_proxy.policy_decisions.return_value = (
                ProxyDecision(
                    host="api.example.com",
                    port=443,
                    decision="allowed",
                    reason="allowed",
                    matched_rule="example.com",
                    count=2,
                ),
                ProxyDecision(
                    host="blocked.example.com",
                    port=443,
                    decision="denied",
                    reason="not-in-allowlist",
                    matched_rule="",
                    count=1,
                ),
            )
            thread = Mock()
            network_info = Mock(ipv4_gateway="192.168.130.1", ipv4_subnet="192.168.130.0/24")
            error_output = io.StringIO()
            with (
                redirect_stderr(error_output),
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.run_preflight"),
                patch("runhaven.cli.inspect_internal_network", return_value=network_info),
                patch("runhaven.cli.create_provider_proxy", return_value=fake_proxy),
                patch("runhaven.cli.threading.Thread", return_value=thread),
                patch("runhaven.cli.delete_container_network"),
                patch("runhaven.cli.subprocess.call", return_value=0),
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
                        "--tty",
                        "never",
                        "--",
                        "/bin/true",
                    ]
                )

            self.assertEqual(code, 0)
            entries = [
                json.loads(line)
                for line in (Path(directory) / "egress-policy.jsonl").read_text().splitlines()
            ]
            self.assertEqual(len(entries), 2)
            self.assertEqual(entries[0]["run_id"], entries[1]["run_id"])
            self.assertEqual(len(entries[0]["run_id"]), 32)
            self.assertEqual(entries[0]["decision"], "allowed")
            self.assertEqual(entries[0]["host"], "api.example.com")
            self.assertEqual(entries[0]["count"], 2)
            self.assertEqual(entries[1]["decision"], "denied")
            self.assertEqual(entries[1]["reason"], "not-in-allowlist")

    def test_provider_run_with_codex_api_key_broker_injects_secret_free_config(self) -> None:
        with TemporaryDirectory() as directory:
            fake_proxy = Mock()
            fake_proxy.server_address = ("0.0.0.0", 49321)
            fake_proxy.policy_decisions.return_value = ()
            fake_broker = Mock()
            fake_broker.server_address = ("0.0.0.0", 48123)
            fake_broker.broker_decisions.return_value = ()
            thread = Mock()
            network_info = Mock(ipv4_gateway="192.168.130.1", ipv4_subnet="192.168.130.0/24")
            with (
                patch.dict(
                    "os.environ",
                    {
                        "OPENAI_API_KEY": "fake-openai-api-key-value",
                        "RUNHAVEN_CACHE_HOME": directory,
                    },
                    clear=True,
                ),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.run_preflight"),
                patch("runhaven.cli.inspect_internal_network", return_value=network_info),
                patch("runhaven.cli.create_provider_proxy", return_value=fake_proxy),
                patch(
                    "runhaven.cli.create_codex_api_key_broker",
                    return_value=fake_broker,
                ) as broker,
                patch("runhaven.cli.threading.Thread", return_value=thread),
                patch("runhaven.cli.delete_container_network"),
                patch("runhaven.cli.subprocess.call", return_value=0) as call,
            ):
                code = main(
                    [
                        "run",
                        "codex",
                        "--workspace",
                        directory,
                        "--network",
                        "provider",
                        "--codex-api-key-broker-env",
                        "OPENAI_API_KEY",
                        "--tty",
                        "never",
                    ]
                )

        self.assertEqual(code, 0)
        broker.assert_called_once_with("fake-openai-api-key-value", network_info)
        self.assertEqual(thread.start.call_count, 2)
        fake_proxy.shutdown.assert_called_once()
        fake_proxy.server_close.assert_called_once()
        fake_broker.shutdown.assert_called_once()
        fake_broker.server_close.assert_called_once()
        command = call.call_args.args[0]
        joined = " ".join(command)
        self.assertIn(
            f"{CODEX_BROKER_PLACEHOLDER_ENV}={CODEX_BROKER_PLACEHOLDER_VALUE}",
            command,
        )
        self.assertIn(f'model_provider="{CODEX_BROKER_PROVIDER_ID}"', command)
        self.assertIn(
            f'model_providers.{CODEX_BROKER_PROVIDER_ID}.base_url='
            '"http://192.168.130.1:48123/v1"',
            command,
        )
        self.assertIn(
            f'model_providers.{CODEX_BROKER_PROVIDER_ID}.env_key='
            f'"{CODEX_BROKER_PLACEHOLDER_ENV}"',
            command,
        )
        self.assertIn("NO_PROXY=localhost,127.0.0.1,::1,192.168.130.1", command)
        self.assertNotIn("fake-openai-api-key-value", joined)
        self.assertNotIn("OPENAI_API_KEY", joined)

    def test_provider_run_with_codex_api_key_broker_writes_secret_free_auth_log(self) -> None:
        with TemporaryDirectory() as directory:
            fake_proxy = Mock()
            fake_proxy.server_address = ("0.0.0.0", 49321)
            fake_proxy.policy_decisions.return_value = ()
            fake_broker = Mock()
            fake_broker.server_address = ("0.0.0.0", 48123)
            fake_broker.broker_decisions.return_value = (
                BrokerDecision(
                    method="POST",
                    path="/v1/responses",
                    decision="allowed",
                    reason="upstream-response",
                    upstream_status=200,
                    count=1,
                ),
            )
            thread = Mock()
            network_info = Mock(ipv4_gateway="192.168.130.1", ipv4_subnet="192.168.130.0/24")
            with (
                patch.dict(
                    "os.environ",
                    {
                        "OPENAI_API_KEY": "fake-openai-api-key-value",
                        "RUNHAVEN_CACHE_HOME": directory,
                    },
                    clear=True,
                ),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.run_preflight"),
                patch("runhaven.cli.inspect_internal_network", return_value=network_info),
                patch("runhaven.cli.create_provider_proxy", return_value=fake_proxy),
                patch("runhaven.cli.create_codex_api_key_broker", return_value=fake_broker),
                patch("runhaven.cli.threading.Thread", return_value=thread),
                patch("runhaven.cli.delete_container_network"),
                patch("runhaven.cli.subprocess.call", return_value=0),
            ):
                code = main(
                    [
                        "run",
                        "codex",
                        "--workspace",
                        directory,
                        "--network",
                        "provider",
                        "--codex-api-key-broker-env",
                        "OPENAI_API_KEY",
                        "--tty",
                        "never",
                    ]
                )

            self.assertEqual(code, 0)
            entries = [
                json.loads(line)
                for line in (Path(directory) / "auth-broker.jsonl").read_text().splitlines()
            ]

        self.assertEqual(len(entries), 1)
        self.assertEqual(entries[0]["broker"], "codex-api-key")
        self.assertEqual(entries[0]["profile"], "codex")
        self.assertEqual(entries[0]["method"], "POST")
        self.assertEqual(entries[0]["path"], "/v1/responses")
        self.assertEqual(entries[0]["decision"], "allowed")
        self.assertEqual(entries[0]["reason"], "upstream-response")
        self.assertEqual(entries[0]["upstream_status"], 200)
        self.assertEqual(entries[0]["count"], 1)
        self.assertNotIn("fake-openai-api-key-value", json.dumps(entries))
        self.assertNotIn("OPENAI_API_KEY", json.dumps(entries))

    def test_provider_run_with_codex_api_key_broker_logs_no_requests(self) -> None:
        with TemporaryDirectory() as directory:
            fake_proxy = Mock()
            fake_proxy.server_address = ("0.0.0.0", 49321)
            fake_proxy.policy_decisions.return_value = ()
            fake_broker = Mock()
            fake_broker.server_address = ("0.0.0.0", 48123)
            fake_broker.broker_decisions.return_value = ()
            thread = Mock()
            network_info = Mock(ipv4_gateway="192.168.130.1", ipv4_subnet="192.168.130.0/24")
            with (
                patch.dict(
                    "os.environ",
                    {
                        "OPENAI_API_KEY": "fake-openai-api-key-value",
                        "RUNHAVEN_CACHE_HOME": directory,
                    },
                    clear=True,
                ),
                patch("runhaven.cli.require_container_cli"),
                patch("runhaven.cli.run_preflight"),
                patch("runhaven.cli.inspect_internal_network", return_value=network_info),
                patch("runhaven.cli.create_provider_proxy", return_value=fake_proxy),
                patch("runhaven.cli.create_codex_api_key_broker", return_value=fake_broker),
                patch("runhaven.cli.threading.Thread", return_value=thread),
                patch("runhaven.cli.delete_container_network"),
                patch("runhaven.cli.subprocess.call", return_value=0),
            ):
                code = main(
                    [
                        "run",
                        "codex",
                        "--workspace",
                        directory,
                        "--network",
                        "provider",
                        "--codex-api-key-broker-env",
                        "OPENAI_API_KEY",
                        "--tty",
                        "never",
                    ]
                )

            self.assertEqual(code, 0)
            entries = [
                json.loads(line)
                for line in (Path(directory) / "auth-broker.jsonl").read_text().splitlines()
            ]

        self.assertEqual(len(entries), 1)
        self.assertEqual(entries[0]["decision"], "no-requests")
        self.assertEqual(entries[0]["count"], 0)

    def test_provider_run_with_codex_api_key_broker_requires_host_env_first(self) -> None:
        with TemporaryDirectory() as directory:
            error_output = io.StringIO()
            with (
                redirect_stderr(error_output),
                patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=True),
                patch("runhaven.cli.require_container_cli") as require_container,
                self.assertRaises(SystemExit) as error,
            ):
                main(
                    [
                        "run",
                        "codex",
                        "--workspace",
                        directory,
                        "--network",
                        "provider",
                        "--codex-api-key-broker-env",
                        "OPENAI_API_KEY",
                        "--tty",
                        "never",
                    ]
                )

        self.assertEqual(error.exception.code, 2)
        require_container.assert_not_called()
        self.assertIn("OPENAI_API_KEY is not set", error_output.getvalue())

    def test_egress_log_prints_recent_policy_entries(self) -> None:
        with TemporaryDirectory() as directory:
            log_path = Path(directory) / "egress-policy.jsonl"
            log_path.write_text(
                "\n".join(
                    [
                        json.dumps(
                            {
                                "timestamp": "2026-06-15T00:00:00Z",
                                "profile": "shell",
                                "workspace": directory,
                                "run_id": "run-allowed",
                                "network": "provider",
                                "host": "api.example.com",
                                "port": 443,
                                "decision": "allowed",
                                "reason": "allowed",
                                "matched_rule": "example.com",
                                "count": 2,
                            }
                        ),
                        json.dumps(
                            {
                                "timestamp": "2026-06-15T00:00:01Z",
                                "profile": "shell",
                                "workspace": directory,
                                "run_id": "run-denied",
                                "network": "provider",
                                "host": "blocked.example.com",
                                "port": 443,
                                "decision": "denied",
                                "reason": "not-in-allowlist",
                                "matched_rule": "",
                                "count": 1,
                            }
                        ),
                    ]
                )
                + "\n"
            )
            output = io.StringIO()
            with patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False):
                with redirect_stdout(output):
                    code = main(["egress", "log", "--limit", "1"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("blocked.example.com:443", text)
        self.assertIn("denied", text)
        self.assertIn("run=run-denied", text)
        self.assertNotIn("api.example.com", text)

    def test_auth_log_prints_recent_broker_entries(self) -> None:
        with TemporaryDirectory() as directory:
            log_path = Path(directory) / "auth-broker.jsonl"
            log_path.write_text(
                "\n".join(
                    [
                        json.dumps(
                            {
                                "timestamp": "2026-06-15T00:00:00Z",
                                "run_id": "run-old",
                                "profile": "codex",
                                "workspace": directory,
                                "network": "provider",
                                "broker": "codex-api-key",
                                "method": "POST",
                                "path": "/v1/responses",
                                "decision": "allowed",
                                "reason": "upstream-response",
                                "upstream_status": 200,
                                "count": 1,
                            }
                        ),
                        json.dumps(
                            {
                                "timestamp": "2026-06-15T00:00:01Z",
                                "run_id": "run-new",
                                "profile": "codex",
                                "workspace": directory,
                                "network": "provider",
                                "broker": "codex-api-key",
                                "method": "-",
                                "path": "-",
                                "decision": "no-requests",
                                "reason": "run-complete",
                                "upstream_status": None,
                                "count": 0,
                            }
                        ),
                    ]
                )
                + "\n"
            )
            output = io.StringIO()
            with patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False):
                with redirect_stdout(output):
                    code = main(["auth", "log", "--limit", "1"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("codex-api-key", text)
        self.assertIn("no-requests", text)
        self.assertIn("run=run-new", text)
        self.assertNotIn("run-old", text)

    def test_auth_log_json_is_secret_free(self) -> None:
        with TemporaryDirectory() as directory:
            log_path = Path(directory) / "auth-broker.jsonl"
            log_path.write_text(
                json.dumps(
                    {
                        "timestamp": "2026-06-15T00:00:00Z",
                        "run_id": "run-allowed",
                        "profile": "codex",
                        "workspace": directory,
                        "network": "provider",
                        "broker": "codex-api-key",
                        "method": "POST",
                        "path": "/v1/responses",
                        "decision": "allowed",
                        "reason": "upstream-response",
                        "upstream_status": 200,
                        "count": 1,
                    }
                )
                + "\n"
            )
            output = io.StringIO()
            with (
                patch.dict(
                    "os.environ",
                    {
                        "RUNHAVEN_CACHE_HOME": directory,
                        "OPENAI_API_KEY": "fake-openai-api-key-value",
                    },
                    clear=True,
                ),
                redirect_stdout(output),
            ):
                code = main(["auth", "log", "--json"])

        self.assertEqual(code, 0)
        self.assertIn('"broker": "codex-api-key"', output.getvalue())
        self.assertNotIn("fake-openai-api-key-value", output.getvalue())
        self.assertNotIn("OPENAI_API_KEY", output.getvalue())

    def test_auth_status_does_not_print_secret_values(self) -> None:
        output = io.StringIO()
        with (
            patch.dict(
                "os.environ",
                {
                    "OPENAI_API_KEY": "fake-openai-api-key-value",
                    "ANTHROPIC_API_KEY": "fake-anthropic-api-key-value",
                },
                clear=False,
            ),
            redirect_stdout(output),
        ):
            code = main(["auth", "status"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("Auth broker: codex-api-key-prototype", text)
        self.assertIn("Credential stores inspected: no", text)
        self.assertIn("Environment values inspected: no", text)
        self.assertIn("Secrets printed: no", text)
        for profile in ("antigravity", "claude", "codex", "copilot", "gemini", "shell"):
            self.assertIn(profile, text)
        self.assertIn("api-key-prototype", text)
        self.assertNotIn("fake-openai-api-key-value", text)
        self.assertNotIn("fake-anthropic-api-key-value", text)

    def test_auth_explain_prints_profile_boundary(self) -> None:
        output = io.StringIO()
        with redirect_stdout(output):
            code = main(["auth", "explain", "codex"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("Profile: codex", text)
        self.assertIn("Auth broker: api-key-prototype", text)
        self.assertIn("OpenAI API key through --codex-api-key-broker-env NAME", text)
        self.assertIn("RUNHAVEN_CODEX_BROKER_TOKEN", text)
        self.assertIn("Provider hosts: api.openai.com, chatgpt.com", text)
        self.assertIn("headless API-key run", text)

    def test_auth_explain_json_is_static_and_secret_free(self) -> None:
        output = io.StringIO()
        with (
            patch.dict(
                "os.environ",
                {"OPENAI_API_KEY": "fake-openai-api-key-value"},
                clear=False,
            ),
            redirect_stdout(output),
        ):
            code = main(["auth", "explain", "codex", "--json"])

        self.assertEqual(code, 0)
        payload = json.loads(output.getvalue())
        self.assertEqual(payload["name"], "codex")
        self.assertFalse(payload["credential_stores_inspected"])
        self.assertFalse(payload["environment_values_inspected"])
        self.assertFalse(payload["secrets_printed"])
        self.assertIn("api.openai.com", payload["provider_hosts"])
        self.assertNotIn("fake-openai-api-key-value", output.getvalue())

    def test_why_host_explains_ip_literal_rejection(self) -> None:
        output = io.StringIO()
        with redirect_stdout(output):
            code = main(["why", "host", "1.1.1.1"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("Host: 1.1.1.1", text)
        self.assertIn("IP literal", text)
        self.assertIn("cannot be allowed", text)

    def test_why_host_explains_profile_allowlist_match(self) -> None:
        output = io.StringIO()
        with redirect_stdout(output):
            code = main(["why", "host", "api.openai.com", "--agent", "codex"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("Host: api.openai.com", text)
        self.assertIn("Provider profile: codex", text)
        self.assertIn("allowed", text)
        self.assertIn("api.openai.com", text)

    def test_why_host_explains_known_unbundled_endpoint(self) -> None:
        output = io.StringIO()
        with redirect_stdout(output):
            code = main(["why", "host", "api.github.com", "--agent", "copilot"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("Provider profile: copilot", text)
        self.assertIn("not allowed by bundled provider profile", text)
        self.assertIn("Known endpoint record", text)
        self.assertIn("candidate", text)
        self.assertIn("specific API paths", text)

    def test_why_host_allows_copilot_subscription_routing(self) -> None:
        output = io.StringIO()
        with redirect_stdout(output):
            code = main(["why", "host", "api.business.githubcopilot.com", "--agent", "copilot"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("Provider profile: copilot", text)
        self.assertIn("allowed by bundled provider profile", text)
        self.assertIn("business.githubcopilot.com", text)

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
