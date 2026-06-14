# Component Inventory

Generated: 2026-06-14

This file records the project boundaries the harness knows about. It is an
inventory, not permission to mutate nested projects.

## Detected Components

- Root Python package: `pyproject.toml`, `src/runhaven/`,
  `tests/`, `scripts/check_pins.py`.
- Bundled image templates:
  `src/runhaven/images/base/`,
  `src/runhaven/images/claude/`,
  `src/runhaven/images/codex/`,
  `src/runhaven/images/gemini/`,
  `src/runhaven/images/antigravity/`,
  `src/runhaven/images/copilot/`, and
  `src/runhaven/images/common/`.
- Harness operating layer: `feature_list.json`, `progress.md`,
  `session-handoff.md`, `init.sh`, `init.ps1`, and `docs/harness/`.
- Human documentation: `README.md`, `SECURITY.md`, `CONTRIBUTING.md`, and
  `docs/`.
- Project assets: `docs/assets/logo.png`.

## Routing Rules

- Treat `.` as the root project boundary unless a task explicitly names a nested
  component.
- Before editing a nested component, inspect that component's own manifests,
  tests, lockfiles, and instructions.
- Run the smallest verification command that covers the changed component, then
  run the root harness check when root behavior or shared policy can change.
- Do not install dependencies, run package scripts, or write generated files in
  nested components unless the task needs it and the command is documented.
- Product runtime support is macOS 26+ on Apple silicon with Python 3.13+ and
  Apple `container` 1.0.0.
- Contributor verification keeps Python import, docs, pin, and
  command-construction behavior reviewable on Windows 11 and Ubuntu 22.04+ where
  Apple `container` runtime checks are not available.
- Generic harness benchmark language may mention macOS 15, Windows 11, and
  Ubuntu 22.04; that does not change this product's macOS 26+ runtime floor.

## Manual Additions

Add components here when discovery cannot infer them safely, such as generated
packages, vendored modules, examples, infrastructure roots, or docs-only
subprojects that have their own release or verification path.
