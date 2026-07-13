# Native GUI Navigation Gap Analysis

Last updated: 2026-07-01 UTC

> Historical implementation tracker, superseded 2026-07-13. It explains the
> earlier navigation build but is not the current product target. The current
> simplified surface is documented in `docs/DESKTOP_APP.md` and
> `current-state.md`.

Correction tracker mapping `docs/design/native-gui-navigation-spec.md` (the
target) against the app as it exists today. Read-only analysis; no behavior
changes here. Line references are to the files as of branch
`docs/native-gui-wireframe` (commit `2a525fa`) and will drift.

Status legend: conforms / partial / missing / contradicts. "Contradicts"
means the current code implements a shape the spec explicitly rejects.

## 1. Conformance Table

| Spec | Requirement | State | Evidence today | What must change |
| --- | --- | --- | --- | --- |
| 1 | Two-column split view, sidebar + one content pane | partial | `ui/src/app/App.tsx` (shadcn `SidebarProvider` + `SidebarInset`) | Shape exists; keep. |
| 1 | Wizard is not a destination | contradicts | `ui/src/app/AppMainContent.tsx:142-148`: default content is `LaunchView` (the wizard); sidebar "Start" row targets it (`RunHavenSidebar.tsx:50`) | Extract wizard into a sheet (Wave A3); default view becomes Home. |
| 2 | Taxonomy: Home / ACTIVE / FINISHED / SYSTEM | contradicts | `ui/src/app/RunHavenSidebar.tsx:71-150`: "Workspace > Start" plus "Sessions (N)" grouped **by agent** (`groupSessionsByAgent`, lines 168-183) | Regroup by lifecycle, add SYSTEM group, drop agent grouping (A1). |
| 2 | Row anatomy: primary label, one status line, trailing attention badge | partial | Rows carry title + identity + auth line + network/input line (4 lines, lines 119-136); badge is the raw `statusLabel` text (`SidebarMenuBadge`, lines 138-140) | Collapse to primary + one status line; replace text badge with symbol-in-dot attention badge (A1). |
| 2 | Newest-first ordering, no reorder on status flap | contradicts | Ordering is agent-label alphabetical (lines 177-182) | Newest-first within ACTIVE/FINISHED (A1). |
| 2 | Empty groups hide headers | partial | Sessions group hides at zero (line 91); no other dynamic groups exist yet | Apply to ACTIVE and FINISHED separately (A1). |
| 2 | Sidebar toggle, persisted width | partial | `SidebarTrigger` in `ui/src/app/AppHeader.tsx:29`; no width persistence; no View menu item | Width persistence (B2), menu item (B1). |
| 3 | Per-view toolbars; every toolbar item also a menu command | missing | One shared `AppHeader` (title + Ready badge + refresh icon); no menu bar at all | Per-view toolbar actions (A2/A6); menu parity (B1). |
| 3.1 | Home destination | missing | No Home. Setup-blocked state replaces **all** views with `SetupCard` (`AppMainContent.tsx:142-143`), instead of Home leading with the blocker while other views stay reachable | Build HomeView; fold SetupCard into it; disable New Run with reason (A2). |
| 3.2 | New Run as modal sheet, Cancel/Back/primary steps | contradicts | In-page 3-step wizard: `LaunchPlanCard.tsx` (agent/project) + `PlanReviewCard.tsx` (review), step state `launchStep` in `useAppController.ts:46` | Sheet extraction (A3). Step content itself largely matches the spec. |
| 3.2 | Worktree toggle top-level in step 2, off by default | partial | Toggle exists, off by default, but buried inside "Advanced options" (`LaunchPlanCard.tsx:253-272`) | Promote out of the disclosure (A3). |
| 3.2 | Recent projects in step 2 (mirrors Open Recent) | missing | Folder picker + free path input only (`LaunchPlanCard.tsx:154-175`) | Needs Rust-side recents store (B1, deferred sub-slice). |
| 3.2 | Rust-owned native dialog before any mutation; sheet is intent capture only | conforms | `src-tauri/src/commands/dialog.rs`; e.g. `run_control.rs:129-138` ("Nothing mutates before the user confirms"); launch under `src-tauri/src/commands/launch/` | Keep; the sheet must not weaken it. |
| 3.2 | On accept: sheet closes, row appears, selection moves to it | conforms | `useAppController.ts:248-254` (`loadSessionsUntilRunPresent` then `selectSession`) | Re-wire to sheet close (A3). |
| 3.3 | Session view order: banner, output, composer; controls in toolbar | contradicts | `AppMainContent.tsx:213-266`: status card, diagnostics card, composer mid-list, controls card **near the bottom**, output last; no attention banner | Reorder, add banner, lift controls (A6). |
| 3.3 | Output behind per-session acknowledgment, never auto-revealed | conforms | `RunOutputCard.tsx` gate; pinned by `ui/e2e/app.spec.ts:63-71` | Keep. |
| 3.3 | Drafts survive navigation/refresh | conforms | `useSessionInputState.ts`; pinned by `app.spec.ts:110-131` | Keep. |
| 3.3 | Details as popover/inspector, not permanent panes | contradicts | `ManagedSessionStatusCard` + `ManagedSessionDiagnosticsCard` are always-stacked cards | Move behind a Details affordance (A6). |
| 3.4 | Worktree review leads the finished-worktree view; four confirmed actions | conforms | `AppMainContent.tsx:221-226`; `WorktreeReviewCard.tsx`; pinned by `app.spec.ts:170-208` | Keep; add `Ready to review` badge (A1). |
| 3.5 | Agents destination | missing | Sign-in lives in wizard step 1 (`AgentSignIn` at `LaunchPlanCard.tsx:142`); image rebuild lives in `MaintenanceCard.tsx:43-115`; auth details in wizard (`AgentAuthDetailsCard`) | Build AgentsView; move rebuild there; share sign-in with the sheet (A4). |
| 3.6 | Diagnostics destination with doctor detail | partial | Accordion on the Start screen only (`AppMainContent.tsx:164-176`) with a manual "Load diagnostics" button; doctor detail lives only in `SetupCard` | Destination view, auto-load, absorb doctor detail (A5). |
| 3.7 | Inline device-code sign-in; Claude terminal instruction; no tokens shown | conforms | `AgentSignIn.tsx` (device flow lines 52-98; Claude terminal note lines 36-47); no Copy button yet (minor) | Add Copy button (A4, S). |
| 3.8 | Maintenance destination: network cleanup only, CLI-only note, empty state | partial | Accordion on every setup-ready view (`AppMainContent.tsx:178-185`); cleanup with count preview conforms (`MaintenanceCard.tsx:117-193`); rebuild wrongly lives here; no empty state, no CLI-only note in view | Destination view; rebuild moves to Agents (A4/A5). |
| 3.9 | Settings window (Cmd-,), one General pane | partial | RunHaven > Settings... (Cmd-,) opens a Settings **dialog in the main window** (`SettingsDialog.tsx`) with the two settings that exist: Dock badge on/off and default agent | Shipped lean (B3): the separate window is a deliberate deferral until settings outgrow one small pane. |
| 4 | Removal fallback: adjacent row, else other group, else Home | contradicts | `useSelectedSessionId.ts:9-17` resets selection to `null`, which renders the wizard | Adjacent-row logic on the new nav model (A1). |
| 4 | Relaunch restores selection after backend re-verification | missing | No persistence anywhere | B2. |
| 4 | No-blink: selection/scroll/drafts survive polls | conforms | Equality guards `sessionList.ts`, `appStatusEquality.ts`; poll loop `useManagedSessionsState.ts:26-49` | Keep; sheet must join the contract (A3). |
| 4 | Focus: banner-first, default buttons, focus return | partial | `useFocusOnChange.ts` lands focus on view/step transitions; no banner, no default-button discipline | A6. |
| 4 | Attention model: prioritized badges, Dock badge count | missing | Data exists (`nextAction.kind` incl. `auth_attention`, `lifecycle` in `ui/src/commands/types.ts:201-215`) but UI shows raw `statusLabel` text only | A1 (rows), B5 (Dock). |
| 5 | Full menu bar (8 menus) | missing | `src-tauri/src/lib.rs` builds no menu; Tauri default menu only | B1. |
| 6 | Keyboard map | missing | No accelerators or app shortcuts; only form-level Return/Escape | B1 (menu accelerators) + A3 (sheet Escape). |
| 7 | Frame/sidebar/selection persistence, re-verified | missing | `src-tauri/tauri.conf.json:13-22` static window; nothing persists | B2. |
| 7 | Window defaults 1080x720 min 720x560 | contradicts | Config is 1180x780 min 920x620 | Judgment call: pick one and amend the other doc (B2). |
| 7 | Per-session auxiliary windows | partial | `Open in New Window` (Session menu) opens a read-only monitor window (`session-<n>` labels, `session-monitor` capability); every mutating capability stays `"windows": ["main"]` | Shipped lean (B4): monitors are read-only; output viewing, row double-click, and the More-menu entry are recorded deferrals in the B4 section. |
| 7 | Full screen via standard controls, no custom frames | conforms | Standard titled window; no custom frame code | Keep. |
| 8 | Filter field ("Filter sessions") | missing | No search anywhere | A2 (Home toolbar) or its own small slice; Cmd-F in B1. |
| 9 | Notifications (future, opt-in) | conforms (N/A) | None implemented; spec marks them future | No action this pass. |
| 9 | Cross-view attention jumps land on the fix | missing | No badges, no Cmd-J, no banner | A1 + A6 + B1. |

