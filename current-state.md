# Current State

Last Updated: 2026-06-24 UTC

## Current Objective

Next release objective: close the `cli-complete-v0.5.0` scope.

Scope for that slice:

- confirm remaining CLI gaps against `docs/V1_RELEASE_PLAN.md`;
- use `docs/RELEASE_GAP_ANALYSIS.md` as the active v0.5/v1 gap tracker;
- finish or explicitly defer any CLI-only product behavior before `v0.5.0`;
- lock CLI docs, output, JSON/data lifecycle, diagnostics, cleanup, and
  profile support tiers;
- apply the secure-easy and maintainability gates to every remaining CLI gap;
- run focused CLI verification and Apple `container` smokes for any claims.

First GUI slice after the `v0.5.0` CLI-complete scope is closed:
`tauri-stop-run-control`.

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
- `v0.5.0` is now the intended CLI-complete release. All CLI product work
  should be done by that tag before broad v1 desktop expansion.
- RunHaven remains alpha/pre-release until after `v0.5.0` is cut.
- The proposed v1.0.0 release boundary now requires the desktop app to become
  first-class, with the CLI as the stable backend and automation surface.
- Above all else, secure defaults must be the easiest path. Supported
  lower-security choices should warn and require explicit intent; unsupported
  or hard-boundary violations still fail closed.
- Apple `container machine` is not the default RunHaven boundary, but explicit
  or user-managed machine workflows should not be blocked solely because they
  are less secure. They should warn, require intent, and fail only for concrete
  unsupported or unsafe states.
- Every stage must consider file size, modularity, duplication, crate/component
  organization, standard-library/native/installed-dependency options, exact
  current-stable pins, and harness state.
- Windows and Linux are not supported runtime or contributor-verification
  targets.
- GitHub Actions CI is disabled during alpha/pre-release. Local verification is
  authoritative until a maintainer explicitly re-enables CI.
- Default safety boundaries remain: no host home mount, no cloud credential
  folder mount, no raw SSH key mount, no arbitrary environment passthrough,
  explicit workspace scope, non-root bundled images, and provider egress
  allowlisting only through reviewed provider mode.

## Key Decisions

Durable rationale that compaction tends to lose. Change these only with new
evidence and a recorded reason.

- Default boundary is task-scoped `container run`, not `container machine`,
  because machine workflows map the host user, home, and credentials into the
  guest. Machine use is warned and explicit, not blocked.
- `--ssh` fails closed because Apple `container` 1.0.0 exposed the forwarded
  socket to the non-root guest but `ssh-add -l` returned permission denied;
  enabling it would need a raw key mount or a root default user. Reopen only
  with a no-secret runtime proof.
- CI stays disabled during alpha because local macOS 26 verification is
  authoritative and hosted CI cannot exercise the Apple `container` boundary.
  Re-enable only by explicit maintainer decision.
- The harness is a three-file startup contract because bulk startup context was
  the failure mode; deeper `docs/harness/` material is on-demand reference.
- Exactly one feature is `active` in `feature_list.json` to hold scope; the
  `active` row is the current slice, distinct from `planned` work.

## Latest Verified Work

- 2026-06-24: Applied a harness gap-analysis pass against the
  learn-harness-engineering course. Added a `feature_list.json` status_legend
  and marked the single current slice `active`; added a startup baseline gate to
  `AGENTS.md`; added this Key Decisions section; added a boundary-journey table
  and independent-evaluator routing to `security-boundary-map.md`; added
  verify-before-refactor and agent-oriented-error gates to `change-contract.md`;
  defined a representative task set in `quality-document.md` and linked it from
  `roadmap.md`; and added a mechanical `capability_guard` test that fails closed
  if a Tauri capability grants a host bridge. Tauri tests (incl. the capability
  guard), pin check, JSON validation, Markdown link check, and diff checks
  passed.
- 2026-06-18: Clarified the Container Machine policy across active docs and
  harness state. Task-scoped `container run` remains the secure-easy default,
  while explicit or user-managed Apple `container machine` workflows should be
  warned and require intent rather than blocked solely because they are less
  secure.
- 2026-06-18: Ran a full active-doc release-status pass. User-facing docs,
  roadmap/planning docs, Tauri planning docs, and harness routing now agree
  that RunHaven remains alpha/pre-release until after `v0.5.0`, `v0.5.0` is
  CLI-complete, and `v1.0.0` is the first-class desktop release. The README now
  names both goals directly. Historical evidence remains historical.
- 2026-06-18: Added `docs/RELEASE_GAP_ANALYSIS.md` as the active v0.5/v1 gap
  tracker. It records observed CLI command coverage, current desktop command
  coverage, maintainability pressure, v0.5 blockers, v1 blockers, v1.x
  deferrals, and immediate next actions. Linked it from README, roadmap,
  release plan, non-UI backlog, feature state, and harness routing.
