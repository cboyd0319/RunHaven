# Native GUI UI/UX audit (2026-07-03)

> Historical audit, superseded 2026-07-13. Home, navigation, settings, menus,
> session review, and system recovery were simplified after this snapshot. See
> `docs/DESKTOP_APP.md` and `current-state.md` for the current contract.

A comprehensive read-only "fine-toothed comb" audit of the RunHaven desktop app,
run as five parallel expert lenses (visual system, accessibility, copy/tone,
macOS-native/HIG, states/flows) over every destination and the Rust user-facing
strings. Measured against the Apple HIG and the app's own stated visual system
(dark navy, glass panels, blue primary, teal/green ready, amber warning, red only
for blocked/destructive; 8th-grade copy; restrained native feel). Every finding
below cites a real `file:line`.

This is a triage list, not a set of applied changes. Fix order: P0, then P1, then
decide on the P2 native-shell pass, then a P3 cleanup sweep.

## Foundation (verified strong, preserve)

- No-blink polling: id-paired data, equality short-circuits, request-seq guards;
  2-4s polls never re-render or flash (`useManagedSessionStatus.ts`, `sessionList`).
- Native menu bar is HIG-correct: full submenu order, About/Settings(Cmd-,)/Quit,
  Cmd-N/W/R/1-9, selection-scoped disabling (`src-tauri/src/menu/`).
- Dialog/Sheet a11y: titles + descriptions, focus trap, Escape, focus return,
  `aria-describedby` on confirmations.
- Tokenized badges/alerts/buttons; red used only for failed/blocked.
- Copy contract clean: no emoji, no AI-tells, no em-dashes; disclosure, sign-in,
  and settings copy is honest and calm.
- Draft retention and confirm-then-act gating across every mutation.
- Real semantics (buttons not divs), `aria-current`, focus rings on all
  primitives, non-color-only status. Muted-text contrast computed ~5-5.7:1 (passes
  WCAG AA), reconciling the visual lens's "unverified" note.

## P0 - broken core flow (fix now)

- **RT-1 Navigating away from a live run kills the agent and strands the user.**
  `AppMainContent.tsx:39` renders the run terminal only under `run-terminal` nav;
  `TerminalPanel.tsx:254` kills the backend child on unmount; the sidebar has no
  run-terminal row, so there is no way back. Regression from the #24 main-content
  move. Fix: keep the run mounted until an explicit End run, or add a navigable
  "Active" handle, or confirm before navigating away while a run is live.
