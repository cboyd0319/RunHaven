# Roadmap

This roadmap records both completed foundation slices and remaining product
direction. The live feature status and verification evidence are tracked in
`feature_list.json`, `current-state.md`, and
`docs/harness/evidence/evidence-log.md`.

RunHaven remains alpha/pre-release until after the `v0.5.0` CLI-complete
milestone. The desktop app is alpha while `v0.5.0` CLI scope is being closed.

The consolidated non-UI backlog lives in
[`NON_UI_BACKLOG.md`](NON_UI_BACKLOG.md). Tauri/UI research lives in
[`TAURI_UI_RESEARCH_PLAN.md`](TAURI_UI_RESEARCH_PLAN.md).
The proposed durable v0.5.0/v1.0.0 release ladder lives in
[`V1_RELEASE_PLAN.md`](V1_RELEASE_PLAN.md).
The active release gap tracker lives in
[`RELEASE_GAP_ANALYSIS.md`](RELEASE_GAP_ANALYSIS.md).

## Current Release Track

| Target | Goal | Status |
| --- | --- | --- |
| `v0.5.0` | CLI-complete release: command set, docs, JSON/data decisions, runtime smokes, profile support tiers, diagnostics, cleanup, secure-easy defaults, and maintainable module boundaries are complete and verified. | Current objective; gaps tracked in `RELEASE_GAP_ANALYSIS.md` |
| `v1.0.0` | First-class desktop release: the Tauri app becomes the easiest safe path for setup, image readiness/rebuild, planning, launch, live status, bounded output, stop, kill, repair, diagnostics, worktree review, cleanup, accessibility, signed/notarized artifact, and provenance. | Planned after `v0.5.0` |
| `v1.x` | New provider credential brokers, broader provider policy, extension/MCP surfaces, updater/installer automation, and other larger surfaces. | Follow-up only |

Design rule for every phase: the secure path must be the easy path. Supported
lower-security choices should warn and require explicit intent; unsupported or
hard-boundary violations still fail closed.

Engineering rule for every phase: avoid large-file debt, remove meaningful
duplication, prefer standard library/native/installed solutions, keep exact
current-stable pins, and keep the harness state current.

## Completed Foundation: Safe Local Baseline

- Rust CLI package and command
- agent profiles
- dry-run planning
- bundled image templates
- doctor checks
- unit tests for command boundaries

## Completed And Ongoing: Network Boundary

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

## v0.5.0 CLI-Complete Scope

- non-mutating guided first-run setup with exact prerequisite fixes
- plain-language explanations for every requested permission
- `runhaven why` diagnostics for blocked hosts, rejected mounts, sensitive
  defaults, network modes, state volumes, and validation failures
- guided `runhaven setup` first-run flow with profile-specific next commands
- goal-based network selection copy for local-only, provider-only, package
  install, and unrestricted internet use cases
- command docs, help text, JSON/data lifecycle decisions, profile support
  tiers, and release notes aligned with the CLI-complete contract
- no known large-file, duplication, or crate-organization debt intentionally
  deferred from CLI code into v1 desktop work

## v1.x Or Design-First Product Candidates

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
- one-command bootstrap or installer automation for Apple `container`

## Completed And Ongoing: Isolation And Recovery

- run records with `runs list`, `runs show`, `runs log`, `runs diff`,
  `runs keep`, `runs recover`, `runs merge`, `runs discard`, `runs active`,
  `runs status`, `runs attach`, `runs logs-follow`, `runs stop`,
  `runs kill`, `runs repair`, `runs repair --all`, and git change metadata
- structured blocked-host and cleanup event records
- stable JSON output for read-only, run-status, and repair-summary commands
- image doctor, image rebuild, state reset/prune, and managed-network
  list/prune commands
- provider auth broker run records that never print secrets

## Pre-Release Codebase Health Gates

- Rust conversion and modularization completed with source organized under
  `src/runhaven/` by ownership boundary.
- keep source size and cohesion checks active so command surfaces, policy logic,
  run-state handling, and verification helpers remain reviewable.
- review touched Rust, Tauri, and frontend files for duplication and
  modularity before calling a slice complete.
- run the `rust-expert` agent with the Rust skill across the entire repo to
  look for correctness, safety, idiomatic Rust, test, packaging, and
  maintainability issues before broadening the product surface.
- use the `apple-container-expert` agent with the Apple Container skill for
  Apple `container` runtime, networking, source, service, registry, machine,
  and security-boundary work.
- add a short top-of-file description to every maintained script so future
  contributors can see what each script is and what it does before running it.

## Runtime Readiness Gates

- Action the coverage gaps in
  [`docs/APPLE_CONTAINER_GAP_ANALYSIS.md`](APPLE_CONTAINER_GAP_ANALYSIS.md)
  before broadening runtime-sensitive CLI or desktop behavior.
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

## v1.0.0 Desktop-First Implementation

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
- Launch planning now warns when other RunHaven runs are active and when the
  selected memory limit plus active runs may be material on the host. The
  post-launch UI shows a sanitized run snapshot.
- The dashboard can read a typed live run-status snapshot for the latest
  launched run, showing marker status, container state, resources, image, and
  network metadata without raw logs or raw Apple inspect payloads.
- Raw log viewing now follows
  [`TAURI_LOG_VIEWING_DESIGN.md`](TAURI_LOG_VIEWING_DESIGN.md): status first,
  then an explicitly requested bounded container-stdio snapshot, with no
  automatic display, no durable frontend storage, and no live stream.
- After the `v0.5.0` CLI-complete scope is closed or explicitly accepted, add
  first-class desktop controls for stop, kill, repair, image build/rebuild,
  diagnostics, worktree review, and safe cleanup.
- Before `v1.0.0`, the desktop app must be keyboard navigable, accessible at
  the minimum supported window size, signed, notarized, checksummed, and backed
  by release provenance.
- Remaining UI controls should still be added one at a time with typed Rust
  commands, explicit confirmation, focused tests, and narrow capabilities.

## Later Repeatable Workflows

- strict project-local workflow files
- container-only setup, main, and teardown steps
- workflow state, resume, and failure policy
- read-only context overlays for docs, skills, prompts, and project memory
- deny-by-default extension and MCP boundary policy documented in
  [`EXTENSION_MCP_BOUNDARY.md`](EXTENSION_MCP_BOUNDARY.md)
- source-of-truth planner and policy objects reusable by future CLI, API, or
  GUI surfaces
