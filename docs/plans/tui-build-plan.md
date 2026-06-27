# RunHaven TUI Build Plan

The master plan for building the RunHaven terminal UI as a first-class,
reference-quality, reusable implementation. This is the build sequence and the
vendoring strategy; the rendering patterns live in
[`tui-architecture.md`](tui-architecture.md) and the brand and graphics vision
lives in [`ratatui-brand-graphics.md`](ratatui-brand-graphics.md). Durable
decisions are logged in `current-state.md`; this doc is the working plan.

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
crate buildable. A bare interactive `runhaven` now opens a temporary read-only
launch preview while the full Codex app shell and bottom pane are adapted.

The dbt-wizard comparison note shows the right production pattern: keep a
coherent terminal app substrate, add a narrow product payload seam, and render
domain-specific cards from that seam. RunHaven should copy that architecture
move, not the dbt product shape and not Codex chat ontology.

dbt-wizard is not the visual target. RunHaven should stay closer to the native
Codex look: a compact intro and status area, the bottom composer and status
line, source-native pet behavior, and small RunHaven cards that feel like Codex
surfaces. Avoid an analytics-dashboard feel unless the user is explicitly in a
diagnostics, history, or dashboard view.

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

Temporary visual check for the native Codex pet renderer:

```bash
RUNHAVEN_TUI_IMAGE_SMOKE=1 cargo run --locked --bin runhaven
```

This is only a smoke path for checking terminal image quality while the full
Codex app shell and bottom pane are being adapted. It loads `custom:cubby` from
`$CODEX_HOME/pets/cubby/` and uses Codex's vendored `AmbientPet`,
`FrameRequester`, and terminal image writer. Quit with `q`.

Immediate integration order:

1. Keep vendored Codex source compiling in small slices.
2. Define presentation-neutral RunHaven UI payloads from existing domain data.
3. Replace temporary manual launch-list drawing with Codex `SelectionViewParams`
   and `ListSelectionView`, fed by the RunHaven payload seam.
4. Adapt Codex bottom pane, status line, key handling, title, pets, tooltips,
   and render lifecycle where they fit the RunHaven product.
5. Remove vendored code only after recording why removal is better than leaving
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
- Delight is the default, not opt-in. The Cubby pet is visible and animated by
  default to make RunHaven approachable to less-technical users; the user can
  toggle it off. This still honors the two rules above: reduced-motion keeps the
  pet visible but static, and the restraint rule keeps all animation off
  confirmation and destructive screens. So "pet on by default" means the idle pet
  animates on safe/idle surfaces unless reduced-motion or the user's toggle says
  otherwise; the pet stays visible either way.

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
| `bottom_pane/list_selection_view.rs` + helpers | staged foundation | native Codex selection list, tab, search, wrapping, side-content, and footer behavior; currently behind a RunHaven facade while the full bottom pane is adapted |
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
| `onboarding/` | referenced at Phase 5, not vendored | RunHaven ships its own first-run guide over the shared planner/run surfaces |
| `notifications/` | referenced at Phase 5, not vendored | RunHaven ships dashboard notices from active-run state and bounded log snapshots |
| `markdown_render.rs` / `markdown.rs` | reference | rich help/remediation (or `pulldown-cmark` directly) |
| `terminal_palette.rs` / `terminal_probe.rs` | reference (heavy) | true terminal-aware default colors |
| `tooltips.{rs,txt}` | superseded by evaluation row above | keep RunHaven's current tips until Codex timing/suppression/accessibility behavior is reviewed |
| `keymap.rs` (6176 lines) | superseded by evaluation row above | full rebindable-keybinding config is probably too broad, but command vocabulary and conflict handling should be evaluated |
| `file_search.rs` | skip | codex-event glue; use a fuzzy crate (e.g. `nucleo`) directly |
| `get_git_diff` | skip | uses `codex_git_utils`; RunHaven has its own git handling |
| chat domain (chatwidget, composer, bottom_pane input, markdown_stream, transcript, token_usage, model_catalog) | skip | agent chat runs inside the container, not in the TUI |
| app-server / MCP / IDE backend | skip | codex backend, not applicable |

Dependencies added for vendored code are pure-Rust and exact-pinned: `base64`,
`image` (png+webp); the foundation adds text-layout crates (`unicode-width`,
`url`, `textwrap` as needed). No C dependencies.

## Build phases

