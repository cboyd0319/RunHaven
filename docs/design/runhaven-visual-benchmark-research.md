# RunHaven visual benchmark research

Status: Research evidence for the approved Calm Control Room direction. The
canonical requirements remain in `runhaven-design-spec.md`.

Reviewed: 2026-07-13

## Research question

How can RunHaven become a beautiful, distinctive Mac app while remaining clear
to nontechnical users, preserving its security boundary, and adopting Liquid
Glass as a functional system layer rather than decoration?

The intended emotional result is calm confidence. RunHaven should feel like a
carefully made control room for protected assistant work, not a security
dashboard, developer console, glass showcase, or imitation of another app.

## Evidence and limits

This review separates four kinds of evidence:

- **Observed product evidence:** the current RunHaven source and a fresh
  packaged Debug app.
- **Apple guidance:** the current HIG, Liquid Glass documentation, and Apple
  framework sessions.
- **Award evidence:** Apple Design Award pages and Apple Newsroom records.
- **Design judgment:** the RunHaven synthesis approved on 2026-07-13. These
  recommendations are not Apple requirements.

The structured Apple Design Award archive was reviewed in full for 2020 through
2026. Official winner records for 2017 through 2019 were also reviewed. Apple
still exposes announcement stubs for 2013 and 2014, but a dependable complete
item-level first-party archive was not located for 2015, 2016, or years before
2013. The research does not pretend those gaps were reviewed.

The pattern labels below are qualitative. The corpus was not coded as a formal
statistical dataset, so terms such as common and rare describe repeated design
evidence rather than a measured percentage.

## Apple Design Award corpus coverage

