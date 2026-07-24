"""Validate Cargo, npm, and TypeScript graphs against repository unit policy."""

from __future__ import annotations

import re
from pathlib import Path

from repository_io import dependency_names, load_json, load_toml
from repository_policy import Policy, Unit


SOURCE_IMPORT = re.compile(r"(?:from\s+|import\s*\(\s*|require\s*\(\s*)['\"]([^'\"]+)['\"]")
ROOT_SCRIPT_PROJECTIONS = {
    "build": "npm --workspace runhaven-ui run build",
    "check": "npm --workspace runhaven-ui run check",
    "tauri:build": "npm --workspace runhaven-ui run tauri:build",
    "test": "npm --workspace runhaven-ui test",
    "test:e2e": "npm --workspace runhaven-ui run test:e2e",
}
UI_TAURI_BUILD = (
    'cd .. && CARGO_TARGET_DIR="$PWD/target" ./ui/node_modules/.bin/tauri build '
    "--config src-tauri/tauri.conf.json --debug --no-bundle"
)


def _cargo_units(policy: Policy) -> list[Unit]:
    return [unit for unit in policy.units if unit.manifest.endswith("Cargo.toml")]


def check_cargo(root: Path, policy: Policy, errors: list[str]) -> None:
    cargo_units = _cargo_units(policy)
    cargo_ids = {unit.identifier for unit in cargo_units}
    root_manifest = load_toml(root / "Cargo.toml", "Cargo.toml", errors)
    if root_manifest is None:
        return
    workspace = root_manifest.get("workspace")
    if not isinstance(workspace, dict):
        errors.append("Cargo.toml must define [workspace]")
        return
    expected_members = {unit.root for unit in cargo_units}
    members = workspace.get("members")
    if not isinstance(members, list) or set(members) != expected_members:
        errors.append("Cargo workspace members disagree with declared Rust units")
    for unit in cargo_units:
        payload = load_toml(root / unit.manifest, unit.manifest, errors)
        if payload is None:
            continue
        package = payload.get("package")
        if not isinstance(package, dict) or package.get("name") != unit.identifier:
            errors.append(f"Cargo package name disagrees with unit id: {unit.identifier}")
        actual = dependency_names(payload) & cargo_ids
        allowed = set(unit.allowed_dependencies) & cargo_ids
        if actual != allowed:
            errors.append(
                f"Rust dependency graph disagrees for {unit.identifier}: "
                f"actual={sorted(actual)}, policy={sorted(allowed)}"
            )
        codex = {name for name in dependency_names(payload) if name.startswith("codex-")}
        allowed_codex = set(unit.allowed_vendored_dependencies)
        if codex != allowed_codex:
            errors.append(
                f"vendored dependency graph disagrees for {unit.identifier}: "
                f"actual={sorted(codex)}, policy={sorted(allowed_codex)}"
            )


def check_javascript(root: Path, policy: Policy, paths: list[str], errors: list[str]) -> None:
    package_units: list[tuple[Unit, dict[str, object]]] = []
    for unit in policy.units:
        path = root / unit.root / "package.json"
        if not path.is_file():
            continue
        package = load_json(path, f"{unit.root}/package.json", errors)
        if package:
            package_units.append((unit, package))
    workspace_units = [(unit, package) for unit, package in package_units if unit.kind != "image"]
    root_package = load_json(root / "package.json", "package.json", errors)
    if root_package:
        expected = {unit.root for unit, _package in workspace_units}
        workspaces = root_package.get("workspaces")
        if not isinstance(workspaces, list) or set(workspaces) != expected:
            errors.append("root npm workspaces disagree with declared JavaScript units")
        if root_package.get("private") is not True:
            errors.append("root npm workspace must be private")
        try:
            node_version = (root / ".node-version").read_text(encoding="utf-8").strip()
        except (OSError, UnicodeError):
            node_version = ""
        engines = root_package.get("engines")
        if not isinstance(engines, dict) or engines.get("node") != node_version:
            errors.append("root npm engines.node must match .node-version")
        npm_version = engines.get("npm") if isinstance(engines, dict) else None
        if not isinstance(npm_version, str) or root_package.get("packageManager") != f"npm@{npm_version}":
            errors.append("root npm packageManager must match engines.npm")
        scripts = root_package.get("scripts")
        if not isinstance(scripts, dict) or any(
            scripts.get(name) != command for name, command in ROOT_SCRIPT_PROJECTIONS.items()
        ):
            errors.append("root npm verification scripts must project to the UI workspace")
    lock = load_json(root / "package-lock.json", "package-lock.json", errors)
    if lock:
        packages = lock.get("packages")
        if not isinstance(packages, dict):
            errors.append("package-lock.json must contain packages metadata")
        else:
            for unit, _package in workspace_units:
                if unit.root not in packages:
                    errors.append(f"package-lock.json is missing workspace unit: {unit.root}")
    root_ts = load_json(root / "tsconfig.json", "tsconfig.json", errors)
    expected_refs = {
        f"./{unit.root}"
        for unit, _ in workspace_units
        if (root / unit.root / "tsconfig.json").is_file()
    }
    if root_ts:
        refs = root_ts.get("references")
        actual_refs = (
            {
                item.get("path")
                for item in refs
                if isinstance(item, dict) and isinstance(item.get("path"), str)
            }
            if isinstance(refs, list)
            else set()
        )
        if actual_refs != expected_refs or root_ts.get("files") != []:
            errors.append("root TypeScript config must contain references only for workspace projects")
    internal_names = {package.get("name"): unit for unit, package in package_units}
    for unit, package in workspace_units:
        if unit.root == "ui":
            scripts = package.get("scripts")
            if not isinstance(scripts, dict) or scripts.get("tauri:build") != UI_TAURI_BUILD:
                errors.append("UI Tauri build must use paths relative to the npm workspace")
        tsconfig = load_json(root / unit.root / "tsconfig.json", f"{unit.root}/tsconfig.json", errors)
        if tsconfig:
            options = tsconfig.get("compilerOptions")
            if not isinstance(options, dict) or options.get("composite") is not True:
                errors.append(f"TypeScript unit must enable composite: {unit.identifier}")
            if not isinstance(options, dict) or options.get("declaration") is not True:
                errors.append(f"TypeScript unit must enable declarations: {unit.identifier}")
            if tsconfig.get("extends") != "../tsconfig.base.json":
                errors.append(f"TypeScript unit must extend the root baseline: {unit.identifier}")
        dependencies: set[str] = set()
        for section in ("dependencies", "devDependencies", "peerDependencies"):
            table = package.get(section)
            if isinstance(table, dict):
                dependencies.update(name for name in table if name in internal_names)
        actual_ids = {internal_names[name].identifier for name in dependencies}
        if actual_ids != set(unit.allowed_dependencies):
            errors.append(f"JavaScript dependency graph disagrees for {unit.identifier}")
        if unit.kind == "reusable" and not isinstance(package.get("exports"), (str, dict)):
            errors.append(f"reusable JavaScript unit needs package exports: {unit.identifier}")
    names = {name for name in internal_names if isinstance(name, str)}
    for unit, _package in workspace_units:
        for relative in paths:
            if not relative.startswith(unit.root + "/"):
                continue
            source = root / relative
            if source.suffix.lower() not in {".js", ".jsx", ".mjs", ".ts", ".tsx"}:
                continue
            if not source.is_file():
                continue
            text = source.read_text(encoding="utf-8", errors="replace")
            for imported in SOURCE_IMPORT.findall(text):
                if any(imported.startswith(name + "/") for name in names):
                    errors.append(f"deep import bypasses a package public API: {relative}: {imported}")
