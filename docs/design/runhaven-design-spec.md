# RunHaven design spec

Status: Canonical product and desktop design direction. Implementation migration
is incremental and remains subject to the current surface audit.

Last reviewed: 2026-07-13

## Purpose and authority

RunHaven gives people a clear, protected place to run coding assistants without
granting unrestricted access to their Mac. The interface should make boundaries,
activity, consequences, and recovery understandable to someone who does not use
Git, containers, or terminals.

This document is the source of truth for RunHaven's brand meaning, visual system,
product voice, component defaults, and design-to-code contract. Use it with:

- `native-gui-uiux-audit.md`, which records the current shipped surface and the
  Apple Human Interface Guidelines (HIG) topic checked for each part.
- `runhaven-visual-benchmark-research.md`, which records the current award and
  Liquid Glass evidence behind the approved Calm Control Room direction.
- `docs/SECURITY_MODEL.md`, which owns the product trust boundary.
- `docs/DESKTOP_APP.md`, which owns current user-facing behavior.
- `PROGRESS.md`, which resolves historical implementation plans.

Historical wireframes and navigation documents explain prior decisions but do
not override this spec, the current audit, or live product behavior.

## Experience definition

### Person and context

The primary person wants help with a software project but may not know what a
container, repository, worktree, network policy, or process is. They may keep
several sessions running and return later to understand what happened.

### Core job

Choose an assistant and project, review what the assistant can reach, start the
session, follow its work, and recover when something needs attention.

### Desired result

The person finishes with useful work, knows what changed, and can stop, review,
retry, or leave without losing context.

### Emotional quality

RunHaven should feel calm, capable, transparent, and protective. It should not
feel technical for its own sake, defensive, alarming, playful during failure,
or visually loud.

### Non-goals

- A cyberpunk security dashboard.
- A landing page inside the app.
- Glass, glow, gradients, and cards as decoration.
- Provider branding as the product's identity.
- Hiding advanced capability or security consequences to appear simple.
- Changing the typed Tauri and Rust trust boundary through visual work.

## Brand foundation

### Brand promise

Run coding assistants with clear boundaries and a clear way back.

This is an internal design promise, not a claim that every session or provider
is completely safe, private, or error-free. User-facing safety statements name
the mechanism, such as a protected copy, limited network access, native
confirmation, or reviewable changes.

### Meaning of the selected mark

The selected master is `docs/assets/app-icon.png`. It contains a clear cube over
three luminous layers on a bright cobalt-blue field.

| Visual element | Meaning | Product expression |
| --- | --- | --- |
| Clear cube | A bounded working space whose edges remain visible | Show what the assistant can reach and what it cannot reach |
| Layered base | Protection, separation, review, and recovery working together | Keep secure defaults easiest and preserve a way back |
| Light through the cube | Capability without a black box | Show status, access, and consequences in plain language |
| Upward composition | Progress and agency | Keep the next useful action obvious |
| Blue-to-teal light | Calm technical confidence | Use a restrained blue accent and rare teal brand highlights |
| Cobalt-blue field | Focus and clarity | Keep the identity legible at small sizes and content quiet around important actions |

The cube and layers are metaphors, not a literal diagram of the security model.
Do not claim that the three visible layers correspond to three technical
boundaries.

### Brand characteristics

Every product decision should support at least one of these traits:

1. **Clear**: state, access, and next actions are visible and specific.
2. **Layered**: common choices stay simple while relevant detail is available in
   context.
3. **Calm**: hierarchy and wording reduce uncertainty without minimizing risk.
4. **Capable**: the interface supports real multi-session work and recovery.
5. **Native**: Mac behavior remains familiar even when the visual identity is
   distinctive.

### Asset roles

| Asset | Role | Rules |
| --- | --- | --- |
| `docs/assets/app-icon.png` | Canonical visual identity and bundle-icon master | Keep 1024 by 1024, square, unmasked, fully opaque, and free of text or provider marks |
| `src-tauri/icons/` | Generated Tauri bundle assets | Regenerate only from the approved master with the pinned Tauri toolchain |
| `ui/src/assets/app-icon.png` | Packaged 256-pixel WebView rendition | Generate from the canonical master with the pinned icon toolchain and use only in the single shell lockup |
| Plain `RunHaven` text | Product name in the interface | Use the system typeface; do not create an ornamental wordmark without a separate brief |

The current app icon is a flattened raster that works in RunHaven's pinned Tauri
bundle pipeline. It is not a complete claim of conformance with Apple's current
layered icon system. Editable layers, appearance variants, and direct system
surface review remain open release work.

### Asset provenance and release clearance

The canonical master was generated during RunHaven app-icon concept work with OpenAI image generation on 2026-07-13, then selected by the maintainer. The
selected 1254-pixel output (SHA-256 `aaeace652e11015381bc20bb25fdca8bc3d162860843b7552c1258fa0eca2363`)
was resized once to 1024 pixels, converted to opaque sRGBA, and stripped of
metadata without changing its composition. It contains no known provider mark.
The reference-input rights record, account type, and applicable service terms
are not recorded in this repository. Before public release, record that
evidence, complete wordmark and design-mark searches plus broader web and
reverse-image similarity searches, obtain appropriate legal review, and retain
the decision with release evidence. Until then, this is approved product art,
not legally cleared brand art.

