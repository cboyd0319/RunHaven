#!/usr/bin/env python3
"""Validate topology, unit ownership, and native dependency graphs."""

from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path, PurePosixPath

from repository_graph import check_cargo, check_javascript
from repository_io import load_json, regular, repository_paths
from repository_platform import check_platform_projection
from repository_policy import Policy, is_source_path, load_policy


REQUIRED_FILES = (
    ".node-version",
    ".python-version",
    ".github/copilot-instructions.md",
    "AGENTS.md",
    "ARCHITECTURE.md",
    "CLAUDE.md",
    "Cargo.lock",
    "Cargo.toml",
    "PROGRESS.md",
    "feature_list.json",
    "docs/harness/boundaries/feature-privacy-labels.json",
    "init.sh",
    "package-lock.json",
    "package.json",
    "pins.toml",
    "repository-structure-policy.json",
    "rust-toolchain.toml",
    "scripts/cargo.sh",
    "scripts/python.sh",
    "scripts/verify-all.sh",
    "src-tauri/tauri.conf.json",
    "tsconfig.base.json",
    "tsconfig.json",
    "ui/package.json",
    "ui/tsconfig.json",
)
REQUIRED_EXECUTABLES = (
    "init.sh",
    "scripts/cargo.sh",
    "scripts/python.sh",
    "scripts/verify-all.sh",
)
ARCHITECTURE_TOKENS = (
    "2,000,000 lines and 50,000 files",
    "`crates/` is the shared-library and CLI/TUI Rust source root",
    "`src-tauri/` is the separate deployable desktop application crate",
    "reviewed at 300 physical lines or 32,768 bytes",
    "more than 500 physical lines or 65,536 bytes",
    "scripts/python.sh scripts/check_repo_layout.py",
    "scripts/python.sh scripts/check_file_limits.py",
)
AGENTS_TOKENS = (
    "the closest scoped",
    "`ARCHITECTURE.md`",
    "`PROGRESS.md`",
    "`feature_list.json`",
    "`./init.sh`",
    "`scripts/verify-all.sh`",
)
COMPONENT_INVENTORY_FORBIDDEN = ("Auth broker prototype",)
FORBIDDEN_RUNTIME_OWNERS = (
    "crates/runhaven-cli/src/runtime",
    "src-tauri/src/runtime",
    "ui/src/runtime",
)
TAURI_CONTAINER_TEST_EXCEPTIONS = {
    "src-tauri/src/commands/tests/live.rs",
    "src-tauri/src/commands/tests/live_support.rs",
}


def _matches(relative: str, candidate: str) -> bool:
    return relative == candidate or relative.startswith(candidate + "/")


def _check_required(root: Path, policy: Policy, errors: list[str]) -> None:
    for relative in REQUIRED_FILES:
        if not regular(root / relative):
            errors.append(f"required repository file is missing or not regular: {relative}")
    for relative in REQUIRED_EXECUTABLES:
        if regular(root / relative) and not os.access(root / relative, os.X_OK):
            errors.append(f"required repository entrypoint is not executable: {relative}")
    for relative in policy.allowed_source_roots:
        path = root / relative
        if path.is_symlink() or not path.is_dir():
            errors.append(f"approved source root is missing or not a regular directory: {relative}")
    for relative in policy.retired_paths:
        if (root / relative).exists() or (root / relative).is_symlink():
            errors.append(f"retired repository path must stay absent: {relative}")
    if (root / "src").exists() or (root / "src").is_symlink():
        errors.append("root src/ is forbidden; deployable entrypoints belong to declared units")
    if (root / "src-tauri/Cargo.lock").exists() or (root / "src-tauri/Cargo.lock").is_symlink():
        errors.append("src-tauri/Cargo.lock is forbidden; root Cargo.lock owns the workspace")


def _check_architecture_owner(root: Path, errors: list[str]) -> None:
    path = root / "ARCHITECTURE.md"
    if not regular(path):
        return
    text = path.read_text(encoding="utf-8")
    for token in ARCHITECTURE_TOKENS:
        if token not in text:
            errors.append(f"ARCHITECTURE.md is missing canonical contract text: {token}")
    product = root / "docs/ARCHITECTURE.md"
    if regular(product) and "Product runtime architecture remains in `docs/ARCHITECTURE.md`" not in text:
        errors.append("root architecture must identify docs/ARCHITECTURE.md as product detail")

    inventory = root / "docs/harness/boundaries/component-inventory.md"
    if regular(inventory):
        inventory_text = inventory.read_text(encoding="utf-8")
        for marker in COMPONENT_INVENTORY_FORBIDDEN:
            if marker in inventory_text:
                errors.append(f"component inventory retains stale architecture label: {marker}")


