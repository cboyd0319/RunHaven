from __future__ import annotations

import io
import subprocess
import unittest
from contextlib import redirect_stdout
from tempfile import TemporaryDirectory
from unittest.mock import patch

from scripts import codex_broker_smoke


class CodexBrokerSmokeTests(unittest.TestCase):
    def test_missing_api_key_env_skips_without_running_command(self) -> None:
        output = io.StringIO()
        with (
            patch.dict("os.environ", {}, clear=True),
            patch("scripts.codex_broker_smoke.subprocess.run") as run,
            redirect_stdout(output),
        ):
            code = codex_broker_smoke.main(["--api-key-env", "RUNHAVEN_SMOKE_KEY"])

        self.assertEqual(code, 0)
        run.assert_not_called()
        self.assertIn("SKIP", output.getvalue())
        self.assertIn("RUNHAVEN_SMOKE_KEY", output.getvalue())

    def test_smoke_command_uses_env_name_without_key_value(self) -> None:
        commands: list[tuple[str, ...]] = []

        def fake_run(
            command: tuple[str, ...],
            *,
            check: bool,
            capture_output: bool,
            text: bool,
            timeout: int,
            env: dict[str, str],
        ) -> subprocess.CompletedProcess[str]:
            commands.append(command)
            self.assertFalse(check)
            self.assertTrue(capture_output)
            self.assertTrue(text)
            self.assertIn("PYTHONPATH", env)
            return subprocess.CompletedProcess(command, 0, "RUNHAVEN_BROKER_SMOKE_OK\n", "")

        with TemporaryDirectory() as directory:
            output = io.StringIO()
            with (
                patch.dict(
                    "os.environ",
                    {"RUNHAVEN_SMOKE_KEY": "fake-openai-api-key-value"},
                    clear=True,
                ),
                patch("scripts.codex_broker_smoke.subprocess.run", side_effect=fake_run),
                redirect_stdout(output),
            ):
                code = codex_broker_smoke.main(
                    [
                        "--api-key-env",
                        "RUNHAVEN_SMOKE_KEY",
                        "--workspace",
                        directory,
                        "--timeout",
                        "5",
                    ]
                )

        self.assertEqual(code, 0)
        self.assertEqual(len(commands), 1)
        command = commands[0]
        joined = " ".join(command)
        self.assertIn("runhaven", joined)
        self.assertIn("run", command)
        self.assertIn("codex", command)
        self.assertIn("--network", command)
        self.assertIn("provider", command)
        self.assertIn("--codex-api-key-broker-env", command)
        self.assertIn("RUNHAVEN_SMOKE_KEY", command)
        self.assertIn("exec", command)
        self.assertIn("--skip-git-repo-check", command)
        self.assertIn("--sandbox", command)
        self.assertIn("read-only", command)
        self.assertNotIn("fake-openai-api-key-value", joined)
        self.assertIn("PASS", output.getvalue())

    def test_output_marker_is_required(self) -> None:
        def fake_run(
            command: tuple[str, ...],
            *,
            check: bool,
            capture_output: bool,
            text: bool,
            timeout: int,
            env: dict[str, str],
        ) -> subprocess.CompletedProcess[str]:
            return subprocess.CompletedProcess(command, 0, "different output\n", "")

        with TemporaryDirectory() as directory:
            with (
                patch.dict("os.environ", {"RUNHAVEN_SMOKE_KEY": "fake-key"}, clear=True),
                patch("scripts.codex_broker_smoke.subprocess.run", side_effect=fake_run),
            ):
                with self.assertRaisesRegex(
                    codex_broker_smoke.SmokeFailure,
                    "expected marker",
                ):
                    codex_broker_smoke.run_smoke(
                        codex_broker_smoke.build_parser().parse_args(
                            [
                                "--api-key-env",
                                "RUNHAVEN_SMOKE_KEY",
                                "--workspace",
                                directory,
                            ]
                        )
                    )


if __name__ == "__main__":
    unittest.main()
