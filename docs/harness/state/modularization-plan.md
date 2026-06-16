# Modularization Plan

Status: active
Last reviewed: 2026-06-16

RunHaven is now a Rust CLI. Keep the crate organized by ownership boundary,
not as a flat `src/` directory. Root `src/` should stay limited to entrypoints
and the crate module root.

## Current Layout

| Area | Primary Files | Boundary |
| --- | --- | --- |
| Entrypoints | `src/main.rs`, `src/lib.rs`, `src/bin/runhaven-check-pins.rs` | Binary startup and exported crate surface |
| CLI | `src/runhaven/cli/app.rs`, `src/runhaven/cli/args.rs`, `src/runhaven/cli/diagnostics.rs`, `src/runhaven/cli/doctor.rs`, `src/runhaven/cli/lock.rs`, `src/runhaven/cli/setup.rs` | Clap schema, command dispatch, diagnostics, prerequisite UX, state-volume locking |
| Runtime planning | `src/runhaven/runtime/plans/`, `src/runhaven/runtime/profiles.rs`, `src/runhaven/runtime/session_state.rs`, `src/runhaven/runtime/state.rs`, `src/runhaven/runtime/network.rs` | Command construction, profiles, state volumes, managed networks |
| Active runs and worktrees | `src/runhaven/runtime/active/`, `src/runhaven/runtime/worktrees/` | Active markers, attach/log/status/repair, worktree lifecycle |
| Provider boundary | `src/runhaven/provider/egress.rs`, `src/runhaven/provider/egress/`, `src/runhaven/provider/runtime.rs`, `src/runhaven/provider/auth_broker.rs`, `src/runhaven/provider/auth_broker/`, `src/runhaven/provider/auth_profiles.rs`, `src/runhaven/provider/endpoints.rs`, `src/runhaven/provider/observability.rs` | Provider proxy, internal network runtime, auth broker, logs |
| Images | `src/runhaven/image/assets.rs`, `src/runhaven/image/build.rs`, `src/runhaven/image/doctor.rs`, `images/` | Bundled image assets, build plans, stale/missing image review |
| Records | `src/runhaven/records/history.rs`, `src/runhaven/records/history/` | Run ledger, log joins, live diff routing |
| Support | `src/runhaven/support/git.rs`, `src/runhaven/support/paths.rs`, `src/runhaven/support/project_checks.rs`, `src/runhaven/support/shell.rs`, `src/runhaven/support/validators.rs` | Shared helpers with no CLI ownership |
| Harness | `src/runhaven/harness/pins.rs` | Repo pin policy executable logic |

## Size Guard

Prefer files under roughly 500 lines. A file may exceed that only when it is a
cohesive command surface or boundary implementation and splitting would make
the behavior harder to follow.

Current largest Rust files after the 2026-06-16 conversion:

- `src/runhaven/cli/app.rs`: 494 lines, dispatcher and command orchestration.
- `src/runhaven/provider/egress.rs`: 492 lines, synchronous CONNECT proxy.
- `src/runhaven/provider/auth_broker.rs`: 473 lines, Codex API-key broker
  lifecycle.
- `src/runhaven/cli/args.rs`: 407 lines, Clap schema.
- `src/runhaven/provider/runtime.rs`: 383 lines, provider run lifecycle.

No Rust source file is currently over the size guard.

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
