# Current State

Last updated: 2026-07-12 UTC

## Current Direction

RunHaven's active product work is now CLI plus native-feeling desktop GUI only.
The CLI remains the complete technical, automation, recovery, and verification
surface. The GUI is the primary easy path for nontechnical users. The TUI and
vendored Codex source are archived outside the active Cargo workspace and
release graph. Reopening them requires an explicit product decision and fresh
dependency/security review.

The maintainer authorized the `v0.7.0` source-version bump. Do not push this
branch, tag, cut a release, notarize, add an updater, or distribute a build
unless the maintainer explicitly asks for that pass. Producing a shareable
desktop artifact requires the paid Apple release credentials and external
verification listed below.

## Startup Contract

Always-read files are:

- `AGENTS.md`
- `feature_list.json`
- `current-state.md`

Do not recreate root `progress.md` or `session-handoff.md`. Load deeper docs
only when the task touches that surface.

## Product Facts

- RunHaven is a Rust 1.97.0 workspace for running AI coding agents inside Apple
  `container` on macOS 26+ on Apple silicon. Repository setup and doctor pins
  require Apple `container` 1.1.0.
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
- `--ssh` forwards the host SSH agent socket only (never raw keys, non-root),
  and is re-enabled after a no-secret live smoke proved it works. On macOS 26+
  with Apple `container` 1.1.0, the default launchd SSH agent socket is
  world-accessible (`srw-rw-rw-`), so the forwarded socket is usable by the
  non-root `agent` user: `runhaven run --ssh` -> `ssh-add -l` returns the
  no-identities status (verified end to end), not permission denied. It is an
  explicit, warned lower-security choice (the sandboxed agent can use the host's
  SSH identities). A private `0600` agent socket (a fresh `ssh-agent`) is
  permission-denied inside the guest, which fails safe (the agent simply cannot
  use it). See `docs/APPLE_CONTAINER_GAP_ANALYSIS.md`.
- User-facing writing is product behavior. UI text, menus, prompts, warnings,
  docs, and setup instructions should be short, concrete, and nontechnical.

## Workspace Ownership

| Area | Path | Owns |
| --- | --- | --- |
| Binary entrypoints | `crates/runhaven/` | `runhaven` and `runhaven-check-pins`. |
| Core library | `crates/runhaven-core/` | Runtime, provider, records, image, doctor, diagnostics, support, harness pins, and shared UI contracts. |
| CLI presentation | `crates/runhaven-cli/` | Clap dispatch, setup text, and human CLI output. |
| Terminal UI archive | `crates/runhaven-tui/`, `crates/codex/` | Parked source outside the release workspace and binaries. |
| Desktop Rust shell | `src-tauri/` | Narrow typed Tauri commands over `runhaven-core`. |
| Frontend | `ui/` | React + shadcn/ui desktop WebView prototype over typed Tauri commands. |

`crates/runhaven` stays binary-only. Shared runtime truth belongs in
`runhaven-core`; active presentation work belongs in CLI, Tauri, or frontend
layers. Archived TUI presentation work is out of scope.

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

Design direction: follow system appearance; stay restrained and native-feeling;
use blue actions, green ready states, amber warnings, and red only for blocked
or destructive states. The canonical logo is `docs/assets/logo.png`.

Refresh, polling, and status updates must keep the current workspace stable.
Session views should update panels in place without full-page blink/remounts,
without clearing user input drafts, and without falling back to the launch
wizard unless the user changes context or the selected session disappears.

In-app maintenance is shipped: Agents repairs images with bounded redacted
status, and Cleanup removes stale managed networks after a count preview. Both
use Rust-owned native confirmation and fail closed. State prune stays CLI-only
because it can erase sign-in state. Live mutation remains a documented
follow-up. End-user guide: `docs/DESKTOP_APP.md`.

Useful GUI design docs:

