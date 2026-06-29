# RunHaven TUI Build Plan

The master plan for building the RunHaven terminal UI as a first-class,
reference-quality, reusable implementation. This is the build sequence and the
vendoring strategy; the rendering patterns live in
[`tui-architecture.md`](tui-architecture.md) and the brand and graphics vision
lives in [`ratatui-brand-graphics.md`](ratatui-brand-graphics.md). The Codex
source capability map lives in
[`codex-tui-capabilities.md`](codex-tui-capabilities.md). Durable decisions are
logged in `current-state.md`; this doc is the working plan.

## Purpose

The TUI is RunHaven's guided front door. A bare `runhaven` on a TTY opens it, so
a non-technical user can pick a working directory and an AI provider, see exactly
what a run will and will not touch (the security boundary made visible), and
launch, with no command to memorize and no hostname to manage. The CLI stays the
complete, inspectable surface underneath; every TUI action maps to a named CLI
command shown in the UI.

This TUI is also the **reference implementation** for several sibling projects,
so it is built fully and right now, not as a deferred minimal launcher. That goal
drives the architecture below.

## Current Reset State

The earlier custom TUI phases below are historical design evidence. The active
source has now been reset to a Codex TUI source baseline in
`crates/runhaven-tui/src/tui/`, with a staged RunHaven `mod.rs` adapter that keeps the
crate buildable. A bare interactive `runhaven` now opens an unreleased
RunHaven-only MVP inside the staged Codex runtime while native Codex `App` and
`ChatWidget` remain deferred.

The dbt-wizard comparison note shows the right production pattern: keep a
coherent terminal app substrate, add a narrow product payload seam, and render
domain-specific cards from that seam. RunHaven should copy that architecture
move, not the dbt product shape and not Codex chat ontology.

dbt-wizard is not the visual target. RunHaven should stay closer to the native
Codex look: a compact intro and status area, the bottom composer and status
line, and small RunHaven cards that feel like Codex surfaces. Avoid an
analytics-dashboard feel unless the user is explicitly in a diagnostics,
history, or dashboard view. Cubby, pet behavior, mascot polish, and Zork are
final-pass work after the core TUI is complete.

The first active payload seam is in `crates/runhaven-core/src/ui_contracts.rs`:
`AgentCatalogData`, `LaunchPlanData`, and tagged `RunHavenComponentPayload`
fixtures. New visual work should feed RunHaven cards from those payloads into
the Codex-vendored shell instead of inventing another custom Ratatui screen.

The first bottom-pane source slice is also staged. RunHaven now compiles the
Codex `ListSelectionView` family from `bottom_pane/list_selection_view.rs` and
its helper modules. `crates/runhaven-tui/src/tui/mod.rs` still owns a small
facade for the event sender, list keymap, paste normalization, cancellation, and
completion types until the full Codex bottom pane is adapted. The facade uses
Codex's default list navigation keys. The upstream list-selection snapshot tests
are opt-in behind `codex-vendored-tests` because their Codex snapshot goldens
were intentionally removed during the vendor reset.

The first Codex text-entry source slice is also staged. Step 4 confirmation now
uses the vendored `bottom_pane/textarea.rs` and `bottom_pane/textarea/vim.rs`
editor primitive through the same temporary facade. This gives the confirmation
field Codex text editing, cursor placement, and Vim-keymap coverage without
adopting the full chat composer yet. Paste is intentionally ignored for the
lower-security typed confirmation phrase so the extra intent still means
typing.

The deeper Codex capability review corrected the next seam: the full
`ChatComposer` is not an isolated widget target unless RunHaven only uses the
small `public_widgets::ComposerInput` wrapper. The native Codex path is the
terminal runtime, `App` event loop, `ChatWidget`, `BottomPane`, and
`app_server_session.rs` typed facade working together. The next source-first
work should therefore align the runtime/event loop and typed app-server seam
before wiring real launch execution through the chat/composer path.

This means RunHaven is taking the capability guide's Strategy C path, a
Codex-compatible client, not Strategy B as the product architecture. Strategy B
remains acceptable only as a temporary compile bridge for low-coupling modules
while the source-first runtime, app-server facade, and native shell are adapted.
RunHaven's domain is close enough to Codex's agent/thread/turn/session model to
reuse more of the native app, but the RunHaven security boundary still controls
which app-server calls are exposed.

