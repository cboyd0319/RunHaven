# Native GUI Navigation And Information Architecture Spec

Last updated: 2026-07-01 UTC

> Historical design record, superseded 2026-07-13 and updated again on
> 2026-07-13 after live multi-session testing. The current product uses a
> smaller information architecture: New session, History, System, a dynamic
> Running group, and Saved setups. Every app launch opens New session.
> Agents, Diagnostics, Maintenance, duplicate run filters, launch/settings
> modals, and placeholder menu commands did not earn permanent surfaces. See
> `docs/DESKTOP_APP.md` and `PROGRESS.md` for the current contract.
> The auxiliary session-window experiment described below was removed on
> 2026-07-13 because it duplicated History without adding an essential task.

The ideal-state navigation and IA contract for the RunHaven desktop app,
grounded in Apple HIG guidance fetched 2026-07-01 (see Sources). It is
deliberately unconstrained by today's implementation; a separate gap-analysis
pass maps current state to this spec. It authorizes no new backend commands
and changes no security boundary. Current visual rules (type, color, dialog
copy) live in `docs/design/runhaven-design-spec.md`; boundaries live in
`docs/SECURITY_MODEL.md`. Evidence tags: `[HIG]` cited Apple guidance, `[AK]`
AppKit/macOS convention not HIG-stated, `[RH]` RunHaven product judgment.
User-facing labels here are final proposed strings for nontechnical users.

## 1. Navigation Model

Decision: one main window, a two-column split view: a source-list sidebar on
the leading side plus one content pane (the Mail/Notes shape). `[HIG]`

Why this fits this product:

- Sessions are the app's content objects and several run at once; a sidebar
  "lets people navigate between areas of your app or top-level collections
  of content" `[HIG sidebars]` and keeps every session and its attention
  badge visible while any view is open.
- The hierarchy is exactly two levels (destination or session -> its view). A
  third column is for hierarchies "deeper than two levels" `[HIG sidebars]`,
  so no content-list middle pane and no persistent right rail.
- The system group (Agents, Diagnostics, Maintenance) gets permanent, honest
  destinations instead of accordions stacked under every view. `[RH]`

Rejected alternatives:

- Tab view: for "closely related areas of content", at most six tabs
  `[HIG tab-views]`. The session list is dynamic and unbounded.
- Three-column split view: hierarchy is not deep enough (above).
- Window-per-session as the primary model: sessions are not documents, and
  scattering lifecycle state across windows hides the attention model.
  Optional per-session auxiliary windows exist instead (section 7).
