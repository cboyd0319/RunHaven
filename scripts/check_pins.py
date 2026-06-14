from __future__ import annotations

import json
import re
import tomllib
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]

TEXT_FILES = (
    ".github/workflows/ci.yml",
    "pyproject.toml",
    "requirements-dev.txt",
    "src/runhaven/profiles.py",
    "src/runhaven/plans.py",
    "src/runhaven/doctor.py",
    "src/runhaven/images/base/Containerfile",
    "src/runhaven/images/claude/Containerfile",
    "src/runhaven/images/codex/Containerfile",
    "src/runhaven/images/gemini/Containerfile",
    "src/runhaven/images/antigravity/Containerfile",
    "src/runhaven/images/copilot/Containerfile",
    "src/runhaven/images/common/debian-packages.txt",
    "src/runhaven/images/common/debian.sources",
    "src/runhaven/images/common/create-agent-user.sh",
    "src/runhaven/images/claude/package.json",
    "src/runhaven/images/codex/package.json",
    "src/runhaven/images/gemini/package.json",
    "src/runhaven/images/copilot/package.json",
)

NPM_PACKAGE_DIRS = (
    "src/runhaven/images/claude",
    "src/runhaven/images/codex",
    "src/runhaven/images/gemini",
    "src/runhaven/images/copilot",
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
    pins = load_pins()

    for relative in TEXT_FILES:
        path = ROOT / relative
        text = path.read_text(encoding="utf-8")
        for pattern, label in FORBIDDEN_PATTERNS:
            for match in pattern.finditer(text):
                match_line = text.count("\n", 0, match.start()) + 1
                failures.append(f"{relative}:{match_line}: {label}")

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
            failures.extend(check_containerfile_against_ledger(relative, text, pins))
        if relative.endswith("debian-packages.txt"):
            failures.extend(check_debian_package_file(relative, text))
            failures.extend(check_debian_packages_against_ledger(relative, text, pins))
        if relative.endswith("debian.sources"):
            failures.extend(check_debian_sources(relative, text))
            failures.extend(check_debian_sources_against_ledger(relative, text, pins))
        if relative == "requirements-dev.txt":
            failures.extend(check_requirements_file(relative, text))
            failures.extend(check_requirements_against_ledger(relative, text, pins))
        if relative == "pyproject.toml":
            failures.extend(check_pyproject_against_ledger(relative, text, pins))
        if relative == ".github/workflows/ci.yml":
            failures.extend(check_ci_against_ledger(relative, text, pins))
        if relative == "src/runhaven/profiles.py":
            failures.extend(check_profiles_against_ledger(relative, text))
        if relative == "src/runhaven/plans.py":
            failures.extend(check_run_plan_against_ledger(relative, text, pins))
        if relative == "src/runhaven/doctor.py":
            failures.extend(check_doctor_against_ledger(relative, text, pins))

        if relative.endswith(".yml"):
            for match in GITHUB_ACTION_RE.finditer(text):
                ref = match.group(1)
                if not IMMUTABLE_SHA_RE.match(ref):
                    match_line = text.count("\n", 0, match.start()) + 1
                    failures.append(
                        f"{relative}:{match_line}: GitHub Action ref is not an immutable SHA"
                    )

    for relative in NPM_PACKAGE_DIRS:
        failures.extend(check_npm_package(relative, pins))

    if failures:
        print("Pin policy failures:")
        for failure in failures:
            print(f"  {failure}")
        return 1

    print("Pin policy passed")
    return 0


def load_pins() -> dict[str, Any]:
    return tomllib.loads((ROOT / "pins.toml").read_text(encoding="utf-8"))


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


def check_pyproject_against_ledger(
    relative: str, text: str, pins: dict[str, Any]
) -> list[str]:
    failures: list[str] = []
    pyproject = tomllib.loads(text)
    python_pins = pins["python"]

    build_requires = pyproject["build-system"]["requires"]
    expected_setuptools = f"setuptools=={python_pins['setuptools']}"
    if build_requires != [expected_setuptools]:
        failures.append(f"{relative}: build-system setuptools does not match pins.toml")

    dev = set(pyproject["project"]["optional-dependencies"]["dev"])
    for name in ("build", "mypy", "ruff"):
        expected = f"{name}=={python_pins[name]}"
        if expected not in dev:
            failures.append(f"{relative}: dev dependency {expected} missing")
    if any(requirement.startswith("pytest") for requirement in dev):
        failures.append(f"{relative}: pytest is not used by the unittest suite")
    return failures


def check_requirements_against_ledger(
    relative: str, text: str, pins: dict[str, Any]
) -> list[str]:
    failures: list[str] = []
    expected = {
        "build": pins["python"]["build"],
        "mypy": pins["python"]["mypy"],
        "ruff": pins["python"]["ruff"],
    }
    requirements = parse_requirements(text)
    for name, version in expected.items():
        if requirements.get(name) != version:
            failures.append(f"{relative}: {name} does not match pins.toml")
    if "pytest" in requirements:
        failures.append(f"{relative}: pytest is not used by the unittest suite")
    return failures


def parse_requirements(text: str) -> dict[str, str]:
    parsed: dict[str, str] = {}
    for line in text.splitlines():
        value = line.strip()
        if not value or value.startswith("#"):
            continue
        requirement = value.split(";", maxsplit=1)[0].strip()
        if "==" not in requirement:
            continue
        name, version = requirement.split("==", maxsplit=1)
        parsed[name] = version
    return parsed


def check_ci_against_ledger(relative: str, text: str, pins: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    github_runners = pins["github_runners"]
    python_pins = pins["python"]
    actions = pins["github_actions"]
    if github_runners["macos"] not in text:
        failures.append(f"{relative}: macOS runner does not match pins.toml")
    if "ubuntu" in text.lower() or "windows" in text.lower():
        failures.append(f"{relative}: CI must run only on macOS 26+")
    for version in (python_pins["minimum_tested"], python_pins["current_stable"]):
        if version not in text:
            failures.append(f"{relative}: Python {version} missing from CI matrix")
    for action_name, action_pin in actions.items():
        sha = action_pin["sha"]
        if sha not in text:
            failures.append(f"{relative}: {action_name} SHA does not match pins.toml")
    return failures


def check_profiles_against_ledger(relative: str, text: str) -> list[str]:
    failures: list[str] = []
    for image in (
        "runhaven/claude:0.1.0",
        "runhaven/codex:0.1.0",
        "runhaven/gemini:0.1.0",
        "runhaven/antigravity:0.1.0",
        "runhaven/copilot:0.1.0",
        "runhaven/base:0.1.0",
    ):
        if image not in text:
            failures.append(f"{relative}: missing pinned image {image}")
    return failures


def check_run_plan_against_ledger(
    relative: str, text: str, pins: dict[str, Any]
) -> list[str]:
    failures: list[str] = []
    digest = pins["container_images"]["debian_trixie_slim"]["digest"]
    if digest not in text:
        failures.append(f"{relative}: volume-prep image digest does not match pins.toml")
    return failures


def check_doctor_against_ledger(relative: str, text: str, pins: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    version = pins["apple_container"]["version"]
    if f'PINNED_APPLE_CONTAINER_VERSION = "{version}"' not in text:
        failures.append(f"{relative}: Apple container version does not match pins.toml")
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


def check_containerfile_against_ledger(
    relative: str, text: str, pins: dict[str, Any]
) -> list[str]:
    failures: list[str] = []
    container_images = pins["container_images"]
    agent_cli = pins["agent_cli"]
    agent_integrity = pins["agent_cli_integrity"]
    node_digest = container_images["node_26_trixie_slim"]["digest"]
    debian_digest = container_images["debian_trixie_slim"]["digest"]

    node_containerfiles = (
        "claude/Containerfile",
        "codex/Containerfile",
        "gemini/Containerfile",
        "copilot/Containerfile",
    )
    if relative.endswith(node_containerfiles):
        if node_digest not in text:
            failures.append(f"{relative}: node base image digest does not match pins.toml")
        if f"npm@{agent_cli['npm']}" not in text:
            failures.append(f"{relative}: npm version does not match pins.toml")
    else:
        if debian_digest not in text:
            failures.append(f"{relative}: Debian base image digest does not match pins.toml")

    if relative.endswith("antigravity/Containerfile"):
        if f"ANTIGRAVITY_CLI_VERSION={agent_cli['antigravity_cli']}" not in text:
            failures.append(f"{relative}: Antigravity version does not match pins.toml")
        if agent_integrity["antigravity_cli"].removeprefix("sha512-") not in text:
            failures.append(f"{relative}: Antigravity checksum does not match pins.toml")
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


def check_debian_packages_against_ledger(
    relative: str, text: str, pins: dict[str, Any]
) -> list[str]:
    failures: list[str] = []
    ledger = pins["debian_trixie_arm64"]
    for line_number, line in enumerate(text.splitlines(), start=1):
        value = line.strip()
        if not value:
            continue
        name, version = value.split("=", maxsplit=1)
        key = debian_package_key(name)
        expected = ledger.get(key)
        if expected != version:
            failures.append(
                f"{relative}:{line_number}: {name}={version} does not match pins.toml"
            )
    return failures


def debian_package_key(name: str) -> str:
    return name.replace("-", "_").replace(".", "_")


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


def check_debian_sources_against_ledger(
    relative: str, text: str, pins: dict[str, Any]
) -> list[str]:
    failures: list[str] = []
    snapshot = pins["debian_snapshot"]
    for key in ("debian_uri", "security_uri"):
        if snapshot[key] not in text:
            failures.append(f"{relative}: {key} does not match pins.toml")
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


def check_npm_package(relative: str, pins: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    package_path = ROOT / relative / "package.json"
    lock_path = ROOT / relative / "package-lock.json"
    package_json = json.loads(package_path.read_text(encoding="utf-8"))
    lock_json = json.loads(lock_path.read_text(encoding="utf-8"))
    agent_versions = {
        "src/runhaven/images/claude": (
            "@anthropic-ai/claude-code",
            pins["agent_cli"]["claude_code"],
            pins["agent_cli_integrity"]["claude_code"],
        ),
        "src/runhaven/images/codex": (
            "@openai/codex",
            pins["agent_cli"]["codex"],
            pins["agent_cli_integrity"]["codex"],
        ),
        "src/runhaven/images/gemini": (
            "@google/gemini-cli",
            pins["agent_cli"]["gemini_cli"],
            pins["agent_cli_integrity"]["gemini_cli"],
        ),
        "src/runhaven/images/copilot": (
            "@github/copilot",
            pins["agent_cli"]["copilot_cli"],
            pins["agent_cli_integrity"]["copilot_cli"],
        ),
    }
    root_name, root_version, root_integrity = agent_versions[relative]

    for section in ("dependencies", "devDependencies", "optionalDependencies"):
        for name, version in package_json.get(section, {}).items():
            if not is_exact_npm_version(version):
                failures.append(f"{relative}/package.json: {name} is not exact-pinned")
    if package_json.get("dependencies", {}).get(root_name) != root_version:
        failures.append(f"{relative}/package.json: {root_name} does not match pins.toml")

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
        if path == f"node_modules/{root_name}" and integrity != root_integrity:
            failures.append(
                f"{relative}/package-lock.json: {name} integrity differs from pins.toml"
            )
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
    name = details.get("name")
    if isinstance(name, str):
        return name
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