The first RunHaven product card over that Codex picker is now active. The
temporary `app_shell.rs` no longer draws its own agent list; it hosts a
RunHaven launch-wizard view model in `tui/runhaven/launch_wizard.rs`, rendered
through Codex `SelectionViewParams` and `ListSelectionView`. The view model is
RunHaven-owned because it maps `AgentCatalogData` and `LaunchPlanData` into
security facts: boundary, host home, credentials, auth scope, network mode, and
the exact command preview. The generic picker remains source-first Codex code.

The temporary app shell now uses Codex shell chrome for the current picker and
review step. It reserves a footer area rendered by the vendored
`bottom_pane/footer.rs`, feeds that footer with RunHaven status text from the
launch-wizard view model, shows `?` help through Codex footer hint rendering,
and writes sanitized terminal titles with the vendored `terminal_title.rs`
helper. The product-specific data still lives under `tui/runhaven/`; the footer
and terminal-title mechanics stay Codex-owned.

The current launch wizard now includes the Step 4 confirmation screen. Enter on
the review step opens confirmation, the exact planner command stays visible,
and plans marked `confirm_required` require typing `launch` before confirmation.
Confirmation now emits a prepared RunHaven launch intent, and `app_shell.rs`
hands the terminal to the foreground launch path only after Codex terminal
restore. The first chooser stays plain; review and confirm show the dense
safety facts and exact command before launch.

Historical visual check for the native Codex pet renderer:

The earlier `RUNHAVEN_TUI_IMAGE_SMOKE=1` hook has been removed from the live
`app_shell.rs` path during core-completion cleanup. The bundled RunHaven Cubby
package and lower Codex pet/image modules remain parked as source-first
infrastructure. Reintroduce a bounded visual smoke only in a final-pass pet
slice or when a core terminal-image check explicitly requires it.

Immediate integration order:

1. Keep vendored Codex source compiling in small slices.
2. Define presentation-neutral RunHaven UI payloads from existing domain data.
3. Follow the capability guide's Strategy C path by adapting a
   Codex-compatible client shape, not a permanent small TUI-kit extraction.
4. Adapt the Codex terminal runtime and event loop: `Tui`, `TuiEventStream`,
   `FrameRequester`, raw-mode restore, bracketed paste, focus, job control, and
   redraw scheduling.
5. Adapt the Codex app-server facade pattern before inventing RunHaven transport
   glue. The facade should own typed planner, launch, status, interrupt,
   history, diagnostics, and session calls. Host-reaching Codex RPCs such as
   remote filesystem, MCP, and IDE actions stay fail-closed unless a RunHaven
   security design explicitly promotes them.
6. Continue replacing temporary shell glue with Codex `App`, `ChatWidget`,
   `BottomPane`, footer, status, title, keymap, pets, and tooltips while keeping
   RunHaven domain data isolated under `tui/runhaven/`. Footer and
   terminal-title basics are now active; launch confirmation now uses Codex
   `TextArea`, but real launch execution is not wired yet.
7. Wire real launch execution from the confirmation step only after the
   Codex-native runtime, app-server facade, and bottom-pane path are clear and
   the command remains owned by RunHaven's planner.
8. Remove vendored code only after recording why removal is better than leaving
   it and adapting it.

## Audience and principles

- Built for less-technical people who sign in with OAuth or a subscription, not
  API keys. They do not manage hosts; exact hostnames appear only when they are
  needed to inspect what a run may reach.
- The secure path is the shortest, clearest path. Supported less-secure choices
  warn and require explicit intent (type-to-confirm); unsupported or
  hard-boundary violations fail closed.
- Plain language for non-technical users, roughly 8th grade reading level. Use
  short sentences, concrete nouns, visible keyboard hints, and contextual
  tooltips that teach the tool and the security model gently.
- Inspectable, never opaque: every action shows and can copy the exact CLI
  command it runs.
- Accessibility is a requirement, not a setting: `NO_COLOR`, a reduced-motion
  switch, colorblind-safe palettes, never color-only state, and a line-mode
  fallback for assistive tech.