def _check_instruction_routes(root: Path, errors: list[str]) -> None:
    agents = root / "AGENTS.md"
    if regular(agents):
        text = agents.read_text(encoding="utf-8")
        for token in AGENTS_TOKENS:
            if token not in text:
                errors.append(f"AGENTS.md is missing repository routing contract: {token}")
    claude = root / "CLAUDE.md"
    if regular(claude):
        text = claude.read_text(encoding="utf-8")
        if not text.startswith("@AGENTS.md\n") or len(text.splitlines()) > 8:
            errors.append("CLAUDE.md must remain a thin import of root AGENTS.md")
    copilot = root / ".github/copilot-instructions.md"
    if regular(copilot):
        text = copilot.read_text(encoding="utf-8")
        required = (
            "Before any work",
            "../AGENTS.md",
            "../PROGRESS.md",
            "../feature_list.json",
            "does not own repository policy",
        )
        if len(text.splitlines()) > 8 or any(token not in text for token in required):
            errors.append("Copilot instructions must remain a thin route to canonical root owners")


def _check_units(root: Path, policy: Policy, paths: list[str], errors: list[str]) -> None:
    unit_roots = {unit.root for unit in policy.units}
    for unit in policy.units:
        unit_root = root / unit.root
        if unit_root.is_symlink() or not unit_root.is_dir():
            errors.append(f"unit root is missing or not regular: {unit.root}")
        test_paths = set(unit.test_entrypoints)
        for relative in (
            unit.manifest,
            *unit.public_entrypoints,
            *unit.test_entrypoints,
            *unit.documentation,
        ):
            if not regular(root / relative):
                errors.append(f"unit {unit.identifier} is missing regular contract file: {relative}")
            elif relative in test_paths and relative.endswith(".sh") and not os.access(
                root / relative, os.X_OK
            ):
                errors.append(f"unit {unit.identifier} shell test is not executable: {relative}")
        package_path = unit_root / "package.json"
        if package_path.is_file():
            package = load_json(package_path, f"{unit.root}/package.json", errors)
            if package and package.get("name") != unit.identifier:
                errors.append(f"unit id and package name disagree: {unit.identifier}")
        if unit.manifest.endswith("manifest.json"):
            manifest = load_json(root / unit.manifest, unit.manifest, errors)
            if manifest and manifest.get("id") != unit.identifier:
                errors.append(f"unit id and manifest id disagree: {unit.identifier}")
    for relative in paths:
        if any(_matches(relative, item) for item in policy.exclusions):
            continue
        in_unitized_root = any(_matches(relative, item) for item in policy.unitized_source_roots)
        if not in_unitized_root:
            continue
        is_contract_file = PurePosixPath(relative).name in {"Cargo.toml", "Containerfile", "package.json"}
        is_source = is_source_path(relative, policy)
        if (is_contract_file or is_source) and not any(_matches(relative, item) for item in unit_roots):
            errors.append(f"source or manifest is outside a declared unit: {relative}")


def _check_graph(policy: Policy, errors: list[str]) -> None:
    graph = {unit.identifier: set(unit.allowed_dependencies) for unit in policy.units}
    visiting: set[str] = set()
    visited: set[str] = set()

    def visit(identifier: str, trail: list[str]) -> None:
        if identifier in visiting:
            cycle = trail[trail.index(identifier) :]
            errors.append("unit dependency cycle: " + " -> ".join(cycle))
            return
        if identifier in visited:
            return
        visiting.add(identifier)
        for dependency in sorted(graph.get(identifier, set())):
            visit(dependency, [*trail, dependency])
        visiting.remove(identifier)
        visited.add(identifier)

    for identifier in sorted(graph):
        visit(identifier, [identifier])


def _check_image_edges(root: Path, policy: Policy, errors: list[str]) -> None:
    common_id = "runhaven-image-common"
    for unit in policy.units:
        if unit.kind != "image" or common_id not in unit.allowed_dependencies:
            continue
        containerfile = root / unit.root / "Containerfile"
        if regular(containerfile) and "COPY common/" not in containerfile.read_text(encoding="utf-8"):
            errors.append(f"image dependency graph disagrees for {unit.identifier}: common inputs not used")


