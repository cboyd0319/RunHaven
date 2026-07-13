# Current State

Last updated: 2026-07-13 UTC

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
or destructive states. The canonical in-app and public logo remains
`docs/assets/logo.png`. The user-selected, unmasked 1024 by 1024 bundle-icon
master is `docs/assets/app-icon.png`; Tauri's generated desktop assets are in
`src-tauri/icons/`.

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
- `docs/design/native-gui-design-resources.md` (Apple HIG and macOS design
  references)
- `docs/design/native-gui-uiux-audit.md` (current per-surface HIG contract)

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
  to return; End session and app-quit still tear down. P1 shipped (gated CONCERNS ->
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
- Superseded 2026-07-13: Home dashboard variant 06 carried setup, recent-run,
  and access panels. The current launchpad intentionally moves those details to
  New Session, History, and System details. Agent readiness still uses the same
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

The latest 2026-07-13 desktop pass supersedes the older navigation shape
recorded below. Current desktop information architecture is New session,
History, System, a dynamic Running group, and Saved setups. Every launch opens
New session. Setup and assistant repair appear after an assistant is chosen.
History owns Needs attention and the remaining chronological runs. Internal
sign-in and shell records are excluded from user history and menu state.

Agents, Cleanup, dedicated run filters, raw whole-app Trust Checks, Recent
Activity wording, Secure Boundary summary, auxiliary session windows, and
placeholder menu commands remain deleted. Assistant sign-in and image repair
are in New session. Setup repair and stale-network cleanup are in System.
Session-specific allow/deny reasons remain under Show details. Output has one
Show output action only while Rust reports process-memory output available.
Finished and failed sessions have no message composer or terminal.

Multiple interactive terminals remain mounted independently. Rust returns the
managed run ID plus a sanitized, path-free sidebar label. Starting one terminal
does not stop another. The sidebar shows every live terminal under Running, and
History offers Return to terminal only when that exact terminal is still live.
Natural nonzero exits are Failed; an explicit RunHaven close is Ended.

Saved setups persist assistant plus canonical project folder in a private,
owner-only, atomic app-data file. Sidebar summaries omit full paths. Selecting
a setup restores secure defaults instead of advanced launch choices. Missing
folders are marked unavailable and pinning is reversible.

Needs attention is now a recovery workflow, not a count. A failed session shows
the sanitized backend reason, distinguishes missing retained output from the
failure, opens same-page issue details and diagnostics, and offers a new session
with the same assistant. Review and output actions appear only when the backing
data exists. The numeric Dock badge was removed because Apple reserves app-icon
badges for unread notifications. Its removal left no valid app preference, so
the empty Settings surface and Cmd-, were removed too.

Protected-copy existence and review attention are now separate backend-owned
states. Rust uses a bounded, name-only Git probe to report changed, clean, or
unreadable without exposing paths or hashing file contents during session-list
polling. Verified changes and unreadable copies with a Check again recovery enter
History's Needs attention section, but only verified changes are called ready.
Clean copies stay under Other sessions and show neutral No changes to review
copy plus optional confirmed deletion. Poll equality includes this state, so a
later clean-to-changed transition is not suppressed. An explicit Check again
refreshes the summary before updating the review, while background state polls
never move keyboard or assistive-technology focus.

Closing the main window now hides the same WebView and preserves live terminal
channels. Dock reopen and menu navigation restore and focus that window. Only
actual app exit ends app-owned terminals and background jobs. The View menu now
supports 80 to 200 percent text, including terminal canvas text, with Increase
Text Size, Decrease Text Size, and Actual Text Size commands. The bounded choice
persists locally.

The selected app icon is separate from the public and in-app logo. Its unmasked
1024 by 1024 RGBA master is `docs/assets/app-icon.png`; the pinned Tauri CLI owns
`src-tauri/icons/`. The 0.7.0 bundle, NSWorkspace mask, and light/dark rendering
passed. Direct Dock, Finder, Spotlight, and About inspection remains release QA.

All native confirmations are parented to the trusted main WebviewWindow. A
cancelled confirmation is a quiet no-op before any mutation; real failures
remain errors. The custom menu surface is reduced to New Session, text-size,
sidebar/full-screen, and contextual Session actions, plus standard macOS items.

Fail-first regressions cover concurrent terminal identity and lifetime,
process-only output availability, ended worktree review, input/composer truth,
saved setup privacy and defaults, main-window lifecycle, text scaling,
attention recovery, confirmed generation-safe terminal end, and
assistant-status refresh. They also cover clean, changed, and unreadable
protected-copy attention, optional clean-copy cleanup, quiet cleanup cancel,
and clean-to-changed polling transitions.

`docs/design/native-gui-uiux-audit.md` now records the current official Apple
HIG topic checked for every shipped GUI surface. Source changes also add one
main landmark, a named sidebar navigation landmark, truthful progress and alert
semantics, ellipses for actions that request more input, and explicit
generative-AI and provider-data disclosures. Future GUI changes must update the
relevant audit row and receive macOS design source and live packaged review.

Development, local-signed, and release builds now have distinct identities.
Development is **RunHaven Debug** with `dev.runhaven.desktop.debug` and a
visible `Debug` version suffix; local signing remains **RunHaven Local** with
`dev.runhaven.desktop.local`; release remains **RunHaven** with
`dev.runhaven.desktop`. This prevents Debug, Local, and release identities from
colliding. Multiple old Debug copies can still share the Debug identity, so
native QA must close stale copies and target the exact newly built app bundle.
Desktop setup status also discards raw doctor
details before WebView serialization and shows only failed checks, so host
paths, commands, URLs, runtime dumps, and successful technical checks do not
reach System details.

A deep v0.7.0 build-hygiene pass removed the generated v0.5.0 release CLI and
app, every alternate GUI target root, all obsolete RunHaven and earlier mca
0.1.0 images, and the retired Gemini 0.7.0 image. Active Cargo metadata, UI
package and lock, Tauri config, pins, preview fixtures, and local product images
now resolve to 0.7.0. Sign-in and state volumes were not changed.

System now keeps passing Apple Container readiness checks collapsed under
**Advanced system tools**. It still shows only actionable failures by
default. The container-service internet repair uses one typed Rust action with
native confirmation, exact stop/start/final-status sequencing, phase-specific
recovery, and an explicit warning that current RunHaven sessions and external
Apple Container workloads stop. Network cleanup remains RunHaven-owned and
count-only. Machine management, registry login, arbitrary exec/copy/export, and
foreign-resource deletion remain CLI-only because they do not earn a safe
one-click product surface.

The final plain-language and security review removed the remaining technical
labels from the ordinary path. Network choices are **Assistant services only**
and **Full internet**, authentication is **Shared assistant sign-in**, and four
raw Apple Container file checks become one actionable **Apple container files**
row with one reinstall action. Destructive alerts keep their explanation in
normal card text with red reserved for the icon and action. Light and dark
semantic colors, pointer hover, keyboard focus, pressed states, and destructive
controls meet the focused accessibility gate. Worktree review IDs now cover the
exact base commit plus all committed, modified, deleted, and untracked files;
Rust revalidates that review before and after native confirmation. Session
messages use the same native-confirmed, bounded, revalidated path for every
send. The stale browser-only launch dialog and remaining session-monitor code
were deleted.

Development servers no longer assume a fixed local port. Playwright owns a
real Vite server on an operating-system-assigned `127.0.0.1` port for each
worker and uses its exact live URL. The canonical Tauri debug launcher starts
the same loopback server first and passes its live URL to Tauri, while manual
Vite development and preview also request an operating-system-assigned port.
Only address-in-use bind failures are retried. No existing local service is
reused, stopped, or treated as RunHaven. Browser Import and bookmarklets do not
exist in this repository. They do not justify a Settings surface.

Release preflight and staging tests pass under Xcode 27.0 beta build 27A5218g.
Distribution remains fail-closed without an exact `v0.7.0` tag, Developer ID
identity, provisioning profile, and notary profile. Human QA still covers
assistive technologies, visual accessibility settings, localization, multiple
displays, nontechnical coworker use, and clean-Mac distribution.

## Next Step

Run the remaining human accessibility checks with a nontechnical coworker.
Keep TUI deferred. Do not tag, sign, notarize, or distribute until explicitly
authorized and the Developer ID and clean-Mac distribution gates pass.