- `docs/plans/native-gui-reference-pass.md`
- `docs/plans/native-gui-stack-spike.md`
- `docs/plans/native-gui-wireframe.md`
- `docs/plans/native-gui-managed-sessions.md`
- `docs/plans/native-gui-session-input-contract.md`
- `docs/plans/native-gui-microsandbox-reference.md`
- `docs/design/native-gui-design-resources.md` (Apple HIG + macOS design
  references for the deferred comprehensive visual redesign)

## Latest State

- The bounded second adversarial pass is remediated in source. Targeted state
  deletion now holds the per-volume lock; active-marker reads and mutations are
  serialized per run and cannot recreate a removed marker; sign-in readiness
  publishes under the same state lock as launch; natural PTY EOF waits for
  cleanup, run recording, and readiness publication before reporting Exit;
  explicit teardown remains bounded. Release WebViews no longer receive the
  arbitrary log-write permission that development uses, and release artifact
  publication holds an atomic per-version lock through the final rename.
  Integrated gates pass: 249 core tests (2 ignored fixtures), 6 CLI tests, 265
  Tauri tests (5 live-only ignored), warnings-denied workspace Clippy, 405
  frontend tests, production build, 11 Playwright/axe flows, release/local-sign
  shell tests, structure/docs/JSON/diff guards, and production plus development
  Tauri config builds.
- Apple `container` 1.1.0 is installed and live-verified. Official release and
  exact-tag source review, installer SHA-256, Apple Team ID, notarization, and
  embedded CLI signature/version are verified. Pins now use release commit
  `5973b9cc626a3e7a499bb316a958237ebe14e2ed`, vminit 0.35.0, and the unchanged
  builder/Kata surface. The installed CLI and apiserver report that exact
  commit; helper properties and the host kernel hash match pins. `runhaven
  doctor`, image doctor, default runtime, provider-egress, no-secret SSH, and
  live two-session Tauri smokes pass with exact RunHaven cleanup.

- Live-test iteration (2026-07-04, adversarial gate CONCERNS -> remediated; all
  suites green): (1) COPY-2 vocabulary unified across 43 files (sign in / run /
  session; "Sign-in sharing", "Shared agent sign-in" / "This project only",
  "Finish sign-in"; display-only, wire values unchanged) with a REAL Rust<->TS
  drift pin that reads `ui/src/app/labels.ts` from a Rust test. (2) macOS shell:
  NAT-1 overlay title bar + hidden title, full-height sidebar, draggable header,
  `window-chrome` capability, minWidth 800; NAT-2 native vibrancy
  (Tauri's native `Effect::Sidebar`, transparent main
  window + macOSPrivateApi + macos-private-api feature -> App Store ineligible,
  RunHaven ships direct; body transparent, sidebar 72% glass, content opaque,
  monitors stay opaque). Vibrancy is cosmetic; tint needs a live-look tune.
  (3) Comprehensive diagnostic logging (tauri-plugin-log =2.8.0, log =0.4.33)
  to `~/Library/Logs/dev.runhaven.desktop/runhaven.log`: development-only
  WebView console + window errors + invoke tracing (name/ms/outcome,
  write/resize excluded), plus release-safe Rust run-id correlation, per-run summary, panic hook, 12 runner
  eprintln->log, >1s SLOW-invoke WARN. SECURITY: a live session leaked the broker
  Bearer token via TRACE http wire-dumps -> fixed by capping the global level
  below TRACE (DEBUG dev/INFO release) + http/tls WARN pins + a redaction filter;
  `scripts/dev-with-logs.sh` auto-scans (`scan-dev-log.sh`) on every exit.
  `logging` cap (log:allow-log, development main-only) + no-console guard;
  release WebViews have no arbitrary log-write capability; leak-gated SOUND.