- 2026-06-18: Locked secure-easy and maintainability gates into `AGENTS.md`,
  `docs/V1_RELEASE_PLAN.md`, `docs/SECURITY_MODEL.md`, and focused harness
  docs. Future slices must make secure defaults the easiest path, warn and
  require intent for supported lower-security choices, fail closed on hard
  boundary violations, avoid deferred large-file debt, remove meaningful
  duplication, prefer standard/native/installed solutions, keep exact
  current-stable pins, and update harness state when scope changes.
- 2026-06-18: Added and revised `docs/V1_RELEASE_PLAN.md` as the proposed
  durable release ladder. The plan now sets `v0.5.0` as CLI-complete, makes
  `v1.0.0` a first-class desktop release for the safe beginner workflow, keeps
  the CLI as the stable backend and automation surface, records missing
  runtime/data/storage/network/auth/UX/accessibility/performance edge cases,
  and defines release milestones and verification gates. Linked it from
  `README.md` and `docs/ROADMAP.md`; added `cli-complete-v0.5.0` and
  `desktop-first-class-v1` to `feature_list.json`.
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

- 2026-06-18 README and Container Machine policy docs checks:
  - `sw_vers` reported macOS 26.5.1 build 25F80.
  - `uname -m` reported `arm64`.
  - `container --version` reported Apple `container` CLI 1.0.0 commit
    `ee848e3`.
  - `container machine --help` passed and showed create, delete, inspect,
    list, logs, run, set, set-default, and stop subcommands.
  - Stale hard-block wording scan for old Container Machine policy phrasing
    passed.
  - `cargo run --locked --bin runhaven-check-pins` passed.
  - `git ls-files '*.json' | xargs -n 1 python3 -m json.tool >/dev/null`
    passed.
  - Local Markdown link check over 54 Markdown files passed.
  - `git diff --check` passed.
- 2026-06-18 v0.5.0/v1.0.0 gap-analysis docs checks:
  - CLI help smokes passed for top-level `runhaven`, `runs`, `image`,
    `network`, `state`, `egress`, `auth`, and `why`.
  - Tauri command/capability scan and source file-size scan completed for
    gap-analysis evidence.
  - `cargo run --locked --bin runhaven-check-pins` passed.
  - `git ls-files '*.json' | xargs -n 1 python3 -m json.tool >/dev/null`
    passed.
  - Local Markdown link check over 54 Markdown files passed.
  - Stale wording scan for old pre-Tauri/release-boundary/package-evidence
    phrasing passed with only intentional README release-plan link text.
  - Explicit trailing-whitespace check over changed docs/state files passed.
  - `git diff --check` passed.
- 2026-06-18 active-doc release-status checks:
  - `cargo run --locked --bin runhaven-check-pins` passed.
  - `git ls-files '*.json' | xargs -n 1 python3 -m json.tool >/dev/null`
    passed.
  - Local Markdown link check over 53 Markdown files passed.
  - Stale wording scans for old pre-Tauri, release-boundary, alpha, `v0.5.0`,
    and `v1.0.0` phrasing passed with only intentional historical/evidence
    matches.
  - Explicit trailing-whitespace check over changed docs/state files passed.
  - `git diff --check` passed.
- 2026-06-18 secure-easy and maintainability docs/harness checks:
  - `cargo run --locked --bin runhaven-check-pins` passed.
  - `git ls-files '*.json' | xargs -n 1 python3 -m json.tool >/dev/null`
    passed.
  - Local Markdown link check over 53 Markdown files passed.
  - Explicit trailing-whitespace check over the changed docs/state files
    passed.
  - `git diff --check` passed.
- 2026-06-18 v0.5.0/v1.0.0 release-ladder docs checks:
  - `cargo run --locked --bin runhaven-check-pins` passed.
  - `git ls-files '*.json' | xargs -n 1 python3 -m json.tool >/dev/null`
    passed.
  - Local Markdown link check over 53 Markdown files passed.
  - Explicit trailing-whitespace check over the changed docs/state files
    passed.
  - `git diff --check` passed.
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
  - `cargo run --locked --bin runhaven -- image build <agent> --dry-run`
    passed for Claude, Codex, Copilot, and Gemini.
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

- Root docs/state: `AGENTS.md`, `feature_list.json`, and `current-state.md`.
- Harness docs: `docs/harness/boundaries/change-contract.md`,
  `docs/harness/boundaries/security-boundary-map.md`,
  `docs/harness/feedback/verification-matrix.md`,
  `docs/harness/feedback/sensor-registry.md`,
  `docs/harness/feedback/quality-document.md`, and
  `docs/harness/state/roadmap.md`.
- Tauri code: `src-tauri/src/capability_guard.rs` (new) and
  `src-tauri/src/lib.rs` (test module declaration).

## Next Step

Close or explicitly accept the `cli-complete-v0.5.0` scope before broad v1 GUI
expansion. Apply the secure-easy and maintainability gates to every remaining
CLI gap. Use `docs/RELEASE_GAP_ANALYSIS.md` for current blockers and
deferrals, and `docs/V1_RELEASE_PLAN.md` for the durable release contract.
`tauri-stop-run-control` remains the first GUI slice toward
`desktop-first-class-v1`, not the whole desktop release requirement.
