# Verification Matrix

Use the smallest check set that can catch likely regressions. Escalate to
`./init.sh` when a change crosses components, affects runtime behavior, or
changes a security boundary.

## Common Checks

| Change Type | Checks |
| --- | --- |
| Harness docs, instructions, or state | `cargo run --locked --bin runhaven-check-pins`; JSON validation for changed JSON; local Markdown link check when links changed; `git diff --check` |
| README or docs-only change | pin check; local Markdown link check when links changed; platform wording scan when support wording changed; `git diff --check` |
| Rust code | `cargo fmt --check`; focused `cargo test` target; `cargo test --locked`; `cargo clippy --all-targets -- -D warnings`; maintainability check for touched modules and duplication |
| Frontend UI | `npm --prefix ui run check`; `npm --prefix ui test`; `npm --prefix ui run test:e2e`; `npm --prefix ui run build`; relevant Tauri command tests; maintainability check for touched components and adapters |
| Tauri shell | Frontend checks plus `cargo fmt --manifest-path src-tauri/Cargo.toml --check`; `cargo test --manifest-path src-tauri/Cargo.toml --locked`; `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings`; capability review |
| CLI command construction | Rust checks plus focused CLI and planning tests |
| Code organization or modularity | Focused tests for moved behavior; stale import/reference scan with `rg`; relevant Rust, Tauri, or frontend checks; verify duplicated logic was deleted or intentionally kept |
| Apple `container` runtime boundary | Rust checks plus `runhaven doctor`, `runhaven plan`, and a focused runtime smoke proving the claimed mount, user, network, or filesystem behavior |
| Provider egress or endpoint policy | Focused egress/provider tests; source review for endpoint changes; `scripts/apple_container_smoke.sh --with-provider` when behavior changes and the host is available |
| SSH forwarding boundary | Planner and CLI tests proving `--ssh` fails closed; `scripts/apple_container_smoke.sh --with-ssh` only when changing that guard |
| Auth or secrets | Focused auth/provider tests; confirm diagnostics and records do not expose secrets; confirm secure defaults stay easiest and lower-security choices are explicit and warned |
| Release prep | `./init.sh`; `runhaven doctor`; relevant Apple container smokes; pin/source review; dirty-tree check |

## Detected Commands

```bash
cargo fmt --check
cargo test --locked
cargo clippy --all-targets -- -D warnings
cargo run --locked --bin runhaven-check-pins
cargo build --locked
npm --prefix ui run check
npm --prefix ui test
npm --prefix ui run test:e2e
npm --prefix ui run build
npm --prefix ui run tauri:build
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
