# Component Inventory

Generated: 2026-06-15
Reviewed: 2026-06-15

This file records the project boundaries the harness knows about. It is an
inventory, not permission to mutate every nested surface.

## Effective Agent Boundary

For RunHaven, changing any of these changes effective agent behavior:

- root and platform instruction files;
- `runhaven` CLI command planning or execution;
- Apple `container` invocation defaults;
- workspace mounts, worktree handling, state volumes, or active-run markers;
- provider allowlists, proxy behavior, auth broker behavior, or SSH/env
  passthrough;
- bundled image templates and image package locks;
- verification entrypoints, CI, pin checks, and harness sensors.

Treat those changes as product changes with scope, verification, and rollback.

## Detected Workspace Markers

- Root Python package: `pyproject.toml`.
- Python runtime pin: `.python-version`.
- Exact dependency and runtime pin ledger: `pins.toml`.
- MacOS-only CI workflow: `.github/workflows/ci.yml`.
- Multiple nested image/package manifests under `src/runhaven/images/`.
- Harness operating layer under `docs/harness/`.

## Detected Routing Markers

- Root canonical instructions: `AGENTS.md`.
- Thin platform routers: `CLAUDE.md`, `GEMINI.md`, and
  `.github/copilot-instructions.md`.
- Product docs: `README.md`, `docs/INSTALLATION.md`, `docs/USAGE.md`,
  `docs/CAPABILITIES.md`, `docs/SECURITY_MODEL.md`, `docs/ARCHITECTURE.md`,
  `docs/AUTH_BROKER.md`, `docs/PROVIDER_ENDPOINTS.md`, and `docs/PINNING.md`.
- Product roadmap: `docs/ROADMAP.md`.
- Harness roadmap and sensors: `docs/harness/roadmap.md` and
  `docs/harness/sensor-registry.md`.

## Detected Components

| Component | Primary Files | Review Notes |
| --- | --- | --- |
| CLI entrypoint and parser | `src/runhaven/cli.py`, `src/runhaven/cli_parser.py`, `src/runhaven/__main__.py` | Keep argparse construction side-effect light. CLI behavior changes need focused command tests plus relevant help smokes. |
| Planning and validation | `src/runhaven/plans.py`, `src/runhaven/validators.py`, `src/runhaven/project_checks.py` | Security-sensitive command construction surface. Use exact subprocess argument lists and fail closed on unsafe inputs. |
| Provider network runtime | `src/runhaven/egress.py`, `src/runhaven/provider_runtime.py`, `src/runhaven/provider_endpoints.py`, `src/runhaven/provider_observability.py` | Provider egress is a core safety boundary. Changes need focused proxy/policy tests and, when behavior changes, Apple `container` smokes. |
| Auth broker prototype | `src/runhaven/auth_broker.py`, `src/runhaven/auth_profiles.py`, `scripts/codex_broker_smoke.py`, `docs/AUTH_BROKER.md` | Secret-handling boundary. Do not read or persist raw credential values in diagnostics, plans, logs, or run records. |
| Run records and active runs | `src/runhaven/run_history.py`, `src/runhaven/active_records.py`, `src/runhaven/active_commands.py`, `src/runhaven/active_repair.py`, `src/runhaven/git_metadata.py` | Observability must stay secret-free and avoid raw command lines, env values, prompts, request bodies, and token values. |
| Worktree lifecycle | `src/runhaven/worktrees.py`, `src/runhaven/worktree_lifecycle.py` | Data-loss boundary. Keep source-checkout validation, RunHaven-owned branch checks, and explicit merge/discard recovery paths. |
| State and network repair UX | `src/runhaven/session_state.py`, `src/runhaven/image_commands.py`, `src/runhaven/network_commands.py`, `src/runhaven/setup_guide.py`, `src/runhaven/doctor.py` | Repair commands should preview before deletion, mutate only RunHaven-owned resources, and print exact next steps. |
| Bundled images | `src/runhaven/images/base/`, `src/runhaven/images/claude/`, `src/runhaven/images/codex/`, `src/runhaven/images/gemini/`, `src/runhaven/images/antigravity/`, `src/runhaven/images/copilot/`, `src/runhaven/images/common/` | Keep image tags, npm packages, Debian snapshot inputs, non-root user setup, and source-digest labels pinned and reviewed. |
| Pin policy | `scripts/check_pins.py`, `scripts/npm_pin_policy.py`, `pins.toml`, `requirements-dev.txt` | Pin checks are a release gate. Dependency and runner changes need primary-source evidence. |
| Runtime smoke scripts | `scripts/provider_egress_smoke.py`, `scripts/codex_broker_smoke.py` | Live smokes may use network or disposable credentials. Keep defaults safe and clearly skipped when required inputs are absent. |
| Test suite | `tests/` | Focused `unittest` modules cover CLI, plans, egress, images, state, worktrees, auth, and repo policy. Do not add pytest-only assumptions. |
| Harness operating layer | `AGENTS.md`, `feature_list.json`, `progress.md`, `session-handoff.md`, `docs/harness/` | Keep root instructions compact and move durable operating detail into focused harness docs. |
| Human documentation | `README.md`, `SECURITY.md`, `CONTRIBUTING.md`, `docs/` | Docs are product surfaces. Keep macOS 26+ only support, Apple `container` 1.0.0, security boundaries, and command examples aligned with code. |
| Project asset | `docs/assets/logo.png` | Required README asset and manifest entry. |

## Routing Rules

- Treat `.` as the root project boundary unless a task explicitly names a
  nested component.
- Before editing a nested component, inspect that component's manifests,
  tests, lockfiles, and instructions.
- Run the smallest verification command that covers the changed component,
  then run the root harness checks when root behavior or shared policy can
  change.
- Do not install dependencies, run package scripts, write generated files, or
  mutate Apple `container` resources unless the task needs it and the command
  is documented.
- Product runtime and contributor verification support is macOS 26+ on Apple
  silicon with Python 3.13+ and Apple `container` 1.0.0.
- Do not add Windows or Linux verification targets; unsupported platforms
  should fail closed or be documented as unsupported.
- Do not commit machine-local absolute paths, private checkout paths, secret
  values, raw Apple `container inspect` payloads, or long command output.

## Manual Additions

Add components here when discovery cannot infer them safely, such as generated
packages, vendored modules, examples, infrastructure roots, docs-only
subprojects, or source ledgers that have their own release or verification
path.