| Year | First-party record | Categories | Award slots | Year-unique titles | Coverage note |
| --- | --- | ---: | ---: | ---: | --- |
| 2026 | [Apple Design Awards](https://developer.apple.com/design/awards/) | 6 | 36 | 33 | Current winners and finalists; three cross-category repeats |
| 2025 | [2025 archive](https://developer.apple.com/design/awards/2025/) | 6 | 36 | 33 | Complete structured page |
| 2024 | [2024 archive](https://developer.apple.com/design/awards/2024/) | 7 | 42 | 40 | Includes six Spatial Computing slots |
| 2023 | [2023 archive](https://developer.apple.com/design/awards/2023/) | 6 | 36 | 32 | Complete structured page |
| 2022 | [2022 archive](https://developer.apple.com/design/awards/2022/) | 6 | 36 | 31 | Complete structured page |
| 2021 | [2021 archive](https://developer.apple.com/design/awards/2021/) | 6 | 36 | 32 | Complete structured page |
| 2020 | [2020 winners](https://developer.apple.com/design/awards/2020/) | Uncategorized | 8 | 8 | Winners only; no finalist list exposed |
| 2019 | [Newsroom record](https://www.apple.com/newsroom/2019/06/apple-design-awards-celebrate-best-in-class-design-for-apps-and-games/) and [ceremony](https://developer.apple.com/videos/play/wwdc2019/104/) | Uncategorized | 9 | 9 | Winners only |
| 2018 | [Newsroom record](https://www.apple.com/newsroom/2018/06/apple-design-awards-highlight-excellence-in-app-and-game-design/) | Uncategorized | 10 | 10 | Winners only |
| 2017 | [Newsroom record](https://www.apple.com/newsroom/2017/06/apple-design-awards-celebrate-the-best-in-innovation-and-creativity/) | Uncategorized | 12 | 12 | Winners only |
| 2013 to 2016 and earlier | Partial announcements or no complete current first-party record located | Unknown | Unknown | Unknown | Explicit archive gap |

The complete structured 2020 through 2026 corpus contains 230 category slots
and 209 year-unique titles. Normalizing obvious cross-year repeats produces
about 205 distinct product titles. Editions, renames, and sequels make a more
absolute deduplication claim inappropriate.

The corpus itself is a useful warning against equating beauty with surface
style. From 2021 onward, Visuals and Graphics is one of six recurring
categories beside Delight and Fun, Inclusivity, Innovation, Interaction, and
Social Impact. Apple repeatedly rewards purpose, accessibility, platform fit,
and interaction quality alongside visual craft.

## Patterns across the corpus

### Near-universal patterns

- One clear product idea is visible before secondary capability.
- The primary state is immediately readable.
- Interaction fits the platform and input method.
- Visual language, copy, motion, and sound feel like one product.
- Technology serves the experience instead of becoming the interface concept.

### Very common patterns

- Progressive disclosure keeps advanced capability available without putting
  every choice on the first screen.
- Recognition replaces recall. Labels, familiar objects, and visible state
  reduce the need to remember commands or technical concepts.
- Default screens are restrained and content-led.
- Cancel, undo, and recovery preserve confidence.
- The app has one memorable expression of delight tied to its purpose.

### Common utility and productivity patterns

- A stable sidebar or library preserves place.
- Compact toolbars expose frequent window-scoped actions.
- Contextual actions stay beside the object they affect.
- Keyboard commands accelerate visible workflows.
- Long-running activity remains attached to the relevant object.
- Tables and lists carry comparable information better than repeated cards.

### Common accessibility patterns

- VoiceOver and text scaling are first-order product behaviors.
- Reduce Motion, increased contrast, outline or focus aids, and alternate input
  paths are designed with the primary interaction.
- Cognitive load is reduced through visible sequencing and predictable state.
- Accessibility options preserve the product's character rather than producing
  a degraded mode.

### Rare patterns

- Large decorative glass fields in the content layer.
- Persistent marketing copy inside the working app.
- Deeply nested cards and borders.
- Dashboards that expose every available fact at once.
- Delight that has no relationship to the product's job.

Apple's Liquid Glass guidance reinforces this result. Glass belongs primarily
to the navigation and control layer. Apple explicitly warns against glass in
the content layer and glass stacked on glass. Standard system material adapts
for legibility, contrast, transparency, motion, content, and active state in a
way a fixed CSS blur cannot reproduce. See [Meet Liquid
Glass](https://developer.apple.com/videos/play/wwdc2025/219/), [Adopting Liquid
Glass](https://developer.apple.com/documentation/technologyoverviews/adopting-liquid-glass),
and [Build an AppKit app with the new
design](https://developer.apple.com/videos/play/wwdc2025/310/).

## RunHaven-relevant exemplars

These products are references, not templates. For the first 13 rows, the
transfer column is RunHaven design judgment based on the cited award-era
evidence. It does not claim that the product retains the same interface today.
DaisyDisk and Transmit use current first-party product documentation.

| Product | Evidence | What transfers to RunHaven | What does not transfer |
| --- | --- | --- | --- |
| Things 3 | [2017 winner](https://www.apple.com/newsroom/2017/06/apple-design-awards-celebrate-the-best-in-innovation-and-creativity/) | Calm hierarchy, keyboard fluency, meaningful completion feedback | Its exact blue, typography, and task-manager object model |
| Agenda | [2018 winner](https://www.apple.com/newsroom/2018/06/apple-design-awards-highlight-excellence-in-app-and-game-design/) | Time and content combined without a dashboard; useful History precedent | Note-taking chronology as the whole navigation model |
| 1Password | [2021 finalist](https://developer.apple.com/design/awards/2021/) | Security state expressed through ordinary Mac structure and disclosure | Vault metaphors or credential-manager density |
| Craft | [2021 finalist](https://developer.apple.com/design/awards/2021/) | Strong content rhythm, focus, and motion continuity | Floating decoration and document-authoring conventions |
| Nova | [2021 finalist](https://developer.apple.com/design/awards/2021/) | Dense professional capability with legible sidebar and content ownership | Developer-first density on the beginner path |
| Flighty | [2023 winner](https://developer.apple.com/design/awards/2023/) and [design interview](https://developer.apple.com/news/?id=970ncww4) | Put the newest, most consequential state first; make failure conditions excellent | Airport visual language and travel-specific data density |
| Halide Mark II | [2022 archive](https://developer.apple.com/design/awards/2022/) | Advanced capability behind a focused primary interface | Camera skeuomorphism and gesture-first controls |
| Crouton | [2024 winner](https://developer.apple.com/design/awards/2024/) | Clear organization and step-by-step information exactly where needed | Recipe imagery and touch-led kitchen interaction |
| Play | [2025 winner](https://developer.apple.com/design/awards/2025/) | Sophisticated authoring made approachable through grouped controls and immediate feedback | A canvas-first editor or inspector-heavy shell |
| iA Writer | [2025 finalist](https://developer.apple.com/design/awards/2025/) | Disciplined reduction and focus on the person's content | Hiding RunHaven state that affects trust or recovery |
| Copilot Money | [2024 finalist](https://developer.apple.com/design/awards/2024/) | Dense information summarized first, with drill-down | Financial-dashboard conventions and chart volume |
| Tide Guide | [2026 winner and finalist](https://developer.apple.com/design/awards/) | Coherent information graphics, platform fit, restrained environmental identity | Decorative full-screen atmosphere on technical work surfaces |
| Structured | [2026 finalist](https://developer.apple.com/design/awards/) | Predictable sequencing, scanability, and cognitive accessibility | Turning every task into a rigid timeline |
| DaisyDisk | [Product guide](https://daisydiskapp.com/guide/4/en/) and [release notes](https://daisydiskapp.com/releases) | One memorable visualization tied directly to the job, with progressive drill-down | A large visualization when no RunHaven decision benefits from it |
| Transmit | [Product](https://www.panic.com/transmit/) and [menu reference](https://help.panic.com/transmit/transmit5/menus/) | Operational legibility, visible activity, restoration, menus, and keyboard support | File-manager density and dual-pane structure as a default |

Command-palette-first products can inform expert acceleration, but they are a
poor primary model for people who do not know what commands exist. RunHaven's
ordinary workflow must remain visible and learnable.

## Current RunHaven assessment

### Strengths to preserve

- New session, History, and System are stable destinations.
- Each destination has one primary task.
- Security and provider uncertainty use plain language.
- Native menus and keyboard commands are discoverable.
- Status meaning does not depend on color alone.
- Text scaling, reduced motion, and semantic HTML already exist.
- Consequential actions use Rust-owned native confirmation.
- The main content is opaque while the sidebar uses native material.

### Observed visual shortcomings

- The initial and minimum dark windows leave a large visually inert field
  around a small stack of bordered content.
- New session gives readiness, sign-in, explanation, and action regions similar
  visual weight through borders inside borders.
- Bright green readiness treatment can compete with the primary blue action.
- The overlay toolbar, traffic lights, and sidebar do not yet form a fully
  resolved active and inactive window composition.
- History and System still use repeated bounded containers where typography,
  list structure, spacing, and disclosure should carry hierarchy.
- Help reads as one long document instead of a short task index.
- Wide-window whitespace feels unused rather than deliberately protective.
- The product has no signature interaction that communicates both protection
  and progress.

These are craft findings, not evidence that the current workflows or security
boundary are wrong.

### Fresh packaged review evidence

The review used the repository's fail-closed packaged-app workflow and inspected
this exact target:

- App: `target/debug/bundle/macos/RunHaven Debug.app`
- Bundle ID: `dev.runhaven.desktop.debug`
- Build ID: `19abf6e8-20260713T221917Z`
- Executable SHA-256:
  `c563fb9a040ffb82342f73ceaece5d70f52ae3bd8957c5597f64ece0540663b9`
- Icon SHA-256:
  `6f018a68eb7e52ed959b4da86140c618c60d7ef84e5f76f1414f426f40adbb86`
- Review fingerprint:
  `b3568f46b32302341d579a7425d3858bebd6c1820caff4b6faa7af61bd0b1f20`

The expert inspected the initial and minimum windows in the current dark
appearance, then closed the reviewed process. A fresh packaged light-appearance
inspection remains unverified because changing the host's global appearance
requires user confirmation. Existing automated light and dark checks are not a
substitute for that packaged visual review.

## Three possible directions

### Quiet Observatory

A restrained Mac utility with native sidebar material, an opaque content
canvas, compact toolbar, low-chroma surfaces, status-forward typography, and
minimal borders.

Strength: strongest Mac fit and longevity.

Risk: can remain too austere and fail to develop a memorable RunHaven identity.

### Guided Workbench

A visible sequence for New session, one principal work region, completed-step
summaries, and contextual detail revealed only when it affects the decision.

Strength: best fit for nontechnical users and complex launch choices.

Risk: can become a rigid wizard if applied to ordinary inspection and recovery.

### Living Haven

A more atmospheric direction using restrained blue and teal environmental
light, a protected-boundary transition, and small teaching illustrations.

Strength: strongest distinction and emotional connection to the app icon.

Risk: highest chance of decorative glass, mascot drift, or visual seriousness
overwhelming operational clarity.

## Recommended north star: Calm Control Room

Combine Quiet Observatory with Guided Workbench. Use Living Haven for one or
two moments that directly explain protection and progress.

The interface should answer these questions in order:

1. What am I doing?
2. What is RunHaven protecting?
3. What is happening now?
4. What needs me?
5. What can I safely do next?

The composition has three layers:

1. **Native frame:** native macOS chrome and Tauri's native sidebar effect
   establish the current window layer. The WebView header remains a documented
   exception until a narrow native toolbar bridge is justified and verified.
2. **Work surface:** an opaque, quiet canvas gives forms, terminal output,
   history, and recovery excellent contrast.
3. **RunHaven signal:** blue marks the next action; status colors retain their
   semantic meaning; a rare blue-to-teal field appears only when it explains a
   protected boundary or transition.

This is more ambitious than today's interface without becoming louder. The
craft comes from composition, hierarchy, state, typography, and continuity.
Glass remains structural.

## Screen concepts

### Window, sidebar, and toolbar

- Use full-height native sidebar material and remove any WebView background
  that competes with it.
- Keep selection restrained and legible in active, inactive, increased
  contrast, and reduced transparency states.
- Let the toolbar contain only window-scoped commands and one contextual
  primary action.
- Treat the traffic-light region, title, toolbar, and sidebar as one
  composition at minimum, initial, wide, and full-screen sizes.
- Keep the RunHaven icon and name to the single compact shell lockup.
- Use no decorative background art in the sidebar.

### New session

- Present Assistant, Project, and Review and start as one vertical sequence.
- Only the current step owns the principal grouped surface.
- Collapse a completed step into a compact summary row with an Edit action.
- Keep readiness beside the assistant name, but quieter than Continue.
- Put advanced access choices one level below the project decision.
- Immediately before Start session, show a concise protection summary beginning
  with “RunHaven will”.
- Preserve every choice when a person goes back, opens Help, cancels native
  confirmation, or corrects an error.

### Running terminal

- Make the terminal the visual center instead of a card inside app chrome.
- Use a compact status strip for assistant, project, elapsed state, network
  boundary, and End session.
- Keep message, output retention, and exact live state attached to this run.
- Put diagnostics and technical detail in one disclosure or contextual detail
  region.
- Do not let routine status colors glow over terminal content.

### History

- Use a stable table or list with status, project, assistant, time, and outcome.
- Put “Needs you” above completed history only when a verified action exists.
- At wide sizes, preserve list context while showing selected-session detail.
- At narrow sizes, open detail in the same content region with an obvious Back
  to History path.
- Keep clean, changed, unreadable, and unavailable evidence visually distinct
  without manufacturing urgency.

### System

- Lead with one overall readiness statement and its next action.
- Show problems as compact rows containing problem, consequence, and repair.
- Keep healthy checks under one disclosure.
- Preserve progress in place during repairs and keep the affected row until the
  result is known.
- Do not turn system readiness into a monitoring dashboard.

### Help

- Lead with a task index: Start a session, Understand protection, Recover a
  session, and Fix setup.
- Open short anchored sections instead of presenting one long page.
- Keep provider and privacy detail available without mixing it into every
  procedural step.
- Return to the preserved originating task.

### Sheets and alerts

- Keep consequential confirmation native and Rust-owned.
- Name affected files, resources, or sessions when the trusted contract can do
  so safely.
- State the consequence and the reversible next step.
- Use specific actions instead of OK.
- Keep routine validation and success inline.

## Signature motion

RunHaven needs one memorable interaction, not a motion system competing for
attention.

The proposed signature is **boundary established**:

1. The final protection summary is visible before confirmation.
2. After approval, a restrained line or field settles around that summary.
3. The summary condenses into the running-session status strip.
4. The terminal appears without a disconnected page flash.

The transition explains that the reviewed boundary became a running session.
It must not imply a stronger guarantee than the Rust plan provides.

Other motion remains functional:

- Completed New session steps condense in place.
- Status updates cross-fade or highlight without remounting the page.
- Inserted, removed, or repaired rows preserve spatial continuity.
- Controls show immediate pressed and progress state.
- Reduced Motion replaces spatial movement with near-immediate fades or
  highlights.
- No continuous glow, shimmer, lensing animation, bounce, or ambient movement
  belongs on working surfaces.

## Liquid Glass contract

### Apple guidance

- Reserve Liquid Glass for important navigation and controls at the top
  functional layer.
- Keep lists, tables, forms, terminals, and other content in the content layer.
- Avoid glass on glass.
- Use standard framework controls and grouping so the material adapts to its
  context.
- Test reduced transparency, increased contrast, reduced motion, active and
  inactive state, different content, and performance.

Primary sources: [Liquid Glass
overview](https://developer.apple.com/documentation/technologyoverviews/liquid-glass),
[materials](https://developer.apple.com/design/human-interface-guidelines/materials),
[sidebars](https://developer.apple.com/design/human-interface-guidelines/sidebars),
[toolbars](https://developer.apple.com/design/human-interface-guidelines/toolbars),
[windows](https://developer.apple.com/design/human-interface-guidelines/windows),
and [accessibility](https://developer.apple.com/design/human-interface-guidelines/accessibility).

### RunHaven ownership

| Need | Owner |
| --- | --- |
| Window lifecycle, title bar, traffic lights, system material, native toolbar behavior | Tauri and narrow macOS Rust code |
| Menus, keyboard commands, file panels, trusted confirmations | Tauri and Rust |
| Forms, lists, tables, Help, session summaries, terminal framing | Semantic React components |
| Security truth, redaction, run identity, validation, destructive authorization | Rust contracts |
| Opaque material fallback | Semantic WebView tokens |

The WebView must not simulate Liquid Glass by adding fixed blur layers. If a
native bridge cannot supply the intended material correctly, the fallback is an
opaque semantic surface, not imitation glass.

## Accessibility and appearance variants

- Light and dark differences belong in semantic tokens.
- Increased Contrast strengthens separators, focus, selection, and status
  outlines without creating a separate visual language.
- Reduced Transparency replaces sidebar material with an opaque semantic
  surface.
- Inactive windows reduce accent prominence while preserving selection and
  status legibility.
- The fixed structural sidebar continues to leave the main task usable at 200
  percent text.
- VoiceOver order follows visible task order. Async announcements remain short
  and specific.
- Every core outcome remains available by pointer, keyboard, menu, and
  appropriate accessibility action.
- Long strings, right-to-left content, malformed project names, Full Keyboard
  Access, Voice Control, and relevant Switch Control paths remain required
  verification states.

## Approved implementation sequence

Calm Control Room was approved on 2026-07-13. Each slice still requires its own
HIG, accessibility, security, test, and packaged-review gates.

1. Flatten New session and establish its current-step hierarchy.
2. Define active, inactive, increased-contrast, and reduced-transparency shell
   states.
3. Convert History to a stable list and detail composition.
4. Rework System into readiness summary, problem rows, and disclosed healthy
   detail.
5. Make terminal and status hierarchy explicit.
6. Reorganize Help around tasks.
7. Add the one boundary-established transition and its reduced-motion form.
8. Run packaged minimum, initial, wide, full-screen, light, dark, keyboard,
   VoiceOver, and nontechnical-user reviews.

The first implementation slice is New session only. It proves composition,
grouping, semantic status, and the RunHaven signal without backend changes.

## Canonical adoption

The canonical spec now adopts these parts of Calm Control Room:

- Expand the emotional quality from calm and capable to calm, composed, and
  quietly luminous.
- Keep the ban on decorative glass, but make system Liquid Glass the explicit
  target for native sidebar and toolbar structure.
- Define one principal work region per destination instead of repeated cards.
- Add the three-layer composition: native frame, opaque work surface, RunHaven
  signal.
- Defer the boundary-established transition until the trusted launch lifecycle
  has a bounded interaction and reduced-motion contract.
- Define New session's completed-step summaries and current-step ownership.
- Define wide History list and detail behavior with a narrow fallback.
- Replace the current migration order with the sequence above.

These deltas change presentation and interaction hierarchy only. They do not
authorize new backend commands, access, settings, provider identity, windows,
dependencies, or weaker security defaults.

## Approval decision

Calm Control Room is the visual north star. This decision approves bounded
screen contracts, not every research idea. Normal HIG, accessibility, security,
test, and packaged-review gates remain mandatory.