### Placement

- Use the app icon in the Dock, Finder, Spotlight, About, installer, release
  artifacts, and store or repository identity.
- In the app shell, use the selected icon in at most one compact identity
  lockup. Do not repeat it in page headers, empty states, cards, and alerts.
- Keep branding subordinate to the person's project, session, status, and next
  action.
- Never use the cube as a generic button icon or status symbol.
- Do not use the full app icon as a background watermark.

### Provider identity

Claude, Codex, Antigravity, and Copilot are choices within RunHaven, not parts of
the RunHaven mark. Use plain names with decorative neutral monograms until each
provider's current trademark terms and asset permissions are confirmed. Keep
the accessible provider name beside each monogram. Do not draw lookalike marks
or combine provider geometry with the RunHaven cube.

### Product name and descriptors

- Write the product name as `RunHaven`, one word with an uppercase R and H.
- Use `runhaven` only for the command, package, file, identifier, or code symbol.
- Do not abbreviate the product name in public or ordinary interface copy.
- No permanent tagline is approved. When context needs a description, prefer
  “Run coding assistants with clear boundaries” or a task-specific sentence.
- Do not use “Safe coding sessions” as a brand line. It is too broad for a
  product whose actual safety depends on the selected access, provider, project,
  and person reviewing the work.

### Public, documentation, and release use

- The maintainer approved provisional public-repository and README use on
  2026-07-13 while formal clearance remains open. This explicit prototype-use
  exception is not legal clearance and does not extend to a public installer,
  store listing, release page, distribution, or marketing campaign.
- The verified in-app lockup is a 32-pixel rendition beside visible `RunHaven`
  text. The README renders the master at 180 CSS pixels above its heading.
- When adjacent text names RunHaven, use empty image alternative text. When the
  image stands alone as identity, use `RunHaven`, not `RunHaven logo`.
- Preserve opaque sRGB PNG output. Derived exports keep the composition, color
  profile, aspect ratio, and lossless encoding.
- Preserve the full composition, colors, orientation, and aspect ratio. Do not
  recolor, rotate, crop, outline, place text inside, or add provider marks.
- Give the icon a quiet solid or simple gradient field when a surrounding
  surface needs a background. Do not add a second glow or shadow to the artwork.
- Use real product screenshots with fictional project and account data. Do not
  stage secrets, personal paths, credentials, or provider conversations.
- Public screenshots show one clear task and one credible state. Avoid collage,
  fake terminal output, and claims not supported by the current build.
- Release notes lead with user impact, changed behavior, recovery, and known
  gaps. Brand art does not substitute for release evidence.
- The generated 16-to-1024-pixel bundle set remains provisional until direct
  system-surface review. No general minimum size, clear space, monochrome mark,
  or alternate lockup is approved; use plain `RunHaven` text when the verified
  artwork contexts do not fit.

### CLI and terminal use

The graphical identity does not require ANSI art, a splash screen, or color in
the CLI. Commands and machine-readable output stay stable and plain. If terminal
terminal branding must preserve no-color,
reduced-motion, line-mode, copy, logging, and small-terminal behavior and earn
its place through a user task.

## Design principles

These product rules apply Apple's current principles to RunHaven.

### Purpose

Each screen has one job. Each decision region has one clear primary action.
Secondary detail earns its place by helping a decision, explaining state, or
enabling recovery.

### Agency

People can leave a flow, switch sessions, inspect what is happening, cancel when
safe, and return without killing work or losing context. Reversible actions are
preferred to repeated warnings.

### Responsibility

Access, provider data sharing, destructive effects, and unavailable evidence are
stated honestly. The interface never calls changes ready, output available, or
sign-in valid unless the backend can support that claim.

### Familiarity

Use Mac windows, menus, sidebar navigation, keyboard behavior, folder pickers,
focus, and native confirmation patterns. Custom styling does not replace
expected behavior.

### Flexibility

Support several concurrent sessions, resizable windows, pointer and keyboard
use, assistive technology, light and dark appearances, and people with different
technical experience.

### Simplicity

Remove unnecessary choices and explain advanced ones at the point of use.
Simplicity is not an empty screen, hidden capability, or missing recovery.

### Craft

Alignment, type, icons, copy, focus, motion, loading, failure, and inactive
window states belong to the same design. Review rendered behavior, not only
source or static mockups.

### Delight

Delight comes from confidence, continuity, and a clear recovery path. Reserve
visual spectacle for brand artwork, not routine controls or failure states.

## Visual system

### Color model

The icon supplies a brand palette, not a set of interface color constants. The
following values were measured from the approved 1024-pixel master and are
reference swatches for brand artwork:

