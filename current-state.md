# Current State

Last updated: 2026-07-01 UTC

## Current Direction

RunHaven is shifting from terminal-first polish to a native-feeling macOS GUI as
the primary easy path. The CLI remains the complete technical, automation,
recovery, and verification surface. The TUI remains a fallback/dev surface and
should be finished only where it hardens the shared launch, session, security,
and verification contracts.

Do not push this branch, tag, version-bump, cut a release, notarize, add an
updater, or do public distribution work unless the maintainer explicitly asks
for that pass. Current GUI prototype work should not require a paid Apple
Developer Program membership.

## Startup Contract

Always-read files are:

- `AGENTS.md`
- `feature_list.json`
- `current-state.md`

Do not recreate root `progress.md` or `session-handoff.md`. Load deeper docs
only when the task touches that surface.

## Product Facts

- RunHaven is a Rust 1.96.0 workspace for running AI coding agents inside Apple
  `container` on macOS 26+ on Apple silicon.
- Windows and Linux are not supported runtime or contributor-verification
  targets.
- The safe default is task-scoped `container run`, not `container machine`.
- Secure defaults must be the easiest path. Supported lower-security choices
  warn and require explicit intent. Unsupported or hard-boundary violations
  fail closed.
- Do not mount host home directories, cloud credential folders, raw SSH keys,
  browser profiles, or arbitrary host environment variables by default.
- Provider egress stays default-deny in provider mode. Agent provider hosts are
  allowlisted; arbitrary data-egress hosts stay closed.
- `--ssh` remains fail-closed until reopened by a no-secret runtime proof.
- User-facing writing is product behavior. UI text, menus, prompts, warnings,
  docs, and setup instructions should be short, concrete, and nontechnical.

## Workspace Ownership

| Area | Path | Owns |
| --- | --- | --- |
| Binary entrypoints | `crates/runhaven/` | `runhaven` and `runhaven-check-pins`; bare-interactive TUI routing. |
| Core library | `crates/runhaven-core/` | Runtime, provider, records, image, doctor, diagnostics, support, harness pins, and shared UI contracts. |
| CLI presentation | `crates/runhaven-cli/` | Clap dispatch, setup text, and human CLI output. |
| Terminal UI | `crates/runhaven-tui/` | Codex-vendored TUI source plus RunHaven TUI adapters. |
| Desktop Rust shell | `src-tauri/` | Narrow typed Tauri commands over `runhaven-core`. |
| Frontend | `ui/` | React + shadcn/ui desktop WebView prototype over typed Tauri commands. |

`crates/runhaven` stays binary-only. Shared runtime truth belongs in
`runhaven-core`; presentation belongs in CLI, TUI, Tauri, or frontend layers.

## Harness Rules

- Repo docs, startup state, repo-local skills, generated docs metadata, and
  examples must not cite machine-local paths, sibling checkouts, download
  folders, or symbolic local-external placeholders. Use repo-relative paths,
  public URLs, or pinned upstream source references.
- `scripts/check-docs-local-paths.sh` is the focused privacy guard.
- `scripts/check-repo-structure.sh` is the structural guard. It enforces static
  file-size ceilings for every tracked file category plus 90% headroom for
  splittable repo-owned text files. It also enforces workspace membership, crate
  dependency direction, binary-only entrypoint ownership, and Tauri/Core boundary
  rules.
- Split by ownership before a file reaches the ceiling. Raise a limit only for
  generated, vendored, or binary artifacts with a recorded reason.

## GUI State

React + shadcn/ui + Tauri is the selected first implementation path unless a
specific blocker proves another stack is better. Keep the existing typed
Tauri/Rust boundary. Do not add generic command execution, filesystem RPC, MCP,
app-server transport, login bridges, resize, attach, or host-reaching Codex
paths without a designed and tested RunHaven boundary.

Design direction: dark by default, logo-informed, restrained, and native
feeling. Use a dark navy base, subtle glass-like panels, blue primary actions,
teal/green ready states, amber warnings, and red only for blocked or destructive
states. Avoid corny safety copy. The canonical logo is `docs/assets/logo.png`.

Useful GUI design docs:

- `docs/plans/native-gui-reference-pass.md`
- `docs/plans/native-gui-stack-spike.md`
- `docs/plans/native-gui-wireframe.md`
- `docs/plans/native-gui-managed-sessions.md`
- `docs/plans/native-gui-session-input-contract.md`
- `docs/plans/native-gui-microsandbox-reference.md`

## Latest State

- CLI-complete v0.5.0 scope remains closed.
- Desktop/Tauri managed-session work has live evidence for launch, bounded
  output, input, stop, cleanup, distinct GUI session identity, and per-session
  state naming.
- Tauri command tests are split under `src-tauri/src/commands/tests/`; the old
  monolithic `tests.rs` file must not return.
- Shared UI contracts are split under `crates/runhaven-core/src/ui_contracts/`.
- Runtime plan tests are split under
  `crates/runhaven-core/src/runtime/plans/tests.rs`.
- The structure guard is wired into `AGENTS.md`, `init.sh`,
  `docs/harness/feedback/verification-matrix.md`, and
  `docs/harness/feedback/sensor-registry.md`.
- Large or near-ceiling owned files were split by ownership across Tauri tests,
  shared UI contracts, runtime/provider helpers, CLI plan output, React GUI
  helpers, TUI launch/MVP/service modules, TUI tests, usage docs, release docs,
  and the Codex TUI source-map doc.

## Current Branch Work

Branch: `docs/native-gui-wireframe`.

Active slice: finish verification for the hard structure guard, then turn the
GUI push from an informal direction into a hard goal prompt aimed at getting
from roughly 20-25 percent polished to 90 percent external-tester-ready.

Smallest relevant checks for this slice:

```bash
scripts/check-repo-structure.sh
scripts/check-docs-local-paths.sh
python3 -m json.tool feature_list.json >/dev/null
git diff --check
```

Add Rust, Tauri, frontend, or live smoke checks when touched files require
them.

## Next Step

Finish focused verification, then start the GUI goal from the hardest problems
first: live container session ownership, real-agent auth/session behavior,
multi-session isolation, typed status reporting, and fail-closed security
boundaries. Keep the safe path short and nontechnical.