- Wizard-as-destination (today's Start view): creating a run is a task, not
  a place; the sheet (3.2) ends the wizard-vs-monitoring contest. `[RH]`

## 2. Sidebar Taxonomy

> Superseded for the Home redesign (variant 06, see
> `docs/design/native-gui-home-dashboard.md`): the sidebar no longer lists
> individual sessions. It carries the four destinations plus a `Runs` summary
> (Needs review / Active / Finished counts); sessions are reached from Home's
> Needs review and Recent activity tables. The rest of this section (navigation
> only, no action buttons, two levels) still holds.

The sidebar is navigation only: no action buttons, no status panels, at most
two levels (group headers plus rows), nothing critical at its bottom edge
`[HIG sidebars]`.

```text
Home                          fixed row, always first
ACTIVE                        dynamic group: starting/running/stopping runs
  <agent> - <project>         session row
FINISHED                      dynamic group: finished/failed, not yet cleaned
  <agent> - <project>         session row
SYSTEM                        fixed group
  Agents
  Diagnostics
  Maintenance
```

- Persistent destinations: Home, Agents, Diagnostics, Maintenance. Always
  present, always render a real view (possibly an empty state). Never dead.
- Transient objects: session rows. A row appears when a launch is accepted
  (lifecycle `starting`) and leaves only when its record is removed (repair,
  clean up, merge, or discard). Empty groups hide their headers.
- Row anatomy: agent icon; primary label `<agent> - <project folder name>`;
  secondary status line ("Running", "Waiting to stop", "Needs sign-in",
  "Not responding", "Ready to review", "Failed"); trailing attention badge.
- Badges are a small symbol-in-dot (amber needs-you, red failed), always
  paired with status text; never color alone. Rows carry states, not counts,
  so numeric badges are app-icon only (section 4). `[RH]`
- Ordering: newest-first in both groups. Rows never reorder on a status
  flap; a row moves only between groups on a lifecycle change. Selection and
  scroll survive refresh (repo no-blink law). `[RH]`
- Sidebar visible by default ("avoid hiding the sidebar by default"), with a
  leading toolbar toggle plus View > Hide Sidebar, width persisted,
  auto-collapse allowed at compact widths, and icons that follow the system
  accent color. `[HIG sidebars]`

## 3. Destinations

Each view names purpose, contents, toolbar, empty state, and how you reach
and leave it. Toolbars everywhere: leading sidebar toggle, view title,
important actions trailing; every item also a menu bar command
`[HIG toolbars]`; at most one prominent action per view (foundation doc).

### 3.1 Home

- Purpose: answer "is this Mac ready, and what needs me right now?"
- Shows: setup health (one line when healthy); agent readiness strip (per
  agent: signed in / needs sign-in, linking to Agents); a "Needs you" list of
  sessions with attention states; recent finished runs. Prominent action:
  `New Run`.
- Setup blocked: Home leads with the single blocker, one fix button, and
  `Check Again` (Cmd-R). Green checks and full doctor detail stay in
  Diagnostics. While blocked, `New Run` is disabled everywhere with a short
  reason; disabled, not hidden `[HIG the-menu-bar]`.
- Toolbar: title "Home"; trailing `New Run` (the one prominent toolbar
  action, trailing per `[HIG toolbars]`) and the search field (section 8).
- Empty state (no sessions ever): small logo, two sentences on what RunHaven
  does and never shares, `New Run`, and a "How RunHaven protects your files"
  Help link. Teach in the app, not a tutorial `[HIG onboarding]`.
- Reach: launch default; Cmd-0; sidebar. Leave: any sidebar selection or the
  New Run sheet.

### 3.2 New Run Flow (sheet)

- Presentation: a modal sheet on the main window (precedent: Xcode's New
  Project sheet `[AK]`). A sheet fits "a simple task that they can complete
  before returning to the parent view" and is always modal on macOS
  `[HIG sheets]`; the dimmed session list stays visible behind it, so
  creation cannot compete with monitoring. One sheet at a time `[HIG]`.
- Steps follow the HIG multi-step sheet pattern (step 1: Cancel leading;
  later steps: Back leading; primary trailing) `[HIG sheets]`:
  1. Agent: single picker; per-agent readiness; inline sign-in when needed
     (section 3.7); `Sign-in details` disclosure.
  2. Project: folder picker plus recent projects (mirrors File > Open
     Recent); `Run in an isolated copy (git worktree)` toggle, off by
     default; `Advanced options` disclosure for network mode, read-only
     project, API key variable name, CPU/memory, custom image, extra hosts,
     env names. Every lower-security choice warns inline when selected.
  3. Review: plain-language access summary (what is shared, what is not, the
     network setting, the sign-in used); warnings render as amber checkboxes
     that must each be checked; exact container command behind a disclosure.
     Primary button: `Start Run`.
- The security boundary: `Start Run` triggers the Rust-owned native dialog
  above the sheet, which restates any warned choices. The sheet is intent
  capture only; nothing starts and no git mutates before confirmation.
- Escape/Cancel closes the sheet with no side effects. On accept, the sheet
  closes, the new session row appears in Active, and selection moves to it
  (user-initiated navigation, so auto-select is allowed `[RH]`).
- Empty state: none. If setup breaks while the sheet is open, step 1 shows
  the blocker and the primary is disabled.
- Reach: `New Run` on Home, Cmd-N, File > New Run..., Dock menu. Leave:
  Cancel/Escape or accepted launch.

### 3.3 Session View

- Purpose: watch and steer one run.
- Shows, top to bottom: status header (agent, project, lifecycle, network,
  input state); an attention banner when the session needs you (needs
  sign-in, not responding, failed), always first in reading order and always
  leading with the one fix; the output region; the message composer. Run
  controls live in the toolbar, never at the window bottom `[HIG windows]`.
- Output region: hidden behind the per-session acknowledgment; until then it
  shows a short privacy note and `Show latest output`. Navigation never
  auto-reveals output; output stays bounded and redacted (repo law).
- Composer: enabled only while input is attached; `Send` (Cmd-Return `[AK]`)
  goes through native confirmation under the metered grant. Drafts survive
  navigation and refresh (repo law).
- Details (mounts summary, network policy, per-session auth broker and
  egress records) open from a toolbar `Details` item as a popover or
  temporary inspector, never a permanent pane. `[RH]`
- Toolbar: title `<agent> - <project>`; trailing `Stop` (Cmd-.), a More menu
  (`Hard Stop`, `Repair Session Record`, `Open in New Window`), `Details`.
  Every mutation sits behind the Rust-owned native dialog.
- Terminal states: a finished plain run shows a short summary plus
  `Clean Up Session...`; a failed run leads with the fix, details second.
- Empty state: none; a session view always shows its object. If the record
  disappears while selected, selection restores per section 4.
- Reach: sidebar row; Cmd-1..9; Cmd-Shift-]/[ cycling; Cmd-J attention jump;
  a notification click (section 9). Leave: sidebar; record removal.

### 3.4 Worktree Review (mode of Session View)

- A finished protected-copy run leads with **Review changes**. It shows
  project-relative filenames with plain Added, Changed, or Removed labels.
  Assistant changes and existing project changes use separate semantic lists.
  Each list shows at most 100 names and states when it is shortened.
- The routine actions are **Apply changes** and **Discard changes**, each behind
  native confirmation. Apply is unavailable while the source project has local
  changes. Leaving the view keeps the protected copy. Recovery appears only
  after an action fails and does not mutate files.
- After Apply or Discard, the record is removed and the row leaves.
- Reach: the badge, Cmd-J, or normal selection. Leave: completing an action
  or navigating away (review state persists until resolved).

### 3.5 Agents

- Purpose: sign in and keep each agent ready; the first-class home for the
  sign-in and image jobs, replacing the accordion afterthoughts. `[RH]`
- Shows: one row per agent (Claude, Codex, Copilot, Antigravity, Shell) with
  sign-in state and `Sign In` action (section 3.7), image state and
  `Rebuild Image...` (native confirm, then background progress with a
  bounded, secret-safe tail), and a `Sign-in details` disclosure (what stays
  on this Mac, what the agent gets, provider hosts; read-only).
- Toolbar: title "Agents"; trailing `Check Again` (Cmd-R). No default
  prominent action; a row's `Sign In` becomes prominent only when that agent
  is the one thing blocking a run. `[RH]`
- Empty state: none (the agent list is static).
- Reach: sidebar; Opt-Cmd-1; links from Home readiness strip and wizard
  step 1. Leave: sidebar.

### 3.6 Diagnostics

- Purpose: read-only support and trust evidence. It never mutates.
- Shows, as sections: setup checks (full doctor detail), trust checks, auth
  broker decisions, provider egress decisions. Tables with bounded, scrubbed
  entries; no secrets, no raw paths.
- Toolbar: title "Diagnostics"; trailing `Check Again` (Cmd-R).
- Empty states per section: "No decisions recorded yet."
- Reach: sidebar; Opt-Cmd-2; "See details" links from Home setup health.
  Leave: sidebar.

### 3.7 Sign-In (flow, not a destination)

- Codex, Copilot, Antigravity: `Sign In` (in Agents or wizard step 1) asks
  for native confirmation, then shows inline progress: the link and short
  code, then success. No second sheet ever opens from the wizard sheet
  ("display only one sheet at a time" `[HIG sheets]`).
- Claude: the row explains sign-in happens in Terminal and shows the exact
  command (`runhaven login claude`) with a Copy button, plus `Check Again`.
- Tokens never appear anywhere in the app (repo law).

### 3.8 Maintenance

- Purpose: host-level upkeep that mutates. Kept separate from read-only
  Diagnostics so the trust story stays legible. `[RH]`
- Shows: `Clean Up Old Networks...` with a count preview, native confirm,
  and count-only outcome. Image rebuild lives in Agents (a note links
  there). Saved-state reset stays CLI-only; this view says so plainly with
  the command.
- Toolbar: title "Maintenance"; trailing `Check Again`.
- Empty state: "Nothing to clean up right now."
- Reach: sidebar; Opt-Cmd-3. Leave: sidebar.

### 3.9 Settings (separate window)

- Warranted, but minimal ("minimize the number of settings you offer"
  `[HIG settings]`). Opens from RunHaven > Settings... (Cmd-,) as its own
  small window with minimize/zoom dimmed `[HIG settings]`.
- One General pane to start: default agent, notification toggles
  (section 9), Dock badge on/off. Grow to a non-customizable toolbar of
  panes only when a second pane truly exists `[HIG settings]`.
- Never in Settings: security defaults, confirmation suppression for any
  boundary action, or anything that makes a lower-security choice sticky and
  silent (repo law).

## 4. Selection, Focus, And Attention

Selection:

- Exactly one sidebar selection at all times, persistently highlighted
  ("persistently highlight the current selection" `[HIG split-views]`).
- On session removal: select the adjacent row in the same group (next below,
  else above), else the first row of the other session group, else Home.
  `[AK]` (list-deletion convention; the HIG has no explicit rule.)
- Selection, scroll position, and composer drafts survive background
  refresh; no view remounts on poll (repo law).
- On relaunch, restore the last selection only if the object still exists
  after backend reconciliation; otherwise Home. `[RH]`

Focus (target behavior for the existing focus management):

- Full Keyboard Access works across sidebar, toolbar, and content
  `[HIG keyboards]`. Tab cycles controls; arrows move within the sidebar
  list; Return activates the single default button; Escape cancels sheets.
- Closing a sheet or native dialog returns focus to the control that opened
  it. `[AK]`
- When navigation lands on a session with an attention banner, the banner is
  first in reading and focus order and its fix button is the default. `[RH]`
- Focus never remains on a removed view; it follows the restored selection.

Attention model:

- Needs-you states, priority order: Failed (red), Needs sign-in, Not
  responding, Ready to review (amber). One badge per row, highest state
  wins; the status text names it. Never color-only.
- App icon (Dock) badge: the count of sessions needing you, nothing else,
  clearing as each is attended ("use a badge only to show people how many
  unread notifications they have" `[HIG notifications]`; one needs-you state
  as one unread item `[RH]`). Badging is never the only signal `[HIG]`.
- The app never navigates by itself. A finishing or failing run changes
  badges and rows; it never steals the current view. `[RH]`

## 5. Menu Bar

Standard menus in standard order, always showing the same items, disabling
rather than hiding, ellipsis when more input follows `[HIG the-menu-bar]`.

- RunHaven: About RunHaven; Settings... (Cmd-,); Services; Hide RunHaven
  (Cmd-H); Hide Others (Opt-Cmd-H); Show All; Quit RunHaven (Cmd-Q). Quit
  does not stop runs; sandboxed runs continue and reappear on relaunch, so
  quit shows no dialog (no warning for expected outcomes). `[RH]`
- File: New Run... (Cmd-N); Open Recent > recent project folders with Clear
  Menu (feeds wizard step 2) `[HIG the-menu-bar]`; Close (Cmd-W).
- Edit: Undo/Redo (text editing only); Cut/Copy/Paste; Select All; Find >
  Filter Sessions (Cmd-F, focuses the toolbar search field).
- View: Show/Hide Sidebar (Opt-Cmd-S `[AK]`, title reflects state `[HIG]`);
  Go to Home (Cmd-0); Go to Agents (Opt-Cmd-1); Go to Diagnostics
  (Opt-Cmd-2); Go to Maintenance (Opt-Cmd-3); Next/Previous Session
  (Cmd-Shift-] / [); Next Needs-You Session (Cmd-J); Enter Full Screen
  (Ctrl-Cmd-F). In-window navigation here follows Xcode's navigators `[AK]`.
- Session (app-specific, between View and Window `[HIG the-menu-bar]`; acts
  on the selected session, disabled otherwise): Send Message...
  (Cmd-Return); Show Latest Output (Shift-Cmd-O; runs the acknowledgment);
  Stop (Cmd-.); Hard Stop (Opt-Cmd-.); Repair Session Record...; Merge
  Changes...; Keep Branch; Recover...; Discard Changes...; Clean Up
  Session...; Open in New Window.
- Agent (app-specific; acts on the Agents-view selection, else the selected
  session's agent): Sign In...; Rebuild Image...; Show Sign-In Details.
  Session before Agent orders app menus most-used first `[HIG the-menu-bar]`.
- Window: Minimize (Cmd-M); Zoom; Bring All to Front; open window list. A
  Window menu exists even with one window `[HIG the-menu-bar]`.
- Help: RunHaven Help (Cmd-?); What Can an Agent Reach? (plain-language
  security explainer); Report a Problem....

Every toolbar item above appears in these menus ("make every toolbar item
available as a command in the menu bar" `[HIG toolbars]`).

## 6. Keyboard Map

Standard shortcuts are never repurposed; Command is the main modifier, Option
marks less-common commands, order Control-Option-Shift-Command `[HIG]`.

| Shortcut        | Action                                    | Basis |
| --------------- | ----------------------------------------- | ----- |
| Cmd-N           | New Run...                                | [HIG] |
| Cmd-0           | Go to Home                                | [RH]  |
| Cmd-1..Cmd-9    | Select session 1-9 (sidebar order)        | [AK]  |
| Cmd-Shift-] / [ | Next / previous session                   | [AK]  |
| Cmd-J           | Next session that needs you               | [RH]  |
| Cmd-.           | Stop selected session (cancel key)        | [AK]  |
| Opt-Cmd-.       | Hard stop selected session                | [RH]  |
| Cmd-Return      | Send message (confirmed)                  | [AK]  |
| Shift-Cmd-O     | Show latest output (acknowledged)         | [RH]  |
| Cmd-F           | Filter sessions                           | [HIG] |
| Cmd-R           | Check again / refresh current view        | [AK]  |
| Opt-Cmd-1/2/3   | Go to Agents / Diagnostics / Maintenance  | [RH]  |
| Opt-Cmd-S       | Show/Hide Sidebar                         | [AK]  |
| Ctrl-Cmd-F      | Enter/Exit Full Screen                    | [HIG] |
| Cmd-, W M Q ?   | Settings, Close, Minimize, Quit, Help     | [HIG] |

Return activates the one default button per view; Escape cancels the sheet
or clears the filter; destructive actions are never the Return default
(foundation doc). Every shortcut appears in its menu item `[HIG]`.

## 7. Windows

- One main window (split view); the sidebar is the canonical session list.
  Default 1080x720; minimum 720x560; frame, sidebar width, and selection
  persist across relaunch, re-verified against the backend before rendering
  (sessions are external truth). `[RH]`
- Per-session auxiliary windows: `Open in New Window` (Session menu, More
  menu, or double-click a row `[AK]`) opens that session in a standard
  titled window without a sidebar, for watching two runs side by side.
  Window-menu listed; Cmd-W closes; on record removal it shows the end state
  once, then closes on dismiss. The main window stays truth. `[RH]`
- No window tabs: tabs suit many peer documents; the sidebar already handles
  many sessions, and duplicate session lists would split attention. `[RH]`
- Settings is its own small window (section 3.9). Native confirmation
  dialogs attach to the window whose content asked for them. `[AK]`
- Full screen: supported through the standard controls only; people choose
  when to enter and exit, the layout keeps its shape, and nothing resizes
  programmatically `[HIG going-full-screen]`. Split View tiling with a
  session window is the two-session answer, not custom layouts.
- System-standard window appearance and controls throughout; no custom
  frames `[HIG windows]`.

## 8. Search

- One search location `[HIG searching]`: a filter field at the toolbar's
  trailing edge `[HIG toolbars]`, placeholder "Filter sessions", scope shown
  by the placeholder ("clearly display the current scope" `[HIG searching]`).
- It filters sidebar session rows live by agent, project name, and status
  word; matching is local and instant. No-match state: "No sessions match."
  with a Clear Filter button. Escape clears.
- No stored search history (entries can name private project folders; "take
  privacy into consideration" `[HIG searching]`) and no Spotlight indexing:
  session content is sensitive by default. `[RH]`
- If a run-history archive ships later, it joins the same field with a
  visible scope control, not a second search location `[HIG searching]`.

## 9. Deep Flows And Notifications

Cross-view jumps (each lands with the fix in focus, section 4):

- Attention badge -> click row (or Cmd-J) -> session view, banner first.
- Worktree run finishes -> `Ready to review` badge -> session view leads
  with the review panel (3.4).
- Setup breaks -> Home shows the blocker; `New Run` disables everywhere with
  the reason; running sessions keep their views untouched.
- Sign-in needed mid-flow -> inline sign-in where you are (wizard step 1 or
  session banner routing to Agents); never a detour that loses the draft.
- Maintenance need (stale image) -> Agents row shows it next to its
  `Rebuild Image...` action; the Home readiness strip mirrors it.

Notifications (future, opt-in, consent-first `[HIG notifications]`):

- Exactly four events may notify: run finished, run failed, needs sign-in,
  not responding. Each fires once per state change; no re-nagging ("avoid
  sending multiple notifications for the same thing" `[HIG notifications]`).
- A click selects that session in the main window ("people expect your app
  to display related content"); no buttons that merely open the app `[HIG]`.
- Content is state-only: agent, project name, state. Never output text,
  paths, codes, or tokens ("avoid including sensitive, personal, or
  confidential information" `[HIG notifications]`).
- While the app is frontmost, no banner: badges and rows already show it
  `[HIG notifications]`.

## 10. Navigation Invariants Contract

Any future feature must obey all of these. A violation is a design bug.

1. Every mutation goes through a Rust-owned native confirmation dialog; no
   sheet, menu, shortcut, or notification may bypass or replace it in-page.
2. The secure default path is the shortest path. Counting interactions from
   Home, no lower-security variant of a task may take fewer steps than the
   secure default; warned choices add explicit steps, never remove them.
3. Supported lower-security choices stay visible and warned (disabled or
   disclosed, never hidden); unsupported choices fail closed.
4. One prominent action per view, and it is the safe one.
5. No dead navigation: persistent destinations always render; commands are
   disabled with a reason, not removed; badges always lead somewhere real.
6. Every toolbar action has a menu bar command; every menu command is
   keyboard-reachable; Full Keyboard Access reaches everything.
7. The sidebar stays navigation-only: two levels max, no action buttons,
   nothing critical at its bottom edge or the window's bottom edge.
8. Sensitive output appears only behind the per-session acknowledgment, and
   no navigation path auto-reveals it.
9. State updates in place: selection, scroll, drafts, and acknowledgments
   survive refresh; background changes never remount or steal the view.
10. The app never self-navigates. Only a user action changes the selected
    view; lifecycle changes may only re-badge and re-group.
11. New surfaces declare their class before build: sidebar destination
    (rare), session-view mode, sheet (self-contained task), native dialog
    (consequence gate), or popover/inspector (transient detail). No new
    accordions on shared views.
12. Escape always cancels or clears (sheet, dialog, filter), never mutates.
13. Attention is reachable in at most two interactions from anywhere (badge
    or Cmd-J, then the fix).
14. Labels stay plain: verb-first actions, no jargon on the default path,
    roughly 8th-grade reading level (repo law).

## Sources

Apple HIG pages, fetched 2026-07-01. The HTML pages are client-rendered;
fetch the DocC JSON endpoints instead
(`https://developer.apple.com/tutorials/data/design/human-interface-guidelines/<page>.json`).
Pages used, all under
`https://developer.apple.com/design/human-interface-guidelines/`:
`navigation-and-search` (now a collection index; its guidance lives in the
component pages), `sidebars`, `toolbars`, `windows`, `the-menu-bar`,
`split-views`, `sheets`, `tab-views`, `keyboards`, `searching`, `settings`,
`notifications`, `going-full-screen`, `onboarding`.

Repo grounding: `docs/design/runhaven-design-spec.md` (visual and dialog
rules), `docs/plans/native-gui-wireframe.md` (prior wireframe),
`docs/DESKTOP_APP.md` (current user-facing behavior), `docs/SECURITY_MODEL.md`
(boundary), `PROGRESS.md` (product laws cited as "repo law").