| Reference color | Value | Use |
| --- | --- | --- |
| Deep harbor | `#023564` | Deepest artwork shadow and dark brand field |
| Harbor blue | `#0C59A7` | Dominant icon background |
| Run blue | `#1472CE` | Mid-blue layer and brand field |
| Core blue | `#2091E6` | Strong brand accent |
| Signal blue | `#5AADF3` | Bright blue highlight |
| Glass cyan | `#61D8F8` | Secondary light and depth |
| Layer teal | `#16B2C0` | Rare high-energy brand highlight |
| Ice | `#CBE8FB` | Glass edge and highlight |

Do not paste these values directly into component code. Interface colors use
semantic tokens that adapt to appearance, contrast, transparency, selection,
and window state.

### Semantic color roles

| Role | Meaning | Current owner |
| --- | --- | --- |
| Background and grouped background | Content hierarchy | `--background`, `--card`, `--popover` |
| Primary action and link | Action or link foreground and primary fill | `--primary`, `--primary-foreground` |
| Neutral hover or focus fill | Menu and quiet-control emphasis | `--accent`, `--accent-foreground` |
| Keyboard focus | Focus indication, not selection | `--ring`, `--sidebar-ring` |
| Sidebar selection | Current navigation row | `--sidebar-accent`, `--sidebar-accent-foreground` |
| Brand teal | Illustration or rare identity detail | No general component role |
| Success | Completed, ready, or verified positive state | `--success` plus text or icon |
| Warning | A correctable condition that needs attention | `--warning` for fill or icon; `--warning-text` for standalone adaptive text |
| Error | Failure or blocked state | Adaptive `--error-text` plus recovery text |
| Destructive | An action that causes loss | Destructive control style and explicit verb |
| Neutral | Ordinary state and secondary information | Foreground, muted, and separator roles |

Rules:

- Blue means primary interaction or keyboard focus. Selection may use a neutral
  fill plus label or weight; do not reuse blue as an informational status.
- Teal is part of the brand, but it does not mean success by itself.
- Green means a verified successful or ready state.
- Amber means a correctable warning or attention state.
- Red means failure, blocked state, or destructive action.
- Pair every state color with a label, symbol, shape, or position.
- Follow system appearance. Do not add an app-specific light or dark setting.
- Dark appearance keeps navy visibly blue and separates sidebar, work surface, and grouped content without near-black planes.
- Test light, dark, increased contrast, and reduced transparency for every custom semantic color.

### Materials and depth

The icon uses glass to show a clear boundary. The interface uses material to
show structure. Calm Control Room has three layers: the macOS frame and Tauri's
native sidebar effect own structural material; an opaque, quiet work surface
keeps task content legible; and a restrained blue RunHaven signal marks the next
action. Status colors keep their meaning and blue-to-teal light remains rare.
The WebView header is a documented exception, not a system-owned toolbar.

- Keep translucent material at the top functional layer, such as the sidebar
  and overlay header.
- Keep forms, tables, session details, terminal content, alerts, and grouped
  content opaque or near-opaque.
- Use brightness, spacing, and separators before shadow or blur to show depth.
- Use one material treatment for the shell. Do not create a different glass
  recipe for each component.
- Keep a fully opaque fallback for Reduce Transparency and unsupported native
  effects.
- Do not stack translucent cards over a translucent content surface.
- Do not add neon glows, gradient text, animated sheen, or a persistent glass
  cube behind content.

### Shape and geometry

The icon contributes three ideas: a stable square field, rounded layers, and a
precise central boundary.

- Use the existing semantic radius scale and system control shapes.
- Reserve larger concentric rounding for brand artwork, a primary empty state,
  or one clearly grouped surface.
- Do not round every nested element. Child geometry should remain quieter than
  its parent.
- Use straight alignment and thin separators for tables, output, and dense
  status information.
- Do not repeat isometric cubes as decoration throughout the product.

### Typography

- Use the system stack already defined in `ui/src/index.css`.
- Use 15-point-equivalent body text for the primary New session task. Use 13 for
  supporting or compact text. RunHaven's ordinary product floor is 11.
- Keep the hierarchy: 11 for captions, 13 for support, 15 for task body and
  section labels, and 17 for task and view titles.
- Use Regular, Medium, Semibold, or Bold. Avoid light weights.
- Use monospaced type only for terminal content, commands, code, identifiers, or
  values that benefit from alignment.
- Preserve the existing View-menu text scaling from 80 to 200 percent, including
  terminal text.
- Do not introduce a custom brand font unless it improves recognition enough to
  justify licensing, accessibility, localization, and loading cost.

### Spacing, alignment, and density

- Use the installed spacing scale and align related controls to shared leading
  edges and baselines.
- Group with proximity and negative space before adding a background or border.
- Keep spacing inside a group tighter than spacing between groups.
- Let readable forms remain compact without adopting touch-first control sizes.
- Give destructive alternatives clear spatial separation from the primary path.
- Keep ordinary text to a comfortable reading measure. Let tables, file lists,
  and terminal content use more width when comparison or scanning benefits.
- Structural window, pane, and identity geometry does not grow with the root
  text-size multiplier. Text and controls may reflow within that stable frame.
- Do not invent a second layout grid for a single screen.

### Icons

