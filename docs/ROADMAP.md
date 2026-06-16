# Roadmap

This roadmap records both completed foundation slices and remaining product
direction. The live feature status and verification evidence are tracked in
`feature_list.json`, `current-state.md`, and
`docs/harness/evidence/evidence-log.md`.

The consolidated non-UI backlog lives in
[`NON_UI_BACKLOG.md`](NON_UI_BACKLOG.md). Tauri/UI research lives in
[`TAURI_UI_RESEARCH_PLAN.md`](TAURI_UI_RESEARCH_PLAN.md).

## Phase 1: Safe Local Baseline

- Rust CLI package and command
- agent profiles
- dry-run planning
- bundled image templates
- doctor checks
- unit tests for command boundaries

## Phase 2: Network Boundary

- provider network mode integrated into normal `runhaven run`
- live smoke harness for host allowlist proxy on an internal network
- provider-specific egress profiles
- reviewed provider endpoint matrix for auth, provider routing, telemetry,
  package, and optional feature paths
- DNS resolution and unsafe-address rejection inside the provider CONNECT proxy
- provider proxy policy logs for allowed and denied CONNECT decisions
- `runhaven why host ...` diagnostic for provider-host decisions
- grouped blocked-host review with rule, count, run id, and suggested next
  action
- provider-profile smoke support for bundled source-backed hosts
- host-side provider credential broker design with `runhaven auth status` and
  `runhaven auth explain AGENT`
- empty-allowlist regression tests for every network policy mode
- first real host-side provider credential broker implementation
- local proxy option for model credentials
- clear offline and package-install network modes
- live provider auth-flow smokes for optional feature paths
- path-aware provider policy for broad hosts such as `github.com`

## Phase 3: Beginner Install Flow

- one-command bootstrap for Apple `container`
- non-mutating guided first-run setup with exact prerequisite fixes
- plain-language explanations for every requested permission
- `runhaven why` diagnostics for blocked hosts, rejected mounts, sensitive
  defaults, network modes, state volumes, and validation failures
- guided `runhaven setup` first-run flow with profile-specific next commands
- goal-based network selection copy for local-only, provider-only, package
  install, and unrestricted internet use cases

## Phase 4: Agent Coverage

- custom profile file support
- per-agent policy presets
- MCP allowlists
- import/export of project profiles
- generated docs checks for profiles, network modes, provider hosts, pins, and
  macOS 26+ support text
- agent profile investigation docs for required files, network hosts, update
  paths, auth paths, and known blocked operations
- devcontainer metadata import for image planning, with host lifecycle hooks
  disabled unless explicitly approved
- task-language usage recipes for review-only, local tests, provider-only,
  undo, and state reset workflows

## Phase 5: Isolation And Recovery

- run records with `runs list`, `runs show`, `runs log`, `runs diff`,
  `runs keep`, `runs recover`, `runs merge`, `runs discard`, `runs active`,
  `runs status`, `runs attach`, `runs logs-follow`, `runs stop`,
  `runs kill`, `runs repair`, `runs repair --all`, and git change metadata
- structured blocked-host and cleanup event records
- stable JSON output for read-only, run-status, and repair-summary commands
- image doctor, image rebuild, state reset/prune, and managed-network
  list/prune commands
- provider auth broker run records that never print secrets

## Pre-Release Codebase Health

- Rust conversion and modularization completed with source organized under
  `src/runhaven/` by ownership boundary.
- keep the Rust source size guard active so command surfaces, policy logic,
  run-state handling, and verification helpers remain reviewable.
- run the `rust-expert` agent with the Rust skill across the entire repo to
  look for correctness, safety, idiomatic Rust, test, packaging, and
  maintainability issues before broadening the product surface.
- use the `apple-container-expert` agent with the Apple Container skill for
  Apple `container` runtime, networking, source, service, registry, machine,
  and security-boundary work.
- add a short top-of-file description to every maintained script so future
  contributors can see what each script is and what it does before running it.

## Pre-Tauri Apple Container Readiness

- Action the coverage gaps in
  [`docs/APPLE_CONTAINER_GAP_ANALYSIS.md`](APPLE_CONTAINER_GAP_ANALYSIS.md)
  before starting Tauri/UI work.
- Keep the consolidated non-UI backlog in
  [`NON_UI_BACKLOG.md`](NON_UI_BACKLOG.md) current before promoting any
  candidate item to implementation.
- Keep opt-in live Apple `container` smoke coverage for command shapes and JSON
  parsing that unit tests cannot prove.
- Keep `runhaven doctor` enforcing the reviewed Apple `container` runtime
  commit, builder image, vminit image, and Kata kernel pins.
- Keep `image doctor` surfacing read-only builder status and resource guidance
  before adding UI flows that trigger rebuilds.
- Keep [`docs/TAURI_UI_GUARDRAILS.md`](TAURI_UI_GUARDRAILS.md) as the active
  contract for UI resource warnings, approval gates, typed Rust commands, and
  narrow Tauri capabilities.

## Tauri/UI Research And Implementation

- Completed 2026-06-16 in
  [`docs/TAURI_UI_RESEARCH_PLAN.md`](TAURI_UI_RESEARCH_PLAN.md).
- Decision: Tauri v2 with Svelte + Vite + TypeScript, npm lockfile, a separate
  `src-tauri` crate that calls the existing Rust library, narrow capabilities,
  and a secure default path with warning-confirmed advanced choices.
- First scaffold work is validated with exact-pinned Tauri/Svelte
  dependencies, a separate `src-tauri` crate, narrow capabilities, setup,
  dashboard, profile, folder-pick, and run-plan surfaces.
- First mutating slice is implemented: `launch_run` reuses the Rust launch
  path, requires explicit launch and warning confirmation, blocks when setup
  checks fail, and lives behind the `launch-run` capability.
- Launch readiness now shows typed image and builder status and blocks launch
  when the selected bundled profile image is missing or stale.
- Remaining UI controls should still be added one at a time with typed Rust
  commands, explicit confirmation, focused tests, and narrow capabilities.

## Phase 6: Repeatable Workflows

- strict project-local workflow files
- container-only setup, main, and teardown steps
- workflow state, resume, and failure policy
- read-only context overlays for docs, skills, prompts, and project memory
- deny-by-default extension and MCP boundary policy documented in
  [`EXTENSION_MCP_BOUNDARY.md`](EXTENSION_MCP_BOUNDARY.md)
- source-of-truth planner and policy objects reusable by future CLI, API, or
  GUI surfaces
