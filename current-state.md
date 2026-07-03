# Current State

Last updated: 2026-07-02 UTC

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
- `--ssh` forwards the host SSH agent socket only (never raw keys, non-root),
  and is re-enabled after a no-secret live smoke proved it works. On macOS 26+
  with Apple `container` 1.0.0, the default launchd SSH agent socket is
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

In-app maintenance (V1 milestone M5, safe subset) is shipped: the Maintenance
panel rebuilds an agent image (background build with sanitized, bounded,
secret-redacted live status) and cleans up RunHaven-managed stale networks
(count preview, confirm, count-only outcome). Both mutations are run-control
commands behind a Rust-owned native confirmation and fail closed. State prune
stays CLI-only on purpose: it can erase agent login/state across sessions and
its preview needs saved-state names the WebView never receives. Live image
build and network delete on a real Apple container host are a documented
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

- Claude OAuth capture (path C) now runs `claude setup-token` under a host PTY,
  not a pipe. Live smoke (F4) proved setup-token is a TTY-required Claude Code
  (ink) polling flow: piped stdout hangs it; under a PTY the user authorizes in
  the browser (no paste) and it polls and prints the one-year token. The capture
  (`provider/claude_oauth_capture.rs`, split from `claude_oauth_keychain.rs`)
  reads until the token appears, extracts it (ANSI/soft-wrap tolerant, rejects a
  half-streamed token, no glue), reaps the child, and stores it; buffers and
  token stay `Zeroizing`; 180s timeout + 256 KiB cap fail closed. Path C stays
  one-click. Remaining live smoke: click Sign in end to end, token lands in the
  Keychain, Touch ID on a later run.
- Provider-network teardown no longer leaks the run container or its network on
  a mid-flow killed login/attach client. `cleanup_provider_network`
  (`provider/runtime/network.rs`) force-removes the run container
  (`container rm --force`, verified on Apple container 1.0.0) before deleting the
  network; idempotent, so a clean `--rm` run still records `cleanup=deleted` and
  a real referring-container failure is surfaced, not swallowed. Single choke
  point for both failed-login and terminal-close paths; hermetic red/green tests
  in `provider/runtime/tests/network.rs`. `docs/DESKTOP_APP.md` caveat tightened.
  Lead's live re-smoke (fail a login, confirm no orphan) is the remaining step.
- Live-runtime verification ran on a real macOS 27 / Apple container 1.0.0
  host: all five live Tauri container tests passed against real containers
  (self-contained smokes on a clean state; the two attach-style tests against a
  deliberately started marker run), `runs repair --all` was exercised on two
  genuinely stale markers (removed=2 kept=0 unverified=0), and
  `scripts/apple_container_smoke.sh --with-provider --with-ssh` passed end to
  end (doctor pins, internal read-only run lifecycle, live SSH agent-socket
  forwarding as the non-root agent, provider allowlist egress, cleanup). The
  stale `--ssh` clap help ("currently disabled") was fixed in the same pass.
- Subprocess-spawning read commands (`get_managed_sessions`,
  `get_network_prune_preview`, `get_image_status`) now coalesce behind a
  keyed single-flight cache with a 2s TTL (`src-tauri/src/read_guard.rs`):
  at most one in-flight `container` subprocess per read family regardless of
  WebView call rate; errors are never cached and mutating commands are
  untouched. This closes the last whole-GUI security-audit finding with code.
- Wire contracts flattened (F8): LaunchRunResponse lost its duplicated nested
  snapshot (containerName hoisted), LogSnapshotResponse collapsed to one
  encoding (the cursor clone that defeated the live-output equality guard is
  gone, so the no-blink guard now holds), preflight_count deleted, and all
  eleven confirm flags take `#[serde(default)]` so an omitted flag fails
  closed with the friendly message instead of a raw serde error.
- The frontend punch list closed: explicit
  session-status reloads now coalesce onto an in-flight poll and take one
  follow-up fetch, so post-action success messages never show against stale
  status; first-load request defaults apply exactly once via an agent-name ref
  (a double-invoked mount refresh can no longer reset networkMode or the Codex
  default); every `same*` equality helper now carries a compile-time
  `satisfies Record<keyof T, true>` checklist so a new backend field fails
  `tsc` until it is compared (shared `sameRunStatusContainer` replaced the
  duplicated container comparison); UX: the global Diagnostics accordion hides
  while a session is open, the duplicate inner Maintenance title is gone, the
  "Run accepted" alert clears once the run appears as a managed session,
  Advanced options say "CPU cores" / "Memory (4g = 4 GB)", and the disabled
  "Load latest output" button points at the ack checkbox with visible helper
  text (ack gate unchanged); the network-cleanup success alert no longer
  self-clears (prune refreshes the count without wiping the result); e2e grew
  to 7/7 covering preview sign-in to `succeeded`, both confirm-gated
  maintenance flows, and worktree review (preview fixtures now finish a stopped
  worktree run with a saved copy and one stale preview network);
  `useAppController` split `sessionActions.ts` out by ownership for the
  structure gate; 191 unit tests pass.