- Use `lucide-react` for standard WebView actions while the shell remains Tauri
  and React. Keep it monochrome with semantic tint only for status.
- Use one optical size and stroke treatment within a control row.
- Pair unfamiliar icons with visible text. Every icon-only control needs an
  accessible name and help text when meaning is not obvious.
- Do not export SF Symbols into the WebView without confirmed license and
  implementation permission. If a surface becomes native AppKit or SwiftUI, use
  system symbols there.
- Create custom icons only for durable RunHaven concepts that no standard symbol
  represents.

### Imagery

The selected mark is the main visual asset. Additional illustration needs a
specific teaching or empty-state purpose. It must use the same deep-navy,
blue, cyan, teal, and ice relationships without copying the cube into every
scene.

No provider logo, generic robot, padlock collage, stock terminal screenshot, or
decorative code rain belongs in the core product.

### Motion

- Use system and installed component transitions first.
- Motion explains continuity, focus, progress, insertion, or removal. It does
  not advertise the interface.
- Keep direct manipulation responsive and interruptible.
- Replace large movement and continuous effects under Reduce Motion while
  preserving state feedback.
- Never animate brand glow, glass refraction, or background layers continuously
  in the working interface.

## Experience architecture

### Product objects and ownership

| Object | Identity and persistence | Primary surface | Owner |
| --- | --- | --- | --- |
| Assistant | One of four supported product choices | New session and sign-in | Rust catalog and readiness contracts |
| Project | User-chosen folder, shown by safe display name | New session and saved setup | Native folder picker and Rust validation |
| Session | Backend-owned run identity with lifecycle state | Running, History, session detail | Rust session model |
| Live terminal | One mounted PTY channel for one exact run | Terminal workspace, selected from Running | Rust registry plus xterm view |
| Protected copy | Optional RunHaven-owned review and recovery copy | Session detail and History | Rust worktree lifecycle |
| Saved setup | Assistant plus canonical project folder and secure defaults | Saved setups and New session | Private app-data record |
| System condition | Secret-free readiness or recovery state | System | Rust diagnostics contracts |

The WebView does not invent object identity, infer safety state, or expose raw
paths and storage names. Selection is a presentation state; the selected object
remains backend-owned.

### Main window

**Observed:** RunHaven uses one resizable 1080 by 720 window, usable down to 800
by 560, with native traffic lights, overlay chrome, Tauri's native sidebar
effect, a custom WebView header, and an opaque pane. It has no native toolbar.

**RunHaven decision:** Preserve this two-region structure. It supports durable
navigation and several sessions without creating a dashboard or third-pane
inspector by default.

- Sidebar: top-level destinations and dynamic collections only.
- Header: sidebar control, current view title, current status, and at most one
  frequent contextual action.
- Content: one terminal workspace in setup or live-terminal state, or the selected History, session, System, or Help task.
- Sheets and native dialogs: bounded input or trusted consequential decisions.
- No persistent right sidebar. Add an inspector only if a real selection owns
  persistent properties that must remain visible.

### Navigation

The current navigation contract is:

- New session
- History
- System
- Running, only when live terminals exist
- Saved setups, only when saved setups exist

These labels remain stable. New session and each Running row select one terminal workspace state.
Dynamic rows have backend-owned identity and a path to their live state. Navigation never kills a session or discards a draft.

### Terminal workspace

- New session is the empty and preflight state: Choose assistant, Choose project, then Review and start, with progress beside the task in wide windows.
- Use a semantic ordered list. Only the current step owns a grouped surface.
- Completed steps become unboxed summaries with **Complete** and a specific Edit
  action. Keep choices, quiet readiness, and advanced access under Project.
- Derive Review from the typed request: assistant, project display name, access,
  network, sign-in sharing, and optional Mac API key. Hide the full host path.
- **Continue** only changes steps. **Start session** plans the run, then the
  Rust-owned native confirmation remains the authoritative launch gate.
- After confirmation, replace setup with the live terminal in the same pane, using the Rust-owned assistant and project label as its title.
- Cancel returns to Review and failures stay local. Explicit actions move focus;
  refresh does not. Summaries and actions stack at 200 percent text.
- **Save setup** is secondary and never stores advanced choices.

### Core surface state matrix

| Surface | Required states | Primary result |
| --- | --- | --- |
| Terminal workspace | Setup blocked, assistant unavailable, signed out, ready, project missing, protected-copy unavailable, preparing, confirmation canceled, launch failed, opening, live, background-hidden, ending, ended, open failed | A reviewed session becomes a reachable live terminal in place, or the person's choices remain available |
| History | Empty, ordinary history, verified attention, failed, clean protected copy, changed protected copy, unreadable protected copy, output available, output unavailable | The person understands what happened and has one truthful next action |
| Session detail | Loading, live, waiting for input, stopping, ended, failed, diagnostics loading or failed, changes clean or ready, recovery complete | Status, evidence, input, review, and recovery remain attached to one session |
| Saved setup | Available, folder missing, repairing, repair failed, selected, unpinned | A repeated secure setup starts quickly or explains how to repair it |
| System | Checking, ready, actionable failure, advanced details, repair progress, repair failed, repair complete | The person fixes the smallest product-owned blocker |
| Help | Ordinary, reached from a task, returning to preserved context, support report creating, canceled, created, Finder failure, GitHub failure, both open actions failed | The person learns the next task without losing work, or creates and reviews a local report with truthful recovery |

