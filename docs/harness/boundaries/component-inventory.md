# Component Inventory

Generated: 2026-06-16
Reviewed: 2026-06-27

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
- verification entrypoints, future CI, pin checks, and harness sensors.

Treat those changes as product changes with scope, verification, and rollback.

## Detected Workspace Markers

- Root Rust package: `Cargo.toml`.
- Tauri desktop crate: `src-tauri/Cargo.toml`.
- Svelte/Vite frontend package: `ui/package.json`.
- Rust toolchain pin: `rust-toolchain.toml`.
- Exact dependency and runtime pin ledger: `pins.toml`.
- Frontend lockfile: `ui/package-lock.json`.
- GitHub Actions CI disabled during alpha/pre-release; no active workflow files.
- Multiple nested image/package manifests under `images/`.
- Harness operating layer under `docs/harness/`.

## Detected Routing Markers

- Root canonical instructions: `AGENTS.md`.
- Thin platform routers: `CLAUDE.md`, `GEMINI.md`, and
  `.github/copilot-instructions.md`.
- Product docs: `README.md`, `docs/INSTALLATION.md`, `docs/USAGE.md`,
  `docs/CAPABILITIES.md`, `docs/SECURITY_MODEL.md`, `docs/ARCHITECTURE.md`,
  `docs/AUTH_BROKER.md`, `docs/PROVIDER_ENDPOINTS.md`, and `docs/PINNING.md`.
- Product roadmap: `docs/ROADMAP.md`.
- Release ladder and gap analysis: `docs/V1_RELEASE_PLAN.md` and
  `docs/RELEASE_GAP_ANALYSIS.md`.
- Harness overview and sensors: `docs/harness/README.md` and
  `docs/harness/feedback/sensor-registry.md`.

## Detected Components

| Component | Primary Files | Review Notes |
| --- | --- | --- |
| CLI entrypoint and parser | `src/main.rs`, `src/runhaven/cli/app.rs`, `src/runhaven/cli/args.rs`, `src/runhaven/cli/diagnostics.rs`, `src/runhaven/cli/setup.rs` | Keep clap construction side-effect light. `cli/` owns argument dispatch and human presentation only; shared runtime, policy, diagnostics data, and prerequisite checks live outside `cli/`. CLI behavior changes need focused command tests plus relevant help smokes. |
| Terminal UI | `src/runhaven/cli/tui/`, `docs/plans/tui-build-plan.md`, `docs/plans/tui-architecture.md`, `docs/plans/ratatui-brand-graphics.md` | First-class interactive terminal surface over shared library data. TUI widgets must not parse CLI prose or own policy/runtime truth; consume planner, records, diagnostics, doctor, provider, and active-run data modules. |
| Desktop shell | `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/`, `src-tauri/capabilities/` | Tauri WebView is untrusted. Keep commands typed, capabilities explicit, and privileged behavior in Rust. No generic shell, filesystem, process, HTTP, or Apple `container` bridge. |
| Frontend UI | `ui/package.json`, `ui/package-lock.json`, `ui/src/`, `ui/vite.config.ts` | Operational desktop UI. Keep secure defaults shortest, supported advanced choices warning-based, and command helpers typed. Frontend must not store secrets, raw logs, command lines, prompts, or workspace contents. |
| Planning and validation | `src/runhaven/runtime/plans/`, `src/runhaven/support/validators.rs`, `src/runhaven/support/project_checks.rs` | Security-sensitive command construction surface. Use exact subprocess argument lists and fail closed on unsafe inputs. |
| Provider network runtime | `src/runhaven/provider/egress.rs`, `src/runhaven/provider/runtime.rs`, `src/runhaven/provider/endpoints.rs`, `src/runhaven/provider/observability.rs` | Provider egress is a core safety boundary. Changes need focused proxy/policy tests and, when behavior changes, Apple `container` smokes. |
| Auth broker prototype | `src/runhaven/provider/auth_broker.rs`, `src/runhaven/provider/auth_broker/`, `src/runhaven/provider/auth_profiles.rs`, `docs/AUTH_BROKER.md` | Secret-handling boundary. Do not read or persist raw credential values in diagnostics, plans, logs, or run records. |
| Secret-free diagnostics data | `src/runhaven/diagnostics.rs`, `src/runhaven/provider/observability.rs`, `src/runhaven/provider/auth_profiles.rs` | Shared log readers/status payloads are presentation-neutral and must stay secret-free. CLI/Tauri/TUI renderers may adapt them but should not own the source data. |
| Run launch, records, and active runs | `src/runhaven/runtime/launch.rs`, `src/runhaven/runtime/lock.rs`, `src/runhaven/records/mod.rs`, `src/runhaven/records/io.rs`, `src/runhaven/records/run_history.rs`, `src/runhaven/records/run_history/`, `src/runhaven/runtime/active/`, `src/runhaven/support/git.rs` | Launch behavior, state-volume locking, and observability must stay secret-free and avoid raw command lines, env values, prompts, request bodies, and token values. `records/` is the facade; `records/history` is compatibility only. |
| Worktree lifecycle | `src/runhaven/runtime/worktrees/` | Data-loss boundary. Keep source-checkout validation, RunHaven-owned branch checks, and explicit merge/discard recovery paths. |
| Host readiness and repair UX | `src/runhaven/doctor.rs`, `src/runhaven/doctor/runtime_pins.rs`, `src/runhaven/runtime/session_state.rs`, `src/runhaven/runtime/state.rs`, `src/runhaven/image/doctor.rs`, `src/runhaven/runtime/network.rs`, `src/runhaven/cli/setup.rs` | `doctor` is shared host-readiness data, not CLI presentation. Repair commands should preview before deletion, mutate only RunHaven-owned resources, and print exact next steps. |
| Bundled images | `images/base/`, `images/claude/`, `images/codex/`, `images/gemini/`, `images/antigravity/`, `images/copilot/`, `images/common/` | Keep image tags, npm packages, Debian snapshot inputs, non-root user setup, and source-digest labels pinned and reviewed. |
| Pin policy | `src/runhaven/harness/pins.rs`, `src/bin/runhaven-check-pins.rs`, `pins.toml`, `Cargo.toml`, `Cargo.lock`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `ui/package.json`, `ui/package-lock.json` | Pin checks are a release gate. Dependency changes and any future workflow or runner changes need primary-source evidence. |
| Test suite | `tests/` plus module tests | Focused Rust tests cover CLI, plans, egress, images, state, worktrees, auth, and repo policy. |
| Harness operating layer | `AGENTS.md`, `feature_list.json`, `current-state.md`, `docs/harness/` | Keep the three startup files compact and load focused harness docs only on demand. |
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
  silicon with Rust 1.96.0 and Apple `container` 1.0.0.
- Do not add Windows or Linux verification targets; unsupported platforms
  should fail closed or be documented as unsupported.
- Do not commit machine-local absolute paths, private checkout paths, secret
  values, raw Apple `container inspect` payloads, or long command output.

## Manual Additions

Add components here when discovery cannot infer them safely, such as generated
packages, vendored modules, examples, infrastructure roots, docs-only
subprojects, or source ledgers that have their own release or verification
path.