- A whole-GUI analysis pass (security, frontend, shell architecture, empirical
  UX with screenshots) ran over the accumulated desktop surface, and its fixes
  landed in three verified waves. Verdict: the trust boundary held under a
  hostile whole-surface audit (every mutation behind a Rust-owned native dialog,
  backend-resolved targets, allowlisted no-leak mappers); findings were
  consistency and hygiene, not boundary breaks. Fixed: session cleanup now
  routes through the same gated worktree discard (post-dialog ownership
  reverify); one shared `confirm_native` dialog helper replaced nine copies;
  poisoned test-lock recovery; five no-blink polling bugs (stale-session flash,
  poll-failure blanking, error flap, missing output equality guard, sign-in
  interval churn) with 15 new hook tests (happy-dom devDependency, pinned);
  status registries merged onto shared bounded helpers (`registry_support.rs`,
  `tail_status.rs`) and run-control collapsed onto the worktree action pattern;
  image-build/sign-in runners now read bounded 8 KB chunks; the session-input
  grant is metered (256 KB or 30 minutes, then a fresh native confirmation;
  documented in SECURITY_MODEL.md); diagnostics strings are control-stripped
  before the WebView; dead code deleted (SafetyCard, no-op output registration,
  unused commands `get_setup_status`/`list_agents`, constant-false
  `ssh_available`); one canonical frontend label vocabulary + safeBackendLabel
  guard; closed status unions; focus management and aria-current; blue primary
  actions (teal/green reserved for ready states); the post-launch Start screen
  no longer dual-controls a run that lives in the sidebar; plain-language copy
  at decision points ("Session record", "API key handling", "Can reach: hosts
  only"); dead sidebar nav removed; Codex is the default agent (in-app
  sign-in); docs/DESKTOP_APP.md realigned with the real UI; e2e specs updated
  to the new flow (4/4 pass). The deferred comprehensive Apple-HIG visual
  redesign has its references in `docs/design/native-gui-design-resources.md`.
- Agent login unified in-sandbox (2026-07-01): all four sign-in agents (Claude,
  Codex, Copilot, Antigravity) log in *inside* the sandbox on the shared home
  volume via `runhaven login <agent>` (path A), which deleted the old host token
  store (`runtime/token_store.rs`, the run-time env injection, the
  `RUNHAVEN_TOKEN_STORE=keychain` seam). CURRENT TRUTH (updated 2026-07-02): the
  desktop app's Claude Keychain broker (path C, the default for Claude sign-in,
  see the broker note below) later reintroduced *opt-in* host custody of the
  Claude OAuth token in the macOS Keychain. So "RunHaven holds no host token" now
  applies to path A and to the non-Claude agents; a path-C Claude token is held
  only in the Keychain. `login claude --clear` and the GUI "Remove saved token"
  delete their respective stores.
- In-app agent sign-in landed seam-first (live verification is a documented
  follow-up). Codex, Copilot, and Antigravity now sign in from the app. A
  mutating `start_agent_sign_in` command lives behind its own isolated
  `agent-sign-in` capability (not run-control): it validates the agent against
  core `supports_sandbox_login`, fails closed on unconfirmed, unknown, or
  non-device-code input, requires a Rust-owned native confirmation, then runs
  the tested core device-code login plan on a background thread through the
  `AgentCommandRunner` seam. Claude is deliberately not an in-app target because
  its in-sandbox sign-in needs a code pasted back into the terminal, which this
  captured-output runner cannot host; the GUI keeps claude out of this device-code
  path via an explicit `agent == "claude"` check (core `supports_sandbox_login`
  is now true for claude). Claude sign-in instead runs in the in-app sign-in
  terminal (Terminal slice 3, separate `open_claude_sign_in_terminal` command),
  with `runhaven login claude` kept as a copy-command fallback. A read-only
  `get_agent_sign_in_status`
  (main-read) returns sanitized progress (starting/needs_browser/succeeded/
  failed) plus a bounded, `redact_run_output`-scrubbed output tail, so the
  sign-in link and short-lived device code show while any leaked token or key is
  scrubbed. The GUI only triggers sign-in and shows progress; the credential
  lives in the agent's isolated home volume and no command returns, logs, or
  stores it. On success the app refreshes auth status. Core reuse:
  `supports_sandbox_login` and `build_sandbox_login_plan` in
  `crates/runhaven-core/src/runtime/login.rs`; `AgentProfileSummary`
  `supportsInAppSignIn` drives the GUI button and excludes claude explicitly
  (core `supports_sandbox_login` now includes it).