Add a state only when the product can enter it. Do not use placeholder states to
imply planned capability.

### Selection, focus, and persistence

- Keep one current destination and one current session selection when relevant.
- Preserve selection through polling and list refresh while the object exists.
- Preserve New session choices when Help temporarily replaces the content view.
- Keep every live terminal mounted while it runs, even when another destination
  is visible.
- Treat React destinations as an implementation detail. Preserve the setup and
  live-terminal state contract so a future SwiftUI or AppKit shell can reuse it.
- Closing the main window hides and preserves the same WebView. App activation
  restores it. App exit owns teardown.
- Persist only bounded preferences and user-created saved setups. Do not persist
  transient errors, raw output, provider credentials, or secret-bearing detail.
- Move focus to the new task region after deliberate navigation, but never move
  it during background refresh.

### Window adaptation

- Preserve the primary task at the minimum size.
- Let guided forms pair context with work, then stack before either gets cramped.
- Let history tables, terminal content, and file review use available width.
- Wrap or reflow secondary controls before truncating an action or consequence.
- Keep visual, keyboard, and accessibility order aligned.
- Test minimum, initial, wide, full-screen, inactive, and restored states.

Whitespace is useful when it protects reading and hierarchy. Do not fill a wide
window with decorative cards. Add a secondary region only when it provides
current context, comparison, or recovery.

### Menu and command model

The menu bar is the complete discoverable command surface. Toolbar and contextual
controls accelerate commands but do not replace the menu. Keep standard app,
File, Edit, View, Window, and Help behavior. Add Session commands only when they
operate on a clear current session.

Do not create Settings until RunHaven owns a real user preference that belongs
outside the task flow. Essential security and launch choices stay in context.

### Deliberately absent surfaces

- **Settings:** absent until a real preference exists. Text size belongs in View.
- **Search:** absent while History remains small and scannable. Add it only for a
  measured retrieval problem.
- **Notifications and Dock badge:** absent until an event is useful outside the
  app and can deep-link to an exact recovery state.
- **Onboarding tour:** absent. New session and contextual Help teach the product
  while doing real work.
- **Persistent dashboard:** absent. New session, History, Running, and System own
  their facts without duplication.
- **Auxiliary session windows:** absent. They previously duplicated History
  without an essential independent task.
- **Generic container administration:** absent from the ordinary GUI because it
  does not improve the core job or typed security boundary.

## Component contract

### Buttons

- Use one filled primary action for the current task.
- Use bordered secondary actions and quiet tertiary actions for alternatives.
- Name actions by result: Start session, Choose folder, Check again, End session.
- Use an ellipsis when more input or confirmation follows.
- Never make a destructive action the default action.
- Keep less-safe choices available, explicitly labeled, and no easier than the
  secure default.

### Forms

- Ask only for information the current task needs.
- Arrange controls in the order a person decides them.
- Use a native folder picker instead of path entry.
- Keep labels visible and use plain language.
- Validate near the field as soon as the value can be judged.
- Preserve entered values after a correctable error.
- Show progress for multistep or long-running preparation.
- Keep advanced choices under one clear disclosure near the decision they
  affect.
- Use a pop-up/select control for one compact choice, radio buttons when a small
  set benefits from visible comparison, and checkboxes only for independent
  yes-or-no options. Never use a checkbox as a launch action.
- Text fields and text views keep visible labels, instructions, preserved input,
  selection, keyboard editing, and a nearby error that states how to recover.
- Disclosure labels name the hidden content and expose expanded state. Help tags
  and tooltips supplement a visible label; they never carry required guidance.

### Grouped content

- Use spacing, headings, and separators before adding a container.
- A grouped surface may have one background step and one boundary.
- Do not place a card inside another card only to create hierarchy.
- Treat status, sign-in, output, input, diagnostics, and controls as regions of
  one session view rather than a stack of unrelated panels.

### Sidebar rows

- Use a familiar icon plus a visible label.
- Show selected state with more than color and expose `aria-current`.
- Keep the hierarchy shallow.
- Keep actions out of navigation rows unless they are a clearly labeled
  contextual alternative with another discoverable path.
- Distinguish duplicate sessions with assistant, project, time, or a stable
  ordinal.

### Tables and history

- Use a semantic table when people compare sessions by state, time, assistant,
  or next action.
- Let the whole row open its session with pointer or keyboard.
- Keep focus separate from the current row, preserve selection through refresh,
  and expose the selected or current state without relying on color.
- Put sessions that require a real action first and state the exact reason.
- Do not claim output, changes, or review data exists when it was not retained or
  verified.

### Status and feedback

