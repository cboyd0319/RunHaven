from __future__ import annotations

import io
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest.mock import Mock, patch

from macos_container_agents.cli import (
    acquire_state_lock,
    ensure_internal_network,
    main,
    state_lock_path,
)


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

    def test_image_build_dry_run_uses_bundled_containerfile(self) -> None:
        output = io.StringIO()
        with redirect_stdout(output):
            code = main(["image", "build", "shell", "--dry-run"])

        self.assertEqual(code, 0)
        text = output.getvalue()
        self.assertIn("container build", text)
        self.assertIn("Containerfile", text)
        self.assertIn("mca/base:0.1.0", text)

    def test_missing_workspace_is_user_error(self) -> None:
        with TemporaryDirectory() as directory:
            missing = Path(directory) / "missing"
            with self.assertRaises(SystemExit) as error:
                main(["plan", "shell", "--workspace", str(missing)])

        self.assertEqual(error.exception.code, 2)

    def test_existing_internal_network_is_reused(self) -> None:
        with patch("macos_container_agents.cli.subprocess.run") as run:
            run.return_value = Mock(returncode=0)

            ensure_internal_network("mca-project-internal")

        run.assert_called_once()
        self.assertEqual(
            run.call_args.args[0],
            ("container", "network", "inspect", "mca-project-internal"),
        )

    def test_missing_internal_network_is_created(self) -> None:
        with patch("macos_container_agents.cli.subprocess.run") as run:
            run.side_effect = [Mock(returncode=1), Mock(returncode=0)]

            ensure_internal_network("mca-project-internal")

        self.assertEqual(run.call_count, 2)
        self.assertEqual(
            run.call_args_list[1].args[0],
            ("container", "network", "create", "--internal", "mca-project-internal"),
        )

    def test_state_lock_rejects_concurrent_same_volume(self) -> None:
        with TemporaryDirectory() as directory:
            with patch.dict("os.environ", {"MCA_CACHE_HOME": directory}, clear=False):
                lock_path = state_lock_path("mca-test-home")

                with acquire_state_lock("mca-test-home"):
                    self.assertTrue(lock_path.exists())
                    with self.assertRaisesRegex(ValueError, "already in use"):
                        with acquire_state_lock("mca-test-home"):
                            pass


if __name__ == "__main__":
    unittest.main()
