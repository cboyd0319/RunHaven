# Current State

Last Updated: 2026-06-18 UTC

## Current Objective

Next product slice: add the first explicit Tauri run-control operation,
`stop_run`.

Scope for that slice:

- one typed Rust command for one validated active run id;
- explicit user confirmation before mutation;
- narrow `run-control` capability coverage;
- frontend state that waits for the command result before implying success;
- focused Rust, Tauri, frontend, and Playwright checks.

## Startup State Contract

- `AGENTS.md`: root instruction map.
- `feature_list.json`: compact feature status and next product slice.
- `current-state.md`: progress, trusted facts, blockers, and handoff.

Do not recreate separate root `progress.md` or `session-handoff.md` files.
Load deeper docs only when the task touches that surface.

## Product Facts

- RunHaven is a Rust 1.96.0 CLI for running AI coding agents inside Apple
  `container` on macOS 26+ on Apple silicon.
- The CLI is the current working product surface.
- The alpha desktop shell lives under `ui/` and `src-tauri/`.
- Windows and Linux are not supported runtime or contributor-verification
  targets.
- GitHub Actions CI is disabled during alpha/pre-release. Local verification is
  authoritative until a maintainer explicitly re-enables CI.
- Default safety boundaries remain: no host home mount, no cloud credential
  folder mount, no raw SSH key mount, no arbitrary environment passthrough,
  explicit workspace scope, non-root bundled images, and provider egress
  allowlisting only through reviewed provider mode.

## Latest Verified Work

- 2026-06-18: Refreshed direct package pins and lockfiles to current stable
  package-manager releases. Tauri Rust pins moved to `tauri` 2.11.3 and
  `tauri-build` 2.6.3; frontend `@tauri-apps/api` moved to 2.11.1; bundled
  image CLIs moved to Claude Code 2.1.181, Codex 0.140.0, and Copilot 1.0.63.
  Cargo and npm lockfiles were refreshed. Playwright now starts an isolated
  strict-port RunHaven dev server instead of reusing an unrelated process on
  port 5173.
- 2026-06-18: Implemented OWASP-informed local hardening from the Cheat Sheet
  review. Tauri commands now reject oversized IPC fields before planning or
  launch confirmation, and RunHaven cache markers, logs, and locks are created
  with owner-only permissions on Unix.
- 2026-06-17: Simplified the repo harness to the lightweight five-subsystem
  model from the referenced harness-learning material. Startup now routes
  through only `AGENTS.md`, `feature_list.json`, and `current-state.md`;
  harness docs are on-demand reference material.
- 2026-06-16: Implemented the first raw-log snapshot slice. `get_log_snapshot`
  lives behind `run-control`, requires sensitive-output acknowledgement,
  validates the run id and RunHaven-owned active container marker, calls only
  bounded `container logs -n`, and keeps raw output out of durable frontend
  state.
- 2026-06-16: Tauri launch flow can confirm launch, check image readiness,
  show resource warnings, render sanitized run snapshots, and refresh live run
  status without exposing raw logs or raw Apple inspect payloads.

## Trusted Verification

