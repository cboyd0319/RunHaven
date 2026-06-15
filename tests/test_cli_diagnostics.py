from __future__ import annotations

import io
import json
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

from runhaven.cli import main


class CliDiagnosticTests(unittest.TestCase):
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


if __name__ == "__main__":
    unittest.main()
