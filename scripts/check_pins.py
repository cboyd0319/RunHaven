from __future__ import annotations

import json
import re
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]

TEXT_FILES = (
    ".github/workflows/ci.yml",
    "pyproject.toml",
    "requirements-dev.txt",
    "src/macos_container_agents/profiles.py",
    "src/macos_container_agents/plans.py",
    "src/macos_container_agents/doctor.py",
    "src/macos_container_agents/images/base/Containerfile",
    "src/macos_container_agents/images/claude/Containerfile",
    "src/macos_container_agents/images/codex/Containerfile",
    "src/macos_container_agents/images/gemini/Containerfile",
    "src/macos_container_agents/images/antigravity/Containerfile",
    "src/macos_container_agents/images/copilot/Containerfile",
    "src/macos_container_agents/images/common/debian-packages.txt",
    "src/macos_container_agents/images/common/debian.sources",
    "src/macos_container_agents/images/common/create-agent-user.sh",
    "src/macos_container_agents/images/claude/package.json",
    "src/macos_container_agents/images/codex/package.json",
    "src/macos_container_agents/images/gemini/package.json",
    "src/macos_container_agents/images/copilot/package.json",
)

NPM_PACKAGE_DIRS = (
    "src/macos_container_agents/images/claude",
    "src/macos_container_agents/images/codex",
    "src/macos_container_agents/images/gemini",
    "src/macos_container_agents/images/copilot",
)

GITHUB_ACTION_RE = re.compile(r"uses:\s*[\w./-]+@([^\s#]+)")
IMMUTABLE_SHA_RE = re.compile(r"^[0-9a-f]{40}$")

FORBIDDEN_PATTERNS = (
    (re.compile(r"(?<![A-Za-z0-9_.-])latest(?![A-Za-z0-9_.-])"), "mutable latest tag"),
    (re.compile(r"npm install(?![^\n]*@[0-9])"), "unpinned npm install"),
)
LOOSE_DEP_RE = re.compile(r'"[^"]*(?:>=|~=|\*).*"')


def main() -> int:
    failures: list[str] = []

    for relative in TEXT_FILES:
        path = ROOT / relative
        text = path.read_text(encoding="utf-8")
        for pattern, label in FORBIDDEN_PATTERNS:
            for match in pattern.finditer(text):
                line = text.count("\n", 0, match.start()) + 1
                failures.append(f"{relative}:{line}: {label}")

        if relative.endswith((".json", ".toml", ".yml")):
            for line_number, line in enumerate(text.splitlines(), start=1):
                if (
                    "requires-python" in line
                    or "package-data" in line
                    or "images/*/Containerfile" in line
                ):
                    continue
                if LOOSE_DEP_RE.search(line):
                    failures.append(f"{relative}:{line_number}: loose dependency version")

        if relative.endswith("Containerfile"):
            failures.extend(check_containerfile_from_pins(relative, text))
            failures.extend(check_apt_install_block(relative, text))
        if relative.endswith("debian-packages.txt"):
            failures.extend(check_debian_package_file(relative, text))
        if relative.endswith("debian.sources"):
            failures.extend(check_debian_sources(relative, text))
        if relative == "requirements-dev.txt":
            failures.extend(check_requirements_file(relative, text))

        if relative.endswith(".yml"):
            for match in GITHUB_ACTION_RE.finditer(text):
                ref = match.group(1)
                if not IMMUTABLE_SHA_RE.match(ref):
                    line = text.count("\n", 0, match.start()) + 1
                    failures.append(f"{relative}:{line}: GitHub Action ref is not an immutable SHA")

    for relative in NPM_PACKAGE_DIRS:
        failures.extend(check_npm_package(relative))

    if failures:
        print("Pin policy failures:")
        for failure in failures:
            print(f"  {failure}")
        return 1

    print("Pin policy passed")
    return 0


def check_apt_install_block(relative: str, text: str) -> list[str]:
    failures: list[str] = []
    lines = text.splitlines()
    for index, line in enumerate(lines):
        if "apt-get install" not in line:
            continue
        block: list[str] = []
        for candidate in lines[index + 1 :]:
            block.append(candidate)
            if not candidate.rstrip().endswith("\\"):
                break
        package_lines = [
            candidate.strip()
            for candidate in block
            if candidate.strip()
            and not candidate.strip().startswith("&&")
            and not candidate.strip().startswith("-")
        ]
        for offset, candidate in enumerate(package_lines, start=1):
            if "=" not in candidate:
                failures.append(f"{relative}:{index + offset + 1}: unpinned apt package")
    return failures


