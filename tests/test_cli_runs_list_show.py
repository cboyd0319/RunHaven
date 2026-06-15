from __future__ import annotations

import io
import json
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import patch

from runhaven.cli import main


class CliRunsListShowTests(unittest.TestCase):
    def test_runs_list_prints_recent_records(self) -> None:
        with TemporaryDirectory() as directory:
            log_path = Path(directory) / "runs.jsonl"
            log_path.write_text(
                "\n".join(
                    [
                        json.dumps(
                            {
                                "timestamp": "2026-06-15T00:00:00Z",
                                "started_at": "2026-06-15T00:00:00Z",
                                "finished_at": "2026-06-15T00:00:01Z",
                                "run_id": "run-old",
                                "profile": "shell",
                                "workspace": directory,
                                "network": "internet",
                                "status": "succeeded",
                                "return_code": 0,
                                "provider_policy": {"entries": 0, "allowed": 0, "denied": 0},
                                "auth_broker": {
                                    "broker": None,
                                    "entries": 0,
                                    "allowed": 0,
                                    "denied": 0,
                                    "no_requests": False,
                                },
                                "cleanup": {"provider_network": "not-applicable"},
                            }
                        ),
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
                                "provider_policy": {"entries": 1, "allowed": 0, "denied": 2},
                                "auth_broker": {
                                    "broker": "codex-api-key",
                                    "entries": 1,
                                    "allowed": 0,
                                    "denied": 1,
                                    "no_requests": False,
                                },
                                "cleanup": {"provider_network": "deleted"},
                            }
                        ),
                    ]
                )
                + "\n"
            )
            output = io.StringIO()
            with patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False):
                with redirect_stdout(output):
                    code = main(["runs", "list", "--limit", "1"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("codex", text)
        self.assertIn("provider", text)
        self.assertIn("failed", text)
        self.assertIn("provider_denied=2", text)
        self.assertIn("auth_denied=1", text)
        self.assertIn("cleanup=deleted", text)
        self.assertIn("run=run-new", text)
        self.assertNotIn("run-old", text)

    def test_runs_show_json_is_secret_free(self) -> None:
        with TemporaryDirectory() as directory:
            log_path = Path(directory) / "runs.jsonl"
            log_path.write_text(
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
                        "provider_policy": {"entries": 1, "allowed": 0, "denied": 2},
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
                code = main(["runs", "show", "run-new", "--json"])

        self.assertEqual(code, 0)
        payload = json.loads(output.getvalue())
        self.assertEqual(payload["run_id"], "run-new")
        self.assertEqual(payload["auth_broker"]["broker"], "codex-api-key")
        self.assertNotIn("fake-openai-api-key-value", output.getvalue())
        self.assertNotIn("OPENAI_API_KEY", output.getvalue())

    def test_runs_show_prints_git_metadata_summary(self) -> None:
        with TemporaryDirectory() as directory:
            log_path = Path(directory) / "runs.jsonl"
            log_path.write_text(
                json.dumps(
                    {
                        "timestamp": "2026-06-15T00:00:02Z",
                        "started_at": "2026-06-15T00:00:02Z",
                        "finished_at": "2026-06-15T00:00:03Z",
                        "run_id": "run-new",
                        "profile": "shell",
                        "workspace": directory,
                        "network": "internet",
                        "status": "succeeded",
                        "return_code": 0,
                        "provider_policy": {"entries": 0, "allowed": 0, "denied": 0},
                        "auth_broker": {
                            "broker": None,
                            "entries": 0,
                            "allowed": 0,
                            "denied": 0,
                            "no_requests": False,
                        },
                        "cleanup": {"provider_network": "not-applicable"},
                        "git": {
                            "available": True,
                            "repo_root": directory,
                            "changed": True,
                            "before": {
                                "head": "1234567890abcdef",
                                "dirty": False,
                                "changed_count": 0,
                                "paths": [],
                                "truncated": False,
                            },
                            "after": {
                                "head": "abcdef1234567890",
                                "dirty": True,
                                "changed_count": 2,
                                "paths": ["created.txt", "tracked.txt"],
                                "truncated": False,
                            },
                        },
                    }
                )
                + "\n"
            )
            output = io.StringIO()
            with patch.dict("os.environ", {"RUNHAVEN_CACHE_HOME": directory}, clear=False):
                with redirect_stdout(output):
                    code = main(["runs", "show", "run-new"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("Git: changed=true", text)
        self.assertIn("before=1234567", text)
        self.assertIn("after=abcdef1", text)
        self.assertIn("files=2", text)


if __name__ == "__main__":
    unittest.main()
