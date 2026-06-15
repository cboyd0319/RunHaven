# Component Inventory

Generated: 2026-06-14

This file records the project boundaries the harness knows about. It is an
inventory, not permission to mutate nested projects.

## Detected Components

- Root Python package: `pyproject.toml`, `src/runhaven/`,
  `src/runhaven/active_records.py`, `src/runhaven/cache_paths.py`,
  `src/runhaven/setup_guide.py`, `src/runhaven/validators.py`,
  `tests/`, `scripts/check_pins.py`, and
  `scripts/provider_egress_smoke.py`.
- Broker smoke harness: `scripts/codex_broker_smoke.py`.
- Runtime cache ledgers and markers: `egress-policy.jsonl`,
  `auth-broker.jsonl`, `runs.jsonl`, and `active-runs/*.json` under the
  RunHaven cache root.
- Bundled image templates:
  `src/runhaven/images/base/`,
  `src/runhaven/images/claude/`,
  `src/runhaven/images/codex/`,
  `src/runhaven/images/gemini/`,
  `src/runhaven/images/antigravity/`,
  `src/runhaven/images/copilot/`, and
  `src/runhaven/images/common/`.
- Harness operating layer: `feature_list.json`, `progress.md`,
  `session-handoff.md`, `init.sh`, and `docs/harness/`.
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
- Product runtime and contributor verification support is macOS 26+ on Apple
  silicon with Python 3.13+ and Apple `container` 1.0.0.
- Do not add Windows or Linux verification targets; unsupported platforms
  should fail closed or be documented as unsupported.

## Manual Additions

Add components here when discovery cannot infer them safely, such as generated
packages, vendored modules, examples, infrastructure roots, or docs-only
subprojects that have their own release or verification path.