def check_containerfile_from_pins(relative: str, text: str) -> list[str]:
    failures: list[str] = []
    for line_number, line in enumerate(text.splitlines(), start=1):
        value = line.strip()
        if not value.startswith("FROM "):
            continue
        image = value.split(maxsplit=1)[1]
        if "@sha256:" not in image:
            failures.append(f"{relative}:{line_number}: base image is not digest-pinned")
    return failures


def check_debian_package_file(relative: str, text: str) -> list[str]:
    failures: list[str] = []
    for line_number, line in enumerate(text.splitlines(), start=1):
        value = line.strip()
        if not value:
            continue
        if "=" not in value:
            failures.append(f"{relative}:{line_number}: unpinned apt package")
    return failures


def check_debian_sources(relative: str, text: str) -> list[str]:
    failures: list[str] = []
    if "snapshot.debian.org" not in text:
        failures.append(f"{relative}: Debian sources must use snapshot.debian.org")
    if "deb.debian.org" in text:
        failures.append(f"{relative}: Debian sources must not use moving mirrors")
    for line_number, line in enumerate(text.splitlines(), start=1):
        if line.startswith("URIs:") and not re.search(r"/\d{8}T\d{6}Z$", line):
            failures.append(f"{relative}:{line_number}: snapshot URI is not timestamp-pinned")
    return failures


def check_requirements_file(relative: str, text: str) -> list[str]:
    failures: list[str] = []
    for line_number, line in enumerate(text.splitlines(), start=1):
        value = line.strip()
        if not value or value.startswith("#"):
            continue
        requirement = value.split(";", maxsplit=1)[0].strip()
        if "==" not in requirement:
            failures.append(f"{relative}:{line_number}: requirement is not exact-pinned")
    return failures


def check_npm_package(relative: str) -> list[str]:
    failures: list[str] = []
    package_path = ROOT / relative / "package.json"
    lock_path = ROOT / relative / "package-lock.json"
    package_json = json.loads(package_path.read_text(encoding="utf-8"))
    lock_json = json.loads(lock_path.read_text(encoding="utf-8"))

    for section in ("dependencies", "devDependencies", "optionalDependencies"):
        for name, version in package_json.get(section, {}).items():
            if not is_exact_npm_version(version):
                failures.append(f"{relative}/package.json: {name} is not exact-pinned")

    allow_scripts = package_json.get("allowScripts", {})
    for name, allowed in allow_scripts.items():
        if allowed is not True:
            failures.append(
                f"{relative}/package.json: {name} install script is not explicitly allowed"
            )
        if "@" not in name.lstrip("@"):
            failures.append(
                f"{relative}/package.json: {name} install script approval is not pinned"
            )

    packages = lock_json.get("packages", {})
    for path, details in packages.items():
        if path == "":
            continue
        if not isinstance(details, dict):
            failures.append(f"{relative}/package-lock.json: invalid package entry {path}")
            continue
        name = npm_package_name(path, details)
        version = details.get("version")
        resolved = details.get("resolved")
        integrity = details.get("integrity")
        if not isinstance(version, str) or not version:
            failures.append(f"{relative}/package-lock.json: {name} missing version")
        if not isinstance(resolved, str) or not resolved.startswith("https://registry.npmjs.org/"):
            failures.append(f"{relative}/package-lock.json: {name} missing registry tarball")
        if not isinstance(integrity, str) or not integrity.startswith("sha512-"):
            failures.append(f"{relative}/package-lock.json: {name} missing sha512 integrity")
        if details.get("hasInstallScript") is True:
            approval = f"{name}@{version}"
            if allow_scripts.get(approval) is not True:
                failures.append(
                    f"{relative}/package.json: missing allowScripts approval for {approval}"
                )

    return failures


def is_exact_npm_version(value: Any) -> bool:
    return (
        isinstance(value, str)
        and re.match(r"^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?$", value) is not None
    )


def npm_package_name(path: str, details: dict[str, Any]) -> str:
    if isinstance(details.get("name"), str):
        return details["name"]
    prefix = "node_modules/"
    if not path.startswith(prefix):
        return path
    remainder = path[len(prefix) :]
    parts = remainder.split("/")
    if remainder.startswith("@") and len(parts) >= 2:
        return "/".join(parts[:2])
    return parts[0]


if __name__ == "__main__":
    raise SystemExit(main())
