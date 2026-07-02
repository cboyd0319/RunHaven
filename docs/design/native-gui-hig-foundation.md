# Native GUI HIG Foundation

Last updated: 2026-07-01 UTC

Distilled Apple Human Interface Guidelines (HIG) foundation for the RunHaven
desktop redesign. Scope: what the HIG says, why it matters for this app, and
what to implement in CSS/React inside the Tauri WKWebView. Native window
options are Tauri-level choices and are marked as such.

Grounding facts observed in this repo: dark navy token set in
`ui/src/index.css`, shadcn/ui components, `lucide-react` icons in about two
dozen app files, a left sidebar plus 3-step launch wizard layout per
`docs/plans/native-gui-wireframe.md`, and Rust-owned native confirmation
dialogs (a security boundary that stays).

HIG pages were fetched 2026-07-01. The HIG changed substantially at WWDC25
(Liquid Glass); treat visual specifics as volatile and re-check before big
bets.

## 1. The HIG Rules That Matter Most Here

Sources:
<https://developer.apple.com/design/human-interface-guidelines/design-principles>,
<https://developer.apple.com/design/human-interface-guidelines/designing-for-macos>,
plus per-rule pages cited in later sections.

1. **One prominent action per view.** "Keep the number of prominent buttons
   to one or two per view." RunHaven's wizard steps and session cards each get
   exactly one filled blue button (Continue, Launch, Send). Everything else is
   bordered or plain.
2. **Show status passively, interrupt rarely.** "The most effective feedback
   tends to match the significance of the information to the way it's
   delivered." Session state, boundary summaries, and diagnostics belong
   inline near the thing they describe. Native dialogs stay reserved for
   consequential choices (stop, delete, boundary changes).
3. **Verb-first, specific button labels.** "A specific button title like
   'Erase,' 'Convert,' 'Clear,' or 'Delete' helps people understand the action
   they're taking." No "OK"/"Yes"/"No" on consequential dialogs. Use "Stop
   Session", "Delete Workspace", "Keep Running".
4. **Never make a destructive action the default button.** "Don't assign the
   primary role to a button that performs a destructive action, even if that
   action is the most likely choice." Return must never confirm data loss.
5. **Hierarchy from a small fixed type set.** macOS body text is 13 pt; light
   weights are out. RunHaven adopts the macOS text-style scale (section 2)
   instead of the current near-uniform `text-sm`.
6. **Color means one thing.** "Avoid using the same color to mean different
   things." Blue = interactive, teal/green = ready/success status, amber =
   needs attention, red = destructive or blocked. Never blue for a status dot
   and a button in the same view meaning different things.
7. **Color is never the only signal.** "Avoid relying solely on color to
   differentiate between objects, indicate interactivity, or communicate
   essential information." Every status dot pairs with a text label or icon.
8. **Dark elevation: brighter means closer.** Base surfaces recede, elevated
   surfaces (popovers, sheets, focused panels) get brighter backgrounds, not
   heavier shadows. Section 3 maps this to the navy ladder.
9. **Group with space, not nested borders.** "Adding nested boxes to define
   subgroups can make your interface feel busy and constrained." Cards inside
   accordions inside cards flatten into grouped sections separated by
   padding and single-level background deltas.
10. **Sidebar is navigation only, two levels max.** "In general, show no more
    than two levels of hierarchy in a sidebar." Workspace nav and the session
    list qualify; actions, status detail, and maintenance controls do not.
11. **Nothing critical at the bottom edge.** "Avoid placing controls or
    critical information at the bottom of a window. People often move windows
    so that the bottom edge is below the bottom of the screen." Same for the
    sidebar bottom. Keep launch/stop controls in the upper content area.
12. **Progressive disclosure over modality.** "Leverage large displays to
    present more content in fewer nested levels and with less need for
    modality." Diagnostics and Maintenance stay collapsed by default, but the
    disclosure itself must be discoverable and keyboard-reachable.
13. **Determinate progress when possible; always let people cancel.** "When
    it's feasible, let people halt processing." Launch and image-pull steps
    show step-based progress ("Step 2 of 3: preparing image") with Cancel.