- Delight is final-pass polish, not the current core-completion gate. Cubby,
  pet behavior, mascot polish, and Zork should wait until the TUI shell, launch
  flow, recovery, active-run logs, diagnostics, guards, and cleanup are finished.
  When pet work resumes, reduced-motion, confirmation/destructive-screen
  restraint, and an explicit user toggle still apply.

## Architecture: the framework / screen seam

Because the TUI is a reference for other projects, it is built with a clean seam
so the reusable core can later be extracted into a shared crate without dragging
RunHaven specifics:

```
crates/runhaven-tui/src/tui/
  codex/        vendored, attributed codex primitives (third-party; see below)
  <framework>   domain-agnostic core: theme/ColorMode, the event+tick loop, the
                widget/card system, key hints, tooltips, the snapshot test harness
  <screens>     RunHaven-specific surfaces that own the domain (profiles, plans,
                egress, runs)
```

Rules that keep the seam clean:

- The framework knows nothing about RunHaven's profiles, egress, or container
  boundary. It takes data and draws it.
- Screens build data from RunHaven's existing planner and policy objects (the
  single source of truth) and hand it to framework widgets. No container calls,
  planning, or policy decisions inside a widget.
- The framework is designed to lift out into a crate later; do not couple it to
  `runhaven`-specific modules.

## Vendoring strategy

The Codex TUI (`openai/codex`, `codex-rs/tui`) is Apache-2.0, so we copy and
adapt its proven code with attribution (preserve the license text and `NOTICE`,
state changes, no trademark use). Attribution lives in `THIRD_PARTY_NOTICES.md`
and `licenses/codex-Apache-2.0.txt`. Vendored files carry a derived-from header
and `#[allow(dead_code, clippy::all, clippy::pedantic)]` until wired.

Source-first rule: for any TUI behavior with an equivalent in the local official
Codex source at `/Users/c/Documents/GitHub/codex/codex-rs/tui`, vendor or adapt
that source before writing custom code. Rare exceptions are RunHaven domain data,
RunHaven security-boundary mapping, RunHaven asset swaps such as
`docs/assets/logo.png`, and small glue where Codex has no equivalent. Every
exception needs a short note in this plan or `docs/plans/tui-architecture.md`.

We vendor the proven primitives first and adapt only the parts tied to Codex's
own runtime (its tokio `FrameRequester`, event system, chat backend) or to
RunHaven's launcher/manager domain. The lesson that drove this: a generic image
crate (`ratatui-image`) auto-selected iTerm2's OSC 1337 protocol and rendered
blank in a full-screen TUI; Codex's code deliberately uses the Kitty graphics
protocol on iTerm2 3.6+ as a direct overlay, which is TUI-safe. Stick to source
for terminal-specific behavior, pet behavior, welcome/header structure, and
terminal image overlay ownership.

### Codex module evaluation