Counts: conforms 8, partial 8, missing 12, contradicts 7 (35 rows).

## 2. Wave A Corrections (Frontend IA)

### A1. Sidebar taxonomy, attention badges, selection model - M

- Regroup `RunHavenSidebar.tsx` by lifecycle: ACTIVE = `starting | running |
  stop-requested | kill-requested`; FINISHED = `finished | failed`
  (`SessionLifecycle`, `ui/src/commands/types.ts:201-210`). Fixed Home row
  first, SYSTEM group (Agents, Diagnostics, Maintenance) last. Newest-first by
  `startedAt`. Empty dynamic groups hide.
- New `sessionAttention.ts` helper: priority Failed > Needs sign-in > Not
  responding > Ready to review, derived from `lifecycle`, `authStatus`,
  `nextAction.kind`, and worktree eligibility (`worktreeReview.ts`). One badge
  per row, symbol-in-dot, paired status text.
- Replace `selectedSessionId: string | null` (null = Start) with a nav union,
  e.g. `{ kind: "home" | "agents" | "diagnostics" | "maintenance" } |
  { kind: "session"; id: string }`. Touches `useSelectedSessionId.ts`,
  `useManagedSessionsState.ts`, `useAppController.ts` (`selectSession`,
  `pageTitle` inputs), `useAppDerivedState.ts:52-55`, `App.tsx:18-19`.
