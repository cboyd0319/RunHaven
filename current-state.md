# Current State

Last updated: 2026-07-01 UTC

## Current Direction

RunHaven's active product work is now CLI plus native-feeling desktop GUI only.
The CLI remains the complete technical, automation, recovery, and verification
surface. The GUI is the primary easy path for nontechnical users. TUI work is
backlogged for the foreseeable future and should not be expanded, polished,
refactored, tested, or planned unless the user explicitly reopens that scope or
a CLI/GUI change requires compile-only maintenance.

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
| Binary entrypoints | `crates/runhaven/` | `runhaven` and `runhaven-check-pins`; bare-interactive TUI routing stays compile-only/backlogged. |
| Core library | `crates/runhaven-core/` | Runtime, provider, records, image, doctor, diagnostics, support, harness pins, and shared UI contracts. |
| CLI presentation | `crates/runhaven-cli/` | Clap dispatch, setup text, and human CLI output. |
| Terminal UI | `crates/runhaven-tui/` | Codex-vendored TUI source plus RunHaven TUI adapters. |
| Desktop Rust shell | `src-tauri/` | Narrow typed Tauri commands over `runhaven-core`. |
| Frontend | `ui/` | React + shadcn/ui desktop WebView prototype over typed Tauri commands. |

`crates/runhaven` stays binary-only. Shared runtime truth belongs in
`runhaven-core`; active presentation work belongs in CLI, Tauri, or frontend
layers. TUI presentation work is backlogged.

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

Refresh, polling, and status updates must keep the current workspace stable.
Session views should update panels in place without full-page blink/remounts,
without clearing user input drafts, and without falling back to the launch
wizard unless the user changes context or the selected session disappears.

Useful GUI design docs:

- `docs/plans/native-gui-reference-pass.md`
- `docs/plans/native-gui-stack-spike.md`
- `docs/plans/native-gui-wireframe.md`
- `docs/plans/native-gui-managed-sessions.md`
- `docs/plans/native-gui-session-input-contract.md`
- `docs/plans/native-gui-microsandbox-reference.md`

## Latest State

- CLI-complete v0.5.0 scope remains closed and was reverified on 2026-07-01:
  CLI surface breadth 39/39, CLI/core tests, CLI/core clippy, non-TTY CLI help
  routing, pin policy, structure guard, docs local-path guard, JSON validation,
  diff hygiene, and the full Apple container smoke with provider egress plus
  SSH fail-closed all passed.
- Desktop/Tauri managed-session work has live evidence for launch, bounded
  output, input, stop, cleanup, distinct GUI session identity, and per-session
  state naming.
- GUI launches now have a Rust-owned launch status path. `launch_run` records
  backend-owned starting, running, finished, or failed state after native
  confirmation, and `get_launch_status` exposes only sanitized status metadata
  while the GUI waits for a managed session to appear.
- Managed-session input visibility now fails closed: sessions are marked
  input-attached only while the backend next action is `monitor`, and selected
  session runtime status detaches input when container status is unavailable.
- Multi-session sidebar rows now show compact agent, auth, login, network, and
  input identity so users can distinguish sessions before selecting them.
- Selected GUI sessions now have a read-only typed runtime status path:
  `get_managed_session_status` resolves the backend-owned session ID in Rust
  and returns sanitized container status for the runtime card without exposing
  raw workspace paths, state volumes, mounts, environment values, or launch
  arguments.
- The GUI quietly refreshes active/recent managed sessions after launch or
  selection and suppresses no-op session-list updates to avoid visible page
  blink while status changes.
- Legacy WebView-readable dashboard and run-status responses now omit state
  volume names and return display-safe workspace labels instead of raw host
  paths. Plan and launch review still carry reviewed launch details.
- API key broker env-var names are fixed per agent: `OPENAI_API_KEY`,
  `ANTHROPIC_API_KEY`, and `GEMINI_API_KEY`. Those names flow through the
  shared planner, typed Tauri plan/launch request, and GUI advanced launch
  field for brokered Codex, Claude, and Gemini profiles only. Raw env
  passthrough remains a separate warned path, and arbitrary host env names fail
  closed.
- GUI launch review now shows a typed access preflight. It defaults to the
  shared agent login state created by `runhaven login`, explicitly says that
  state is shared across projects on this Mac, still lets users choose a
  project session, and blocks launch before background start if an API key
  broker variable name is present but not set. The GUI success message says
  `Run accepted`, not `Run started`, because the Tauri command returns before
  the background container is fully proven running.
- The real-agent GUI live smoke requires an explicit broker env for the chosen
  profile, waits for a running monitor session, and requires bounded non-empty
  output. It is brokered-launch coverage, not TTY or OAuth/login parity.
- Selected-session runtime status now returns typed recovery guidance when live
  container status is unavailable. Generic failures use refresh/diagnostics
  guidance first; marker repair is reserved for a proven available next action.
  The GUI can show a plain next step without exposing backend errors, raw
  paths, mounts, state volumes, environment values, or launch arguments.
- Selected sessions now have on-demand session-scoped diagnostics. The GUI sends
  only the managed session ID; Rust resolves the backend-owned run and returns
  bounded auth broker and provider network records filtered to that run.
