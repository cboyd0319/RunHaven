# Current State

Last Updated: 2026-06-16 UTC

## Current Objective

Continue Tauri/UI implementation after the first mutating launch slice. The
desktop shell now has setup, dashboard, profile, folder-pick, run-plan review,
and explicitly confirmed launch surfaces. The next implementation should add
one missing launch-readiness or run-control operation at a time only after typed
Rust validation, explicit confirmation, narrow capability review, and focused
tests are in place.

## State Contract

- `feature_list.json`: machine-readable feature state and durable product
  evidence.
- `docs/harness/evidence/evidence-log.md`: meaningful verification, source
  review, packaging, or harness evidence.
- `current-state.md`: current objective, trusted verification, touched
  surfaces, blockers, and next step.
- Do not recreate separate root `progress.md` or `session-handoff.md` files.

## Product State

- RunHaven is a Rust 1.96.0 CLI for running AI coding agents inside Apple
  `container` on macOS 26+ on Apple silicon.
- The application code is organized as a Cargo crate under `src/runhaven/` with
  CLI, runtime, provider, image, records, harness, and support modules. Bundled
  image templates live under top-level `images/`.
- An alpha desktop scaffold lives under `ui/` and `src-tauri/`. It uses
  Svelte + Vite + TypeScript and Tauri v2, calls the root Rust library through
  typed commands, and currently exposes setup/dashboard/profile, native folder
  picking, run-plan review, and an explicitly confirmed launch path.
- The old Python package, Python tests, Python scripts, `pyproject.toml`,
  `.python-version`, and `requirements-dev.txt` have been removed.
- Windows and Linux are not supported runtime or contributor-verification
  targets.
- GitHub Actions CI is disabled during alpha/pre-release to avoid hosted-runner
  cost. Local verification remains authoritative until a maintainer explicitly
  re-enables CI.
- Default product safety boundaries remain: no host home mount, no cloud
  credential folder mount, no raw SSH key mount, no arbitrary environment
  passthrough, explicit workspace scope, non-root bundled images, and
  provider egress allowlisting only through reviewed provider mode.
- HarnessForge output is advisory unless a maintainer promotes a recommendation
  into repo-owned docs, tests, policy, code, or packaging checks.

## Latest Verified Work

- Added `docs/NON_UI_BACKLOG.md` as the consolidated backlog for all remaining
  non-UI gates, accepted polish, candidates, and deferred end-stage packaging
  work.
- Added `docs/TAURI_UI_RESEARCH_PLAN.md` as the active research plan for Tauri
  architecture, frontend framework selection, comparable product analysis, UX
  questions, information architecture, command contracts, and scaffold gates.
- Promoted and then validated Tauri/UI research and framework selection in the
  active harness roadmap.
- Completed the Tauri/UI research phase. The accepted direction is Tauri v2
  with Svelte + Vite + TypeScript, npm lockfile, a separate `src-tauri` crate
  calling the existing Rust library, narrow capabilities, typed command
  contracts, a read-only first scaffold, and no generic shell/filesystem/process
  bridge.
- Added the first Tauri/Svelte scaffold under `ui/` and `src-tauri/` with
  exact-pinned dependencies, checked-in npm and Cargo lockfiles, typed
  read-only Rust commands, and narrow `main-read` plus `folder-pick`
  capabilities.
- Added a Svelte operational UI for setup checks, active/recent run counts,
  agent selection, folder selection, secure-default network choices, resource
  inputs, read-only workspace selection, plan review, and explicit launch
  confirmation.
- Added the first mutating desktop slice: typed Tauri `launch_run`, separate
  `launch-run` capability, shared Rust runtime launch module, shared
  state-volume lock module, frontend launch helper, confirmation controls, and
  Playwright preview coverage. Launch requires plan confirmation, every warning
  acknowledgement, and passing setup checks before it starts a background run.