| State | Presentation | Required content |
| --- | --- | --- |
| Loading | Activity indicator near the affected region | Specific operation name |
| Progress | Determinate progress when known | Current step and safe cancellation when available |
| Success | Quiet inline state or status | Result, only when not already obvious |
| Warning | Inline warning | Condition, consequence, and next action |
| Failure | Alert near the failed task | What failed, what remains safe, and recovery |
| Needs attention | Persistent session state | Verified reason and one useful next action |
| Unavailable evidence | Neutral or warning state | What was not kept or could not be checked |
| Empty | Purposeful empty state | What belongs here and one relevant action |

Do not use alerts for routine success. Do not use a numeric Dock badge for
session problems. Do not let background refresh steal focus.

### Native confirmation

Rust-owned native confirmation remains the trusted boundary for consequential
launch, access, message, stop, cleanup, and destructive actions.

- State the decision or consequence in the title.
- Add only facts that change the decision.
- Use result verbs, a safe default, and Cancel when nothing should change.
- Parent the dialog to the initiating trusted window.
- Revalidate the backend-owned target before mutation.
- Treat cancellation as a quiet no-op.

### Terminal and technical detail

- Terminal content is utilitarian, stable, selectable, and monospaced.
- Keep terminal input and output visually separate from app controls.
- Always provide a keyboard path out of the terminal.
- State output-retention limits before they surprise the person.
- Put raw commands, identifiers, logs, and policy detail under an Advanced or
  Details disclosure unless they are the task's content.

## Product voice

### Voice

RunHaven is calm, direct, precise, and respectful. It explains unfamiliar ideas
without talking down to people.

### Vocabulary

Use these ordinary-path terms consistently:

| Use | Avoid in ordinary UI |
| --- | --- |
| Assistant | Agent, provider client |
| Project | Repository, repo, workspace path |
| Protected copy | Worktree |
| Assistant services only | Provider-only egress |
| Shared assistant sign-in | Agent-scoped auth volume |
| This project only | Project-scoped auth volume |
| Session | Container, process, run when not historical |
| End session | Kill, terminate |
| Changes ready to review | Worktree exists |
| Could not check changes | Git probe failed |

Technical terms remain available in Advanced details, documentation, support,
and exact security disclosures when accuracy requires them.

### Copy rules

- Use sentence case, active voice, short sentences, and visible labels.
- Put the most important fact first.
- Use a verb for buttons and links.
- Match control names exactly in help.
- Write failures as problem, preserved state, and next action.
- Avoid blame, jokes, exclamation marks, vague urgency, and generic `OK`.
- Do not use `safe`, `secure`, `private`, `isolated`, or `verified` as an
  absolute. Name what RunHaven actually did.
- Write for localization. Do not concatenate sentence fragments.

## Accessibility and inclusion

Accessibility is part of the component contract, not a final visual check.

- Support pointer, keyboard-only, Full Keyboard Access, and VoiceOver for every
  core task.
- Keep focus and selection distinct, visible, and stable through refresh.
- Restore focus after dialogs, navigation, deletion, retry, and disclosure.
- Provide useful names, roles, values, states, and live announcements.
- Do not rely on color, motion, hover, sound, or a timed response.
- Support light, dark, increased contrast, reduced transparency, and reduced
  motion.
- Preserve usable content at 200 percent text and the 800 by 560 minimum window.
- Keep ordinary text at 4.5:1 contrast through 17 points. Give controls a 28 by
  28 point hit area by default and never less than 20 by 20 points.
- Test long English, a long-string locale, right-to-left layout, malformed
  content, and user-generated project names.
- Provide alternatives to precise dragging and context-menu-only actions.

### Localization and content resilience

- Treat dates, times, durations, counts, and lists as locale-aware data.
- Allow names, labels, buttons, and state messages to expand without clipping.
- Keep project and session names user-generated and potentially long or
  malformed; sanitize control characters before display.
- Mirror directional structure and symbols only when their meaning is
  directional. Keep code, commands, paths, and terminal content in their native
  direction.
- Do not use flags, color, or culturally narrow metaphors as the only identity
  for an assistant or state.

## Privacy and trust

Visual simplicity cannot weaken the product boundary.

- The protected copy and assistant-services-only network remain easiest.
- Less-safe supported choices remain visible, warned, and explicit.
- Unsupported or invalid boundary changes fail closed.
- Provider sharing, storage, training uncertainty, and human-review needs are
  disclosed at the relevant decision and in Help.
- Host paths, state-volume names, credentials, environment values, and raw
  diagnostics do not enter ordinary WebView content.
- The UI never invents capability that the typed Rust contract does not expose.

## Implementation contract

### Current framework

**Observed:** Tauri v2 owns the macOS window, native menus, trusted dialogs,
effects, and typed command boundary. React, shadcn/ui, Tailwind CSS, and Lucide
render semantic HTML inside WKWebView. The terminal-workspace state contract is
framework-neutral; a future native rewrite must not inherit WebView route names.

### Token ownership

- `ui/src/index.css` owns system font stacks, semantic colors, shape roles, and
  accessibility media queries.
- Components consume semantic roles. They do not import measured icon swatches.
- Appearance differences belong in token definitions, not per-component theme
  branches.
- New visual tokens need a named product role and light, dark, contrast, and
  transparency behavior.
