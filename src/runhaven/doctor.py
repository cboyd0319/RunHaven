from __future__ import annotations

import platform
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass

PINNED_APPLE_CONTAINER_VERSION = "1.0.0"


@dataclass(frozen=True)
class Check:
    name: str
    ok: bool
    detail: str
    remedy: str = ""


def collect_checks() -> tuple[Check, ...]:
    checks: list[Check] = []

    py_version = sys.version_info
    checks.append(
        Check(
            "python",
            py_version >= (3, 13),
            f"{py_version.major}.{py_version.minor}.{py_version.micro}",
            "Use Python 3.13+; Python 3.14.6 is the recommended current runtime.",
        )
    )

    system = platform.system()
    checks.append(
        Check(
            "operating system",
            system == "Darwin",
            system or "unknown",
            "RunHaven only supports macOS 26+ on Apple silicon.",
        )
    )

    mac_version = platform.mac_ver()[0]
    major = parse_major_version(mac_version)
    checks.append(
        Check(
            "macOS",
            major is not None and major >= 26,
            mac_version or "unknown",
            "Use a macOS 26+ host.",
        )
    )

    machine = platform.machine()
    checks.append(
        Check(
            "architecture",
            machine in {"arm64", "aarch64"},
            machine or "unknown",
            "Use an Apple silicon Mac.",
        )
    )

    container_path = shutil.which("container")
    checks.append(
        Check(
            "Apple container CLI",
            container_path is not None,
            container_path or "not found on PATH",
            "Install Apple container 1.0.0 and run `container system start`.",
        )
    )

    if container_path is not None:
        checks.append(container_version_check())
        checks.append(container_status_check())

    return tuple(checks)


def parse_major_version(value: str) -> int | None:
    if not value:
        return None
    try:
        return int(value.split(".", maxsplit=1)[0])
    except ValueError:
        return None


def container_version_check() -> Check:
    try:
        result = subprocess.run(
            ["container", "--version"],
            check=False,
            capture_output=True,
            text=True,
            timeout=10,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        return Check("Apple container version", False, str(exc), "Check `container --version`.")

    output = (result.stdout or result.stderr).strip()
    version = parse_container_version(output)
    ok = result.returncode == 0 and version == PINNED_APPLE_CONTAINER_VERSION
    expected = f"expected {PINNED_APPLE_CONTAINER_VERSION}"
    detail = f"{output or f'exit {result.returncode}'}; {expected}"
    return Check(
        "Apple container version",
        ok,
        detail,
        f"Install the reviewed Apple container {PINNED_APPLE_CONTAINER_VERSION} release.",
    )


def parse_container_version(value: str) -> str | None:
    match = re.search(r"\b(\d+\.\d+\.\d+)\b", value)
    return match.group(1) if match else None


def container_status_check() -> Check:
    try:
        result = subprocess.run(
            ["container", "system", "status"],
            check=False,
            capture_output=True,
            text=True,
            timeout=10,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        return Check("container system", False, str(exc), "Run `container system start`.")

    detail = (result.stdout or result.stderr).strip() or f"exit {result.returncode}"
    return Check(
        "container system",
        result.returncode == 0,
        detail,
        "Run `container system start`.",
    )