- Remaining launch-readiness gaps before the UI launch flow is complete are
  image status, builder status, active-run resource warnings, and richer
  post-start status/log feedback. Stop, kill, attach, repair, image build,
  state cleanup, network cleanup, and worktree review remain CLI-first.
- Fixed the Svelte 5 blank-page runtime failure by replacing the old
  `new App(...)` entrypoint with `mount(App, ...)`.
- Added exact-pinned Playwright browser coverage for the UI shell so runtime
  errors, console errors, and blank-page regressions are caught by
  `npm --prefix ui run test:e2e`.
- Updated the local harness so `./init.sh` now runs frontend npm ci, Svelte
  typecheck, Vitest, Playwright, Vite build, Tauri Rust fmt/test/clippy, and
  debug no-bundle Tauri build in addition to the root Rust checks.
- Added the project-wide secure-easy-path rule: the default and shortest path
  should be secure; supported advanced choices warn and require confirmation
  but are not hidden or blocked only because they are advanced.
- Added non-mutating CLI explainers for `runhaven why workspace PATH`,
  `runhaven why network MODE`, and `runhaven why state AGENT`; existing
  `why host` remains the provider-host explainer.
- Added focused integration tests and usage docs for the expanded `why`
  commands.
- Added task recipes for read-only review, local-only checks, provider-only
  runs, worktree review, and exact state reset.
- Added `docs/EXTENSION_MCP_BOUNDARY.md` and linked it from the security model
  so future extension or MCP work has a deny-by-default policy before
  implementation.
- Added a `runhaven-check-pins` sensor that enforces top-of-file descriptions
  for maintained shell scripts and bundled image templates.
- Updated the active harness roadmap and source-mined UX notes so completed
  CLI explainers, script-header enforcement, and extension/MCP policy do not
  keep resurfacing as current backlog.
- Rebuilt the CLI in Rust with exact-pinned Cargo dependencies and a checked-in
  `Cargo.lock`.
- Replaced the Python pin checker with `runhaven-check-pins`.
- Updated `init.sh`, root docs, installation docs, usage docs, pinning docs,
  harness docs, component inventory, verification matrix, manifest metadata,
  and former CI routing for the Rust stack.
- Kept file organization nested by responsibility instead of flattening the
  Rust source tree.
- Split large Rust modules so every Rust source file is under 500 lines; the
  current largest file is `src/runhaven/provider/auth_broker.rs` at 499 lines.
- Updated `.gitignore` for Rust build, coverage, profiling, local
  RunHaven/container, macOS, editor, and pre-Rust Python tooling artifacts.
- Completed the final active-document accuracy sweep for the Rust conversion
  across product docs, GitHub instructions, harness boundaries, roadmap,
  packaging controls, and source-mined ideas.
- Removed ignored local cleanup artifacts from the working tree, including
  stale Python cache/build output and `.DS_Store` files.
- Deduped the main README after the overview refresh so the top-level page now
  keeps one compact product/value narrative, one quick-start path, and routes
  detailed feature and command coverage to `docs/CAPABILITIES.md` and
  `docs/USAGE.md`.
- Reworked `docs/CAPABILITIES.md` into a scan-friendly overview with compact
  tables for runtime defaults, profiles, network modes, credentials,
  sessions/state, observability, and current limits.
- Corrected the Cargo development command in installation docs to name the
  `runhaven` binary explicitly.
- Removed the active GitHub Actions workflow and updated pin-policy and harness
  guidance so no hosted CI jobs run during alpha/pre-release.
- Started the Apple Container pre-Tauri coverage review and added
  `docs/APPLE_CONTAINER_GAP_ANALYSIS.md` as the action ledger for remaining
  runtime, security, and verification gaps.
- Added `scripts/apple_container_smoke.sh` as an opt-in local Apple
  `container` smoke harness. The default path covers `doctor`, shell image
  readiness, internal read-only workspace behavior, active-run
  status/logs-follow/stop cleanup, provider planning, and exact cleanup.
  `--with-provider` adds live provider allowlist and egress-denial coverage.
  `--with-ssh` is a no-secret live SSH-forwarding connectivity check with a
  disposable empty `ssh-agent`; it currently exposes an Apple `container` 1.0.0
  non-root socket permission blocker.
