#!/usr/bin/env python3
"""Run the standard macOS initialization and representative baseline."""

from __future__ import annotations

import argparse
import os
import shlex
import subprocess
import sys
import time
from pathlib import Path


def run(root: Path, command: list[str], reason: str) -> None:
    print(f"== {reason}: {shlex.join(command)} ==", flush=True)
    started = time.monotonic()
    result = subprocess.run(command, cwd=root, check=False)
    elapsed = time.monotonic() - started
    if result.returncode != 0:
        raise SystemExit(
            f"initialization failed ({reason}); exit {result.returncode}; {elapsed:.2f}s"
        )
    print(f"== completed {reason} in {elapsed:.2f}s ==", flush=True)


def dependencies_ready(root: Path) -> bool:
    required = (
        root / "ui/node_modules/.bin/vite",
        root / "ui/node_modules/.bin/vitest",
        root / "ui/node_modules/.bin/playwright",
        root / "ui/node_modules/.bin/tauri",
    )
    root = root.resolve()
    for path in required:
        try:
            resolved = path.resolve(strict=True)
            resolved.relative_to(root)
        except (OSError, ValueError):
            return False
        if not resolved.is_file():
            return False
    return True


def main() -> int:
    started = time.monotonic()
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--bootstrap",
        action="store_true",
        help="synchronize root npm dependencies from package-lock.json",
    )
    parser.add_argument("--skip-e2e", action="store_true", help="skip only the browser smoke")
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    python = sys.executable

    print(f"Repository: {root}")
    run(root, [python, "scripts/check_environment.py"], "readiness")
    run(root, [python, "scripts/test_environment.py"], "environment violation fixtures")
    run(root, ["bash", "scripts/test_scan_dev_log.sh"], "secret scanner fixtures")
    run(root, [python, "scripts/check_harness_state.py"], "durable state")
    run(root, [python, "scripts/test_harness_state.py"], "state violation fixtures")
    run(root, [python, "scripts/test_file_limits.py"], "source-limit fixtures")
    run(root, [python, "scripts/test_repo_layout.py"], "topology fixtures")
    run(root, [python, "scripts/check_file_limits.py"], "source limits")
    run(root, [python, "scripts/check_repo_layout.py"], "topology and dependency graph")

    if args.bootstrap:
        print("bootstrap writes only ignored node_modules content from package-lock.json")
        run(root, ["npm", "ci", "--ignore-scripts"], "locked dependency synchronization")
    elif not dependencies_ready(root):
        raise SystemExit(
            "dependencies are not ready; run ./init.sh --bootstrap, then rerun ./init.sh"
        )

    run(root, ["npm", "run", "build"], "frontend production build")
    run(
        root,
        ["scripts/cargo.sh", "check", "--workspace", "--locked"],
        "Rust workspace baseline",
    )
    run(root, ["npm", "run", "check"], "TypeScript project graph")
    run(root, ["npm", "test"], "frontend unit baseline")
    if not args.skip_e2e:
        run(root, ["npm", "run", "test:e2e:smoke"], "representative frontend journey")
    run(root, ["git", "diff", "--check"], "diff hygiene")

    start = ["npm", "run", "tauri:dev"]
    print(f"Start command: {shlex.join(start)}")
    if os.environ.get("RUN_START_COMMAND") == "1":
        os.execvp(start[0], start)
    print(f"initialization baseline passed in {time.monotonic() - started:.2f}s")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
