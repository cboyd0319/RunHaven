# Current State

Last Updated: 2026-06-16 UTC

## Current Objective

RunHaven has been converted from a Python project to a fully functional Rust
CLI while preserving the macOS Apple `container` harness contract, exact pin
policy, and repo-owned verification route.

## State Contract

- `feature_list.json`: machine-readable feature state and durable product
  evidence.
- `docs/harness/evidence/evidence-log.md`: meaningful verification, source
  review, release, or harness evidence.
- `current-state.md`: current objective, trusted verification, touched
  surfaces, blockers, and next step.
- Do not recreate separate root `progress.md` or `session-handoff.md` files.

## Product State

- RunHaven is a Rust 1.96.0 CLI for running AI coding agents inside Apple
  `container` on macOS 26+ on Apple silicon.
- The application code is organized as a Cargo crate under `src/runhaven/` with
  CLI, runtime, provider, image, records, harness, and support modules. Bundled
  image templates live under top-level `images/`.
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
  into repo-owned docs, tests, policy, code, or release checks.

## Latest Verified Work

- Rebuilt the CLI in Rust with exact-pinned Cargo dependencies and a checked-in
  `Cargo.lock`.
- Replaced the Python pin checker with `runhaven-check-pins`.
- Updated `init.sh`, root docs, installation docs, usage docs, pinning docs,
  harness docs, component inventory, verification matrix, manifest metadata,
  and former CI routing for the Rust stack.
- Kept file organization nested by responsibility instead of flattening the
  Rust source tree.
- Split large Rust modules so every Rust source file is under 500 lines; the
  current largest file is `src/runhaven/cli/app.rs` at 494 lines.
- Updated `.gitignore` for Rust build output.
- Completed the final active-document accuracy sweep for the Rust conversion
  across product docs, GitHub instructions, harness boundaries, roadmap,
  release controls, and source-mined ideas.
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
- Added `docs/TAURI_UI_GUARDRAILS.md` as the pre-implementation contract for
  future Tauri resource warnings, approval gates, typed Rust commands, narrow
  capabilities, and denied-by-default WebView access.
- Added an accepted backlog item to give every maintained script a short
  top-of-file description explaining what it is and what it does.
- Added provider-mode troubleshooting guidance that distinguishes allowlist
  denials from host-side proxy reachability or macOS Local Network privacy
  failures.

## Trusted Verification

- `cargo fmt --check`: passed.
- `cargo test --locked`: passed with 36 library tests and 2 integration tests.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo run --locked --bin runhaven-check-pins`: passed.
- `cargo build --locked`: passed.
- `./init.sh`: passed. The full local harness ran Cargo format, tests, clippy,
  pin policy, and build.
- Rust source size scan: passed; no Rust source file is over 500 lines.
- Direct CLI smokes passed: `target/debug/runhaven agents`,
  `target/debug/runhaven plan shell --workspace . -- /bin/bash -lc pwd`,
  `target/debug/runhaven doctor`, and
  `target/debug/runhaven image build shell --dry-run`.
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

## Touched Surfaces

- `AGENTS.md`
- `.github/workflows/` active workflow removal
- `.gitignore`
- `Cargo.toml`
- `Cargo.lock`
- `rust-toolchain.toml`
- `init.sh`
- `current-state.md`
- `pins.toml`
- `README.md`
- `CONTRIBUTING.md`
- `docs/`
- `docs/APPLE_CONTAINER_GAP_ANALYSIS.md`
- `docs/TAURI_UI_GUARDRAILS.md`
- `docs/harness/`
- `feature_list.json`
- `images/`
- `scripts/`
- `src/`
- `src/runhaven/cli/doctor/`
- `tests/`
- `tests/fixtures/apple_container/`

## Blockers

- None known.

## Next Step

Decide whether to do the accepted script-description cleanup or the planned
Rust expert plus Rust skill repo-wide review next. Remaining Apple
`container` pre-Tauri items are P2 SSH-forwarding smoke coverage and the
release-update playbook. Keep verification local while alpha CI is disabled.