- Fixed the Rust provider CONNECT proxy relay after the live smoke exposed TLS
  tunnel failures. Accepted/tunnel sockets are forced back to blocking mode,
  and CONNECT header reads no longer consume tunneled bytes.
- Added fixture-backed parser tests for Apple `container` JSON shapes covering
  image list, network inspect, container inspect, source-backed legacy
  attachment aliases, invalid shapes, and missing-container repair stderr.
- Tightened parser behavior found during the fixture pass: image matching now
  trusts only actual image-name fields instead of all descriptor annotations,
  and active-run repair no longer removes markers for unrelated `not found`
  inspect failures.
- Extended `runhaven doctor` with JSON-backed Apple `container` runtime pin
  checks. It now fails closed on mismatched runtime commit, builder image,
  vminit image, or Kata kernel fields from the structured
  `container system version --format json` and
  `container system property list --format json` probes.
- Split the new doctor runtime-pin implementation into
  `src/runhaven/cli/doctor/runtime_pins.rs` so the main doctor module stays
  well under the repo's Rust file-size ceiling.
- Extended `runhaven image doctor` with read-only Apple builder diagnostics
  from `container builder status --format json`. The output reports sanitized
  builder state, image, CPU/memory allocation, Rosetta mode, start time, and
  network address while avoiding builder mounts and environment.
- Updated `scripts/apple_container_smoke.sh` to assert that
  `image doctor shell` reports builder status and large-build resource
  guidance.
- Added `docs/TAURI_UI_GUARDRAILS.md` as the active UI contract for Tauri
  resource warnings, approval gates, typed Rust commands, narrow capabilities,
  and denied-by-default WebView access.
- Added an accepted backlog item to give every maintained script a short
  top-of-file description explaining what it is and what it does.
- Added provider-mode troubleshooting guidance that distinguishes allowlist
  denials from host-side proxy reachability or macOS Local Network privacy
  failures.
- Changed `--ssh` to fail closed in planner and run paths while Apple
  `container` 1.0.0 non-root SSH forwarding is blocked. The flag stays visible
  so users get an explicit refusal instead of an implied working private-Git
  path.
- Updated `scripts/apple_container_smoke.sh` so the default path verifies
  `runhaven plan --ssh` refusal, and `--with-ssh` verifies `runhaven run --ssh`
  refusal before launch.
- Added `docs/harness/release/apple-container-update-playbook.md` for Apple
  `container` runtime, helper image, installer, Kata kernel, CLI help, smoke,
  cleanup, and rollback evidence before future pin changes.
- Added top-of-file descriptions to every maintained shell script and bundled
  image template.
- Ran whole-repo Rust expert review and addressed the findings: auth broker
  upstream requests now use the configured global timeout, standard and provider
  launch failures remove active markers and record failed runs when a run
  started, cleanup failures stay visible in run records, the crate is marked
  `publish = false`, and tests were moved out of near-limit files to preserve
  the source-size ceiling.
- Fixed the provider-mode bind regression exposed by live verification. Apple
  `container` 1.0.0 reports a guest gateway that is not bindable on macOS, so
  RunHaven uses the gateway URL for guests while allowing a wildcard host
  listener only with explicit Apple-container subnet rejection.

## Trusted Verification

- Tauri launch red checks failed first for missing launch contract pieces:
  `cargo test --manifest-path src-tauri/Cargo.toml launch_run` and
  `npm --prefix ui test -- --run`.
- `cargo fmt`: passed.
- `npm --prefix ui run check`: passed with 0 Svelte errors and 0 warnings.
- `npm --prefix ui test -- --run`: passed with 1 test file and 9 tests.
- `npm --prefix ui run build`: passed with Vite 8.0.16.
- `npm --prefix ui run test:e2e`: passed with 2 Chromium tests, including the
  preview launch confirmation flow.
