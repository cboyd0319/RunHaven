# Native GUI Help, Tips, and Privacy Communication

Last updated: 2026-07-02 UTC

Design guidance for how the RunHaven desktop app explains itself: privacy copy,
in-app help, and first-encounter tips. Scope is what to say and when to show it,
implemented in React and CSS inside the Tauri WKWebView. This doc is design
only; it proposes no component code.

It extends `docs/design/native-gui-hig-foundation.md` (the visual and interaction
bar) and applies the privacy discipline already set in
`docs/design/native-gui-claude-oauth-broker.md` and
`docs/design/native-gui-terminal-design.md`.

## Framing: this is a WKWebView, not AppKit

RunHaven's GUI is a Tauri v2 shell with a React plus shadcn/ui front end running
in a WKWebView. It is not AppKit or SwiftUI. That has one hard consequence for
this doc:

TipKit is a native Apple framework (SwiftUI, UIKit, and AppKit; iOS 17 and
macOS 14 and later). It cannot run in a WebView. RunHaven uses TipKit only as a
pattern source: its rules for when and how to show a tip, re-implemented in
React and CSS on the components already installed. Every recommendation here is
buildable in the current stack. Where a rule names a native affordance, the
web-implementable equivalent is given.

Grounding facts observed in this repo:

- The menu bar already has a Help menu with three disabled placeholders in
  `src-tauri/src/menu/spec.rs`: `help` ("RunHaven Help"), `help-agent-reach`
  ("What Can an Agent Reach?"), and `help-report-problem` ("Report a
  Problem..."). The code comment says help content is "reserved and disabled."
  This doc fills them.
- shadcn/ui components present under `ui/src/components/ui/` include
  `tooltip.tsx`, `dialog.tsx`, `sheet.tsx`, `alert.tsx`, and `alert-dialog.tsx`.
  There is no `popover.tsx` yet, but the installed `radix-ui` package (1.6.1,
  the same metapackage that backs Tooltip and Dialog) already ships the Popover
  primitive, so a Popover wrapper is a new component file, not a new dependency.
- UI state saved to `localStorage` is allow-listed and sanitized in
  `ui/src/app/uiStatePersistence.ts` (four keys today, all validated before
  write). Tip "seen" state must join that allow-list, not bypass it.
- The launch security review lives in `ui/src/app/PlanReviewCard.tsx` inside
  `ui/src/app/LaunchSheet.tsx`; agent sign-in lives in
  `ui/src/app/AgentSignIn.tsx`.
- Consequential consent stays in Rust-owned native confirmation dialogs
  (`trusted_launch_confirmed`, `confirm_native`). That security boundary does
  not move into web tips or popovers.

Sources were fetched 2026-07-02. Apple pages are client-rendered and change; the
distilled JSON endpoints are listed at the end.

Label key: [S] sourced from Apple HIG or TipKit docs. [J] product judgment for
RunHaven.

## 1. Privacy communication

Sources: <https://developer.apple.com/design/human-interface-guidelines/privacy>

Four HIG principles [S], each mapped to a real RunHaven privacy surface [J]. The
privacy surfaces are the Claude OAuth token, the terminal no-filter and no-store
disclosure, the sandbox boundary, and provider egress.

### 1.1 Explain before you ask [S]

The HIG says to request access only when a feature needs it, to wait until a
person uses that feature, and to avoid launch-time requests unless the need is
obvious. A purpose string should be a brief, specific, active sentence.

RunHaven moments [J]:

- The "Sign in to Claude" confirm (path C in the broker doc). The Rust native
  confirm names Claude and states that the token will be saved to the Mac's
  Keychain before `claude setup-token` runs. A "?" help popover next to the
  button carries the reason first, so the native confirm is not the first time
  the user learns what is about to happen.
- Nothing privacy-relevant is requested at app launch. Sign-in, terminal open,
  and run launch each ask in context, at the moment the user chooses them.

### 1.2 Request in context, with a clear reason [S]

Purpose strings must be plain, specific, and active, not vague ("for a better
experience") or bare imperatives ("Turn on access").

RunHaven moments [J]:

- The native confirm text and the "?" reason use one clear reason in plain
  words: what is saved, where it is saved, and why. Written at roughly an 8th
  grade reading level, active voice, one idea per sentence.
- The terminal-open dialog states the reason and the facts together: you are
  opening a live shell inside this sandbox, and what you type goes straight to
  it. That copy already exists in `docs/design/native-gui-terminal-design.md`.

### 1.3 Be transparent about what is collected, stored, and sent [S]

People share data only when they understand how it is used. Say what is
collected, where it is kept, and to whom it is sent.

RunHaven moments [J]:

- Terminal: reuse the see and cannot-see facts from the terminal doc. The live
  stream is not filtered, not stored, and not logged; the only copy is bounded
  xterm scrollback in WebView memory. Show this as an always-on on-screen line,
  not buried in a menu.
- Launch review (`PlanReviewCard.tsx`): the review already lists Network,
  Image, Project, and Login state. The privacy copy on that view states what
  leaves the Mac (provider egress to allowlisted hosts) and what stays local
  (run records, diagnostics, redacted saved output). Transparency here is a
  feature, not a warning.
- Sandbox boundary: the "What Can an Agent Reach?" help (section 2.3) is the
  home for the plain-language boundary summary: non-root, read-only root
  filesystem, provider-only egress, no host home, no raw keys.

### 1.4 Never overclaim [S][J]

The HIG bars deceptive privacy framing (imitating system alerts, adding
incentives, misleading pre-alert screens). RunHaven extends this into a copy
rule: any privacy claim in a tooltip, tip, popover, or doc must be checkable
against the mechanism. If it cannot be proven true, weaken it.

The broker doc already rejected three overclaims. Its corrections are the
standing ceiling for all privacy copy [S]:

- Not "RunHaven never sends your token anywhere." The broker sends the token as
  a Bearer to Anthropic on every run. Correct: "RunHaven sends your token only
  to Anthropic, to authorize your runs, and nowhere else."
- Not "Only RunHaven can ever read this Keychain item." A same-user process with
  RunHaven's identity, or one the user grants at a keychain prompt, is a real
  case. Correct: "Other apps do not get silent access to this item."
- Not "The token is fully wiped from memory after sign-in." Zeroization has
  documented limits. Correct: "RunHaven overwrites its own copy of the token as
  soon as it is saved."

Discipline for writers: prefer the narrower true claim over the broader
comforting one. State the residual honestly. Do not imply more privacy than the
code delivers, and do not imply surveillance the app does not do.

## 2. Offering help

Sources:
<https://developer.apple.com/design/human-interface-guidelines/offering-help>

HIG rules that matter here [S]:

- Relate help to the exact task the person is doing, and make help easy to
  dismiss or skip.
- Use consistent platform terms ("click," not "tap," on a Mac).
- Do not bloat help or explain how standard controls work; describe only what a
  control does in this app.
- Tooltips (help tags) describe one control, start with a verb, do not repeat
  the control's name, stay brief (about 60 to 75 characters), and use sentence
  case.
- Simple one or two step tasks get inline help; multistep tasks get a fuller
  walkthrough. The Help menu is the menu-bar home for help.

### 2.1 The help architecture (headline)

Three layers, each mapped to a surface that already exists or is a small add
[J]:

1. Tooltip: hover or focus, one short line, for a single control. Built on the
   installed `tooltip.tsx`. Never the only place a critical fact appears.
2. Help popover ("?"): an on-demand, per-view "?" button that opens one to three
   short sentences plus an optional "Learn more" link into `docs/DESKTOP_APP.md`.
   This is the home for "why this choice" and "what it means for you." Built on
   the installed `radix-ui` Popover primitive (new wrapper file, no new
   dependency).
3. Full docs: `docs/DESKTOP_APP.md` sections, reached from the Help menu and
   from popover "Learn more" links. One canonical doc; the app links to it and
   does not restate it.

Decision rule for which layer [J]: use a tooltip for "what this control does";
use a help popover for "why this choice, and what it means for you"; use full
docs for step-by-step and the complete security model.

### 2.2 Fill the Help menu placeholders

Map each disabled item in `src-tauri/src/menu/spec.rs` to a destination [J]:

- `help` "RunHaven Help": opens the how-to, `docs/DESKTOP_APP.md`. In the
  prototype this can open the doc in the user's browser or an in-app Help view.
  Name the honest current behavior when it ships; do not promise a bundled help
  book that does not exist yet.
- `help-agent-reach` "What Can an Agent Reach?": opens the boundary explainer,
  the plain-language sandbox and egress summary that also underlies the launch
  review's Network and Access rows. This is the transparency home for section
  1.3, linking to `docs/SECURITY_MODEL.md`.
- `help-report-problem` "Report a Problem...": opens the Diagnostics export flow
  the app already has (see `docs/DESKTOP_APP.md`, "Check diagnostics"), so a
  report carries local diagnostics, never secrets.

### 2.3 The per-view "?" pattern

Each primary view gets exactly one "?" button in its header: the launch review,
agent sign-in, the terminal, and diagnostics [J]. One "?" per view, not one per
control. The "?" opens a help popover scoped to that view with a short
explanation and a "Learn more" link to the matching `docs/DESKTOP_APP.md`
anchor. Candidate anchors: "Sign in to an agent," "Open a terminal," "Review a
worktree run," "What still needs the terminal," and "Menu bar and keyboard
shortcuts." Keep the doc as the single source of truth (DRY): the popover
summarizes, the doc carries the full steps.

## 3. Tips and tooltips pattern

Sources: <https://developer.apple.com/documentation/TipKit> and the tips section
of the Offering Help page.

Reminder: TipKit is native-only and cannot run here (see the framing section).
The rules below are TipKit's behavior re-implemented in React.

### 3.1 The rules [S]

- Show a tip on the first encounter of a non-obvious feature, not on every
  visit. Tips are for discovery, not onboarding walls or ads.
- Use tips sparingly. When several exist, set a display frequency (for example
  no more than once per day) so they do not pile up.
- Define eligibility. Do not show a tip about a feature the person has already
  used.
- Show once. Persist "seen" and "dismissed" across launches, and respect a
  dismissal by never re-showing that tip.
- Placement: an inline tip keeps surrounding information visible; a popover tip
  preserves the content flow. Place the tip near the feature.
- Keep a tip to one or two action-oriented sentences. If a feature needs more
  than three steps, it is too complex for a tip.

### 3.2 RunHaven implementation [J]

- Persistence through the allow-list. Store seen tip ids in `localStorage`, but
  only by extending the sanitized allow-set in
  `ui/src/app/uiStatePersistence.ts`. Add one new allowed key, for example
  `runhaven.seenTips`, holding an array of opaque tip ids validated against the
  existing opaque-token pattern with a capped count. Do not write arbitrary
  keys; guard this key the same way the four existing keys are guarded. This is
  a deliberate note: tip state is sanitized UI state, nothing more.
- Eligibility. A tip is eligible when its id is not in `runhaven.seenTips` and
  its trigger condition holds (for example, the first time the launch review
  opens). Dismissing writes the id to `runhaven.seenTips`. A dismissed tip never
  returns.
- Frequency. Show at most one tip at a time. If several are eligible, show the
  highest-priority one and defer the rest, mirroring TipKit's TipGroup. An
  optional soft cap of one new tip per session prevents fatigue.

### 3.3 The tip and help component shape [J]

One small component family, `HelpPopover`, built on the installed `radix-ui`
Popover primitive, added as `ui/src/components/ui/popover.tsx` in the existing
shadcn style. No new dependency. Design shape, not code:

- Props: `id` (opaque, for seen-state), `title` (short), `body` (one or two
  sentences), optional `learnMoreHref` (into `docs/DESKTOP_APP.md`), `placement`,
  `variant` ("tip" for auto first-encounter, "help" for on-demand), and
  `onDismiss`.
- Two entry modes on the same content. Auto tip ("tip" variant): appears once on
  the first eligible encounter, anchored near the feature, dismissible. On-demand
  help ("help" variant): the "?" button opens the same content any time, not
  gated by seen-state.
- Short text uses the installed `tooltip.tsx` directly; only the richer content
  with a link needs the popover.

### 3.4 Accessibility [J]

- Reachable by keyboard and screen reader, not hover-only. The "?" is a real
  focusable button with an `aria-label`; the popover is a labeled region; Escape
  closes it and focus returns to the trigger.
- Tooltips open on focus, not only on hover (the installed Radix Tooltip does
  this), so keyboard and touch users reach them.
- A tip never traps focus and never blocks the view.

### 3.5 The first three tip locations [J]

1. The Claude auth choice A versus C, in `ui/src/app/AgentSignIn.tsx`. First time
   the sign-in view opens, a tip near the "Sign in to Claude" control explains
   the default (path C) in one line, with a "?" for the full custody
   explanation (section 4).
2. The terminal disclosure, in the terminal view. First time the terminal opens,
   the no-filter and no-store line appears as a tip, alongside the always-on
   on-screen disclosure from the terminal doc.
3. The launch security review, in `ui/src/app/PlanReviewCard.tsx`. First time the
   review opens, a tip points at the Network and Access rows and explains that
   this is where you see what a run can reach before it starts.

### 3.6 Anti-patterns (do not ship) [J]

- Tooltip-only critical information. Never gate a security or privacy fact behind
  hover; put it in visible copy or a focusable help popover as well.
- Hover-only on keyboard or touch. Every tip and tooltip must open on focus and
  be reachable without a pointer.
- Repeating a dismissed tip. Once dismissed, a tip stays gone.
- Tips that block. Tips are dismissible and non-blocking. Consequential consent
  stays in the Rust-owned native dialog, never a tip.
- Tips as ads or onboarding walls. A tip highlights one non-obvious feature, and
  nothing more.

## 4. Concrete application: the Claude auth A versus C choice

Sources: HIG Privacy and Offering Help (above), plus the approved claim set in
`docs/design/native-gui-claude-oauth-broker.md`.

The broker doc fixes the decision: path C (host Keychain token broker) is the
default; path A (in-sandbox login) is an explicit opt-in. This section shows how
the patterns above present that choice honestly, in
`ui/src/app/AgentSignIn.tsx`.

### 4.1 The sign-in affordance [J]

One prominent action per view (the HIG foundation rule): a primary "Sign in to
Claude" button (path C) and a secondary "Sign in inside the sandbox instead"
opt-in (path A). Choosing A runs the existing in-sandbox login unchanged.

### 4.2 Layer the explanation

- Tooltip (about 75 characters, verb-first) [J]: the broker doc's "tooltip
  strings" are longer than a help tag should be, so reclassify them. The true
  tooltip is a short fragment, for example "Save your Claude token to your Mac's
  Keychain." The full custody explanation moves into the "?" help popover.
- Help popover ("?") next to the choice: the plain 8th-grade custody explanation,
  quoting the approved claim set verbatim [S]:
  - "RunHaven saves your token to your Mac's Keychain and keeps no copy of its
    own outside the Keychain."
  - "RunHaven reads only the one Keychain item it created. It does not read your
    other Keychain logins, including Claude's own saved login."
  - "RunHaven uses your token only to sign your runs. It does not log it, display
    it, or save a copy of its own."
  - "Your token never goes inside the sandbox. The sandbox holds only a
    placeholder; RunHaven adds the real token on your Mac, only for requests to
    Anthropic."
- Full docs: a "Learn more" link to `docs/DESKTOP_APP.md` ("Sign in to an
  agent") and `docs/SECURITY_MODEL.md`, where Keychain custody, the broker, and
  token removal are documented in full.

### 4.3 Surface the tradeoff without overclaiming [J]

- The "?" popover states the honest A-versus-C tradeoff in plain words: "Signing
  in inside the sandbox keeps nothing on your Mac, but your login then lives in
  the agent's shared space, which any run using that space can read." And the
  corrected egress line: "RunHaven sends your token only to Anthropic, to
  authorize your runs, and nowhere else."
- The popover uses "Other apps do not get silent access to this item." It does
  not say "only RunHaven can ever read it" (a rejected overclaim).
- The native Rust confirm (path C) names Claude and states that the token will be
  saved to the Keychain before `claude setup-token` runs. That is the consent
  moment. The tip, tooltip, and popover only inform; they never replace the
  native confirm.
- Removal: a "Remove the Claude token RunHaven saved" affordance uses the broker
  doc's clear-action string. The popover notes that removal deletes only
  RunHaven's Keychain item and does not revoke the token on Anthropic's side, and
  links to the revoke steps in the docs.

All quoted copy above is already at roughly an 8th grade reading level and free
of em-dashes. Keep any new copy short, plain, and checkable against the
mechanism.

## Source endpoints

Fetched 2026-07-02 as client-rendered pages, distilled via the JSON endpoints:

- Privacy:
  <https://developer.apple.com/design/human-interface-guidelines/privacy>
- Offering Help:
  <https://developer.apple.com/design/human-interface-guidelines/offering-help>
- TipKit:
  <https://developer.apple.com/documentation/TipKit>