14. **Don't warn on expected outcomes.** "Don't warn people when data loss is
    the expected result of their action." Stopping a session the user just
    asked to stop needs no dialog; deleting a worktree with uncommitted
    changes does.
15. **Keyboard is a first-class input.** Return activates the single default
    button, Escape cancels sheets and dialogs, and common actions get
    shortcuts surfaced in the native menu bar (Tauri menu API).

## 2. Typography

Source:
<https://developer.apple.com/design/human-interface-guidelines/typography>.

macOS built-in text styles (points; in a WKWebView at default scaling treat
1 pt as 1 CSS px):

| Style       | Weight   | Size | Line height | Emphasized |
|-------------|----------|------|-------------|------------|
| Large Title | Regular  | 26   | 32          | Bold       |
| Title 1     | Regular  | 22   | 26          | Bold       |
| Title 2     | Regular  | 17   | 22          | Bold       |
| Title 3     | Regular  | 15   | 20          | Semibold   |
| Headline    | Bold     | 13   | 16          | Heavy      |
| Body        | Regular  | 13   | 16          | Semibold   |
| Callout     | Regular  | 12   | 15          | Semibold   |
| Subheadline | Regular  | 11   | 14          | Semibold   |
| Footnote    | Regular  | 10   | 13          | Semibold   |
| Caption 1   | Regular  | 10   | 13          | Medium     |

Hard rules: macOS default text is 13 pt, minimum is 10 pt, macOS has no
Dynamic Type, and "avoid Ultralight, Thin, and Light font weights."

CSS system font stack (macOS-only app, so the fallbacks barely matter):

```css
--font-sans: -apple-system, BlinkMacSystemFont, "Helvetica Neue", sans-serif;
--font-mono: ui-monospace, "SF Mono", Menlo, monospace;
```

Change from today: `ui/src/index.css` names `"SF Pro Text"` directly. Drop
it. `-apple-system` already resolves to San Francisco with correct optical
sizing and tracking; naming the font family directly bypasses that and is the
kind of thing the SF font license's mockup-only grant does not clearly cover.
`--font-mono` is new and is for session output and command lines.

Recommended RunHaven scale (define as CSS variables, replace ad hoc Tailwind
sizes):

| Token             | Maps to     | Use in RunHaven                          |
|-------------------|-------------|------------------------------------------|
| `--text-title`    | Title 2, 17/22 semibold | Main view title (wizard step title, session title) |
| `--text-section`  | Title 3, 15/20 semibold | Grouped-section headers (Diagnostics, Maintenance) |
| `--text-heading`  | Headline, 13/16 bold    | Card/row headings, dialog-adjacent emphasis |
| `--text-body`     | Body, 13/16 regular     | Default: labels, descriptions, inputs, buttons |
| `--text-callout`  | Callout, 12/15          | Dense metadata rows, table cells          |
| `--text-caption`  | Subheadline, 11/14      | Timestamps, secondary meta, sidebar section labels |

Do not go below 11 px in practice (10 pt is the floor; 11 reads better on
non-Retina externals). Today's near-universal `text-sm` (14 px) becomes
13 px body; hierarchy comes from the weight and the 15/17 px steps, not from
more sizes. Large Title and Title 1 are for full-window empty states at most;
this utility window rarely needs them.

## 3. Color and Dark Mode

Sources:
<https://developer.apple.com/design/human-interface-guidelines/color>,
<https://developer.apple.com/design/human-interface-guidelines/dark-mode>,
<https://developer.apple.com/design/human-interface-guidelines/materials>.

**Dark-only is defensible here, with eyes open.** The HIG says "avoid
offering an app-specific appearance setting" and allows dark-only interfaces
"in rare cases." RunHaven ships dark-by-default as a brand decision; the
honest HIG-aligned posture is: dark is the supported appearance, there is no
in-app toggle, and the existing light token block in `ui/src/index.css` is
either finished to the same quality bar someday or left unused. Do not ship a
half-supported light mode.