- Removal fallback: adjacent row in group, else other group's first row, else
  Home (extend `sessionSelection.ts`); kills the reset-to-wizard behavior in
  `useSelectedSessionId.ts:9-17`.
- Tests that break: all 3 cases in `ui/src/test/runhaven-sidebar.test.tsx`
  (`"Sessions (3)"`, `aria-label="Claude sessions"` ordering, `">2<"`/`">1<"`
  count badges); `ui/src/test/session-selection.test.ts` (fallback semantics).
  E2e: every `getByRole("button", { name: "Start" })` (`app.spec.ts:55, 85,
  121`) becomes Home; row-click selectors (`getByText("Codex -
  runhaven-preview")`, lines 56, 128) survive.
- Structure risk: low (`RunHavenSidebar.tsx` 183 lines; helper is a new file).

### A2. Home destination + destination routing - L

- `AppMainContent.tsx` switches on the nav union. New `HomeView`: setup health
  line (healthy) or the single blocker with fix + Check Again (folding today's
  full-screen `SetupCard` takeover at `AppMainContent.tsx:142-143` into Home);
  agent readiness strip linking to Agents; "Needs you" list; recent finished
  runs; prominent `New Run` (disabled with reason while blocked); first-run
  empty state.
- Structure gate: `AppMainContent.tsx` is 420 lines with a 500 ceiling and a
  450 split trigger (`scripts/check-repo-structure.sh`, `CUSTOM_SOURCE_LINES=500`,
  90% headroom). This slice **must** split it by ownership: per-destination
  view files (Home, Session, Agents, Diagnostics, Maintenance) plus a thin
  router.
- Tests that break: e2e test 1 (`heading level 1 "Launch"` -> Home,
  `app.spec.ts:14`); `getByText("Ready", { exact: true })` (line 18) may
  collide with Home content; `setup-card.test.tsx` partially (content reused).
- Docs: `docs/DESKTOP_APP.md` in the same slice.

### A3. New Run flow as a modal sheet - L

- Move `LaunchPlanCard`, `PlanReviewCard`, `LaunchSteps`, `LaunchAccess`
  pieces, and `WhyBoundaryCard` inside a modal dialog component (shadcn dialog
  primitive; no new dependency). Components move unchanged where possible;
  step chrome changes to Cancel (step 1) / Back (later) leading, primary
  trailing.