- `cargo test --manifest-path src-tauri/Cargo.toml --locked`: passed with 7
  Tauri command tests.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings`:
  passed.
- `cargo test --locked`: passed with 41 library tests and 6 integration tests.
- `cargo fmt --check`: passed.
- `cargo clippy --all-targets --locked -- -D warnings`: passed.
- `cargo run --locked --bin runhaven-check-pins`: passed.
- `npm --prefix ui run tauri:build`: passed and built
  `src-tauri/target/debug/runhaven-tauri` without bundling.
- `./init.sh`: initially found one Tauri rustfmt issue, then passed after
  `cargo fmt --manifest-path src-tauri/Cargo.toml`.
- JSON validation for `feature_list.json` and `docs/harness/manifest.json`:
  passed.
- Local Markdown link check over 51 tracked Markdown files: passed.
- `git diff --check`: passed.
- Rust source size guard: passed; no Rust source file under `src/` or
  `src-tauri/src/` is over 500 lines. The current largest files are
  `src/runhaven/provider/auth_broker.rs` at 499 lines and
  `src/runhaven/provider/egress.rs` at 495 lines.
- `./init.sh`: passed after the harness was extended for the desktop scaffold.
  It ran root Rust format/tests/clippy/pin policy/build,
  `npm --prefix ui ci --ignore-scripts`, frontend typecheck/tests/build, Tauri
  Rust fmt/test/clippy, and `npm --prefix ui run tauri:build`.
- `npm --prefix ui ci --ignore-scripts`: passed; frontend checks and Tauri
  build still passed with install scripts disabled.
- `npm --prefix ui run check`: passed with 0 Svelte errors and 0 warnings.
- `npm --prefix ui test`: passed with 1 test file and 6 tests.
- `npm --prefix ui run test:e2e`: passed with 1 Chromium browser test after
  verifying the dashboard renders without page or console errors.
- `npm --prefix ui run build`: passed with Vite 8.0.16.
- `cargo test --manifest-path src-tauri/Cargo.toml --locked`: passed with 4
  Tauri command tests.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings`:
  passed.
- `npm --prefix ui run tauri:build`: passed and built
  `src-tauri/target/debug/runhaven-tauri` without bundling.