- Do not recreate system material with multiple fixed CSS blur recipes.

Use three token tiers:

1. **Primitive values** define raw color, alpha, type, and geometry relationships
   inside the system. Product components do not consume them directly.
2. **Semantic roles** name intent, including window, content, grouped and
   elevated surfaces; primary, secondary and tertiary text; primary, secondary
   and destructive actions; success, warning, error, informational and neutral
   status; separator, control and focus strokes; selection; and brand accent.
3. **Component aliases** map a component state to a semantic role. A button,
   sidebar row, alert, field, or table does not create a new raw palette.

New semantic roles define light, dark, increased-contrast,
reduced-transparency, inactive-window, focused, selected, and disabled behavior
where relevant. Current WebView tokens do not model inactive-window state; that
remains migration work rather than a shipped claim.
Structural pane and icon geometry uses fixed or native window metrics and does
not scale with the root text-size multiplier.

### Native and WebView ownership

| Need | Preferred owner |
| --- | --- |
| Window lifecycle, frame, title-bar behavior, native material | Tauri and narrow macOS Rust code |
| Menu bar and keyboard commands | Tauri menu model |
| Folder choice and consequential confirmation | Native dialog owned from Rust |
| Navigation, forms, tables, session content, Help | Semantic React components |
| Backend truth, validation, run identity, security decisions | Rust typed commands and core contracts |
| Terminal canvas and input | Existing xterm component behind Rust-owned session identity |

### Performance and resilience

- First paint and navigation remain responsive while Rust-owned checks run.
- Polling updates changed regions in place and does not remount the page, clear a
  draft, close a disclosure, or steal focus.
- Bound and redact any process output before it can reach the WebView.
- Keep material, blur, shadow, and motion costs small enough for the minimum
  supported Mac and several concurrent sessions.
- Treat missing, late, duplicated, stale, partial, and out-of-order responses as
  normal states. The current selection and newest request win.
- Preserve a usable opaque shell when native material is unavailable.
- A visual failure never authorizes a broader command, filesystem, environment,
  or process bridge.

### Current migration priorities

1. Complete packaged accessibility review of the three-step New session.
2. Flatten nested cards and move **Unpin setup** out of navigation.
3. Audit semantic colors across the remaining product surfaces.
4. Review the WebView header in narrow, inactive, full-screen, contrast, and
   reduced-transparency states while keeping its Tauri exception explicit.
5. Defer the boundary transition until the trusted launch lifecycle has a
   bounded interaction and reduced-motion contract.

These are design migrations, not authorization for new backend commands,
dependencies, settings, windows, or security behavior.

## Evaluation criteria

The design system succeeds when representative people can:

1. Choose an assistant and project without needing container, Git, or terminal
   knowledge.
2. Explain what the assistant can reach before they approve a launch.
3. Keep the protected copy and assistant-services-only network without extra
   work.
4. Tell two concurrent sessions apart and return to the correct live terminal.
5. Understand why a session needs attention and take the next action without
   searching another screen.
6. Distinguish unavailable output from a failed session and unavailable changes
   from verified changes.
7. Recover a missing saved project folder without rebuilding the setup.
8. Complete the core workflow with keyboard and VoiceOver.

For human testing, record task completion, wrong turns, errors, requests for
help, recovery success, time to first session, and whether the person can explain
the access boundary in their own words. Do not use visual preference alone as a
success measure.

Before public release, at least five people who do not routinely use Git,
containers, or terminals must attempt the core tasks above. At least four must
finish without moderator rescue. Record prompted and unprompted completion
separately. Any misunderstanding of the approved access boundary, unrecoverable
dead end, or loss of work blocks the release claim until corrected and retested.

## Governance

- The selected icon, product name, vocabulary, semantic role model, and security
  language are controlled assets. Change them through an explicit design review.
- `runhaven-design-spec.md` owns target direction. `native-gui-uiux-audit.md`
  owns current shipped truth. `PROGRESS.md` owns active implementation
  status. Do not duplicate those roles in new root documents.
- A new component must name its purpose, states, input behavior, accessibility
  contract, appearance behavior, framework owner, and focused verification.
- A new raw token, custom material, custom icon, custom control, window, setting,
  notification, or persistent surface requires a current user need and the
  smallest viable implementation.
- Historical plans remain evidence, not active design authority.
- Review this spec when the deployment target, framework, app icon tooling,
  primary navigation, security model, or supported assistant set changes.

## Design review gate

For every GUI change:

1. Name the person, task, surface, states, and recovery path.
2. Check the exact current Apple HIG topic for every changed window, menu,
   control, pattern, material, input, status, and accessibility behavior.
3. Prefer deletion, native behavior, or an installed component before custom
   code.
4. Add focused fail-first coverage for changed behavior.
5. Inspect the rendered result at minimum and initial window sizes in light and
   dark appearance.
6. Check increased contrast, reduced transparency, reduced motion, 200 percent
   text, keyboard focus, and accessibility semantics when the change touches
   them.
7. Update `native-gui-uiux-audit.md` with the result and any intentional Tauri
   or security exception.
