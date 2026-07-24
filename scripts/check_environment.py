#!/usr/bin/env python3
"""Read-only validation of the macOS 26+ RunHaven toolchain contract."""

from __future__ import annotations

import argparse
import json
import platform
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path


def _read_version(path: Path, label: str, errors: list[str]) -> str:
    if path.is_symlink() or not path.is_file():
        errors.append(f"missing regular runtime owner: {label}")
        return ""
    try:
        value = path.read_text(encoding="utf-8").strip()
    except (OSError, UnicodeError) as exc:
        errors.append(f"cannot read runtime owner {label}: {exc}")
        return ""
    if not value or any(character.isspace() for character in value):
        errors.append(f"{label} must contain one exact version")
        return ""
    return value


def _command(name: str, errors: list[str]) -> str:
    path = shutil.which(name)
    if not path:
        errors.append(f"required command not found: {name}")
        return ""
    return path


def _capture(command: list[str], label: str, errors: list[str]) -> str:
    try:
        result = subprocess.run(
            command,
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            timeout=30,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        errors.append(f"cannot inspect {label}: {exc}")
        return ""
    output = result.stdout.strip()
    if result.returncode != 0:
        errors.append(f"{label} check failed with exit {result.returncode}: {output}")
        return ""
    return output


def check_environment(root: Path, *, full: bool = False) -> list[str]:
    root = root.resolve()
    errors: list[str] = []
    if platform.system() != "Darwin":
        errors.append("RunHaven requires macOS 26 or newer")
    else:
        release = platform.mac_ver()[0]
        try:
            major = int(release.split(".", 1)[0])
        except (ValueError, IndexError):
            major = 0
        if major < 26:
            errors.append(f"RunHaven requires macOS 26 or newer; found {release or 'unknown'}")
    if platform.machine() != "arm64":
        errors.append(f"RunHaven requires Apple silicon; found {platform.machine() or 'unknown'}")

    python_expected = _read_version(root / ".python-version", ".python-version", errors)
    python_actual = platform.python_version()
    if python_expected and python_actual != python_expected:
        errors.append(f"Python {python_expected} is required; found {python_actual}")

    node_expected = _read_version(root / ".node-version", ".node-version", errors)
    npm_expected = ""
    try:
        package = json.loads((root / "package.json").read_text(encoding="utf-8"))
        engines = package["engines"]
        if engines.get("node") != node_expected:
            errors.append("package.json engines.node must match .node-version")
        npm_expected = str(engines["npm"])
        if package.get("packageManager") != f"npm@{npm_expected}":
            errors.append("package.json packageManager must match engines.npm")
    except (OSError, KeyError, TypeError, json.JSONDecodeError) as exc:
        errors.append(f"cannot read package.json runtime owners: {exc}")
    node = _command("node", errors)
    if node:
        node_actual = _capture([node, "--version"], "Node.js", errors).removeprefix("v")
        if node_expected and node_actual != node_expected:
            errors.append(f"Node.js {node_expected} is required; found {node_actual or 'unknown'}")
    npm = _command("npm", errors)
    if npm:
        npm_actual = _capture([npm, "--version"], "npm", errors)
        if npm_expected and npm_actual != npm_expected:
            errors.append(f"npm {npm_expected} is required; found {npm_actual or 'unknown'}")

    rustup = _command("rustup", errors)
    rust_expected = ""
    toolchain_path = root / "rust-toolchain.toml"
    if toolchain_path.is_file() and not toolchain_path.is_symlink():
        try:
            with toolchain_path.open("rb") as handle:
                rust_expected = str(tomllib.load(handle)["toolchain"]["channel"])
        except (OSError, KeyError, TypeError, tomllib.TOMLDecodeError) as exc:
            errors.append(f"invalid rust-toolchain.toml: {exc}")
    else:
        errors.append("missing regular runtime owner: rust-toolchain.toml")
    if rustup and rust_expected:
        rustc = _capture([rustup, "run", rust_expected, "rustc", "--version"], "Rust", errors)
        if rustc and not rustc.startswith(f"rustc {rust_expected} "):
            errors.append(f"Rust {rust_expected} is required; found {rustc}")

    container = _command("container", errors)
    if container:
        output = _capture([container, "--version"], "Apple container", errors)
        if "version 1.1.0" not in output:
            errors.append(f"Apple container 1.1.0 is required; found {output or 'unknown'}")
    _command("git", errors)
    if full:
        for name in ("cargo", "xcodebuild"):
            _command(name, errors)
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument("--full", action="store_true", help="also require full-build tools")
    args = parser.parse_args()
    errors = check_environment(args.root, full=args.full)
    if errors:
        print("environment readiness check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        print("remediation: install the exact versions named by root runtime owners", file=sys.stderr)
        return 1
    print("environment readiness check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
