---
name: codex-tui
description: "Use for RunHaven TUI work that vendors, adapts, audits, or verifies Codex Rust TUI source."
---

# RunHaven Codex TUI

Use this skill for RunHaven TUI work that vendors, adapts, audits, or verifies
Codex Rust TUI source. This includes terminal ownership, `App` and
`ChatWidget`, `BottomPane`, transcript rendering, status/footer chrome, pets,
terminal images, snapshot tests, vendored `codex-*` crates, and drift guards.

RunHaven follows a source-first Strategy C reset. The active goal is to replace
the former custom TUI with Codex's TUI architecture while keeping RunHaven's
security boundary intact.

## Required Source Skill

Read the Persona Codex TUI skill before making source-level Codex claims:

```text
/Users/c/Documents/GitHub/persona/content/skills/codex-tui
```

Then load only the Persona references that match the touched surface:

- `references/source-map.md` for navigation and source claims.
- `references/architecture-patterns.md` for app loop, widget ownership, input,
  render, async, or adaptation decisions.
- `references/testing-and-build.md` for snapshots, VT100 tests, terminal
  behavior, features, workspace, or packaging.

If the Persona skill or local Codex checkout is unavailable, stop and report
that gap instead of guessing Codex TUI behavior.

## Evidence Order

Use evidence in this order:

1. RunHaven live files and docs.
2. Local Codex checkout:
   `/Users/c/Documents/GitHub/codex/codex-rs/tui`.
3. Pinned upstream Codex baseline recorded in `current-state.md` and
   `feature_list.json`.
4. DeepWiki or generated docs as source maps only, verified against local
   source before acting.

For active strategy work, read the focused plan file under:

```text
docs/plans/codex-tui-strategy-c/
```

`04-implementation-and-verification.md` is the implementation gate for the
current Phase 4 path.

## RunHaven Rules

- Preserve original Codex package, crate, and module names by default.
- Prefer real vendored Codex crates and modules over RunHaven shims.
- Use local bridges only when activating the real surface would compile in or
  execute unreviewed host-reaching behavior.
- Keep host-reaching Codex paths dormant or fail-closed until explicitly
  promoted through RunHaven security design and focused verification. This
  includes app-server transport, filesystem RPC, MCP execution, login,
  external editors, hooks, shell execution, cloud or browser credential access,
  and broad host environment capture.
- Do not mount host home, cloud credential folders, raw SSH keys, browser
  profiles, or arbitrary host environment variables by default.
- Do not expand `crates/runhaven-tui/src/tui/app_shell.rs` as a product
  screen. Shrink it toward native Codex `App`/`ChatWidget` ownership.
- Keep the active foreground launch path owned by
  `tui/runhaven/launch_handoff.rs` and the prepared RunHaven plan. Do not route
  RunHaven launches through Codex workspace-command, app-server transport, or
  shell-execution surfaces.
- Keep RunHaven product behavior in thin adapters and shared UI contracts in
  `runhaven-core`.
- Defer Cubby/pet polish, terminal mascot work, and the hidden Zork easter egg
  until the core RunHaven TUI is complete. Keep existing pet/image code as
  parked source-first infrastructure; do not add a live env-gated smoke path
  unless a final-pass pet slice or core terminal-image check explicitly requires
  it.
- Snapshot goldens from upstream stay external unless intentionally promoted.
  Default tests must not leave `.snap.new` files.

## Current Promotion Path

For Phase 4, prefer this order unless live plan files say otherwise:

1. Keep Codex `Tui`, `TuiEventStream`, `FrameRequester`, and `BottomPane`
   ownership active.
2. Treat real vendored `branch_summary.rs` and the `workspace_command.rs`
   contract as active for the next `ChatWidget` status-line path.
3. Remove or shrink temporary bridge types as their real owners become active.
4. Add drift and security guards for each promoted surface.
5. Keep native `App`, real `app_server_session`, app-server transport,
   filesystem RPC, MCP, login, and host-reaching paths dormant or fail-closed
   until their markers are removed or guarded.
6. Continue toward native `App`/`ChatWidget` ownership without adding product
   screens to `app_shell.rs`.

## Guard Expectations

Every non-trivial promotion needs at least one focused guard where practical:

- Source drift guard for copied or locally adapted Codex files.
- Dependency guard for reduced `codex-*` crates that must not pull backend,
  login, MCP, filesystem, exec-server, rollout, state, or thread-store
  dependencies.
- Module guard for dormant upstream files that must not be declared before
  fail-closed design exists.
- Adapter import guard so RunHaven-owned code does not bypass the approved
  boundary through backend Codex modules.
- Snapshot hygiene guard when upstream snapshot tests are parked or gated.

## Verification

Before implementation, run the smallest baseline check for the touched TUI
surface from `docs/harness/feedback/verification-matrix.md`. For ordinary
RunHaven TUI code, start with:

```bash
cargo test -p runhaven-tui --locked
```

For implementation slices, choose the smallest reliable set, then usually end
with:

```bash
cargo fmt --check
cargo check -p runhaven-tui --locked
cargo test -p runhaven-tui --locked
cargo test -p runhaven-tui --locked --features codex-vendored-tests --no-run
cargo clippy -p runhaven-tui --all-targets --locked -- -D warnings
cargo run --locked --bin runhaven-check-pins --quiet
scripts/compare-codex-tui.sh
python3 -m json.tool feature_list.json >/dev/null
find crates/runhaven-tui/src/tui -name '*.snap.new' -print
git diff --check
```

Use broader workspace checks only when the change crosses crate boundaries,
dependency graph behavior, runtime security boundaries, or release controls.

## End-of-Slice Gate

For RunHaven TUI implementation slices, run the requested review sequence
before commit:

1. `rust`: crate/tooling correctness, ownership, features, and tests.
2. Persona `codex-tui`: source-pattern alignment with local Codex.
3. `adversarial-review`: boundary, overclaim, dormant-path, and simplification
   review.

Record only durable facts and verification in `feature_list.json` and
`current-state.md`. Keep raw logs out of startup files.