8. Obtain macOS design source review and live packaged review for rendering or
   native-behavior changes. Start the exact target with `scripts/review-gui.sh`
   so a stale process cannot stand in for the reviewed build.
9. Record unverified human, hardware, display, localization, or assistive-
   technology gaps.

## Open work

- If release polish warrants it, recreate the approved icon as editable vector
  layers in Icon Composer and test default, dark, clear, and tinted appearances.
- Complete direct Dock, Finder, Spotlight, and About inspection.
- Complete and record right-to-use, trademark, and product-name clearance for
  the public identity.
- Complete VoiceOver, Voice Control where relevant, Full Keyboard Access,
  increased-contrast, reduced-transparency, localization, multiple-display, and
  live-terminal checks.
- Test the primary workflow with people who do not use Git, containers, or
  terminals.

## Sources

Current Apple sources checked on 2026-07-13:

- [Design principles](https://developer.apple.com/design/human-interface-guidelines/design-principles), [designing for macOS](https://developer.apple.com/design/human-interface-guidelines/designing-for-macos), [branding](https://developer.apple.com/design/human-interface-guidelines/branding), and [app icons](https://developer.apple.com/design/human-interface-guidelines/app-icons)
- [Color](https://developer.apple.com/design/human-interface-guidelines/color), [materials](https://developer.apple.com/design/human-interface-guidelines/materials), [motion](https://developer.apple.com/design/human-interface-guidelines/motion), [typography](https://developer.apple.com/design/human-interface-guidelines/typography), and [layout](https://developer.apple.com/design/human-interface-guidelines/layout)
- [Accessibility](https://developer.apple.com/design/human-interface-guidelines/accessibility) and [writing](https://developer.apple.com/design/human-interface-guidelines/writing)
- [Windows](https://developer.apple.com/design/human-interface-guidelines/windows), [sidebars](https://developer.apple.com/design/human-interface-guidelines/sidebars), [toolbars](https://developer.apple.com/design/human-interface-guidelines/toolbars), [menus](https://developer.apple.com/design/human-interface-guidelines/menus), [keyboards](https://developer.apple.com/design/human-interface-guidelines/keyboards), and [focus and selection](https://developer.apple.com/design/human-interface-guidelines/focus-and-selection)
- [Buttons](https://developer.apple.com/design/human-interface-guidelines/buttons), [pop-up buttons](https://developer.apple.com/design/human-interface-guidelines/pop-up-buttons), [toggles](https://developer.apple.com/design/human-interface-guidelines/toggles), [text fields](https://developer.apple.com/design/human-interface-guidelines/text-fields), and [text views](https://developer.apple.com/design/human-interface-guidelines/text-views)
- [Disclosure controls](https://developer.apple.com/design/human-interface-guidelines/disclosure-controls) and [lists and tables](https://developer.apple.com/design/human-interface-guidelines/lists-and-tables)
- [Alerts](https://developer.apple.com/design/human-interface-guidelines/alerts), [progress indicators](https://developer.apple.com/design/human-interface-guidelines/progress-indicators), [offering help](https://developer.apple.com/design/human-interface-guidelines/offering-help), [privacy](https://developer.apple.com/design/human-interface-guidelines/privacy), [generative AI](https://developer.apple.com/design/human-interface-guidelines/generative-ai), [file management](https://developer.apple.com/design/human-interface-guidelines/file-management), and [settings](https://developer.apple.com/design/human-interface-guidelines/settings)

Brand-rights sources checked on 2026-07-13:

- [OpenAI Terms of Use](https://openai.com/policies/terms-of-use/), [OpenAI Service Terms](https://openai.com/policies/service-terms/), and [USPTO federal trademark searching](https://www.uspto.gov/trademarks/search/federal-trademark-searching)

Supporting nontechnical-user guidance:

- [WowTo on user guides for nontechnical users](https://wowto.ai/blog/how-to-design-user-friendly-software-user-guides-for-non-technical-users)
- The user-supplied WebSiteMatics excerpt on minimal fields, logical flow, clear
  labels, visual errors, responsive forms, large controls, whitespace, immediate
  validation, and progress indicators
- [LinkedIn collection on interfaces for nondevelopers](https://www.linkedin.com/top-content/user-experience/user-experience-for-non-technical-users/creating-user-friendly-interfaces-for-non-developers/)
- [Lightweight Solutions on digital literacy](https://lightweightsolutions.co/building-ui-ux-for-non-tech-savvy-users-designing-for-digital-literacy/)
- [IIC Lab on interfaces for nontechnical users](https://iiclab.com/blogs/designing-tech-interfaces-for-non-tech-users/)
- [AlterSquare on domain-heavy nontechnical users](https://altersquare.io/blog/designing-software-non-technical-domain-heavy-users)

Supporting project evidence:

- `docs/assets/app-icon.png`
- `ui/src/index.css`
- `src-tauri/tauri.conf.json`
- `docs/design/native-gui-macos-kit-notes.md`
- `docs/design/native-gui-uiux-audit.md`
- `docs/plans/native-gui-wireframe.md`
