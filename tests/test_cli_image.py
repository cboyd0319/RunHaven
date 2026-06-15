from __future__ import annotations

import io
import unittest
from contextlib import redirect_stdout
from unittest.mock import Mock, patch

from runhaven.cli import main


class CliImageTests(unittest.TestCase):
    def test_image_doctor_reports_missing_bundled_image(self) -> None:
        output = io.StringIO()
        with (
            redirect_stdout(output),
            patch("runhaven.cli.require_container_cli"),
            patch("runhaven.cli.subprocess.run") as run,
        ):
            run.return_value = Mock(
                returncode=0,
                stdout='[{"configuration": {"name": "runhaven/base:0.1.0"}}]',
                stderr="",
            )

            code = main(["image", "doctor", "claude"])

        self.assertEqual(code, 1)
        text = output.getvalue()
        self.assertIn("missing claude", text)
        self.assertIn("runhaven/claude:0.1.0", text)
        self.assertIn("runhaven image rebuild claude", text)
        self.assertIn("Preflight recovery", text)
        self.assertIn("runhaven network list", text)
        self.assertIn("runhaven network prune", text)
        self.assertIn("runhaven state reset claude --workspace PATH --yes", text)
        run.assert_called_once_with(
            ("container", "image", "list", "--format", "json"),
            check=False,
            capture_output=True,
            text=True,
        )

    def test_image_doctor_reports_present_image(self) -> None:
        output = io.StringIO()
        with (
            redirect_stdout(output),
            patch("runhaven.cli.require_container_cli"),
            patch("runhaven.cli.subprocess.run") as run,
        ):
            run.return_value = Mock(
                returncode=0,
                stdout='[{"configuration": {"name": "docker.io/runhaven/base:0.1.0"}}]',
                stderr="",
            )

            code = main(["image", "doctor", "shell"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("ok shell", text)
        self.assertIn("runhaven/base:0.1.0", text)
        self.assertNotIn("missing shell", text)

    def test_image_doctor_checks_all_profiles_by_default(self) -> None:
        output = io.StringIO()
        with (
            redirect_stdout(output),
            patch("runhaven.cli.require_container_cli"),
            patch("runhaven.cli.subprocess.run") as run,
        ):
            run.return_value = Mock(
                returncode=0,
                stdout='[{"configuration": {"name": "runhaven/base:0.1.0"}}]',
                stderr="",
            )

            code = main(["image", "doctor"])

        self.assertEqual(code, 1)
        text = output.getvalue()
        self.assertIn("ok shell", text)
        self.assertIn("missing claude", text)
        self.assertIn("missing codex", text)
        self.assertIn("runhaven image rebuild claude", text)

    def test_image_doctor_rejects_invalid_image_list_json(self) -> None:
        with (
            patch("runhaven.cli.require_container_cli"),
            patch("runhaven.cli.subprocess.run") as run,
        ):
            run.return_value = Mock(returncode=0, stdout="not json", stderr="")

            with self.assertRaises(SystemExit) as error:
                main(["image", "doctor", "shell"])

        self.assertEqual(error.exception.code, 2)
        run.assert_called_once_with(
            ("container", "image", "list", "--format", "json"),
            check=False,
            capture_output=True,
            text=True,
        )


if __name__ == "__main__":
    unittest.main()