- State: extract from `useAppController.ts` into a new `useLaunchFlow.ts`:
  `request`, `launchStep`, `plan`, `planning`, `launching`,
  `launchConfirmation`, `confirmedWarnings`, `launchMessage`, plus new
  `sheetOpen`. `useAppController.ts:43-54, 152-268` shrink accordingly (also
  relieves its 359-line count). Close resets flow state; Escape/Cancel has no
  side effects (Radix default; verify Escape is inert while the Rust native
  dialog is up - the native dialog is app-modal so the WebView should not see
  the key, but test it).
- Focus and no-blink: the dialog's focus trap replaces the launch-view
  `useFocusOnChange` mount behavior (`AppMainContent.tsx:270-273`); keep
  step-change focus inside the sheet. On accepted launch, close the sheet
  first, then `selectSession` - `SessionView`'s `focusOnMount`
  (`AppMainContent.tsx:208-211`) already lands focus. The sheet must not
  remount on the 4s session poll: keep its state independent of
  `managedSessions` identity (it already is; keep it that way).
- Move the worktree toggle out of Advanced options to step-2 top level
  (`LaunchPlanCard.tsx:253-272`).
- `lastLaunch`/`RunView` fallback (`AppMainContent.tsx:150-159, 354-379`)
  stays outside the sheet on Home ("Run started" notice / failed-launch
  controls).
- Tests that break: e2e helper `startPreviewRun` (`app.spec.ts:210-226`) needs
  a leading `New Run` click and loses the `Advanced options` click for
  worktree (lines 218-221); tests 1, 2, 4, 5, 7 all traverse the wizard
  (`Continue`, `Review access`, `Launch run`, `Choose agent`/`Choose project`
  headings). Unit: `launch-plan-card.test.tsx` (step chrome, Back/Continue
  button assertions) and `plan-review-card.test.tsx` (Back button) change;
  card-internal assertions survive.

### A4. Agents destination - M

- New `AgentsView`: one row per agent from `dashboard.agents` composing the
  existing `AgentSignIn` (shared with sheet step 1, not moved),
  `AgentAuthDetailsCard`, image status (`useStatusLoaders`/`getImageStatus`),
  and the rebuild flow moved from `MaintenanceCard.tsx:43-115`
  (`ImageRebuildSection` + `useImageRebuild` from `useMaintenance.ts`).
  Toolbar: Check Again. Add the Copy button to the Claude terminal command.
- Tests that break: `maintenance-card.test.tsx` (rebuild cases move to a new
  agents-view test); e2e test 6 first half (`"Rebuild image"`,
  `"Confirm agent image rebuild"`, `"Succeeded (preview)"`,
  `app.spec.ts:151-157`) re-targets Agents; e2e test 5 (sign-in from the Agent
  step) keeps working once the sheet is open (A3 dependency for selectors
  only). `agent-sign-in-card.test.tsx` survives unchanged.

### A5. Diagnostics and Maintenance destinations; kill the accordions - S

- Delete both `<details>` accordions (`AppMainContent.tsx:164-185`).
  `DiagnosticsView`: reuse `DiagnosticsCard` sections, add full doctor/setup
  detail (reusing `SetupCard` check rendering), load on entry with Check Again
  instead of the manual "Load diagnostics" button, per-section empty states.
  `MaintenanceView`: `NetworkCleanupSection` + link-note that rebuild lives in
  Agents + plain CLI-only note for state reset + "Nothing to clean up right
  now." empty state.
- Tests that break: e2e test 3 (`getByText("Diagnostics").click()` accordion
  open, `"Load diagnostics"` button, `app.spec.ts:101-103`); e2e test 6
  accordion open (line 149); e2e test 2 assertion that `Diagnostics` has count
  0 in the session view (line 49) - the sidebar will now always contain a
  Diagnostics row, so the selector must scope to the content pane. Unit:
  `diagnostics-card.test.tsx` (load-button flow), `maintenance-card.test.tsx`
  (split).

### A6. Session view reorder, attention banner, details demotion - M

- Reorder `SessionView`: attention banner first (new component driven by
  `sessionAttention.ts` from A1; leads with the one fix and takes focus per
  spec 4), then output (`RunOutputCard`), then composer (`SessionInputCard`).
  Move stop/hard-stop/repair/cleanup out of the bottom `RunStatusCard`
  placement into the view's header action row (toolbar approximation until
  B1); keep the per-action confirmation pattern. Demote
  `ManagedSessionStatusCard`/`ManagedSessionDiagnosticsCard` behind a Details
  affordance (popover or disclosure).