**Elevation model.** In dark mode the system uses "base and elevated"
background sets: "the base colors are dimmer... and the elevated colors are
brighter, making foreground interfaces appear to advance." Reconcile with the
navy brand by making lightness, not hue, the elevation signal. Keep the navy
hue (~hue 219-224 in oklch, as today) and define a ladder:

- Window base: darkest (current `--background` L≈0.18 is right).
- Grouped section / card: one step brighter, low or no alpha.
- Popover, sheet, focused inspector: brightest step (current `--popover` is
  barely above base; raise elevated surfaces to roughly L 0.26-0.30).
- Borders: white/teal at low alpha (as today), 1px, not shadows. Shadows are
  a light-mode idiom; in dark mode brightness does the layering work.

**Materials and glass.** Translucency belongs to the functional layer
(sidebar, toolbar/header), not the content layer: "Don't use Liquid Glass in
the content layer." So: `backdrop-filter: blur()` plus a translucent navy fill
for the sidebar and window header only. Session cards, wizard body, and
output panes use opaque or near-opaque elevated navy. This also matches the
macOS note that some window-background transparency lets desktop tinting
work. Cap the glass: one blur strength, used in at most two places.

**Semantic colors, per HIG intent:**

- Blue: interactive. Primary buttons, links, selected sidebar row, focus
  rings. Not a status color.
- Teal/green: success and ready states only. "Apply color sparingly... reserve
  it for elements that truly benefit from emphasis, such as status indicators
  or primary actions."
- Amber: warnings that need attention before proceeding; pair with an icon
  and text.
- Red: destructive actions and blocked/error states only. Red buttons only
  for destructive verbs.
- Grays for everything else: most icons and secondary text are monochrome so
  the four semantic colors keep meaning.

**Contrast requirements (dark mode page):** "make sure the contrast ratio
between colors is no lower than 4.5:1. For custom foreground and background
colors, strive for a contrast ratio of 7:1, especially in small text." Action
items: verify `--muted-foreground` on `--background` and on card surfaces at
4.5:1 minimum; verify amber and teal text-on-navy at 4.5:1; body text targets
7:1. Add these as a one-time checklist when tokens change.

## 4. Layout and Spacing

Sources:
<https://developer.apple.com/design/human-interface-guidelines/layout>,
<https://developer.apple.com/design/human-interface-guidelines/windows>,
<https://developer.apple.com/design/human-interface-guidelines/split-views>.

The HIG gives principles, not a numeric grid: "group related items," "align
components," "make essential information easy to find by giving it sufficient
space," and avoid the bottom edge for critical controls. The numbers below
are standard AppKit-derived conventions (inference, not HIG-quoted):

- Base unit 8 px; fine adjustments in 4 px steps.
- Window content margins: 20 px from window edges to content.
- Related control spacing 8 px; between groups 20-24 px; between major
  sections 32 px.
- Control row height: 28-32 px for buttons/inputs (macOS controls are
  shorter than the 44 pt touch target; that rule is for touch platforms).

Sidebar: resizable, roughly 200 px min, 220-240 px default, 320 px max
(convention; the HIG specifies behavior, not widths). macOS sidebar list
rows come in small/medium/large sizes; medium is ~28 px row height with 16 px
icons (convention). Persist the user's chosen width (Tauri store or
localStorage).

Content column: the wizard and card column should max out around 640-680 px
and center in wider windows, keeping body text near a 65-75 character
measure. Session output panes may use the full width.

Split view: thin 1 px divider ("prefer the thin divider style"), sensible
min/max pane sizes so the divider never vanishes, and a toolbar button plus
menu command to hide/show the sidebar ("avoid hiding the sidebar by
default"). Current 1080x720 default window with 720x560 minimum (from the
wireframe doc) is consistent with this.

List/table rows (session list, diagnostics tables): 13 px primary text,
11 px secondary line, ~36-44 px two-line rows; alternating row backgrounds
only for wide multi-column tables, per the lists-and-tables page.

## 5. Component Mapping