- GUI sensitive actions no longer rely on WebView booleans as the trust
  boundary. Launch, raw output viewing, stop, hard stop, and repair use
  Rust-owned native confirmation before the backend starts or mutates anything.
  Session input uses a backend-held grant bound to the selected session ID plus
  run ID, and the grant is cleared when the Rust-owned writer unregisters.
  Frontend confirmation fields remain user-intent preconditions, not security
  approval.
- Selected GUI sessions now support narrow worktree cleanup for eligible stopped
  managed sessions. The frontend sends only the managed session ID plus user
  intent; Rust resolves the backend run, rejects active markers, stale
  worktrees, unsupported history, and unconfirmed requests before mutation,
  asks for native confirmation, discards only the verified RunHaven worktree,
  and clears stale cleanup affordances from the response.
- Managed-session cleanup availability now fails closed from the Rust-owned read
  model: active records never advertise cleanup, and recent records advertise it
  only when the recorded worktree lifecycle still verifies.
- The GUI command-surface guards now positively pin sensitive run-control
  commands, including cleanup, to the `run-control` capability and scan
  production frontend code for direct Tauri imports or forbidden host-reaching
  command names.
- GUI session switching preserves per-session message drafts, sending state,
  accepted notices, and errors. Quiet managed-session refreshes now compare all
  rendered identity, display, input, and next-action fields so meaningful state
  changes are not hidden while no-op refreshes stay stable.
- The GUI live smoke script keeps real-agent broker coverage opt-in via
  `RUNHAVEN_GUI_LIVE_REAL_AGENT=1`, bounds and redacts captured output on
  failure, and keeps the default smoke focused on live shell multi-session
  input/output/control without requiring credentials.
- Pre-release native design resources are recorded in
  `docs/plans/native-gui-reference-pass.md` for the later macOS design and app
  asset audit. That audit is deferred until CLI and GUI workflows are
  functionally complete.
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
  helpers, usage docs, release docs, and prior TUI checkpoint files. Current
  active work should not continue TUI splits.
- Managed-session auth status is now truthful from run-owned proof instead of
  static agent capability. The Rust read model distinguishes API key broker,
  sign-in-needed, no-auth-needed, and unknown, and never lets a live session
  claim broker auth just because its agent supports it. The active-run record
  writer persists the broker env-var name (never a secret value), so live
  brokered sessions prove API key broker instead of conservatively showing
  sign-in-needed; finished brokered sessions prove it through the history
  auth-broker channel. Both proof channels plus the shell no-auth-needed path
  are covered by read-model tests, session-input readiness fixtures were moved to
  the new trust model, and the managed-session UI-contract tests were split by
  ownership into `session_cleanup.rs` to stay under the structure ceiling.
- The GUI managed-session list now surfaces orphaned runs for recovery using the
  authoritative container signal. `get_managed_sessions` reconciles each active
  (running or starting) marker against Apple `container inspect` (the same check
  marker repair uses); a marker whose container is proven gone is shown as "Not
  responding" with a `recover` action via the pure core
  `ManagedSessionListData::mark_stale_sessions`. This can never contradict repair
  (identical signal), never false-flags a live session, is fail-safe (a missing
  `container` CLI or transient error leaves the session untouched), and injects
  the container check so tests stay hermetic. The pure list read model is
  unchanged; the container check lives at the Tauri command layer, and input
  stays fail-closed (only a `monitor` action accepts input). An earlier
  `host_pid` liveness probe was rejected in review because a launcher-dead but
  container-alive session would be mislabeled and dead-end against repair.

## Current Branch Work

Branch: `docs/native-gui-wireframe`.

Active slice: bring CLI plus native-feeling desktop GUI to 100 percent ready
for real external user testing. Start by keeping TUI work out of scope, then
continue hardest-first through live session ownership, real-agent auth/session
behavior, multi-session isolation, typed status, run control, recovery, and
fail-closed boundaries before visual polish.

For this CLI+GUI completion goal, local downloads, installs, tool runs, and
external research are authorized when they directly reduce risk or accelerate
the work. Multiple subagents are authorized for real bounded implementation,
review, and test work when their scopes are independent; the lead agent remains
responsible for architecture, integration, and final verification.

Smallest relevant checks for this slice:

```bash
scripts/check-repo-structure.sh
scripts/check-docs-local-paths.sh
python3 -m json.tool feature_list.json >/dev/null
git diff --check
```

Add Rust, Tauri, frontend, or live smoke checks when touched files require
them. Per-crate `cargo test -p runhaven-core` and `-p runhaven-tauri` plus
`cargo build --workspace` are clean. `cargo test --workspace` has pre-existing,
out-of-scope failures that stay on the backlog and are not RunHaven CLI/GUI
regressions: the backlogged TUI test fixtures need `ActiveRunSummaryData`'s
`auth_scope`, and the vendored `codex-exec-server` integration tests do not
compile (unresolved `codex_test_binary_support`). Do not spend the CLI/GUI push
fixing TUI.

## Next Step

CLI is reverified and should stay contract-preserving except for bug fixes,
security fixes, docs corrections, pin updates, or internal GUI support that
preserves CLI semantics. Continue hardest-first on real-agent auth/session
correctness beyond shell, GUI recovery beyond verified worktree cleanup,
CLI/GUI parity checks, fail-closed security boundary checks, and a current
visual screenshot pass. Keep the safe path short, stable, and nontechnical.