- Tests that break: e2e test 2 relies on presence not order for most
  assertions, but `"Session controls"` heading (line 72) and
  `"Load session diagnostics"` (line 50) move/change; `run-status-card` and
  `managed-session-*` unit tests survive as component tests, placement
  assertions live only in e2e.

Wave A sizing: 2L (A2, A3) + 3M (A1, A4, A6) + 1S (A5).

## 3. Wave B Corrections (Rust/Tauri)

Every new command here also updates `EXPECTED_INVOKED_COMMANDS` in
`ui/src/test/command-surface-guard.test.ts` and a capability file; that test
is the tripwire.

### B1. Menu bar + keyboard accelerators - L

- Tauri v2 core `tauri::menu` (Menu/Submenu/MenuItem/PredefinedMenuItem with
  accelerators). No extra cargo feature expected for desktop menus on the
  pinned `tauri =2.11.3` (`Cargo.toml:111`, `features = []`); verify against
  the pinned docs at implementation time (volatile fact).
- New `src-tauri/src/menu.rs`; wire in `lib.rs` via `.menu()` +
  `.on_menu_event()`. Dispatch: menu events **emit** to the frontend
  (`app.emit("menu", id)`); a new `ui/src/app/useMenuCommands.ts` listens
  (`@tauri-apps/api/event`, needs the core event listen permission in a
  capability) and maps ids onto existing controller actions. Mutations stay on
  the existing invoke commands and their Rust-owned native dialogs
  (`stop_managed_session`, `kill_managed_session`, `repair_managed_session`,
  `cleanup_managed_session`, worktree actions, `rebuild_agent_image`,
  `start_agent_sign_in`, `launch_run`), so the menu adds no new mutation path.
- Existing-command mapping: Session menu -> run-control/worktree/cleanup/input
  commands above; Agent menu -> sign-in/rebuild/auth-details (all exist); View
  menu -> pure frontend navigation; File > New Run -> opens the sheet
  (frontend). New backend surface needed only for: enable-state sync (shipped:
  a narrow `sync_menu_state` command in its own `menu-state` capability drives
  Rust-side `set_enabled` on retained item handles as the selection and session
  list change; see the 2026-07-01 follow-up below and `src-tauri/src/menu/`),
  Open Recent (new
  `get_recent_projects`/`record_recent_project` commands with a Rust-side
  store; can ship as a follow-up sub-slice), and Settings/aux-window items
  (B3/B4).
- Keyboard: accelerators ride the menu items (Cmd-N, Cmd-., Opt-Cmd-., Cmd-0,
  Opt-Cmd-1/2/3, Cmd-J, Cmd-Shift-]/[, Shift-Cmd-O, Cmd-R, Cmd-F, Opt-Cmd-S).
  Cmd-1..9 as nine static menu items updated/disabled via the sync path.
  Cmd-Return stays a frontend key handler in the composer.
- Tests: no existing e2e covers menus (Playwright cannot drive native menus);
  add unit coverage for the frontend event-to-action map; `command-surface-guard`
  updates for any new invokes.

### B2. Window frame, sidebar width, selection persistence - M

- Frame: either `tauri-plugin-window-state` (new exact-pinned dependency) or a
  ~40-line custom store in the app state dir. Lean ladder decision at
  implementation; the plugin is the boring choice but is a new dependency for
  what a few lines do - flag for the lean gate.
- Selection: persist only sanitized enums and opaque ids (destination kind or
  session id). Never project paths, commands, or anything from
  `detailCommand`. Store Rust-side (new `get_ui_state`/`set_ui_state`
  commands, `main`-window capability only); on startup the frontend loads
  sessions first (`getManagedSessions`) and applies the persisted selection
  only if the object still exists, else Home - exactly the spec 4 rule. The
  WebView is untrusted: Rust validates the shape (enum + id pattern) before
  writing.
- Sidebar width is frontend-owned (shadcn `SidebarProvider`); persisting it in
  the WebView (cookie/localStorage) is acceptable since it holds no secrets -
  judgment call.
- Also reconcile the default/min window size mismatch
  (`tauri.conf.json:13-22` 1180x780/920x620 vs spec 1080x720/720x560) and
  amend whichever doc loses.
- Tests: new Rust unit tests for the sanitized store; `command-surface-guard`
  and `main-read`-style capability updates.

### B3. Settings - M (shipped lean as an in-window dialog)