- **RT-2 The interactive run has no durable session handle.** The interactive
  `startRun` branch (`useLaunchFlow.ts:251`) never calls `onLaunched` /
  `loadManagedSessions` / `selectSession`, so the run has no last-launch card and
  no sidebar row. Fix: register the interactive run as a managed session on start
  (also resolves RT-1's "no way back").

## P1 - high value, bounded

- **A11Y-1 Terminals are a keyboard trap** (`TerminalPanel.tsx:204`, WCAG 2.1.2):
  no non-pointer escape from xterm. Fix: `attachCustomKeyEventHandler` that on
  Escape blurs and moves focus to End run.
- **FLOW-1 Sign-in has no cancel or timeout** (`useClaudeOAuthSignIn.ts:62`,
  `useMaintenance.ts:194`): stuck in "signing" forever if the browser step is
  abandoned. Fix: visible Cancel + elapsed-time message; do not block restart on
  `busy` once stalled.
- **COPY-1 Launch confirmation shows raw codes** (`launch/mod.rs:298`): the
  consent dialog reads `Auth: agent` / `Network: provider` instead of the friendly
  labels the WebView already has. Fix via COPY-2's shared source.
- **COPY-2 One vocabulary, one label source**: sign in / login / auth used
  interchangeably; "Login state" names three concepts; Stop says "run" vs
  "session" (`labels.ts:49`, `LaunchAccess.tsx:52`, `RunStatusCard.tsx:121`). Fix:
  one vocabulary decision + one shared label module reused by the WebView and the
  Rust dialog builders (collapses COPY-1 too).
- **VIS-1 Status icons bypass the tokens, inconsistently**
  (`HomeView.tsx:117`, `DiagnosticsSystemPanels.tsx:62,64`, `ClaudeSignIn.tsx:156`):
  `green-600`/`emerald-500`/`amber-500` hardcoded; success green drawn three ways.
  Fix: `text-success` / `text-warning` tokens everywhere.
- **VIS-2 "Ready to review" renders in warning amber** (`sessionAttention.ts:44`,
  `RunHavenSidebar.tsx:264`): a positive/ready state in amber, conflicting with the
  teal/green = ready rule (and a nav-spec comment). Fix: decide the rule, document,
  drive from a token.

## P2 - macOS-native shell (larger, phased)

This is the deferred native visual pass the design-resources doc anticipates. Best
done as one coordinated pass with a real bundle build; several items need
Tauri/Rust window/menu work, not just CSS.

- **NAT-1 (done)** Double chrome: default OS title bar + a separate web header.
  Shipped `titleBarStyle: Overlay` + `hiddenTitle`, full-height sidebar,
  drag-region header.
- **NAT-2 (done)** Native vibrancy. Tauri's built-in `Effect::Sidebar` is
  applied to the transparent main window in
  `lib.rs` setup; `body` is transparent so the frosted material shows, the
  sidebar paints a 72%-alpha tint over it (glass), and the main content
  (`SidebarInset`, `bg-background`) stays opaque. Requires `transparent: true`
  on the main window plus `macOSPrivateApi: true` and the `macos-private-api`
  Tauri feature. Tradeoff: apps using the private API cannot ship through the
  Mac App Store; RunHaven distributes directly (notarized), so this does not
  constrain its distribution model. Main window only; the read-only session
  monitors stay opaque. Vibrancy failure is non-fatal (logs a warning, keeps
  the per-region backgrounds).
- **NAT-3** Full-pill buttons, not macOS rounded-rects (`button.tsx:8`).
- **NAT-4** New Run is a center-zoom modal, not a top-attached sheet
  (`LaunchSheet.tsx:49`, `dialog.tsx:62`).
- **NAT-5** Icons are lucide, not SF Symbols (design-resources intent).
- **NAT-6** Header is a web bar, not a toolbar; window title never updates
  (`AppHeader.tsx:28`, `tauri.conf.json:15`).
- **NAT-7** Web-y overlay motion + no `prefers-reduced-motion` (`dialog.tsx:62`,
  `sheet.tsx:65`, `index.css`).
- **NAT-8** Cmd-C does nothing in the terminals (xterm canvas selection not wired
  to the clipboard) (`TerminalPanel.tsx:204`).

## P3 - polish and nits

Grouped, lower severity; cleanup sweep after P0-P2. Full list with `file:line` in
the published audit Artifact and the finder transcripts.

- Visual (5): subsection heading inconsistency; destructive-dot contrast/token;
  no backdrop-blur on main panels; arbitrary values duplicating tokens; dead
  light theme.
- Accessibility (5): header status pill not announced; whole-card `aria-live`;
  unlabeled first Settings select; `aria-expanded` without `aria-controls`; focus
  on an `outline-none` landing region.
- Copy (9): networkHelp effect-vs-flow; authScopeHelp points at a CLI command;
  sandbox vs container; "Skipped N still in use" possibly wrong; "Input is not
  attached" jargon; spaced hyphen; "Claude subscription (Keychain)" badge clip;
  `break-all` on label cells; mixed loading verbs.
- States (10): polling stops at zero sessions; failed-load "Checking…" forever;
  stale Agents image status; sign-in nav-away drops the device code; "Message
  sent" never clears / shown twice; Diagnostics reload blanks on a transient
  error; top error no dismiss; manual refresh disables New run; ended-run button
  still says "End run"; status-read failure mislabeled as sign-in failure.

## Method and limits

Five parallel read-only expert lenses, results deduped/reconciled/ranked by user
impact and fix cost. Static audit: a live desktop smoke on macOS 26+ would confirm
the rendered-only items (vibrancy, motion, real contrast). The two P0 items were
verified against the live tree.