- 2026-06-18 package pin refresh checks:
  - `rustup check` reported stable `1.96.0` up to date.
  - `cargo info`, `cargo search`, and `npm view` checked current stable direct
    package versions.
  - `cargo update` and `cargo update --manifest-path src-tauri/Cargo.toml`
    refreshed Cargo lockfiles to the latest Rust 1.96-compatible versions.
  - `npx -y npm@11.17.0 --prefix <package> install --package-lock-only
    --ignore-scripts` refreshed UI and bundled-image npm lockfiles.
  - `npx -y npm@11.17.0 --prefix <package> audit --audit-level=moderate`
    passed for the UI and bundled-image npm packages.
  - `cargo update --dry-run --verbose` reported zero remaining root Cargo
    lockfile updates.
  - `cargo update --manifest-path src-tauri/Cargo.toml --dry-run --verbose`
    reported zero remaining Tauri lockfile updates; remaining newer transitive
    releases are outside upstream semver constraints.
  - `cargo tree --manifest-path src-tauri/Cargo.toml --locked --target
    aarch64-apple-darwin -i glib` found no macOS dependency path for `glib`.
  - `cargo fmt --check` passed.
  - `cargo fmt --manifest-path src-tauri/Cargo.toml --check` passed.
  - `cargo run --locked --bin runhaven-check-pins` passed.
  - `git ls-files '*.json' | xargs -n 1 python3 -m json.tool >/dev/null`
    passed.
  - `git diff --check` passed.
  - `cargo test --locked` passed.
  - `cargo test --manifest-path src-tauri/Cargo.toml --locked` passed.
  - `cargo clippy --all-targets --locked -- -D warnings` passed.
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked
    -- -D warnings` passed.
  - `npx -y npm@11.17.0 --prefix ui test -- --run` passed.
  - `npx -y npm@11.17.0 --prefix ui run check` passed.
  - `npx -y npm@11.17.0 --prefix ui run build` passed.
  - `npx -y npm@11.17.0 --prefix ui run test:e2e` passed after Playwright was
    isolated from the unrelated JobSentinel dev server on port 5173.
  - `cargo build --locked` passed.
  - `npx -y npm@11.17.0 --prefix ui run tauri:build` passed.
- 2026-06-18 security hardening checks:
  - Red checks first failed for oversized IPC payloads and default active-run
    marker permissions.
  - `cargo fmt --check` passed.
  - `cargo fmt --manifest-path src-tauri/Cargo.toml --check` passed.
  - `cargo test --locked` passed.
  - `cargo test --manifest-path src-tauri/Cargo.toml --locked` passed.
  - `cargo clippy --all-targets --locked -- -D warnings` passed.
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked
    -- -D warnings` passed.
  - `cargo run --locked --bin runhaven-check-pins` passed.
  - `npm --prefix ui test -- --run` passed.
  - `npm --prefix ui run check` passed.
  - `git ls-files '*.json' | xargs -n 1 python3 -m json.tool >/dev/null`
    passed.
  - `git diff --check` passed.
- 2026-06-17 harness simplification checks:
  - `git ls-files '*.json' | xargs -n 1 python3 -m json.tool >/dev/null`
    passed.
  - Local Markdown link check over 52 tracked Markdown files passed.
  - `cargo run --locked --bin runhaven-check-pins` passed.
  - `git diff --check` passed.
  - Stale-reference scans for retired root `progress.md`/`session-handoff.md`,
    old Python pin-check commands, and old mandatory harness roadmap routing
    found only intentional archive or historical evidence references.
  - `./init.sh` was not run because this pass changed documentation, harness
    instructions, and state only; no runtime code, lockfile, package, image, or
    Tauri capability behavior changed.
- 2026-06-16 Tauri raw-log snapshot checks passed:
  - `cargo test --manifest-path src-tauri/Cargo.toml --locked`
  - `cargo fmt --manifest-path src-tauri/Cargo.toml --check`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings`
  - `cargo test --locked`
  - `cargo clippy --all-targets --locked -- -D warnings`
  - `npm --prefix ui test -- --run`
  - `npm --prefix ui run check`
  - `npm --prefix ui run test:e2e`
  - `npm --prefix ui run build`
  - `scripts/apple_container_smoke.sh`

## Blockers

- `--ssh` remains fail-closed. Apple `container` 1.0.0 exposes an SSH agent
  socket to the non-root guest user, but `ssh-add -l` returns permission
  denied. Do not re-enable SSH forwarding, mount raw SSH keys, or switch the
  default agent user to root without explicit security review and no-secret
  runtime proof.

## Touched Surfaces In This Harness Pass

- `AGENTS.md`
- `.agents/skills/harness/SKILL.md`
- `.agents/skills/harness/references/repo-harness.md`
- `README.md`
- `feature_list.json`
- `current-state.md`
- `docs/HARNESS_EVALUATION.md`
- `docs/NON_UI_BACKLOG.md`
- `docs/TAURI_LOG_VIEWING_DESIGN.md`
- `docs/harness/README.md`
- `docs/harness/manifest.json`
- `docs/harness/authoritative-facts.md`
- `docs/harness/boundaries/change-contract.md`
- `docs/harness/boundaries/component-inventory.md`
- `docs/harness/feedback/quality-document.md`
- `docs/harness/feedback/sensor-registry.md`
- `docs/harness/feedback/verification-matrix.md`
- `docs/harness/operations/agent-operating-model.md`
- `docs/harness/release/release-controls.md`
- `docs/harness/research/sources.md`
- `docs/harness/state/entropy-control.md`
- `docs/harness/state/first-agent-task.md`
- `docs/harness/state/roadmap.md`
- `docs/harness/evidence/evidence-log.md`

## Next Step

Implement `tauri-stop-run-control` as the next product slice. Keep every later
run-control operation separate.