- Honest baseline at implementation (2026-07-01): the app had exactly two
  real preferences to offer - the Dock badge on/off (B5's promised toggle)
  and the default agent for new runs (previously a fixed heuristic,
  `preferredDefaultAgent`). Notification toggles wait for notifications
  (spec 9, future); appearance is dark-only (stated as prose, not a dead
  control). Never security defaults or confirmation suppression.
- Lean deviation from spec 3.9, recorded here: two settings do not justify a
  separate WebView window and its whole surface (a runtime window-creation
  command, a new `"windows": ["settings"]` capability, `get_settings`/
  `set_settings` commands, and a Rust-side store). Shipped instead:
  RunHaven > Settings... (Cmd-,) opens a Settings dialog in the main window
  (`ui/src/app/SettingsDialog.tsx`, state in `useSettings.ts`), routed over
  the existing `menu://command` event; no new IPC surface, no capability
  change, no new mutation path. Choices persist through the sanitized
  `uiStatePersistence.ts` allowed set (a boolean plus an opaque agent name,
  re-verified against the live agent list before it picks anything).
- The separate settings window (minimize/zoom dimmed) returns only when
  settings outgrow one small pane (spec 3.9's own "minimize the number of
  settings" bar) or a setting needs Rust-owned storage or authority.

### B4. Per-session auxiliary windows - L, security review required (shipped as read-only monitors)

Shipped 2026-07-01 as strictly read-only monitor windows; this section records
the as-built boundary and every deviation from the sketch above it replaced.

- `Open in New Window` (Session menu, enabled; acts on the selected session)
  routes menu -> frontend -> `open_session_window`. The Rust command
  validates the id shape (registry-id rules) and existence against the
  backend read model, mints the window label itself (`session-<n>` from a
  counter, never user data), caps concurrent monitors at 6, and focuses an
  already-open monitor instead of duplicating it. The frontend never gets
  window-creation permissions.
- Capability decision, narrower than the original sketch on purpose: the
  early sketch listed "input write, run-control, worktree" for aux windows;
  that would have multiplied the mutation surface across N WebViews. Instead
  the `session-monitor` capability (windows `["session-*"]`; glob semantics
  verified in the pinned tauri 2.11.3 / tauri-utils 2.9.3 sources:
  `acl/resolved.rs` parses `windows` as `glob::Pattern`, `ipc/authority.rs`
  matches labels) grants exactly three reads: `get_session_window_target`,
  `get_managed_session_status`, `get_managed_session_diagnostics`. Rust
  additionally pins the two session-keyed reads to the window's own
  registered session (`session_windows::authorize_session_read`); unknown
  labels are denied. Every mutating capability stays `"windows": ["main"]`,
  now pinned by a `capability_guard` test over all capability files.
- Output viewing stays main-window-only (fail closed). The snapshot's
  Rust-owned confirmation dialog is app-level today, not parented to the
  asking window, and redacted output can still carry sensitive text.
  Verified return path if monitoring output becomes a real need:
  `tauri-plugin-dialog` 2.7.1 `MessageDialogBuilder::parent` flows into
  `rfd::set_parent`, so the dialog can be parented to the invoking
  `WebviewWindow`; that change plus adding the snapshot permission to
  `session-monitor` is the reviewed upgrade, not a default.
- Monitor identity flows Rust -> WebView, not through URLs: monitors load a
  constant `index.html?monitor=1` and ask `get_session_window_target`
  (answered from the calling window's label). The `?session=` parameter is a
  browser-preview/test affordance only, token-validated.
- Per-window state audit (the read_guard/session_input/session_output check
  the sketch demanded): `get_managed_session_status` gained a session-keyed
  `ReadGuard` (single-flight, 2s TTL) so main plus up to six monitors
  polling one session coalesce into one `container` probe; monitors never
  touch the input/output registries (no permission). WebView persistence
  (`uiStatePersistence.ts`) no-ops in monitor windows so a monitor can never
  move the main window's remembered place or settings.
- End-state behavior: on record removal the monitor shows the gone notice
  and keeps the last view; "closes on dismiss" is read as the person closing
  the window (Cmd-W / close button), never auto-close and never a
  WebView-held `core:window` close permission (which would be label-global).
  Main window remains truth (no second selection model). Monitors are not
  persisted or restored across relaunch (window-state plugin filters
  `session-*` labels), and closing the main window closes them too.
- Deferred, recorded: row double-click and the More-menu entry point (menu
  item only this slice), and output viewing per above.

### B5. Dock badge - S

- `set_badge_count` on the main window (present in Tauri 2.x since 2.2; verify
  on the pinned 2.11.3 docs - volatile fact). Count = sessions in a needs-you
  state, computed from the same attention rules as A1. Prefer computing in
  Rust from `managed sessions` truth rather than trusting a WebView-supplied
  number; simplest slice recomputes on each `get_managed_sessions` call.
  Settings toggle arrives with B3.

Wave B sizing: 2L (B1, B4) + 2M (B2, B3) + 1S (B5).

## 4. Sequencing Plan

Dependency-ordered slices; parallel tracks marked. Each slice ships its
`docs/DESKTOP_APP.md` update and its own test repairs.

1. **A1** sidebar taxonomy + nav union + attention helper + removal fallback.
   Foundation for everything. Breaks: `runhaven-sidebar.test.tsx` (all),
   `session-selection.test.ts`, e2e nav selectors (`"Start"` button, heading
   assertions).
2. **A2** Home + destination routing + `AppMainContent.tsx` split (the
   structure gate forces the split here: 420 lines vs a 450 split trigger).
   Breaks: e2e test 1, parts of tests 3/6 routing.
3. **A3** launch sheet (after A2 for the Home `New Run` entry; can be built in
   parallel on a branch, merged second). Breaks: e2e `startPreviewRun` and
   tests 1, 2, 4, 5, 7; `launch-plan-card`/`plan-review-card` step chrome.
   Relieves `useAppController.ts` line pressure.
4. **A4 + A5** Agents / Diagnostics / Maintenance destinations (parallel with
   A3; both depend on A2's routing). Breaks: e2e tests 3 and 6,
   `maintenance-card`/`diagnostics-card` unit tests, e2e line 49 scoping.
5. **A6** session view reorder + banner (after A1's attention helper; parallel
   with A4/A5). Breaks: e2e test 2 placement selectors.
6. **B1** menu bar + accelerators (after A2 so destinations exist to target;
   parallel with A6). **B2** persistence (after A1's nav union; parallel with
   B1). **B5** Dock badge (after A1; anytime, S).
7. **B3** Settings (after B1). Shipped lean as an in-window dialog; see B3
   for the recorded deviation.
8. **B4** aux session windows last, gated on security review; touches
   `command-surface-guard.test.ts`, new capability, `docs/SECURITY_MODEL.md`.
   Shipped as read-only monitors; see the B4 section for the as-built
   boundary and recorded deferrals.

Structure-gate watchlist: `ui/src/app/AppMainContent.tsx` (420) splits in A2;
`ui/src/commands/client.ts` landed at 441 after B4's two commands, still
under the 450 trigger - the next command addition must do the domain split
(sessions/launch/system) first; `ui/e2e/app.spec.ts` (226) grows across A
slices - split by flow before it nears 450.

## 5. Invariants Audit (Today's App vs Spec Section 10)

| # | Invariant | Today | Evidence |
| --- | --- | --- | --- |
| 1 | Mutations behind Rust-owned native dialog | holds | `src-tauri/src/commands/dialog.rs`; `run_control.rs:129-146`; launch and worktree commands follow the same pattern |
| 2 | Secure default is the shortest path | holds | Secure defaults preselected (`secureNetworkDefault`); warnings add checkboxes (`PlanReviewCard`) |
| 3 | Lower-security choices visible and warned | holds | Advanced options disclose and warn; nothing hidden |
| 4 | One prominent action per view | partial violation | Session view stacks Send, Stop run, Hard stop, Repair, Load output as peer primaries (`AppMainContent.tsx` SessionView) |
| 5 | No dead navigation | holds | Sidebar rows are Start + live sessions only; prior dead nav is gone (verified: every `SidebarMenuButton` has a live handler, `RunHavenSidebar.tsx:75-143`). The spec's persistent destinations are *absent*, not dead. Menu items no longer silently no-op: session-scoped and Cmd-1..9 items gray out until they apply, via `sync_menu_state` (`src-tauri/src/menu/state.rs`, `ui/src/app/useMenuStateSync.ts`); see the 2026-07-01 follow-up |
| 6 | Toolbar/menu/keyboard parity | violated | No menu bar, no shortcuts (B1) |
| 7 | Sidebar nav-only; nothing critical at bottom edges | partial violation | Sidebar has no action buttons (good) but rows carry 4 info lines; session controls + composer sit at the content bottom (A6) |
| 8 | Output only behind acknowledgment | holds | `RunOutputCard` gate; `app.spec.ts:63-71` |
| 9 | State updates in place | holds | Equality guards + draft-survival e2e (`app.spec.ts:110-131`) |
| 10 | App never self-navigates | violated | Record removal resets selection to the wizard (`useSelectedSessionId.ts:9-17`); a setup break swaps every view for `SetupCard` (`AppMainContent.tsx:142`) |
| 11 | New surfaces declare their class; no new accordions | violated | Diagnostics and Maintenance are exactly the banned accordions (`AppMainContent.tsx:164-185`) |
| 12 | Escape always cancels, never mutates | vacuously holds | No sheet or filter exists; nothing binds Escape to a mutation |
| 13 | Attention reachable in two interactions | violated | No attention badges, no Cmd-J; a needs-sign-in session requires reading sidebar text and knowing sign-in lives in wizard step 1 |
| 14 | Plain labels | holds | `labels.ts` scrubbing + plain copy across cards; pinned by sidebar leak tests |

Current violations to burn down: 6, 10, 11, 13 (full) and 4, 7 (partial).

## 6. Acceptance-Audit Follow-Ups (2026-07-01)

A post-Wave-A/B invariant-acceptance pass found small residual gaps against
spec sections 2, 4, 6, and 10. Resolved in this slice (frontend plus a menu
selector), all baselines green:

- M1 (undocumented Cmd-B): removed the vendored shadcn global Cmd/Ctrl-B
  sidebar keydown from `ui/src/components/ui/sidebar.tsx` (it fired inside the
  composer textarea and collided with Bold, and was never a documented
  toggle). Opt-Cmd-S and the sidebar trigger button remain the toggle.
- M2 (Cmd-Return to send, spec section 6): added a Cmd/Ctrl-Return handler on
  the session composer (`ui/src/app/SessionInputCard.tsx`) that presses the
  same confirm-gated Send path (no native-dialog bypass). Decision: the
  composer handler is the real shortcut; the **Send Message** menu item keeps
  focusing the composer with no accelerator, so Cmd-Return is not double-bound.
  Documented in `docs/DESKTOP_APP.md`.
- M3 (row anatomy, spec section 2): collapsed each sidebar session row in
  `ui/src/app/RunHavenSidebar.tsx` to the spec anatomy (primary label, one
  status line, trailing attention badge). Removed the identity, auth/scope, and
  network/input lines; that detail already lives in the session view and its
  Details disclosure. Invariant 7's "rows carry 4 info lines" gap is resolved.
- M4 (jargon labels, invariant 14): renamed the finished-session cleanup
  control from "Delete worktree" / "Confirm deleting this session's saved
  worktree." to "Clean up session" / "Confirm cleaning up this session's saved
  copy." (`ui/src/app/RunStatusCard.tsx`), and updated the menu focus selector
  (`ui/src/app/useMenuEvents.ts`) and its drift test together. Copy only; the
  confirm gate and worktree-ownership check are unchanged.
- NAV EDGE (invariant 10): `navigationAfterSessionRemoval`
  (`ui/src/app/navigation.ts`) now returns Home when the removed session was
  never in the previous list, instead of selecting a remaining session the user
  never chose.
- L1 (invariant 4, Maintenance view): made the safe "Check for stale networks"
  preview button the one prominent action and demoted the mutating "Remove
  stale networks" to an outline button (`ui/src/app/views/MaintenanceView.tsx`).
- L2 (view-aware Cmd-R refresh) and L3 (focus the attention banner's fix button
  on a jump) were reviewed and left as recorded deferrals: neither is needed
  for the invariants above, and both are cross-cutting enough to belong with
  the menu/focus work rather than this copy-and-shortcut slice.
- MENU ENABLE-STATE (invariant 5, "no dead affordances"): B1 shipped the menu
  bar with session-scoped and Cmd-1..9 items statically enabled and a no-op in
  the frontend when nothing applied, to avoid a chatty protocol. Those items
  now gray out instead. A narrow `sync_menu_state` command (its own
  `menu-state` capability, main-window only, booleans and one count in and
  nothing out) drives Rust-side `set_enabled` on menu item handles retained at
  build (`Menu::get`/`Submenu::get` are not recursive in the pinned tauri
  2.11.3, so nested items are kept by handle). The frontend sends state only
  when it actually changes (`ui/src/app/useMenuStateSync.ts`, deduped, no-op in
  browser preview), computed from the same helpers the sidebar and attention
  badge use. Session-scoped and Cmd-1..9 items build disabled so the
  no-selection launch state is correct; placeholders (Filter Sessions, Help)
  still gray out for their missing surfaces. The B4 focus rule (session-scoped
  ids drop while a monitor window is focused) is unchanged. Invariant 5 now
  CONFORMS for the menu bar; the deviation is retired.