- `cargo fmt --check`: passed.
- `cargo test --locked`: passed with 41 library tests and 6 integration tests.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo run --locked --bin runhaven-check-pins`: passed.
- `cargo build --locked`: passed.
- `./init.sh`: passed. The full local harness ran Cargo format, tests, clippy,
  pin policy, and build.
- Rust source size scan: passed; no Rust source file is over 500 lines. The
  current largest file is `src/runhaven/provider/auth_broker.rs` at 499 lines.
- Direct CLI smokes passed: `target/debug/runhaven agents`,
  `target/debug/runhaven plan shell --workspace . -- /bin/bash -lc pwd`,
  `target/debug/runhaven doctor`, and
  `target/debug/runhaven image build shell --dry-run`.
- Expanded `why` CLI smokes passed: `target/debug/runhaven why network
  provider`, `target/debug/runhaven why workspace .`,
  `target/debug/runhaven why state shell`, and
  `target/debug/runhaven why host api.openai.com --agent codex`.
- Active-doc stale-reference scan: passed for old Python project paths,
  Python-package guidance, and pre-Rust source paths.
- Cleanup scan: passed; no Python project artifacts, Python caches, old Python
  packaging files, or `.DS_Store` files remain outside ignored build output.
- JSON validation, local Markdown link check, `git diff --check`, and Rust
  source size guard: passed.
- README docs checks: pin check, local Markdown link check, platform
  wording/stale-command scan, duplicate-section scan, `git diff --check`, and
  CLI `plan` smoke passed.
- Capabilities docs cleanup checks: pin check, local Markdown link check,
  platform/stale-command scan, `git diff --check`, `runhaven agents`, and
  `runhaven plan shell` smokes passed.
- Current Apple Container host evidence: macOS 26.5.1, arm64,
  `/usr/local/bin/container`, Apple `container` CLI 1.0.0 build `release`
  commit `ee848e3`, and `container system status` running.
- Alpha CI disablement and Apple Container gap-analysis checks passed:
  `cargo fmt --check`, `cargo test --locked`,
  `cargo clippy --all-targets -- -D warnings`,
  `cargo run --locked --bin runhaven-check-pins`, `cargo build --locked`,
  JSON validation for `feature_list.json` and `docs/harness/manifest.json`,
  local Markdown link check, active workflow file scan, stale active-CI scan,
  `cargo run --locked --bin runhaven -- doctor`,
  `cargo run --locked --bin runhaven -- plan shell --workspace . -- /bin/bash -lc pwd`,
  `cargo run --locked --bin runhaven -- image build shell --dry-run`,
  `container system version`, `container system property list`, and
  `git diff --check`.
- Apple Container smoke harness checks passed: `bash -n
  scripts/apple_container_smoke.sh`, `cargo fmt --check`,
  `cargo test --locked provider::egress -- --nocapture`,
  `scripts/apple_container_smoke.sh`, and
  `scripts/apple_container_smoke.sh --with-provider`.
- Cleanup checks after smoke/debug runs passed: `target/debug/runhaven state
  list` reported no RunHaven state volumes, `target/debug/runhaven runs active`
  reported no active runs, and `target/debug/runhaven network list` showed only
  the shared `runhaven-volume-prep-internal` network.
- Apple Container JSON parser fixture checks passed: `cargo fmt --check` and
  `cargo test --locked` ran 22 library tests and 2 integration tests covering
  the fixture-backed parsers and existing CLI behavior.
- Apple Container runtime pin enforcement checks passed: Apple Container expert
  read-only review, `container system version --format json`,
  `container system property list --format json`, focused doctor parser tests,
  `cargo fmt --check`, `cargo test --locked`,
  `cargo clippy --all-targets -- -D warnings`,
  `cargo run --locked --bin runhaven-check-pins`,
  `cargo run --locked --bin runhaven -- doctor`,
  `scripts/apple_container_smoke.sh`, JSON validation, local Markdown link
  check, Rust source size scan, and `git diff --check`.
- Apple Container builder diagnostic checks passed: Apple Container CLI/source
  review, focused image-doctor parser tests, `cargo fmt --check`,
  `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`,
  `cargo run --locked --bin runhaven-check-pins`,
  `cargo run --locked --bin runhaven -- image doctor shell`,
  `cargo run --locked --bin runhaven -- setup --agent shell`,
  `scripts/apple_container_smoke.sh`, JSON validation, local Markdown link
  check, Rust source size scan, and `git diff --check`.
- Tauri guardrail and backlog-doc checks passed: official Tauri v2 security,
  capabilities, and permissions docs reviewed; pin policy, JSON validation,
  local Markdown link check, active-doc platform/stale-command scan, and
  `git diff --check` passed.
- Provider local-network/privacy troubleshooting checks passed:
  `scripts/apple_container_smoke.sh --with-provider`, cleanup checks with
  `target/debug/runhaven runs active`, `target/debug/runhaven state list`, and
  `target/debug/runhaven network list`, pin policy, JSON validation, local
  Markdown link check, active-doc platform/stale-command scan, and
  `git diff --check` passed.
- SSH forwarding smoke review found a blocker: Bash syntax check and the
  default `scripts/apple_container_smoke.sh` path passed, corrected
  `scripts/apple_container_smoke.sh --with-ssh` failed as expected with
  `Error connecting to agent: Permission denied`, and cleanup checks found no
  active runs or RunHaven state volumes. The script now treats that as a failed
  live SSH smoke instead of passing on socket existence.
- SSH fail-closed guard checks passed: `cargo fmt --check`,
  `cargo test --locked ssh_forwarding_fails_closed_until_non_root_runtime_is_verified`,
  `cargo test --locked plan_ssh_fails_closed_until_runtime_boundary_is_verified`,
  `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`,
  `cargo run --locked --bin runhaven-check-pins`, `bash -n
  scripts/apple_container_smoke.sh`, `scripts/apple_container_smoke.sh`,
  `scripts/apple_container_smoke.sh --with-ssh`, direct
  `target/debug/runhaven plan shell --workspace . --ssh -- /bin/bash -lc true`
  expected refusal, `target/debug/runhaven setup --agent shell`, cleanup checks,
  JSON validation, local Markdown link check, active-doc stale SSH wording scan,
  Rust source size guard, and `git diff --check`.
- Apple Container release-playbook, script-header, `.gitignore`, and Rust expert
  review checks passed: Apple Container expert review, Antigravity sandbox
  research, Rust expert review, script/image header scan, shell syntax checks,
  `cargo fmt --check`, `cargo test --locked`, `cargo clippy --all-targets -- -D
  warnings`, `cargo run --locked --bin runhaven-check-pins`, `cargo build
  --locked`, `scripts/apple_container_smoke.sh --with-provider`,
  `scripts/apple_container_smoke.sh --with-ssh`, cleanup checks, tracked-ignored
  check, JSON validation, local Markdown link check, Rust source size guard, and
  `git diff --check`.
- Tauri/UI research setup checks passed: Antigravity read-only research,
  current official Tauri/Vite/frontend source review, comparable UI source
  review, JSON validation for `feature_list.json` and
  `docs/harness/manifest.json`, local Markdown link check, stale-text scan,
  `cargo run --locked --bin runhaven-check-pins`, and `git diff --check`.
- Tauri/UI research completion checks passed: Antigravity read-only research,
  current official Tauri/Tauri plugin/Vite/Svelte/React/Solid/Vue/comparable UI
  source review, current npm and Cargo package-version checks, JSON validation,
  local Markdown link check, stale-text scan, `cargo run --locked --bin
  runhaven-check-pins`, and `git diff --check`.

## Touched Surfaces

- `.gitignore`
- `README.md`
- `current-state.md`
- `feature_list.json`
- `init.sh`
- `pins.toml`
- `docs/ARCHITECTURE.md`
- `docs/PINNING.md`
- `docs/ROADMAP.md`
- `docs/TAURI_UI_GUARDRAILS.md`
- `docs/TAURI_UI_RESEARCH_PLAN.md`
- `docs/harness/boundaries/component-inventory.md`
- `docs/harness/evidence/evidence-log.md`
- `docs/harness/feedback/sensor-registry.md`
- `docs/harness/feedback/verification-matrix.md`
- `docs/harness/manifest.json`
- `docs/harness/state/roadmap.md`
- `src/runhaven/harness/pins.rs`
- `src/runhaven/runtime/launch.rs`
- `src/runhaven/runtime/lock.rs`
- `src-tauri/`
- `ui/`

## Blockers

- Apple `container` 1.0.0 exposes an SSH agent socket to the RunHaven non-root
  `agent` user with `--ssh`, but the guest cannot use it: `ssh-add -l` returns
  permission denied. RunHaven now fails closed on `--ssh`; do not re-enable it,
  mount raw SSH keys, or switch the default agent user to root without explicit
  security review and no-secret runtime proof.

## Next Step

Choose the next Tauri/UI slice deliberately. The best next step is launch
readiness and run feedback before broader controls: add typed image/builder
status to the dashboard or launch panel, warn on concurrent active runs and
material memory usage, and show post-start active-run status/log snapshots.
After that, add stop, kill, attach, repair, image build, state cleanup,
network cleanup, and worktree review one at a time with typed Rust commands,
explicit confirmation, focused tests, and narrow capabilities. Keep `--ssh`
fail-closed until a no-secret non-root Apple `container` smoke proves usable
forwarding.