| Module | Status | Purpose / RunHaven use |
| --- | --- | --- |
| `terminal-detection` (crate) | vendored | terminal identification for image-protocol selection |
| `pets/image_protocol.rs` | vendored | Kitty/iTerm2(3.6+ `t=f`)/Sixel image emission, the logo + pet image tier |
| `pets/sixel.rs` | vendored | pure-Rust Sixel encoder (Sixel-only terminals) |
| `pets/model.rs` | vendored | pet atlas/manifest/animation model |
| `pets/frames.rs` | vendored | atlas spritesheet -> per-frame extraction |
| `pets/catalog.rs` | vendored | catalog/dimension constants |
| `pets/ambient.rs` (extract) | vendored as `animation.rs` | `current_animation_frame`, the elapsed->frame timing |
| `pets/ambient.rs` (placement/rendering) | vendored | ambient pet anchor, target size, composer gap, clear-area, and image-overlay lifecycle |
| `render/` (Renderable trait) | foundation | layout composition (Column/Flex/Row/Inset); base for cards |
| `key_hint.rs` | foundation | consistent keyboard-hint rendering |
| `wrapping.rs` (+ `width`, `line_truncation`) | foundation | URL-aware, unicode-correct wrapping/truncation |
| `bottom_pane/list_selection_view.rs` + helpers | vendored bottom pane | native Codex selection list, tab, search, wrapping, side-content, and footer behavior; active through the temporary RunHaven shell until native `App` owns the flow |
| `bottom_pane/textarea.rs` + `textarea/vim.rs` | vendored bottom pane | native Codex text editor for the Step 4 confirmation phrase; deterministic upstream tests run by default, snapshot/randomized tests stay opt-in |
| `bottom_pane/chat_composer.rs` + `public_widgets/composer_input.rs` | evaluate through runtime/app-server seam | `ComposerInput` is the Strategy B standalone wrapper; full `ChatComposer` belongs with the Strategy C Codex-compatible path: `App`, `ChatWidget`, `BottomPane`, event loop, and app-server facade |
| `terminal_hyperlinks.rs` | foundation | OSC 8 clickable paths and URLs |
| `selection_list.rs` | foundation | reusable selection primitive for the pickers |
| `clipboard` (OSC 52) | foundation | copy the equivalent CLI command, a path, a run receipt |
| `color.rs` | Phase 0 (with theme) | pure color math (light/dark, blend, distance) |
| `chatwidget/status_surfaces.rs` | evaluate next | status line and terminal-title model; useful signal, but must map to RunHaven runs/plans rather than Codex chat turns |
| `status/card.rs` | evaluate next | `/status` card structure; useful for diagnostics/status display, but data comes from RunHaven records/runtime/auth modules |
| `theme_picker.rs` + `render/highlight` | evaluate next | syntax and highlighting themes; likely useful for diffs/log snippets, gated by dependency and light/dark behavior review |
| `keymap.rs` + `chatwidget/keymap_picker.rs` | evaluate next | shortcut/accessibility model; native code is large, so evaluate for command vocabulary and conflict handling before vendoring |
| `chatwidget/session_flow.rs` thread-name handling | evaluate next | thread naming concept may map to RunHaven run labels/history names |
| `session_archive_commands.rs` + `resume_picker.rs` | evaluate next | resume/session picker patterns; RunHaven should map this to run history and relaunch/attach semantics, not Codex chat replay |
| `chatwidget/slash_dispatch.rs` status handling | evaluate next | `/status` command routing pattern; RunHaven may expose TUI command palette/status actions instead of slash chat commands |
| `terminal_title.rs` + `chatwidget/status_surfaces.rs` | evaluate next | terminal title updates can carry run state, with cleanup on exit and no secret/path leakage |
| `tooltips.rs` | evaluate next | richer tooltip/announcement system; RunHaven already has local footer tips, so evaluate for timing, suppression, and accessibility before replacing them |
| `diff_render.rs` + `diff_model.rs` | evaluated at Phase 4, not vendored | RunHaven uses its own `records::run_diff_text` data path and text diff view rather than pulling Codex git helpers |
| `pager_overlay.rs` | evaluated at Phase 3, not vendored | upstream transcript/chat overlay is tied to Codex history cells, keymaps, and app events; RunHaven ships a dedicated bounded log viewer instead |
| `status_indicator_widget.rs` / throbber | not vendored | doctor and diagnostics remain static/plain until a real async spinner is needed |
| `onboarding/` | evaluate during product-screen reattachment | RunHaven ships its own first-run guide over the shared planner/run surfaces |
| `notifications/` | evaluate during product-screen reattachment | RunHaven ships dashboard notices from active-run state and bounded log snapshots |
| `markdown_render.rs` / `markdown.rs` | reference | rich help/remediation (or `pulldown-cmark` directly) |
| `terminal_palette.rs` / `terminal_probe.rs` | reference (heavy) | true terminal-aware default colors |
| `tooltips.{rs,txt}` | superseded by evaluation row above | keep RunHaven's current tips until Codex timing/suppression/accessibility behavior is reviewed |
| `keymap.rs` (6176 lines) | superseded by evaluation row above | full rebindable-keybinding config is probably too broad, but command vocabulary and conflict handling should be evaluated |
| `file_search.rs` | skip | codex-event glue; use a fuzzy crate (e.g. `nucleo`) directly |
| `get_git_diff` | skip | uses `codex_git_utils`; RunHaven has its own git handling |
| chat domain (`App`, `ChatWidget`, markdown_stream, transcript, token_usage, model_catalog) | evaluate through native Codex path | do not rebuild chat/thread plumbing; adapt the native shell and typed app-server seam where it fits RunHaven, then map RunHaven run/session data into those surfaces |
| `app_server_session.rs` typed facade | evaluate next | major Codex TUI seam for keeping typed client calls out of `App` and `ChatWidget`; adapt the facade pattern before inventing RunHaven transport glue, but keep remote filesystem, MCP, IDE, and other host-reaching RPCs fail-closed unless a RunHaven security design explicitly permits them |
| MCP / IDE backend | skip unless promoted by security design | Codex product backend surfaces are not RunHaven defaults; review only after the boundary and user outcome are clear |

