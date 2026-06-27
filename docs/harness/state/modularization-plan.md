# Modularization Plan

Status: active
Last reviewed: 2026-06-27

RunHaven is a Rust workspace. Keep ownership at the crate boundary first, then
at module boundaries inside each crate. Do not rebuild a root compatibility
facade.

## Current Layout

| Area | Primary Files | Boundary |
| --- | --- | --- |
| Binary entrypoints | `crates/runhaven/src/main.rs`, `crates/runhaven/src/bin/runhaven-check-pins.rs` | Process startup only. No shared runtime truth. |
| Core library | `crates/runhaven-core/src/` | Runtime planning/control, provider boundary, records, images, doctor checks, diagnostics, support helpers, harness pin logic, and shared UI contracts. |
| CLI presentation | `crates/runhaven-cli/src/` | Clap schema, command dispatch, setup text, and human CLI output. |
| Terminal UI | `crates/runhaven-tui/src/tui/` | Codex-vendored terminal UI source plus RunHaven TUI adapters over core data. |
| Desktop shell | `src-tauri/src/`, `src-tauri/capabilities/` | Typed Tauri commands over `runhaven-core`; no generic host bridge. |
| Frontend | `ui/src/` | Alpha desktop UI over typed Tauri commands. |
| Bundled images | `images/` | Agent image templates, package pins, and non-root runtime setup. |
| Docs and harness | `docs/`, `AGENTS.md`, `feature_list.json`, `current-state.md` | Product docs, startup state, and verification routing. |

## Size Guard

Prefer RunHaven-owned Rust files under roughly 500 lines. A file may exceed that
only when it is a cohesive command surface or boundary implementation and
splitting would make behavior harder to follow.

The Codex-vendored TUI baseline is a special case. Many files in
`crates/runhaven-tui/src/tui/` are large because they preserve upstream Codex
structure for app shell, bottom pane, composer, keymaps, resume picker,
history cells, and tests. Do not churn those files only to satisfy the local
size guard. During TUI integration, cull or split only with a recorded reason
showing why removal or adaptation is better than leaving the vendored source in
place.

Current largest Rust files outside generated build output (2026-06-27):

- `crates/runhaven-tui/src/tui/bottom_pane/chat_composer.rs`: about 11,203 lines, vendored Codex composer.
- `crates/runhaven-tui/src/tui/resume_picker.rs`: about 6,351 lines, vendored Codex resume picker.
- `crates/runhaven-tui/src/tui/app/tests.rs`: about 6,263 lines, vendored Codex app tests.
- `crates/runhaven-tui/src/tui/bottom_pane/textarea.rs`: about 3,837 lines, vendored Codex textarea.
- `crates/runhaven-tui/src/tui/keymap.rs`: about 2,947 lines, vendored Codex keymap setup.

For non-vendored RunHaven-owned code, split when a file crosses boundaries or
new work would make review harder.

## Split Triggers

Split a Rust file or crate when:

- a change requires touching unrelated behavior in the same file;
- tests need to mock internals instead of public behavior;
- code crosses responsibilities between CLI parsing, runtime execution,
  persistence, network security, TUI rendering, Tauri IPC, or harness policy;
- duplicated logic appears in more than one crate;
- a cohesive extraction is obvious and makes the secure path easier to review.

Avoid splitting when the only benefit is a smaller line count and the result
would hide a security-sensitive flow across too many files.

## Verification

For structural changes, run:

```bash
cargo fmt --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo run --locked --bin runhaven-check-pins
```

Run `./init.sh` before ending broad architecture or harness-maintenance work on
macOS 26+ when frontend and Tauri packaging should also be rechecked.
