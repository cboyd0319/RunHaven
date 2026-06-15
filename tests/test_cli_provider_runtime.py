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
from runhaven.cli import ensure_internal_network, main
from runhaven.egress import ProxyDecision


class CliProviderRuntimeTests(unittest.TestCase):
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
            f'model_providers.{CODEX_BROKER_PROVIDER_ID}.base_url="http://192.168.130.1:48123/v1"',
            command,
        )
        self.assertIn(
            f'model_providers.{CODEX_BROKER_PROVIDER_ID}.env_key="{CODEX_BROKER_PLACEHOLDER_ENV}"',
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

    def test_provider_run_writes_run_record_with_policy_auth_and_cleanup_summary(self) -> None:
        with TemporaryDirectory() as directory:
            fake_proxy = Mock()
            fake_proxy.server_address = ("0.0.0.0", 49321)
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
                    count=3,
                ),
            )
            fake_broker = Mock()
            fake_broker.server_address = ("0.0.0.0", 48123)
            fake_broker.broker_decisions.return_value = (
                BrokerDecision(
                    method="POST",
                    path="/v1/responses",
                    decision="allowed",
                    reason="upstream-response",
                    upstream_status=200,
                    count=2,
                ),
                BrokerDecision(
                    method="GET",
                    path="<unsupported>",
                    decision="denied",
                    reason="method-not-allowed",
                    upstream_status=None,
                    count=1,
                ),
            )
            thread = Mock()
            network_info = Mock(ipv4_gateway="192.168.130.1", ipv4_subnet="192.168.130.0/24")
            error_output = io.StringIO()
            with (
                redirect_stderr(error_output),
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
                patch("runhaven.cli.delete_container_network", return_value=0),
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
            records = [
                json.loads(line)
                for line in (Path(directory) / "runs.jsonl").read_text().splitlines()
            ]
            egress_entries = [
                json.loads(line)
                for line in (Path(directory) / "egress-policy.jsonl").read_text().splitlines()
            ]
            auth_entries = [
                json.loads(line)
                for line in (Path(directory) / "auth-broker.jsonl").read_text().splitlines()
            ]

        self.assertEqual(len(records), 1)
        record = records[0]
        self.assertEqual(record["profile"], "codex")
        self.assertEqual(record["status"], "succeeded")
        self.assertEqual(record["return_code"], 0)
        self.assertEqual(record["provider_policy"]["entries"], 2)
        self.assertEqual(record["provider_policy"]["allowed"], 2)
        self.assertEqual(record["provider_policy"]["denied"], 3)
        self.assertEqual(record["auth_broker"]["broker"], "codex-api-key")
        self.assertEqual(record["auth_broker"]["entries"], 2)
        self.assertEqual(record["auth_broker"]["allowed"], 2)
        self.assertEqual(record["auth_broker"]["denied"], 1)
        self.assertFalse(record["auth_broker"]["no_requests"])
        self.assertEqual(record["cleanup"]["provider_network"], "deleted")
        self.assertEqual(record["run_id"], egress_entries[0]["run_id"])
        self.assertEqual(record["run_id"], auth_entries[0]["run_id"])
        self.assertNotIn("fake-openai-api-key-value", json.dumps(records))
        self.assertNotIn("OPENAI_API_KEY", json.dumps(records))

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


if __name__ == "__main__":
    unittest.main()