Sources:
<https://developer.apple.com/design/human-interface-guidelines/sidebars>,
<https://developer.apple.com/design/human-interface-guidelines/toolbars>,
<https://developer.apple.com/design/human-interface-guidelines/buttons>,
<https://developer.apple.com/design/human-interface-guidelines/boxes>,
<https://developer.apple.com/design/human-interface-guidelines/alerts>,
<https://developer.apple.com/design/human-interface-guidelines/progress-indicators>,
<https://developer.apple.com/design/human-interface-guidelines/feedback>,
<https://developer.apple.com/design/human-interface-guidelines/lists-and-tables>.

**Sidebar as source list.** Navigation and top-level collections only.
Selected row: full-width rounded rectangle in blue accent with white text
(the one place blue fill is always on). Icons default to monochrome/accent;
"if you use them sparingly, fixed colors can help clarify the meaning of an
icon" covers small teal/amber/red session-status dots in rows. Group the
session list under a caption-size header; keep at most two levels with
disclosure. No buttons pinned to the sidebar bottom.

**Cards become grouped sections.** macOS groups content with boxes/grouped
sections, not floating web cards. Concretely: one background step above the
window base, 8-10 px radius, hairline border or none, no drop shadow, section
title above or at the top in Title 3 / Headline, and never a box inside a
box. The current per-session card stack keeps its content but flattens:
status header, output, input, and controls are regions of one section
separated by padding and hairlines, not nested cards.

**Header as toolbar.** Treat the top strip as a macOS toolbar: leading
sidebar-toggle, view title, trailing important actions. "Prefer simple,
recognizable symbols for items instead of text," keep at most three groups,
and mirror every toolbar action as a native menu-bar command (Tauri menu),
since "it can't be the only place that presents a command."