Each phase ships at the reference-quality bar (below). Phases are sequenced so
later screens build on earlier foundations.

### Phase 0 - Foundation (complete)

The reusable spine. Vendor the foundation primitives (`render`, `key_hint`,
`wrapping`, `terminal_hyperlinks`, `selection_list`, OSC 52 clipboard). Build:

- the theme system: `color.rs` plus a `Palette` and `ColorMode` (dark/light
  detection), honoring `NO_COLOR` and a reduced-motion switch;
- the event + tick loop (RunHaven's own, `event::poll`-driven, supporting
  animation, replacing codex's tokio `FrameRequester`);
- the VT100 + `insta` snapshot test harness used by every later screen.

### Phase 1 - Brand complete (complete)

- Header logo: render `docs/assets/logo.png` through the Codex-derived terminal
  image overlay path on graphics terminals, with a half-block fallback on plain
  terminals.
- Native Cubby pet: the idle loop (blink, spark pulse) is driven by
  `codex::animation` and the Phase 0 tick loop, with Codex-derived ambient
  placement/rendering where supported and half-block fallback everywhere else.
  Cubby is enabled and visible by default on safe/idle surfaces; `p` and
  `RUNHAVEN_TUI_PET=0` hide the pet without hiding the logo. Reduced-motion keeps
  Cubby visible but static.
- RunHaven-authored rotating tooltips that teach shortcuts and the security
  model. Evaluate Codex's tooltip timing/suppression/accessibility behavior
  before replacing them.

### Phase 2 - The launcher flow (complete)

The directory-and-provider front door.

- Workspace/folder picker (fuzzy search) so a user points RunHaven at their
  project.
- Agent/provider picker (extends the existing home list).
- Plan + egress review card: the defining security screen, what the run mounts,
  the network mode, the provider hosts it may reach, and what it explicitly will
  not touch (host home, credentials), built from RunHaven's planner/policy.
- Confirm-launch modal: names the exact mounts, network mode, and egress posture;
  type-to-confirm only for less-secure choices.
- Launch a real run.

### Phase 3 - Run management (complete)

- Live run dashboard: active run list, sanitized live status, resource summary,
  network attachments, and a streaming egress ledger backed by the provider
  runtime's decision-delta flusher.
- Bounded log viewer: explicit active-run log snapshots with search, scrolling,
  tail-following, and ANSI parsing through `vt100` so escape sequences are not
  replayed into the user's terminal.
- Stop / kill / repair with plain typed-confirm screens (no motion on
  destructive surfaces), routed through the existing validated run-control
  cores.

### Phase 4 - History and diagnostics (complete)

- Run history with per-run records and "what changed" diff review
  (`diff_render`).
- Diagnostics: egress log, auth status, and a terminal/render capability probe.
- `doctor`: prerequisite checks with spinners and inline remediation.

### Phase 5 - Polish (complete)

- Guided onboarding: a fresh cache opens the RunHaven Guide first, and `?`/F1
  opens it later from the main screens.
- Run-done / waiting-for-input notifications: the dashboard surfaces explicit
  notices for stale/done runs, control transitions, status errors, and output
  that appears to be waiting for interactive input.
- Full accessibility pass: `NO_COLOR`, reduced-motion, colorblind-safe palettes,
  line-mode fallback, and `RUNHAVEN_TUI_COLOR_MODE=light|dark` are documented
  and covered by focused render tests.
- Themes and Zork easter egg: light/dark palette selection is implemented, and
  the hidden Home-only `~` screen runs the bundled MIT-licensed Zork I story
  through an attributed Ferrif-derived Z-machine.
- Complete snapshot coverage; the architecture doc is finalized as the reference
  guide for sibling projects.

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

## Dependencies on RunHaven domain APIs

Some phases need RunHaven's own planner/run code to expose structured data to the
TUI rather than printed text:

- Phase 2 (plan/egress review, confirm) needs the resolved run plan, mounts,
  network mode, and egress allow-set as structured values.
- Phase 3 (dashboard, egress ledger) needs a live run-status and
  network-decision stream. Complete: the TUI consumes the existing active-run
  status/log cores and the provider runtime writes egress decision deltas during
  provider-mode execution.
- Phase 4 (history, diagnostics) needs run records and the egress/auth logs as
  data. Complete: `crates/runhaven-core/src/records/` exposes a real facade over
  `run_history` and JSONL IO, `crates/runhaven-core/src/diagnostics.rs` owns secret-free log
  readers/status payloads, and `crates/runhaven-core/src/doctor.rs` owns shared host
  readiness checks. TUI Phase 4 consumes those data modules, not CLI prose.

Any structured output the TUI needs that the CLI does not yet expose is a CLI gap
to close in the shared library, not text to re-parse. These are surfaced per
phase as they arise.

## Status

- Vendored: the pet/image rendering core (`tui/codex/`: terminal detection, image
  protocol, sixel, pet model, frames, catalog, animation timing) with attribution
  and the `base64`/`image` deps.
- Complete: Phase 0 foundation primitives. The TUI now has a theme/settings
  layer with `NO_COLOR`, reduced-motion, and line-mode switches; a synchronous
  `event::poll` tick loop; Codex-derived color helpers; a Codex-derived VT100
  backend; and `insta` snapshots for the current home/detail screens at multiple
  sizes.
- Complete: Phase 1 brand. The launcher loads the validated Cubby Codex pet
  package from `crates/runhaven-tui/src/tui/assets/cubby/`, drives the idle loop with
  Codex animation timing, renders the current atlas frame as a half-block
  fallback, and emits the Codex Kitty/iTerm2/Sixel image overlay after the
  ratatui draw when the terminal supports it. Cubby is visible by default,
  `p` toggles it for the session, `RUNHAVEN_TUI_PET=0` starts with it hidden,
  reduced-motion keeps it visible but static, and line-mode omits it. The copied
  QA evidence for the pet lives in `docs/assets/cubby-pet/`.
- Complete: Phase 2 launcher flow. The TUI now has a workspace picker with
  simple fuzzy filtering and typed paths, keeps the existing agent picker, builds
  `AgentRunPlan` through RunHaven's shared planner, renders the workspace mount,
  state volume, network mode, provider egress posture, explicit non-mounts, and
  equivalent CLI command, requires typed confirmation for plans with security
  notices, restores the terminal, and launches through `launch_run_plan`.
- Complete: Phase 3 run management. The TUI now has a run dashboard (`d`) with
  active runs, sanitized status/resource/network details, and a provider egress
  ledger; provider-mode runs stream egress decisions to the JSONL log as deltas
  while the run is active; logs open as explicit bounded snapshots with search,
  scroll, tail-following, and ANSI parsing through `vt100`; and stop, hard-stop,
  and stale-marker repair use plain typed-confirm screens over the existing
  validated run-control cores.
- Complete: pre-Phase 4 organization lock. Shared TUI data dependencies now
  live outside CLI presentation: host readiness in
  `crates/runhaven-core/src/doctor.rs`, secret-free diagnostics in
  `crates/runhaven-core/src/diagnostics.rs`, auth posture labels in
  `crates/runhaven-core/src/provider/auth_profiles.rs`, and run history behind
  `crates/runhaven-core/src/records/` with `records/run_history.rs` plus
  `records/io.rs`. The workspace now uses crate ownership boundaries instead
  of a root compatibility facade.
- Complete: Phase 4 history and diagnostics. The TUI now has run history (`h`),
  per-run diff review, diagnostics (`g`) for egress/auth metadata and terminal
  render capabilities, and a doctor screen (`d` from diagnostics) with
  prerequisite checks plus inline remediation. Diff review uses the shared
  `records::run_diff_text` API; diagnostics and doctor consume
  `diagnostics.rs` and `doctor.rs` data rather than CLI prose.
- Complete: Phase 5 polish. A fresh cache starts on the RunHaven Guide and
  `?`/F1 opens it later; the dashboard emits plain notices for status errors,
  control transitions, stale/done runs, and output that appears to be waiting for
  input; accessibility controls cover no-color, reduced-motion, line-mode, and
  explicit light/dark palette selection; VT100 snapshots now cover guide,
  launcher, dashboard, logs, control, history, diagnostics, and doctor screens;
  and the architecture guide is finalized around the framework/screen seam.
- Complete: post-polish Zork easter egg. The Home-only `~` screen runs the
  bundled MIT-licensed Zork I story through a vendored, attributed
  Ferrif-derived Z-machine. The implementation adds no new Cargo dependencies,
  runs in-process only, performs no subprocess/network/workspace/container
  access, validates the bundled story by exact length and SHA-256, and constrains
  save/restore to one private RunHaven cache slot with Quetzal/IFF validation
  before restore.
