#!/usr/bin/env python3
"""Adversarial fixtures for repository topology and dependency enforcement."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path

from check_repo_layout import check_repository


class LayoutFixture:
    def __init__(self, root: Path) -> None:
        self.root = root
        subprocess.run(
            ["git", "init", "-b", "main"],
            cwd=root,
            check=True,
            stdout=subprocess.DEVNULL,
        )
        for directory in (
            ".github",
            "crates/app/src",
            "crates/core/src",
            "docs",
            "images",
            "scripts",
            "src-tauri/src",
            "ui/src",
        ):
            (root / directory).mkdir(parents=True, exist_ok=True)
        self.policy = {
            "schema_version": 1,
            "source_limits": {
                "counting": "physical_lines_including_blank_and_comment_lines",
                "review_lines": 300,
                "hard_lines": 500,
                "review_bytes": 32768,
                "hard_bytes": 65536,
            },
            "included_extensions": [".json", ".py", ".rs", ".sh", ".toml", ".ts", ".yml"],
            "included_source_paths": [],
            "startup_context": {
                "files": ["AGENTS.md", "PROGRESS.md", "feature_list.json", "CLAUDE.md", ".github/copilot-instructions.md"],
                "state_reads": ["PROGRESS.md", "feature_list.json"],
                "loading_hops": {"AGENTS.md": 0, "PROGRESS.md": 0, "feature_list.json": 0, "CLAUDE.md": 1, ".github/copilot-instructions.md": 1},
                "warning_percent": 90,
                "hard_lines": 700,
                "hard_bytes": 49152,
            },
            "modularization_review_file": "docs/modularization-reviews.json",
            "structure": {
                "allowed_source_roots": [
                    ".github",
                    "crates",
                    "docs",
                    "images",
                    "scripts",
                    "src-tauri",
                    "ui",
                ],
                "unitized_source_roots": ["crates", "images", "src-tauri", "ui"],
                "allowed_root_source_files": [
                    "Cargo.toml",
                    "feature_list.json",
                    "init.sh",
                    "package.json",
                    "pins.toml",
                    "repository-structure-policy.json",
                    "rust-toolchain.toml",
                    "tsconfig.base.json",
                    "tsconfig.json",
                ],
                "forbidden_root_source_files": True,
                "architecture_check": "scripts/python.sh scripts/check_repo_layout.py",
                "source_limit_check": "scripts/python.sh scripts/check_file_limits.py",
                "units": [
                    {
                        "id": "core",
                        "kind": "reusable",
                        "root": "crates/core",
                        "manifest": "crates/core/Cargo.toml",
                        "public_entrypoints": ["crates/core/src/lib.rs"],
                        "allowed_dependencies": [],
                    },
                    {
                        "id": "app",
                        "kind": "deployable",
                        "root": "crates/app",
                        "manifest": "crates/app/Cargo.toml",
                        "public_entrypoints": ["crates/app/src/main.rs"],
                        "allowed_dependencies": ["core"],
                    },
                    {
                        "id": "tauri-app",
                        "kind": "deployable",
                        "root": "src-tauri",
                        "manifest": "src-tauri/Cargo.toml",
                        "public_entrypoints": ["src-tauri/src/lib.rs", "src-tauri/tauri.conf.json"],
                        "allowed_dependencies": ["core"],
                    },
                    {
                        "id": "ui-app",
                        "kind": "deployable",
                        "root": "ui",
                        "manifest": "ui/package.json",
                        "public_entrypoints": ["ui/src/main.ts"],
                        "allowed_dependencies": [],
                    },
                ],
            },
            "non_hand_authored_exclusions": [
                {
                    "path": "Cargo.lock",
                    "category": "lock",
                    "owner": "Cargo workspace",
                    "reason": "generated dependency lock",
                },
                {
                    "path": "package-lock.json",
                    "category": "lock",
                    "owner": "npm workspace",
                    "reason": "generated dependency lock",
                },
            ],
            "hand_authored_inclusions": [],
            "retired_paths": [],
            "exceptions": [],
        }
        for unit in self.policy["structure"]["units"]:
            unit["test_entrypoints"] = [f"{unit['root']}/contract.test.sh"]
            unit["documentation"] = [f"{unit['root']}/README.md"]
        self.write_contract_files()
        self.write_policy()

    def write(self, relative: str, content: str = "fixture\n") -> None:
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        if relative.endswith(".sh"):
            path.chmod(0o755)

    def write_json(self, relative: str, payload: object) -> None:
        self.write(relative, json.dumps(payload, indent=2) + "\n")

    def write_policy(self) -> None:
        self.write_json("repository-structure-policy.json", self.policy)

    def cargo_manifest(self, root: str, name: str, dependencies: list[str]) -> None:
        deps = "\n".join(f'{dependency} = "1"' for dependency in dependencies)
        self.write(
            f"{root}/Cargo.toml",
            f'[package]\nname = "{name}"\nversion = "1.0.0"\n\n[dependencies]\n{deps}\n',
        )

    def write_contract_files(self) -> None:
        architecture = "\n".join(
            (
                "# Repository Architecture",
                "2,000,000 lines and 50,000 files",
                "`crates/` is the shared-library and CLI/TUI Rust source root",
                "`src-tauri/` is the separate deployable desktop application crate",
                "reviewed at 300 physical lines or 32,768 bytes",
                "more than 500 physical lines or 65,536 bytes",
                "scripts/python.sh scripts/check_repo_layout.py",
                "scripts/python.sh scripts/check_file_limits.py",
                "Product runtime architecture remains in `docs/ARCHITECTURE.md`",
            )
        )
        self.write("ARCHITECTURE.md", architecture + "\n")
        self.write("docs/ARCHITECTURE.md")
        self.write(
            "docs/harness/boundaries/component-inventory.md",
            "# Component inventory\n\nClaude and Codex API-key brokers.\n",
        )
        for relative in (
            ".python-version",
            "AGENTS.md",
            "Cargo.lock",
            "PROGRESS.md",
            "feature_list.json",
            "init.sh",
            "pins.toml",
            "rust-toolchain.toml",
            "scripts/cargo.sh",
            "scripts/init_repository.py",
            "scripts/python.sh",
            "scripts/verify-all.sh",
            "tsconfig.base.json",
        ):
            self.write(relative)
        self.write(
            "scripts/init_repository.py",
            'run(root, ["npm", "run", "build"])\n'
            + 'run(root, ["scripts/cargo.sh", "check", "--workspace", "--locked"])\n',
        )
        self.write_json(
            "feature_list.json",
            {
                "features": [
                    {"id": "multi-provider-broker", "status": "passing"},
                    {"id": "ssh-forwarding-boundary", "status": "passing"},
                ]
            },
        )
        self.write_json(
            "docs/harness/boundaries/feature-privacy-labels.json",
            {
                "features": [
                    {
                        "id": "multi-provider-broker",
                        "defaultDataFlow": "Claude and Codex use OPENAI_API_KEY and ANTHROPIC_API_KEY.",
                    },
                    {
                        "id": "ssh-forwarding-boundary",
                        "defaultDataFlow": "The SSH-agent socket is forwarded. Raw SSH keys stay outside, and an unusable private socket fails safely.",
                    },
                ]
            },
        )
        self.write("CLAUDE.md", "@AGENTS.md\n")
        self.write(
            ".github/copilot-instructions.md",
            "Before any work, open [the root repository contract](../AGENTS.md). "
            "Read [`PROGRESS.md`](../PROGRESS.md) and "
            "[`feature_list.json`](../feature_list.json). This compatibility "
            "file does not own repository policy.\n",
        )
        self.write(
            "AGENTS.md",
            "the closest scoped\n`ARCHITECTURE.md`\n`PROGRESS.md`\n`feature_list.json`\n`./init.sh`\n`scripts/verify-all.sh`\n",
        )
        self.write(".node-version", "1.0.0\n")
        self.write("crates/core/src/lib.rs", "pub fn core() {}\n")
        self.write("crates/app/src/main.rs", "fn main() {}\n")
        self.write("src-tauri/src/lib.rs", "pub fn run() {}\n")
        self.write_json(
            "src-tauri/tauri.conf.json",
            {"build": {"beforeBuildCommand": "npm run build", "frontendDist": "../ui/dist"}},
        )
        self.write("ui/src/main.ts", "export {}\n")
        self.cargo_manifest("crates/core", "core", [])
        self.cargo_manifest("crates/app", "app", ["core"])
        self.cargo_manifest("src-tauri", "tauri-app", ["core"])
        members = '\n'.join(f'  "{item}",' for item in ("crates/app", "crates/core", "src-tauri"))
        self.write("Cargo.toml", f"[workspace]\nmembers = [\n{members}\n]\nresolver = \"3\"\n")
        self.write_json(
            "package.json",
            {
                "name": "fixture",
                "private": True,
                "packageManager": "npm@1.0.0",
                "workspaces": ["ui"],
                "engines": {"node": "1.0.0", "npm": "1.0.0"},
                "scripts": {
                    "build": "npm --workspace runhaven-ui run build",
                    "check": "npm --workspace runhaven-ui run check",
                    "tauri:build": "npm --workspace runhaven-ui run tauri:build",
                    "test": "npm --workspace runhaven-ui test",
                    "test:e2e": "npm --workspace runhaven-ui run test:e2e",
                },
            },
        )
        self.write_json(
            "ui/package.json",
            {
                "name": "ui-app",
                "private": True,
                "dependencies": {},
                "scripts": {
                    "tauri:build": "cd .. && CARGO_TARGET_DIR=\"$PWD/target\" ./ui/node_modules/.bin/tauri build --config src-tauri/tauri.conf.json --debug --no-bundle"
                },
            },
        )
        self.write_json(
            "package-lock.json",
            {"lockfileVersion": 3, "packages": {"": {}, "ui": {"name": "ui-app"}}},
        )
        self.write_json("tsconfig.json", {"files": [], "references": [{"path": "./ui"}]})
        self.write_json(
            "ui/tsconfig.json",
            {
                "extends": "../tsconfig.base.json",
                "compilerOptions": {"composite": True, "declaration": True},
            },
        )
        for unit in self.policy["structure"]["units"]:
            self.write(unit["test_entrypoints"][0], "#!/bin/sh\nexit 0\n")
            self.write(unit["documentation"][0], f"# {unit['id']}\n")
class RepoLayoutTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.fixture = LayoutFixture(self.root)
    def tearDown(self) -> None:
        self.temp.cleanup()

    def errors(self) -> list[str]:
        return check_repository(self.root)

    def assert_error(self, text: str) -> None:
        errors = self.errors()
        self.assertTrue(any(text in error for error in errors), errors)

    def test_valid_repository_layout_passes(self) -> None:
        self.assertEqual(self.errors(), [])

    def test_rejects_undeclared_crate_and_source(self) -> None:
        self.fixture.write("crates/rogue/Cargo.toml", "[package]\nname='rogue'\nversion='1'\n")
        self.fixture.write("crates/rogue.rs", "fn rogue() {}\n")
        self.assert_error("outside a declared unit: crates/rogue/Cargo.toml")
        self.assert_error("outside a declared unit: crates/rogue.rs")

    def test_rejects_missing_manifest_or_entrypoint(self) -> None:
        (self.root / "crates/core/src/lib.rs").unlink()
        self.assert_error("unit core is missing regular contract file")

    def test_rejects_nonexecutable_entrypoint(self) -> None:
        (self.root / "init.sh").chmod(0o644)
        self.assert_error("required repository entrypoint is not executable")

    def test_rejects_missing_unit_test_or_documentation(self) -> None:
        (self.root / "crates/core/contract.test.sh").unlink()
        (self.root / "crates/core/README.md").unlink()
        self.assert_error("unit core is missing regular contract file")

    def test_rejects_dependency_direction_drift(self) -> None:
        self.fixture.cargo_manifest("crates/core", "core", ["app"])
        self.assert_error("Rust dependency graph disagrees for core")

    def test_resolves_allowed_and_forbidden_cargo_dependency_aliases(self) -> None:
        self.fixture.write(
            "crates/app/Cargo.toml",
            '[package]\nname = "app"\nversion = "1.0.0"\n\n[dependencies.runtime]\npackage = "core"\nversion = "1"\n',
        )
        self.assertEqual(self.errors(), [])
        self.fixture.write(
            "crates/core/Cargo.toml",
            '[package]\nname = "core"\nversion = "1.0.0"\n\n[dependencies.hidden]\npackage = "app"\nversion = "1"\n',
        )
        self.assert_error("Rust dependency graph disagrees for core")

    def test_rejects_undeclared_vendored_dependency(self) -> None:
        self.fixture.write(
            "crates/core/Cargo.toml",
            '[package]\nname = "core"\nversion = "1.0.0"\n\n[dependencies]\ncodex-shim = "1"\n',
        )
        self.assert_error("vendored dependency graph disagrees for core")

    def test_rejects_policy_cycle(self) -> None:
        self.fixture.policy["structure"]["units"][0]["allowed_dependencies"] = ["app"]
        self.fixture.write_policy()
        self.assert_error("unit dependency cycle")

    def test_rejects_workspace_member_drift(self) -> None:
        self.fixture.write("Cargo.toml", '[workspace]\nmembers = ["crates/core"]\n')
        self.assert_error("Cargo workspace members disagree")

    def test_rejects_staged_and_worktree_contract_divergence(self) -> None:
        subprocess.run(["git", "add", "."], cwd=self.root, check=True)
        valid = (self.root / "Cargo.toml").read_text(encoding="utf-8")
        self.fixture.write("Cargo.toml", '[workspace]\nmembers = []\n')
        subprocess.run(["git", "add", "Cargo.toml"], cwd=self.root, check=True)
        self.fixture.write("Cargo.toml", valid)
        self.assert_error("staged and working-tree content diverge")

    def test_rejects_npm_workspace_or_typescript_drift(self) -> None:
        package = json.loads((self.root / "package.json").read_text(encoding="utf-8"))
        package["workspaces"] = []
        self.fixture.write_json("package.json", package)
        self.assert_error("root npm workspaces disagree")
        package["workspaces"] = ["ui"]
        self.fixture.write_json("package.json", package)
        self.fixture.write_json("ui/tsconfig.json", {"compilerOptions": {"composite": False}})
        self.assert_error("TypeScript unit must enable composite")

    def test_rejects_npm_runtime_projection_drift(self) -> None:
        package = json.loads((self.root / "package.json").read_text(encoding="utf-8"))
        package["packageManager"] = "npm@2.0.0"
        package["engines"] = {"node": "2.0.0", "npm": "2.0.0"}
        self.fixture.write_json("package.json", package)
        self.assert_error("root npm engines.node must match .node-version")

    def test_rejects_tauri_workspace_path_drift(self) -> None:
        package = json.loads((self.root / "ui/package.json").read_text(encoding="utf-8"))
        package["scripts"]["tauri:build"] = "tauri build --config src-tauri/tauri.conf.json"
        self.fixture.write_json("ui/package.json", package)
        self.assert_error("UI Tauri build must use paths relative to the npm workspace")

    def test_rejects_tauri_frontend_build_drift(self) -> None:
        self.fixture.write_json(
            "src-tauri/tauri.conf.json",
            {"build": {"beforeBuildCommand": "npm --prefix .. run build", "frontendDist": "../ui/dist"}},
        )
        self.assert_error("Tauri beforeBuildCommand must use the root npm workspace")

    def test_rejects_fresh_clone_build_order_drift(self) -> None:
        self.fixture.write(
            "scripts/init_repository.py",
            'run(root, ["scripts/cargo.sh", "check", "--workspace", "--locked"])\n'
            + 'run(root, ["npm", "run", "build"])\n',
        )
        self.assert_error("initialization must build the Tauri frontend before checking the Rust workspace")

    def test_rejects_hosted_ci_artifacts(self) -> None:
        self.fixture.write(".github/workflows/ci.yml", "name: CI\n")
        self.assert_error("hosted CI is disabled; workflow artifact must be absent")

    def test_rejects_unsupported_platform_projection(self) -> None:
        marker = '#[cfg(target_os = "' + "win" + 'dows")]\nfn alternate() {}\n'
        self.fixture.write("crates/app/src/platform.rs", marker)
        self.assert_error("unsupported platform projection")

    def test_rejects_retired_path(self) -> None:
        self.fixture.policy["retired_paths"] = [".agents"]
        self.fixture.write_policy()
        (self.root / ".agents").mkdir()
        self.assert_error("retired repository path must stay absent: .agents")

    def test_rejects_architecture_owner_drift(self) -> None:
        self.fixture.write("ARCHITECTURE.md", "# drift\n")
        self.assert_error("ARCHITECTURE.md is missing canonical contract text")

    def test_rejects_stale_component_inventory_label(self) -> None:
        self.fixture.write(
            "docs/harness/boundaries/component-inventory.md",
            "# Component inventory\n\nAuth broker prototype\n",
        )
        self.assert_error("component inventory retains stale architecture label")

    def test_rejects_compatibility_instruction_drift(self) -> None:
        self.fixture.write("CLAUDE.md", "# duplicated policy\n")
        self.assert_error("CLAUDE.md must remain a thin import")
        self.fixture.write("CLAUDE.md", "@AGENTS.md\n")
        self.fixture.write(".github/copilot-instructions.md", "# duplicated policy\n")
        self.assert_error("Copilot instructions must remain a thin route")

    def test_rejects_direct_tauri_container_process(self) -> None:
        self.fixture.write("src-tauri/src/direct.rs", 'fn bad() { Command::new("container"); }\n')
        self.assert_error("Tauri bypasses the runhaven-core runtime owner")

    def test_rejects_stale_security_privacy_projection(self) -> None:
        privacy = json.loads(
            (self.root / "docs/harness/boundaries/feature-privacy-labels.json").read_text(
                encoding="utf-8"
            )
        )
        privacy["features"][0]["defaultDataFlow"] = "Codex prototype only."
        self.fixture.write_json("docs/harness/boundaries/feature-privacy-labels.json", privacy)
        self.assert_error("privacy projection disagrees with passing security feature")


if __name__ == "__main__":
    unittest.main(verbosity=2)