**Button prominence ladder.** Exactly three styles: filled blue (one per
view, responds to Return), bordered/gray (secondary), plain text (tertiary,
in-context). Distinguish by style, not size. Verb-first title-style labels;
trailing ellipsis when a button opens another window or view ("Review
Access..."). Red fill only on destructive verbs, and a destructive button is
never the Return-key default.

**Progress and feedback.** Wizard/launch: determinate or stepped progress
with a short, specific description ("avoid vague terms like loading");
"switch a progress bar from indeterminate to determinate" once duration is
known. Background work (image pull, health checks): small unlabeled spinner
near the affected row. Always render a Cancel next to long operations.
Success confirmations stay quiet: inline state change, not a dialog; "they
only need to know when it doesn't" succeed.

**Confirmation dialogs (Rust-owned, stays native).** Copy contract: title is
a specific, complete statement or question (never "Are you sure?"); message
only if it adds facts (what is lost, what keeps running); at most three
buttons; verb-first titles; Cancel always present when anything is
destructive; default (trailing/Return) is the safe action; destructive style
on the loss-causing button only when the user did not deliberately choose it.
macOS suppression checkbox ("Don't ask again") is allowed for repeating
non-boundary confirmations only; never for security-boundary changes.

## 6. Iconography

Sources:
<https://developer.apple.com/design/human-interface-guidelines/sf-symbols>,
<https://developer.apple.com/sf-symbols/>,
<https://developer.apple.com/fonts/> (license text, checked 2026-07-01).

SF Symbols conventions worth copying regardless of icon set: symbols come in
nine weights matched to text weights and three scales "defined relative to
the cap height" of the adjacent text; default rendering is monochrome with
tint; color in icons is reserved for meaning (multicolor `trash.slash` uses
red "to signal data loss").

Licensing reality for a WKWebView app: SF Symbols are system-provided images
under the Xcode and Apple SDKs agreement, with an explicit "prohibition
against using symbols — or images that are confusingly similar — in app
icons, logos, or any other trademarked use." There is no web/CSS API for
symbols; using them in a WebView means exporting SVGs from the SF Symbols app
or embedding SF fonts, and the SF font license grants use "solely for
creating mock-ups of user interfaces to be used in software products running
on Apple's" operating systems and states "You may not embed the Apple Font in
any software programs." A macOS-only app is at least in-platform, but
shipping exported symbol SVGs as web assets is not clearly authorized
anywhere I can cite. That is a legal gray zone, not a design decision.

**Recommendation: keep `lucide-react` (ISC license, zero constraints) and
align it to SF Symbols metrics.** Honest tradeoff: lucide will not be
pixel-identical to native symbols, but it is already in ~24 app files, is
legally clean, and reads native when tuned:

- Default icon size 16 px next to 13 px text; 14 px next to 11-12 px text.
- Lucide's default 2 px stroke is calibrated for 24 px art. At 16 px render
  with `strokeWidth={1.5}` to match SF regular weight; 2 at most for
  emphasized contexts.
- Monochrome via `currentColor`; semantic tint only on status icons.
- Vertically align to text cap height (flex baseline alignment), one optical
  size per context, no mixed sizes in a row.
- Every icon-only control gets an `aria-label`.

Revisit only if the shell ever goes native, where real SF Symbols become the
default again.

## 7. App Icon Brief

Source:
<https://developer.apple.com/design/human-interface-guidelines/app-icons>.

Executable brief:

1. Rebuild the `docs/assets/logo.png` glass-cube mark as vector layers:
   background plus one or two foreground layers (cube faces, layer stack).
   "Prefer vector graphics when bringing layers into Icon Composer."
2. Background: navy-to-teal gradient defined in Icon Composer itself
   ("supports solid colors and gradients for background layers").
3. Foreground: crisp, hard-edged shapes; "avoid soft and feathered edges";
   vary layer opacity for depth. Do not bake in highlights, bevels, or
   shadows; the system applies specular/Liquid Glass effects.
4. Canvas 1024x1024 square, unmasked; the system applies the rounded-corner
   mask. Keep the cube centered with generous margins.
5. In Icon Composer (ships with Xcode 26): place layers, annotate default,
   dark, and mono/tinted appearance variants, preview, export.
6. No text, no photo, no screenshot, no Apple hardware, and no SF Symbols
   glyphs in the icon (license prohibition above).
7. Wire the exported icon through Tauri's bundle config (`.icns`/icon set)
   and verify at 16, 32, 128, 256, 512 px; the small sizes decide whether the
   cube needs simplification.

## 8. Anti-Patterns To Remove (Current App vs HIG)

Each item is something the current UI does or is at risk of doing, per the
context above and the wireframe doc.

1. Shadowed floating cards everywhere. macOS groups with boxes and spacing;
   replace card shadows with elevation-by-brightness and hairlines.
2. Nested containers: cards inside accordions inside cards violates "adding
   nested boxes... can make your interface feel busy and constrained."
3. One-size typography (`text-sm` for nearly everything). No hierarchy;
   adopt the section 2 scale.
4. Naming `"SF Pro Text"` in the font stack instead of `-apple-system`.
5. Glass on content surfaces. Blur/translucency is for sidebar and header
   only; "don't use Liquid Glass in the content layer."
6. Multiple filled/prominent buttons visible at once across stacked cards.
   One per view.
7. Web-style accordions as the primary disclosure for important status.
   Diagnostics may collapse; current session state and boundary summary must
   not require expansion to read.
8. Status conveyed by color alone (teal vs amber dot with no label/icon).
9. Reusing blue for both interactivity and informational status.
10. "OK"/generic confirm labels or question-mark-shrug copy in native
    dialogs; use verb-first specifics and safe defaults.
11. Critical actions at the bottom edge of the window or sidebar.
12. Indeterminate spinners with labels for long launches instead of stepped
    or determinate progress with Cancel.
13. Warning dialogs on expected outcomes (e.g., confirming a stop the user
    just clicked) while real irreversible cases ride on inline text.
14. An in-app light/dark toggle. Dark-only is the deliberate exception; do
    not add an appearance setting.

## Tauri-Level Native Options (Separate Track)

These are window/native choices, not CSS: title bar style (overlay or
transparent title bar so the web header reads as a unified toolbar, with
traffic-light inset), optional window vibrancy behind the sidebar region,
native menu bar mirroring toolbar commands and keyboard shortcuts, minimum
window size, and persisted window frame. Each needs its own security review
before adoption; none of the CSS work above depends on them.
