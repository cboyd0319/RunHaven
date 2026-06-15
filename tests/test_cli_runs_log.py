from __future__ import annotations

import io
import json
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

from runhaven.cli import main


class CliRunsLogTests(unittest.TestCase):
    def test_runs_log_prints_joined_secret_free_run_events(self) -> None:
        with TemporaryDirectory() as directory:
            Path(directory, "runs.jsonl").write_text(
                json.dumps(
                    {
                        "timestamp": "2026-06-15T00:00:02Z",
                        "started_at": "2026-06-15T00:00:02Z",
                        "finished_at": "2026-06-15T00:00:03Z",
                        "run_id": "run-new",
                        "profile": "codex",
                        "workspace": directory,
                        "network": "provider",
                        "status": "failed",
                        "return_code": 1,
                        "provider_policy": {"entries": 2, "allowed": 1, "denied": 2},
                        "auth_broker": {
                            "broker": "codex-api-key",
                            "entries": 2,
                            "allowed": 1,
                            "denied": 1,
                            "no_requests": False,
                        },
                        "cleanup": {"provider_network": "deleted"},
                    }
                )
                + "\n"
            )
            Path(directory, "egress-policy.jsonl").write_text(
                "\n".join(
                    [
                        json.dumps(
                            {
                                "timestamp": "2026-06-15T00:00:01Z",
                                "run_id": "run-old",
                                "profile": "codex",
                                "workspace": directory,
                                "network": "provider",
                                "host": "old.example.com",
                                "port": 443,
                                "decision": "denied",
                                "reason": "not-in-allowlist",
                                "matched_rule": "",
                                "count": 1,
                            }
                        ),
                        json.dumps(
                            {
                                "timestamp": "2026-06-15T00:00:02Z",
                                "run_id": "run-new",
                                "profile": "codex",
                                "workspace": directory,
                                "network": "provider",
                                "host": "api.openai.com",
                                "port": 443,
                                "decision": "allowed",
                                "reason": "allowed",
                                "matched_rule": "api.openai.com",
                                "count": 1,
                            }
                        ),
                        json.dumps(
                            {
                                "timestamp": "2026-06-15T00:00:03Z",
                                "run_id": "run-new",
                                "profile": "codex",
                                "workspace": directory,
                                "network": "provider",
                                "host": "blocked.example.com",
                                "port": 443,
                                "decision": "denied",
                                "reason": "not-in-allowlist",
                                "matched_rule": "",
                                "count": 2,
                            }
                        ),
                    ]
                )
                + "\n"
            )
            Path(directory, "auth-broker.jsonl").write_text(
                "\n".join(
                    [
                        json.dumps(
                            {
                                "timestamp": "2026-06-15T00:00:01Z",
                                "run_id": "run-old",
                                "profile": "codex",
                                "workspace": directory,
                                "network": "provider",
                                "broker": "codex-api-key",
                                "method": "GET",
                                "path": "<unsupported>",
                                "decision": "denied",
                                "reason": "method-not-allowed",
                                "upstream_status": None,
                                "count": 1,
                                "return_code": 1,
                            }
                        ),
                        json.dumps(
                            {
                                "timestamp": "2026-06-15T00:00:02Z",
                                "run_id": "run-new",
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
                                "return_code": 1,
                            }
                        ),
                        json.dumps(
                            {
                                "timestamp": "2026-06-15T00:00:03Z",
                                "run_id": "run-new",
                                "profile": "codex",
                                "workspace": directory,
                                "network": "provider",
                                "broker": "codex-api-key",
                                "method": "GET",
                                "path": "<unsupported>",
                                "decision": "denied",
                                "reason": "method-not-allowed",
                                "upstream_status": None,
                                "count": 1,
                                "return_code": 1,
                            }
                        ),
                    ]
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
                code = main(["runs", "log", "run-new"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("Run id: run-new", text)
        self.assertIn("Provider policy decisions:", text)
        self.assertIn("api.openai.com:443", text)
        self.assertIn("blocked.example.com:443", text)
        self.assertIn("Auth broker decisions:", text)
        self.assertIn("POST /v1/responses", text)
        self.assertIn("GET <unsupported>", text)
        self.assertNotIn("old.example.com", text)
        self.assertNotIn("run-old", text)
        self.assertNotIn("fake-openai-api-key-value", text)
        self.assertNotIn("OPENAI_API_KEY", text)

    def test_runs_log_json_is_secret_free(self) -> None:
        with TemporaryDirectory() as directory:
            Path(directory, "runs.jsonl").write_text(
                json.dumps(
                    {
                        "timestamp": "2026-06-15T00:00:02Z",
                        "started_at": "2026-06-15T00:00:02Z",
                        "finished_at": "2026-06-15T00:00:03Z",
                        "run_id": "run-new",
                        "profile": "codex",
                        "workspace": directory,
                        "network": "provider",
                        "status": "failed",
                        "return_code": 1,
                        "provider_policy": {"entries": 1, "allowed": 0, "denied": 1},
                        "auth_broker": {
                            "broker": "codex-api-key",
                            "entries": 1,
                            "allowed": 0,
                            "denied": 1,
                            "no_requests": False,
                        },
                        "cleanup": {"provider_network": "deleted"},
                    }
                )
                + "\n"
            )
            Path(directory, "egress-policy.jsonl").write_text(
                json.dumps(
                    {
                        "timestamp": "2026-06-15T00:00:02Z",
                        "run_id": "run-new",
                        "profile": "codex",
                        "workspace": directory,
                        "network": "provider",
                        "host": "blocked.example.com",
                        "port": 443,
                        "decision": "denied",
                        "reason": "not-in-allowlist",
                        "matched_rule": "",
                        "count": 1,
                    }
                )
                + "\n"
            )
            Path(directory, "auth-broker.jsonl").write_text(
                json.dumps(
                    {
                        "timestamp": "2026-06-15T00:00:03Z",
                        "run_id": "run-new",
                        "profile": "codex",
                        "workspace": directory,
                        "network": "provider",
                        "broker": "codex-api-key",
                        "method": "GET",
                        "path": "<unsupported>",
                        "decision": "denied",
                        "reason": "method-not-allowed",
                        "upstream_status": None,
                        "count": 1,
                        "return_code": 1,
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
                code = main(["runs", "log", "run-new", "--json"])

        self.assertEqual(code, 0)
        payload = json.loads(output.getvalue())
        self.assertEqual(payload["run"]["run_id"], "run-new")
        self.assertEqual(payload["provider_policy"][0]["host"], "blocked.example.com")
        self.assertEqual(payload["auth_broker"][0]["reason"], "method-not-allowed")
        self.assertNotIn("fake-openai-api-key-value", output.getvalue())
        self.assertNotIn("OPENAI_API_KEY", output.getvalue())


if __name__ == "__main__":
    unittest.main()
