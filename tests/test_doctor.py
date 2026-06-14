from __future__ import annotations

import subprocess
import unittest
from unittest.mock import Mock, patch

from runhaven.doctor import (
    PINNED_APPLE_CONTAINER_VERSION,
    container_status_check,
    container_version_check,
    parse_container_version,
)


class DoctorTests(unittest.TestCase):
    def test_parse_container_version(self) -> None:
        self.assertEqual(
            parse_container_version("container CLI version 1.0.0 (build: release)"),
            "1.0.0",
        )

    def test_pinned_container_version_passes(self) -> None:
        output = f"container CLI version {PINNED_APPLE_CONTAINER_VERSION} (build: release)"
        with patch("runhaven.doctor.subprocess.run") as run:
            run.return_value = Mock(returncode=0, stdout=output, stderr="")

            check = container_version_check()

        self.assertTrue(check.ok)

    def test_unreviewed_container_version_fails(self) -> None:
        with patch("runhaven.doctor.subprocess.run") as run:
            run.return_value = Mock(returncode=0, stdout="container CLI version 1.0.1", stderr="")

            check = container_version_check()

        self.assertFalse(check.ok)
        self.assertIn(f"expected {PINNED_APPLE_CONTAINER_VERSION}", check.detail)

    def test_container_version_timeout_fails(self) -> None:
        with patch(
            "runhaven.doctor.subprocess.run",
            side_effect=subprocess.TimeoutExpired(["container", "--version"], 10),
        ):
            check = container_version_check()

        self.assertFalse(check.ok)
        self.assertIn("timed out", check.detail)

    def test_container_status_os_error_fails(self) -> None:
        with patch("runhaven.doctor.subprocess.run", side_effect=OSError("not available")):
            check = container_status_check()

        self.assertFalse(check.ok)
        self.assertIn("not available", check.detail)


if __name__ == "__main__":
    unittest.main()
