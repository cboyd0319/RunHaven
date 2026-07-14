# Verification Matrix

Use the smallest check set that can catch likely regressions. Use the scoped
CLI plus desktop gate for the active coworker-test release. `./init.sh` covers
the complete active workspace. The archived TUI/Codex source is intentionally
outside the release workspace until that product surface is reopened.

## Common Checks

| Change Type | Checks |
| --- | --- |
| Harness docs, instructions, or state | `cargo run --locked --bin runhaven-check-pins`; `scripts/check-repo-structure.sh`; JSON validation for changed JSON; local Markdown link check when links changed; `scripts/check-docs-local-paths.sh`; `git diff --check` |
| README or docs-only change | pin check; `scripts/check-repo-structure.sh`; local Markdown link check when links changed; platform wording scan when support wording changed; `scripts/check-docs-local-paths.sh`; `git diff --check` |
| Rust code | `cargo fmt --check`; focused `cargo test` target; `cargo test --workspace --locked`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; `scripts/check-repo-structure.sh`; maintainability check for touched modules and duplication |
| Frontend UI | `npm --prefix ui run check`; `npm --prefix ui test`; `npm --prefix ui run test:e2e`; `npm --prefix ui run build`; relevant Tauri command tests; `scripts/check-repo-structure.sh`; maintainability check for touched components and adapters |
| Tauri shell | Frontend checks plus `cargo fmt --check`; `cargo test --workspace --locked` (includes the `capability_guard` scope test); `cargo clippy --workspace --all-targets --locked -- -D warnings`; `scripts/check-repo-structure.sh`; capability review |
| GUI visual or macOS design review | `scripts/review-gui.test.sh`; close every existing RunHaven app; `scripts/review-gui.sh`; give the reviewer its exact app path, bundle ID, PID/start time, build ID, executable hash, icon hash, and review fingerprint |
| GUI/Tauri live managed-session path | `scripts/gui_tauri_live_smoke.sh` on macOS 26+ Apple silicon with Apple `container`; set `RUNHAVEN_GUI_LIVE_REAL_AGENT=1` plus `--with-agent codex` or another built real-agent profile and the fixed broker env for that profile when changing real-agent launch/session ownership. Confirms no-bundle Tauri build plus live selected-session bounded output, confirmed stop, multi-session isolation, and opt-in real-agent launch through typed Tauri commands. This is a brokered-launch smoke, not a TTY or OAuth/login parity check |
| CLI command construction | Rust checks plus focused CLI and planning tests |
| Full CLI surface confirmation | `scripts/cli_surface_check.sh` (breadth: every command family); `scripts/apple_container_smoke.sh --with-provider --with-ssh` (depth: provider egress denial and SSH fail-closed); coverage indexed in `docs/CLI_SURFACE_COVERAGE.md` |
| Code organization or modularity | Focused tests for moved behavior; stale import/reference scan with `rg`; relevant Rust, Tauri, or frontend checks; verify duplicated logic was deleted or intentionally kept |
| Apple `container` runtime boundary | Rust checks plus `runhaven doctor`, `runhaven plan`, and a focused runtime smoke proving the claimed mount, user, network, or filesystem behavior |
| Provider egress or endpoint policy | Focused egress/provider tests; source review for endpoint changes; `scripts/apple_container_smoke.sh --with-provider` when behavior changes and the host is available |
| SSH forwarding boundary | Planner and CLI tests proving `--ssh` fails closed; `scripts/apple_container_smoke.sh --with-ssh` only when changing that guard |
| Auth or secrets | Focused auth/provider tests; confirm diagnostics and records do not expose secrets; confirm secure defaults stay easiest and lower-security choices are explicit and warned |
| Support report or issue intake | focused core, CLI, Tauri, and frontend tests; `scripts/check-support-issue-form.sh`; isolated-home report smoke; inspect permissions and poison-value absence; confirm no automatic upload or submission |
| Bundled image definitions | `cargo run --locked --bin runhaven-check-pins`; isolated `npm ci --ignore-scripts` plus audit for each npm image; build all six `0.7.0` images; `runhaven image doctor`; smoke every bundled CLI as the non-root agent with a read-only root and dropped capabilities; verify Node/npm versions where used |
| Desktop installer and first setup | `scripts/release-desktop-common.test.sh`; `scripts/release-desktop-preflight.test.sh`; `scripts/local-sign-desktop.test.sh`; focused Tauri setup-recovery/capability tests; focused Agents/New Run frontend tests; pin check; build and mount the signed DMG; verify its exact drag-to-Applications layout, copied app signature, ticket, Gatekeeper result, bundle metadata, architecture, and clean-Mac Finder launch |
| CLI plus desktop coworker-test prep | `cargo fmt --check`; tests and Clippy for `runhaven-core`, `runhaven-cli`, `runhaven-tauri`, and the `runhaven` CLI integration; frontend check, unit, E2E, and build; no-bundle Tauri build; `runhaven doctor`; relevant Apple container and GUI live smokes; pin/source and dependency audit; structure/docs/JSON/diff checks; signed/notarized app bundle plus clean-machine launch when distributing an artifact. Record TUI exclusion explicitly |
| Release prep | `./init.sh`; `runhaven doctor`; relevant Apple container smokes; pin/source review; dirty-tree check |

## Detected Commands

```bash
cargo fmt --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo run --locked --bin runhaven-check-pins
cargo build --workspace --locked
npm --prefix ui run check
npm --prefix ui test
npm --prefix ui run test:e2e
npm --prefix ui run build
npm --prefix ui run tauri:build
scripts/review-gui.test.sh
scripts/review-gui.sh
scripts/gui_tauri_live_smoke.sh
scripts/check-docs-local-paths.sh
scripts/check-repo-structure.sh
scripts/check-support-issue-form.sh
git diff --check
```

## When Checks Cannot Run

Record the skipped command, exact reason, risk, next best check that did run,
and follow-up needed to close the gap.

## Optional Structural Review

HarnessForge commands are optional owner tools. Use them only when available
and relevant; do not treat their output as the source of truth unless a
maintainer promotes a finding into repo-owned docs, tests, policy, or code.

Manual maintainability review is not optional for non-trivial code. Before
completion, report whether touched files, modules, crates, components,
dependencies, and duplicated logic stayed within the project quality bar.
