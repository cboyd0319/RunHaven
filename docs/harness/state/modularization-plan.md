# Modularization Plan

Status: active
Last reviewed: 2026-06-27

RunHaven is now a Rust CLI. Keep the crate organized by ownership boundary,
not as a flat `src/` directory. Root `src/` should stay limited to entrypoints
and the crate module root.

## Current Layout

| Area | Primary Files | Boundary |
| --- | --- | --- |
| Entrypoints | `src/main.rs`, `src/lib.rs`, `src/bin/runhaven-check-pins.rs` | Binary startup and exported crate surface |
| CLI | `src/runhaven/cli/app.rs`, `src/runhaven/cli/args.rs`, `src/runhaven/cli/diagnostics.rs`, `src/runhaven/cli/setup.rs` | Clap schema, command dispatch, and human presentation |
| Terminal UI | `src/runhaven/cli/tui/` | Interactive launcher/run manager over shared planner, diagnostics, doctor, records, and active-run APIs |
| Host readiness | `src/runhaven/doctor.rs`, `src/runhaven/doctor/runtime_pins.rs` | macOS, Rust, Apple `container`, and runtime pin checks shared by CLI, Tauri, TUI, and launch preflight |
| Shared diagnostics data | `src/runhaven/diagnostics.rs` | Secret-free egress/auth log readers and auth status payloads shared by CLI, Tauri, and TUI |
| Runtime planning | `src/runhaven/runtime/plans/`, `src/runhaven/runtime/profiles.rs`, `src/runhaven/runtime/session_state.rs`, `src/runhaven/runtime/state.rs`, `src/runhaven/runtime/lock.rs`, `src/runhaven/runtime/network.rs` | Command construction, profiles, state volumes, state-volume locks, managed networks |
| Active runs and worktrees | `src/runhaven/runtime/active/`, `src/runhaven/runtime/worktrees/` | Active markers, attach/log/status/repair, worktree lifecycle |
| Provider boundary | `src/runhaven/provider/egress.rs`, `src/runhaven/provider/egress/`, `src/runhaven/provider/runtime.rs`, `src/runhaven/provider/auth_broker.rs`, `src/runhaven/provider/auth_broker/`, `src/runhaven/provider/auth_profiles.rs`, `src/runhaven/provider/endpoints.rs`, `src/runhaven/provider/observability.rs` | Provider proxy, internal network runtime, auth broker, logs |
| Images | `src/runhaven/image/assets.rs`, `src/runhaven/image/build.rs`, `src/runhaven/image/doctor.rs`, `images/` | Bundled image assets, build plans, stale/missing image review |
| Records | `src/runhaven/records/mod.rs`, `src/runhaven/records/io.rs`, `src/runhaven/records/run_history.rs`, `src/runhaven/records/run_history/` | Records facade, generic JSONL IO, run ledger, log joins, live diff routing |
| Support | `src/runhaven/support/git.rs`, `src/runhaven/support/paths.rs`, `src/runhaven/support/project_checks.rs`, `src/runhaven/support/shell.rs`, `src/runhaven/support/validators.rs` | Shared helpers with no CLI ownership |
| Harness | `src/runhaven/harness/pins.rs` | Repo pin policy executable logic |

## Size Guard

Prefer files under roughly 500 lines. A file may exceed that only when it is a
cohesive command surface or boundary implementation and splitting would make
the behavior harder to follow.

Current largest Rust files in `src/` (2026-06-27):

- `src/runhaven/cli/tui/mod.rs`: about 748 lines, TUI orchestration.
- `src/runhaven/cli/tui/runs.rs`: about 648 lines, run dashboard state.
- `src/runhaven/cli/tui/pet.rs`: about 642 lines, Cubby pet rendering.
- `src/runhaven/provider/runtime.rs`: about 580 lines, provider run lifecycle.
- `src/runhaven/runtime/plans/mod.rs`: about 584 lines, run plan construction.
- `src/runhaven/provider/egress.rs`: about 525 lines, synchronous CONNECT proxy.
- `src/runhaven/provider/auth_broker.rs`: about 512 lines, API-key broker
  lifecycle.
- `src/runhaven/cli/app.rs`: about 499 lines, dispatcher and command
  orchestration.

The TUI files exceed the soft size guard while the reference implementation is
being built. Split them by screen/framework boundary whenever a new phase would
make review harder. Provider/runtime files are security boundaries and should
split only along clear ownership lines.

## Split Triggers

Split a Rust file when:

- a change requires touching unrelated behavior in the same file;
- tests need to mock internals instead of public behavior;
- a file crosses responsibilities between CLI parsing, runtime execution,
  persistence, network security, and docs/harness policy;
- the module exceeds the size guard and a cohesive extraction is obvious.

Avoid splitting when the only benefit is a smaller line count and the result
would hide a security-sensitive flow across too many files.

## Verification

For structural changes, run:

```bash
cargo fmt --check
cargo test --locked
cargo clippy --all-targets -- -D warnings
cargo run --locked --bin runhaven-check-pins
```

Run `./init.sh` before ending broad architecture or harness-maintenance work on
macOS 26+.