Dependencies added for vendored code are pure-Rust and exact-pinned: `base64`,
`image` (png+webp); the foundation adds text-layout crates (`unicode-width`,
`url`, `textwrap` as needed). No C dependencies.

## Active Strategy C Phases

The active execution plan is now split under
`docs/plans/codex-tui-strategy-c/`. That plan supersedes the old custom
Ratatui phase list in this document. The old phases are historical design
evidence only; do not use them as current completion status.

Current phase order:

1. Lock the vendor baseline.
2. Stop growing the temporary shell.
3. Build the Codex-shaped backend facade.
4. Adapt `App` and `BottomPane`.
5. Adapt `ChatWidget` transcript and status.
6. Reattach RunHaven product screens.
7. Cull or stub unsupported Codex product features.

The runtime-spine compile and terminal-handoff proof completed on 2026-06-27 as
supporting gates for Phase 3. They do not renumber the canonical Strategy C
plan.

Key corrections from the Strategy C review:

- `app_shell.rs` and staged `mod.rs` are compile bridges, not architecture.
- `launch_wizard.rs` stays UI-owned. The RunHaven service returns payloads and
  events; the UI turns them into views.
- Foreground launch is prepared through the typed facade, but the UI loop owns
  terminal restore and `launch_run_plan`.
- Vendor Codex protocol, utility, and TUI-adjacent crates first, preserving
  original crate names where practical. Keep active RunHaven behavior behind
  the RunHaven backend boundary.
- Upstream `.snap` files remain external by default in the local Codex checkout.
  RunHaven snapshots are generated only for wired RunHaven behavior.

## Reference-quality bar (every phase)

- Snapshot-tested with the VT100 backend + `insta` for each screen, at a few
  sizes, with deterministic (injected-clock) animation frames.
- Keyboard-complete with visible hints; no hidden keyboard-only product
  workflow. Easter eggs must be documented, attributed, non-operational, and
  isolated from RunHaven runtime boundaries.
- Plain language; the secure choice is never harder than the insecure one.
- Accessibility honored, not bolted on.
- Vendored code attributed; new dependencies pure-Rust and exact-pinned.
- The framework / screen seam kept clean so the core stays extractable.

## Dependencies On RunHaven Domain APIs

The Strategy C TUI must consume structured data from `runhaven-core`, not CLI
text:

- planner data: resolved run plan, mounts, state volume, network mode, provider
  hosts, auth scope, security notices, and exact command arguments
- run control data: active run records, status payloads, log snapshots, stop,
  kill, and repair results
- records data: run history, run detail, and `records::run_diff_text`
- diagnostics data: egress log, auth broker log, auth status, terminal
  capability facts, and doctor checks

Any structured output the TUI needs that `runhaven-core` does not yet expose is
a shared-library gap to close. Do not parse CLI prose in the TUI.

## Current Status

- The source copy is broad enough for Strategy C, and the live TUI is now a
  RunHaven-only MVP hosted in the staged shell. Native Codex `App` and
  `ChatWidget` remain deferred.
- Active staged pieces: terminal title, footer, `BottomPane`,
  `ListSelectionView`, `TextArea`, workspace picker, plain agent chooser,
  policy mutation, review, typed confirmation, foreground launch handoff,
  post-run recovery, active-run list, confirmation-gated log snapshots, and
  secret-free diagnostics. The native Cubby pet package and Codex pet/image
  renderer remain available as parked source-first infrastructure, but are not
  part of the current core-completion gate.
- The first agent chooser is intentionally plain. Dense launch details such as
  auth scope, provider hosts, not-shared host data, safety notes, and the exact
  `container run` command belong in review and confirm.
- Not yet active: native Codex `App` loop, `ChatWidget`, full app-server
  transport, filesystem RPC, MCP, login, workspace command execution, Codex
  session recording, unrelated Codex product surfaces, full history/diff
  dashboard, and Zork.
