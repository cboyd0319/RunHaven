# Current State

Last updated: 2026-06-27 UTC

## Current Objective

The active slice is the TUI Codex vendor reset plus the Rust workspace
normalization needed to make that TUI a clean reference implementation.

RunHaven is replacing its previous custom TUI with the local Codex TUI source
baseline from:

```text
/Users/c/Documents/GitHub/codex/codex-rs/tui/src/
```

The RunHaven TUI is the reference implementation for several sibling projects.
Keep Codex vendoring source-first, RunHaven behavior in thin adapters, shared
data contracts in `runhaven-core`, every culling decision documented, and
user-facing copy plain enough for non-technical users.

Do not publish a release from the interim vendor-reset state. After the TUI is
fully integrated, verified, and confirmed, do a full release bump to `v0.6.0`.

## Startup State Contract

- `AGENTS.md`: root instruction map.
- `feature_list.json`: compact feature status and next product slice.
- `current-state.md`: current facts, blockers, and handoff.

Do not recreate separate root `progress.md` or `session-handoff.md` files.
Load deeper docs only when the task touches that surface.

## Product Facts

- RunHaven is a Rust 1.96.0 workspace for running AI coding agents inside Apple
  `container` on macOS 26+ on Apple silicon.
- The `v0.5.0` CLI-complete pre-release was cut and published on 2026-06-26.
- The CLI remains the complete automation and recovery backend.
- The terminal UI is unreleased and active. A bare interactive `runhaven`
  should open the TUI when the TUI is integrated; pipes, redirection, and
  explicit subcommands stay CLI-first.
- The alpha desktop shell lives under `ui/` and `src-tauri/`. `src-tauri` is a
  Rust workspace member over typed `runhaven-core` commands. The desktop shell
  remains deferred to a later first-class release phase.
- Windows and Linux are not supported runtime or contributor-verification
  targets.
- GitHub Actions CI is disabled during alpha/pre-release. Local macOS 26+
  verification is authoritative until a maintainer explicitly re-enables CI.

## Rust Source Layout

| Area | Path | Owns |
| --- | --- | --- |
| Binary entrypoints | `crates/runhaven/` | `runhaven` and `runhaven-check-pins` startup only. |
| Core library | `crates/runhaven-core/` | Runtime, provider, records, image, doctor, diagnostics, support, harness pin logic, and shared UI contracts. |
| CLI presentation | `crates/runhaven-cli/` | Clap dispatch, setup text, and human CLI output. |
| Terminal UI | `crates/runhaven-tui/` | Codex-vendored TUI source plus RunHaven TUI adapters. |
| Desktop Rust shell | `src-tauri/` | Narrow typed Tauri commands over `runhaven-core`. |
| Frontend | `ui/` | Alpha Svelte desktop UI. |

`crates/runhaven` is binary-only. Do not rebuild a root compatibility facade.
Shared runtime truth belongs in `runhaven-core`; presentation belongs in CLI,
TUI, Tauri, or frontend layers.

## Key Decisions

- Secure defaults must be the easiest path. Supported lower-security choices
  warn and require explicit intent. Unsupported or hard-boundary violations
  fail closed.
- Default runs use task-scoped `container run`, not `container machine`,
  because normal machine workflows can expose host home and credentials.
  Explicit or user-managed machine workflows are not blocked solely for being
  less secure, but any RunHaven machine integration must warn and require
  intent.
- Do not mount host home directories, cloud credential folders, raw SSH keys,
  browser profiles, or arbitrary host environment variables by default.
- `--ssh` remains fail-closed. Apple `container` 1.0.0 exposes the forwarded
  socket to the non-root guest, but `ssh-add -l` returns permission denied.
  Reopen only with a no-secret runtime proof.
- Provider egress stays default-deny in provider mode. Agent provider domains
  can be expressed as reviewed stable domain-family patterns, but data-egress
  hosts stay closed by not being in the allowlist.
- Log sanitization and host-held secret storage are separate non-TUI security
  slices. Because RunHaven is macOS 26+ only, RunHaven-owned host secrets should
  prefer macOS Keychain where practical. This does not authorize reading
  provider-owned Keychain items, browser profiles, cloud credential stores, or
  arbitrary host credentials.
- TUI image and pet rendering must follow Codex source behavior. Use the local
  Codex TUI source and local Codex config evidence before writing custom pet,
  terminal image, statusline, bottom-pane, keymap, title, or resume behavior.
- User-facing writing is product behavior. UI text, menus, prompts, warnings,
  README/usage docs, and setup instructions target non-technical users at about
  an 8th grade reading level.
- The hidden Zork I easter egg remains wanted. The current reset keeps the
  MIT-licensed `historicalsource/zork1` collection under `third_party/zork1/`.
  The earlier Ferrif-derived TUI engine was removed with the old custom TUI and
  is recoverable from git history. If reintroduced, it must stay TUI-local,
  attributed, offline, and carefully validate save/restore files.
- The glib advisory GHSA-wrw7-89jp-8q8g remains treated as not affected because
  `glib` enters only through Tauri's Linux GTK backend and is absent from the
  macOS build graph. See `docs/PINNING.md`.

## Latest Verified Work

2026-06-27: Workspace crate split complete. The Rust codebase now uses
workspace crates:

- `crates/runhaven` for binary entrypoints.
- `crates/runhaven-core` for shared runtime truth and UI contracts.
- `crates/runhaven-cli` for CLI presentation.
- `crates/runhaven-tui` for the Codex-vendored TUI.
- `src-tauri` as a workspace member.

This phase also removed the obsolete separate Tauri lockfile, made root Cargo
commands cover Tauri, narrowed public crate exports, kept `runhaven`
binary-only, updated `init.sh`, and refreshed active architecture, harness,
pinning, TUI, and state docs to the new layout.

Verified:

- `cargo fmt --check`
- `cargo test --workspace --locked --quiet`
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo run --locked --bin runhaven-check-pins --quiet`
- `cargo build --workspace --locked --quiet`
- `jq empty feature_list.json`
- active stale-reference scans
- `git diff --check`
- `./init.sh`

## Blockers

- SSH forwarding remains fail-closed as described above.

## Next Step

Commit and push this architecture phase, then do a repo-wide organization audit
for stale directories, junk files, stray docs, and layout drift.