- The GUI now has parity for git-worktree runs and their review lifecycle (V1
  milestone M4). The launch flow has an opt-in worktree choice (secure default
  off): `RunPlanRequest.worktree` threads into the planner, the plan review and
  native confirm dialog both show that an isolated branch will be created, and
  the branch is created only after confirmation (no git mutation before
  approval). A read-only `get_managed_session_worktree` (main-read capability)
  returns a sanitized review (branch, change counts, status, available actions)
  for eligible stopped worktree sessions. Four run-control commands
  (`keep`/`merge`/`recover`/`discard`_managed_session_worktree) resolve the
  backend-owned run in Rust, verify the worktree is RunHaven-owned and not
  active/stale/foreign (the same `verify_lifecycle` gate cleanup uses), require
  native confirmation, reuse the tested core functions, and return sanitized
  results. No worktree command exposes raw host paths, state volumes, or
  secrets (proven by no-leak tests). Shared launch-plan and capability-guard
  files were split by ownership to stay under structure ceilings.
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
takeover, Agents with image readiness and the Claude copy button, Diagnostics
with auto-loaded doctor detail, and the session view reordered behind
`SessionAttentionBanner.tsx` with runtime detail demoted to Show details.

Wave B slices B1 (commit `cb9d5a9`), B2+B5 (commit `1b85d21`), and B3
(commit `d7a29b3`) are complete; per-slice detail lives in the gap tracker
sections and the `desktop-first-class-v1` evidence entries. In short: the
native macOS menu bar and keyboard map emit item ids over a listen-only
`menu-events` capability with no new mutation path; window frame, the
sanitized last destination, and the sidebar cookie persist and re-verify on
relaunch; the Dock badge reports the needs-you count through the narrow
`attention-badge` capability; and Settings (Cmd-,) is a lean in-window
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

B4 read-only session monitor windows are complete (detail in the gap tracker
B4 section): Session menu > Open in New Window mints the `session-<n>` label in
Rust, caps monitors at 6, focuses instead of duplicating, and grants monitors
exactly three reads over a `session-monitor` (`session-*`) capability pinned to
their own session; every mutation stays main-only (fail closed). Live
multi-window behavior is part of the pending `tauri dev` smoke.

Active slice: bring CLI plus native-feeling desktop GUI to 100 percent ready
for real external user testing. Start by keeping TUI work out of scope, then
continue hardest-first through live session ownership, real-agent auth/session
behavior, multi-session isolation, typed status, run control, recovery, and
fail-closed boundaries before visual polish.

Claude OAuth Keychain broker (`docs/design/native-gui-claude-oauth-broker.md`
holds the full spec and per-slice status). Slices 1, 3, 4, and 5 landed; Gate 1
passed and authorized the GUI flip; Gate 2 (pre-merge full verification + live
Keychain / real setup-token maintainer smokes + a final claim-set read) and the
double-coverage review remain before merge. Slices 1/3/4 (custody + broker +
commands): macOS-gated `provider/claude_oauth_keychain.rs` custody (user-presence
`SecAccessControl`, `Zeroizing` capture, no disk/log copy; deps
`security-framework =3.5.1`, `zeroize =1.9.0`); `ClaudeAuthSource` keeps any
token off persisted plan/marker fields (`keychain-oauth` tag only, F1);
`CLAUDE_OAUTH_BROKER` reuses `BearerAuth` + `ANTHROPIC_BASE_URL`, pins
`api.anthropic.com` + `/v1/`; three `claude-oauth`-capability commands
(`start_claude_oauth_signin`, `get_claude_oauth_status`, `clear_claude_oauth`),
main-only, no token/prefix/path in any response. Slice 5 (GUI + rotation + docs):
`ClaudeSignIn.tsx`/`useClaudeOAuthSignIn.ts` make path C ("Sign in on your Mac")
the primary action and keep the unchanged T3 terminal login as the "Sign in
inside the sandbox instead" opt-in; the three approved tooltip strings ship
verbatim (asserted). `get_claude_oauth_status` now returns a non-secret `ageDays`
from the item's own `kSecAttrCreationDate` via the SAME attributes-only presence
query (no `kSecReturnData`, no prompt, no token read; parsed with `time`, split
into `provider/claude_oauth_age.rs`); past the Settings cadence (default 90 days)
the card prompts a fresh sign-in (a reminder, not an expiry). Docs shipped:
`AUTH_BROKER.md` (mechanism + verbatim claim set + removal + rotation),
`SECURITY_MODEL.md` (custody-reversal trade-off), `DESKTOP_APP.md`, `USAGE.md`.
No secret reaches the WebView; browser preview shows the placeholder and never
invokes Tauri. Selection wiring fix (F1 + F3, the double-coverage review's
load-bearing finding): `KeychainOAuth` was never constructed in a production
launch path, so a signed-in token was unused. Selection precedence now lives once
in `broker_secret.rs::select_claude_auth_source_with_presence` (env-var broker
NAME wins; else a PRESENT claude token via non-prompting `claude_oauth_token_present`
drives `KeychainOAuth`; else none) and both CLI (`make_run_plan`) and GUI
(`agent_run_options`) call it; the GUI decides from real Keychain presence, never
a WebView value. The prompting token read stays deferred to broker startup
(`require_api_key_broker_secret`) after the trusted confirm and fires exactly once;
plan prep and pre-confirm validation are presence-only. Hermetic tests (injected
presence/reader): `provider/runtime/tests/selection.rs`, CLI `app/tests.rs`, GUI
`commands/tests/claude_auth_source.rs`. Lead still owns the review re-verification
and the Gate 2 live Keychain smokes before merge.

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