- Comprehensive UI/UX audit shipped (task #25,
  `docs/design/native-gui-uiux-audit.md`): five parallel read-only lenses
  (visual, accessibility, copy, macOS-native/HIG, states/flows) over every
  destination plus the Rust user-facing strings, ranked P0-P3. Headline is one
  P0 data-loss bug (navigating away from a live interactive run kills the agent
  and strands the user, a #24 regression) plus no durable session handle for
  interactive runs. P1: terminal keyboard trap, sign-in cancel/timeout, native
  launch-confirm raw codes plus a one-vocabulary shared label source, status-icon
  tokens. P2 is the phased macOS-native shell (title bar, vibrancy, button shape,
  sheet, SF Symbols). Foundation verified strong. The P0 (#28) is fixed and gated
  SOUND: the interactive run terminal now stays mounted (hidden) across navigation
  instead of unmounting and killing the run, with a sidebar "Run terminal" handle
  to return; End run and app-quit still tear down. P1 shipped (gated CONCERNS ->
  fixed): the native launch confirm shows friendly labels instead of raw enum
  codes (COPY-1), status icons use success/warning
  tokens + ready-to-review is teal not amber (VIS-1/2), and a Shift+Esc terminal
  keyboard escape (A11Y-1). Agent sign-in now uses the terminal registry and its
  shared two-session cap; app teardown kills and reaps terminal children.
  Real CLI/GUI image mutations repeat the pinned supported-runtime gate.
  Deferred: the run/session + sign-in vocabulary rename (#33, awaiting user). P2
  Wave 1 (rounded-rect buttons, motion, terminal Cmd-C/V) + high-value P3 state
  bugs shipped. P2 NAT-1 overlay chrome + NAT-2 native vibrancy shipped; P3 nits
  held (TerminalPanel at its line ceiling needs a split first).
- Home dashboard redesign (variant 06) is shipped with secret-free setup,
  safety, recent-run, and access panels. Agent readiness now uses the same
  secret-free successful sign-in marker for Claude, Codex, Antigravity, and
  Copilot.
- Native container OAuth is the only ordinary desktop sign-in path as of
  2026-07-12. All four providers run their own login command in the shared
  terminal surface on an isolated agent volume. The host Claude setup-token,
  Keychain OAuth broker, captured-output sign-in runner, polling registries,
  Tauri commands/capabilities, token-age setting, signing entitlements, and
  direct Keychain dependencies were deleted. The optional explicit
  environment-variable API-key broker remains for Claude and Codex only.

- Managed run-network teardown no longer leaks the run container or its network on
  a mid-flow killed login/attach client. `cleanup_run_network`
  (`provider/runtime/network.rs`) force-removes the run container
  (`container rm --force`, verified on Apple container 1.0.0) before deleting the
  network; idempotent, so a clean `--rm` run still records `cleanup=deleted` and
  a real referring-container failure is surfaced, not swallowed. Single choke
  point for provider failed-login and terminal-close paths. Standard local-only
  runs now use the same cleanup after success or runner failure, preventing one
  stale network per run after per-run identity isolation. Hermetic red/green
  tests live in `provider/runtime/tests/network.rs` and `runtime/launch/tests.rs`;
  a live isolated-worktree shell run proved source isolation, diff capture,
  discard, and automatic network removal. The CLI parent now owns macOS
  termination signals through exact-pinned `signal-hook` 0.4.4, kills and
  reaps its child, then completes normal recording and cleanup. A real Ctrl-C
  smoke exited 130 with a failed record, no active marker or run container, and
  `cleanup=deleted`; SIGKILL remains recoverable, not cleanable in-process.
- Earlier verification and hardening milestones are complete (detail in
  `feature_list.json` evidence and git): live-runtime verification on a real
  macOS 27 / Apple container 1.0.0 host (five live Tauri container tests, `runs
  repair --all`, the `--with-provider --with-ssh` smoke); a keyed single-flight
  read cache (`read_guard.rs`) coalescing container subprocesses; wire-contract
  flattening (F8, `#[serde(default)]` fail-closed confirm flags); and the
  frontend punch list (coalesced status reloads, compile-time `same*` checklists,
  UX copy fixes, e2e to 7/7).
- A whole-GUI hostile analysis pass (security, frontend, shell architecture,
  empirical UX) landed in three verified waves (detail in `feature_list.json`
  and git). Verdict: the trust boundary held (every mutation behind a Rust-owned
  native dialog, backend-resolved targets, no-leak mappers); findings were
  consistency/hygiene, not boundary breaks. Fixes: a shared `confirm_native`
  dialog helper, five no-blink polling fixes, bounded status/registry helpers,
  the metered session-input grant (SECURITY_MODEL.md), control-stripped
  diagnostics, one canonical label vocabulary + accessibility/focus, dead-code
  deletion, and docs/e2e realignment. The deferred Apple-HIG visual redesign has
  its references in `docs/design/native-gui-design-resources.md`.
- Agent authentication is consistent across CLI and GUI. Claude wraps
  `claude auth login` and requires `claude auth status`; Codex runs `codex login
  --device-auth`; Antigravity wraps first-launch `agy`; Copilot runs `copilot
  login`. After Antigravity's ambiguous exit 1, its wrapper requires the fixed
  token file plus a successful non-generative `agy models` probe before the
  secret-free marker is written. Every GUI login uses a native-confirmed PTY,
  provider-only egress, non-root execution, no project
  mount, and a secret-free success marker. PTY bytes are never written by
  RunHaven. Provider CLIs own credential storage inside their isolated volumes.
  Current OpenAI source confirms Codex polls at the server interval for up to 15
  minutes before token exchange and storage under `CODEX_HOME`. After taking
  the state lock, launch also checks Apple's captured container list for the
  exact volume; conflicts fail before preparation with a fixed path-free message
  and no automatic container removal.

- GUI git-worktree launch and review parity is shipped (M4). The GUI defaults
  to recommended isolated-copy protection, validates it before review and
  native confirmation, lets users turn it off, and warns before writable direct
  mode; all possible runtime, image, configuration, and auth-selection checks run
  before creation. A later launch failure writes a failed recoverable run record
  and keeps the worktree. Read-only review and confirmed keep, merge, recover,
  and discard commands resolve backend-owned records, reject active, stale, or
  foreign targets through `verify_lifecycle`, and return sanitized results.
  No command exposes host paths, state volumes, or secrets. Shared launch and
  capability files remain split below structure ceilings.
- CLI-complete v0.5.0 scope remains closed and was reverified on 2026-07-01:
  CLI surface breadth 39/39, CLI/core tests, CLI/core clippy, non-TTY CLI help
  routing, pin policy, structure guard, docs local-path guard, JSON validation,
  diff hygiene, and the full Apple container smoke with provider egress plus
  SSH fail-closed all passed.
- Earlier GUI milestones (compact; details in `feature_list.json` evidence,
  `docs/SECURITY_MODEL.md`, and `docs/DESKTOP_APP.md`): managed sessions have
  live evidence for launch, bounded output, input, stop, and cleanup; launch,
  session status, session diagnostics, and recovery guidance are Rust-owned,
  typed, and sanitized (no raw paths, volumes, env values, or launch args);
  sensitive actions use Rust-owned native confirmation, never WebView
  booleans, and session input uses a backend-held run-bound grant; the API-key
  broker uses fixed per-agent env names with arbitrary names failing closed;
  quiet refresh compares all rendered fields so updates are in place without
  blink and drafts survive switching; worktree cleanup verifies RunHaven
  ownership before discard and availability fails closed; command-surface
  guards pin run-control commands to their capability; the GUI live smoke
  keeps real-agent broker coverage opt-in and bounded.
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
- Earlier managed-session and diagnostics milestones are complete (detail in
  `feature_list.json` evidence, `docs/DESKTOP_APP.md`, and
  `docs/SECURITY_MODEL.md`): truthful managed-session auth status from run-owned
  proof; orphaned-run recovery via container-inspect reconcile; the end-user
  guide; read-only auth-explain and why-boundary parity in the GUI; and shared
  run-output redaction at the single snapshot choke point (`runs logs -f` live
  stream warns and is not redacted).

## Current Branch Work

Branch: `docs/native-gui-wireframe`.

Navigation correction Wave A (slices A1-A6 of
`docs/plans/native-gui-navigation-gap.md`) is complete; per-slice detail lives
in the gap tracker and the `desktop-first-class-v1` evidence entries. In
short: spec sidebar taxonomy with prioritized symbol-in-dot attention badges
(`sessionAttention.ts`), the navigation union with the spec removal fallback
(`navigation.ts`, `useNavigation.ts`), per-destination views under
`ui/src/app/views/` behind a thin `AppMainContent.tsx` router, the launch
wizard extracted into the modal New Run sheet (`LaunchSheet.tsx`,
`useLaunchFlow.ts`; the Rust-owned native confirmation stays the mutation
boundary), Home as the spec 3.1 destination replacing the `SetupCard`
takeover, Agents with image readiness and the Claude copy button, System Check
with auto-loaded doctor detail, and the run detail view reordered behind
`SessionAttentionBanner.tsx` with runtime detail demoted to Show details.

The 2026-07-10 pass added setup recovery, Run filters, progressive disclosure,
plain vocabulary, system appearance, and small-window coverage. Follow-up fixed
premature reads, network failures, past-run polling, and cleanup; owned JSONL
appends now lock, and malformed records warn without content.

Wave B slices B1 (commit `cb9d5a9`), B2+B5 (commit `1b85d21`), and B3
(commit `d7a29b3`) are complete; per-slice detail lives in the gap tracker
sections and the `desktop-first-class-v1` evidence entries. In short: the
native macOS menu bar and keyboard map emit item ids over a listen-only
`menu-events` capability with no new mutation path; window frame, the
sanitized last destination, and the sidebar cookie persist and re-verify on
relaunch; the Dock badge reports the needs-you count through the narrow
`attention-badge` capability; and Settings (sidebar or Cmd-,) is a lean in-window
dialog holding the two real preferences (deviation recorded in the
tracker's B3 section). Menu, relaunch-restore, badge, and Settings behavior
share one still-pending `tauri dev` smoke.

The in-app terminal is complete and committed
(`docs/design/native-gui-terminal-design.md`; T1-T4). Host-PTY backend in
`runhaven-core::runtime::terminal` owns the `container` spawn for both attach
(`open_gui_attach_pty`) and Claude login (`open_gui_login_pty`, path A running
`claude auth login` through the real provider launch path so egress stays
enforced); `src-tauri` keeps the fail-closed `TerminalSessionRegistry` (cap 2),
the write/resize/close commands, a `TerminalMessage` Channel, and a main-only
`terminal` capability with `capability_guard` pins and a no-persist reader
tripwire (registry split into `terminal/registry.rs`). The xterm frontend shows
the standing no-filter/no-store disclosure and kills the child on unmount. Gates
A/B/C passed. Live-smoke fixes landed: failed-login container/network teardown
leak (force-remove before delete), `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1`
in the sandbox, and the sidebar System group pinned to a footer. The live
terminal + `claude auth login` round-trip is the lead's `tauri dev` smoke.

Interactive agent runs (task #23, Option A) reuse that machinery: core
`open_gui_run_pty` (sharing new `run_plan_through_pty` with login) runs a full RUN
plan under a PTY, and main-only `open_agent_run_terminal` reuses
`prepare_confirmed_launch` then forces `--interactive --tty` (`make_interactive_tty`)
so Claude runs its TUI; the New Run flow opens a run-mode `TerminalPanel` for
interactive agents (shared cap, disclosure, no-persist tripwire; background kept).
Placement fix (frontend-only): the interactive run terminal no longer renders
inside the New Run modal (it grew unbounded off-screen there). Confirming an
interactive launch closes the sheet and navigates to a new `run-terminal`
main-content destination (`NavDestination`, `RUN_TERMINAL_NAV`; ephemeral, never
persisted) where `AppMainContent` renders the run-mode `TerminalPanel` with a new
`fill` prop: the xterm container is `flex-1 min-h-0` in a bounded full-height
column (replacing the old `min-h-72` no-max), so FitAddon settles and cannot
drive a growth loop. `Close`/`End run` (and Escape/unmount) tear down via the
existing close-on-unmount + `closeTerminalSession` and return Home. Backend
contract, `open_agent_run_terminal`, disclosure, and native confirm unchanged.

Terminal viewport QA (2026-07-12) closed the same resize-loop risk for login
and attach modes: their non-fill xterm container is now fixed at `h-72` instead
of `min-h-72`, so FitAddon cannot grow its own ResizeObserver target. The
fail-first component regression was red before the one-class fix and green
after it. Browser and Playwright matrices cover every primary GUI destination
at 800x560 and 1080x720, including all four Agent states, both System Check
tabs, Cleanup, every run filter, Settings, all New Run steps, and run-terminal
navigation. The pass also corrected stale System Check copy from Maintenance to
Cleanup. The real Tauri entry surface rendered correctly at both sizes and its
live log plus secret scan stayed clean. macOS denied assistive click access, so
the credentialed native terminal interaction remains in the existing live
provider gate rather than being claimed from browser evidence.

Session-display fix: interactive TTY runs exit non-zero when the user closes
the agent UI, so `write_run_record` records the neutral status `ended` for any
`AgentRunPlan::is_interactive_tty()` plan (`--tty`) absent an explicit status.
Background runs keep the exit-code succeeded/failed split. `ended` threads as a
neutral terminal state through `ui_contracts` and the frontend and never uses
the red failed styling. `DESKTOP_APP.md` documents the Ended status.

B4 read-only session monitor windows are complete (detail in the gap tracker
B4 section): Run menu > Open in New Window mints the `session-<n>` label in
Rust, caps monitors at 6, focuses instead of duplicating, and grants monitors
exactly three reads over a `session-monitor` (`session-*`) capability pinned to
their own session; every mutation stays main-only (fail closed). Live
multi-window behavior is part of the pending `tauri dev` smoke.

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
them. The active workspace is CLI plus desktop only; full workspace test,
Clippy, and build gates must pass. Archived TUI/Codex source is not a release
dependency.

The 2026-07-10 hostile review closed the then-reproducible runtime and release
findings: Internet runs clean their exact container, process-global signal
handling preserves normal termination after child exit and fails safe on a
second signal, worktree validation finishes before mutation, and release
staging is temporary and fail closed. Expanded axe coverage passes across 11
desktop flows. The later Claude Keychain design was removed on 2026-07-12.

After the `v0.7.0` version bump, `./init.sh` passes with the repository-pinned
Rust and Cargo 1.97.0 rather than an unrelated Homebrew toolchain. The full gate
covers Rust tests, warnings-denied Clippy, pin, structure and release tests, 424
frontend tests, production build, 11 Playwright/axe flows, Tauri debug build,
workspace build, JSON validation, and diff hygiene. npm audit reports zero vulnerabilities.
Cargo audit reports zero vulnerabilities and 17 allowed upstream warnings: five
Tauri/macOS Unicode crates and twelve Linux-only GTK/GLib lockfile entries.

The 2026-07-11 refresh updated every `0.7.0` image, package, lock,
checksum, and agent pin. On 2026-07-12 the desktop agent set became Claude,
Codex, Antigravity, and Copilot only. Shell stays CLI-only; legacy Gemini names
remain cleanup-only.

## Next Step

The native-auth verification slice is complete: all four real GUI paths report
signed in, provider-owned readiness checks pass where required, no Mac Keychain
prompt or terminal-byte persistence occurred, and live cleanup left no run
resources. Keep TUI deferred and do not distribute until Developer ID
notarization and clean-Mac distribution gates pass.