def _check_no_ci(root: Path, errors: list[str]) -> None:
    workflows = root / ".github/workflows"
    if not workflows.exists():
        return
    if workflows.is_symlink() or not workflows.is_dir():
        errors.append("CI workflow path must be absent, not redirected")
        return
    for path in sorted(workflows.iterdir()):
        errors.append(f"hosted CI is disabled; workflow artifact must be absent: {path.name}")


def _check_tauri_runtime_boundary(root: Path, errors: list[str]) -> None:
    source_root = root / "src-tauri/src"
    if not source_root.is_dir():
        return
    marker = 'Command::new("container")'
    for path in source_root.rglob("*.rs"):
        relative = path.relative_to(root).as_posix()
        if relative not in TAURI_CONTAINER_TEST_EXCEPTIONS:
            text = path.read_text(encoding="utf-8", errors="replace")
            if marker in text:
                errors.append(f"Tauri bypasses the runhaven-core runtime owner: {relative}")
    for relative in FORBIDDEN_RUNTIME_OWNERS:
        if (root / relative).exists() or (root / relative).is_symlink():
            errors.append(f"shared runtime truth must not exist at {relative}")


def _check_tauri_build_contract(root: Path, errors: list[str]) -> None:
    config = load_json(root / "src-tauri/tauri.conf.json", "src-tauri/tauri.conf.json", errors)
    if not config:
        return
    build = config.get("build")
    if not isinstance(build, dict):
        errors.append("Tauri config must define the root-owned frontend build contract")
        return
    if build.get("beforeBuildCommand") != "npm run build":
        errors.append("Tauri beforeBuildCommand must use the root npm workspace")
    if build.get("frontendDist") != "../ui/dist":
        errors.append("Tauri frontendDist must resolve to the UI build output")

    init_path = root / "scripts/init_repository.py"
    if not regular(init_path):
        errors.append("initialization core is missing or not regular: scripts/init_repository.py")
        return
    init_text = init_path.read_text(encoding="utf-8")
    frontend_marker = '["npm", "run", "build"]'
    rust_marker = '["scripts/cargo.sh", "check", "--workspace", "--locked"]'
    frontend_index = init_text.find(frontend_marker)
    rust_index = init_text.find(rust_marker)
    if frontend_index < 0 or rust_index < 0 or frontend_index > rust_index:
        errors.append("initialization must build the Tauri frontend before checking the Rust workspace")


def _check_security_fact_alignment(root: Path, errors: list[str]) -> None:
    features = load_json(root / "feature_list.json", "feature_list.json", errors)
    privacy = load_json(
        root / "docs/harness/boundaries/feature-privacy-labels.json",
        "feature privacy labels",
        errors,
    )
    if not features or not privacy:
        return
    passing = {
        item.get("id")
        for item in features.get("features", [])
        if isinstance(item, dict) and item.get("status") == "passing"
    }
    privacy_by_id = {
        item.get("id"): item
        for item in privacy.get("features", [])
        if isinstance(item, dict) and isinstance(item.get("id"), str)
    }
    required_tokens = {
        "multi-provider-broker": ("Claude", "Codex", "OPENAI_API_KEY", "ANTHROPIC_API_KEY"),
        "ssh-forwarding-boundary": ("SSH-agent socket", "Raw SSH keys", "fails safely"),
    }
    for identifier, tokens in required_tokens.items():
        if identifier not in passing:
            continue
        item = privacy_by_id.get(identifier)
        if item is None:
            errors.append(f"passing security feature lacks a privacy projection: {identifier}")
            continue
        flow = item.get("defaultDataFlow", "")
        if not isinstance(flow, str) or any(token not in flow for token in tokens):
            errors.append(f"privacy projection disagrees with passing security feature: {identifier}")


def check_repository(root: Path) -> list[str]:
    root = root.resolve()
    policy, policy_errors = load_policy(root)
    errors = list(policy_errors)
    if policy is None:
        return errors
    paths, path_errors = repository_paths(root)
    errors.extend(path_errors)
    _check_required(root, policy, errors)
    _check_architecture_owner(root, errors)
    _check_instruction_routes(root, errors)
    _check_units(root, policy, paths, errors)
    _check_graph(policy, errors)
    check_cargo(root, policy, errors)
    check_javascript(root, policy, paths, errors)
    _check_image_edges(root, policy, errors)
    _check_no_ci(root, errors)
    _check_tauri_runtime_boundary(root, errors)
    _check_tauri_build_contract(root, errors)
    _check_security_fact_alignment(root, errors)
    check_platform_projection(root, policy, paths, errors)
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    args = parser.parse_args()
    errors = check_repository(args.root)
    if errors:
        print("repository layout check failed", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("repository layout check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
