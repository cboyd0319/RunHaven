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

## Audience and principles

- Built for less-technical people who sign in with OAuth or a subscription, not
  API keys. They manage no hosts and see no hostnames.
- The secure path is the shortest, clearest path. Supported less-secure choices
  warn and require explicit intent (type-to-confirm); unsupported or
  hard-boundary violations fail closed.
- Plain language, no jargon. Visible keyboard hints on every screen. Contextual,
  rotating tooltips that teach the tool and the security model gently.
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
src/runhaven/cli/tui/
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

We vendor the proven, domain-agnostic primitives and re-implement the parts tied
to codex's own runtime (its tokio `FrameRequester`, event system, chat UI). The
lesson that drove this: a generic image crate (`ratatui-image`) auto-selected
iTerm2's OSC 1337 protocol and rendered blank in a full-screen TUI; codex's code
deliberately uses the Kitty graphics protocol on iTerm2 3.6+ as a direct overlay,
which is TUI-safe. Stick to source for the terminal-specific hard parts.

### Codex module evaluation

| Module | Status | Purpose / RunHaven use |
| --- | --- | --- |
| `terminal-detection` (crate) | vendored | terminal identification for image-protocol selection |
| `pets/image_protocol.rs` | vendored | Kitty/iTerm2(3.6+ `t=f`)/Sixel image emission, the hero + pet image tier |
| `pets/sixel.rs` | vendored | pure-Rust Sixel encoder (Sixel-only terminals) |
| `pets/model.rs` | vendored | pet atlas/manifest/animation model |
| `pets/frames.rs` | vendored | atlas spritesheet -> per-frame extraction |
| `pets/catalog.rs` | vendored | catalog/dimension constants |
| `pets/ambient.rs` (extract) | vendored as `animation.rs` | `current_animation_frame`, the elapsed->frame timing |
| `render/` (Renderable trait) | foundation | layout composition (Column/Flex/Row/Inset); base for cards |
| `key_hint.rs` | foundation | consistent keyboard-hint rendering |
| `wrapping.rs` (+ `width`, `line_truncation`) | foundation | URL-aware, unicode-correct wrapping/truncation |
| `terminal_hyperlinks.rs` | foundation | OSC 8 clickable paths and URLs |
| `selection_list.rs` | foundation | reusable selection primitive for the pickers |
| `clipboard` (OSC 52) | foundation | copy the equivalent CLI command, a path, a run receipt |
| `color.rs` | Phase 0 (with theme) | pure color math (light/dark, blend, distance) |
| `diff_render.rs` + `diff_model.rs` | vendor at Phase 4 | "what did the agent change" run/worktree diff |
| `pager_overlay.rs` | vendor at Phase 3 | scrollable log/help viewer |
| `status_indicator_widget.rs` / throbber | Phase 3 (or `throbber-widgets-tui`) | spinners during waits |
| `onboarding/` | reference at Phase 5 | guided first-run pattern (build our own) |
| `notifications/` | Phase 5 | run-done / waiting-for-input alerts |
| `markdown_render.rs` / `markdown.rs` | reference | rich help/remediation (or `pulldown-cmark` directly) |
| `terminal_palette.rs` / `terminal_probe.rs` | reference (heavy) | true terminal-aware default colors |
| `tooltips.{rs,txt}` | reference only | idea is great; write RunHaven's own tips + a ~10-line picker |
| `keymap.rs` (6176 lines) | skip | full rebindable-keybinding config; overkill for fixed keys |
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

### Phase 0 — Foundation (in progress)

The reusable spine. Vendor the foundation primitives (`render`, `key_hint`,
`wrapping`, `terminal_hyperlinks`, `selection_list`, OSC 52 clipboard). Build:

- the theme system: `color.rs` plus a `Palette` and `ColorMode` (dark/light
  detection), honoring `NO_COLOR` and a reduced-motion switch;
- the event + tick loop (RunHaven's own, `event::poll`-driven, supporting
  animation, replacing codex's tokio `FrameRequester`);
- the VT100 + `insta` snapshot test harness used by every later screen.

### Phase 1 — Brand complete

- Hero image tier: render the high-resolution Cubby via `codex::image_protocol`
  (Kitty overlay emitted after the ratatui draw, positioned over the banner) on
  graphics terminals, with the xterm-256 half-block sprite as the fallback. Needs
  an iTerm2 test cycle.
- Animated pet: the idle loop (blink, spark pulse) driven by `codex::animation`
  and the Phase 0 tick loop, half-block on every terminal, the image tier where
  supported. Enabled and visible by default (the approachable, fun first
  impression); an explicit user toggle (a setting and/or env var, surfaced in the
  UI) turns it off. Reduced-motion keeps the pet visible but static; the restraint
  rule keeps it off confirmation and destructive surfaces.
- RunHaven-authored rotating tooltips that teach shortcuts and the security
  model.

### Phase 2 — The launcher flow

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

### Phase 3 — Run management

- Live run dashboard: status, resource use, and the streaming egress ledger
  (every network decision in real time, allowed muted, blocked in the danger
  token).
- Bounded log viewer: search, smart tailing, ANSI-preserving, ring-buffered.
- Stop / kill / repair with plain confirm modals (no motion on destructive
  surfaces).

### Phase 4 — History and diagnostics

- Run history with per-run records and "what changed" diff review
  (`diff_render`).
- Diagnostics: egress log, auth status, and a terminal/render capability probe.
- `doctor`: prerequisite checks with spinners and inline remediation.

### Phase 5 — Polish

- Guided onboarding (first-run).
- Run-done / waiting-for-input notifications.
- Full accessibility pass: `NO_COLOR`, reduced-motion, colorblind-safe, line-mode
  fallback verified.
- Themes; the lighthouse easter egg (per the brand doc's guardrails).
- Complete snapshot coverage; the architecture doc finalized as the reference
  guide for sibling projects.

## Reference-quality bar (every phase)

- Snapshot-tested with the VT100 backend + `insta` for each screen, at a few
  sizes, with deterministic (injected-clock) animation frames.
- Keyboard-complete with visible hints; no hidden keyboard-only workflow.
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
  network-decision stream.
- Phase 4 (history, diagnostics) needs run records and the egress/auth logs as
  data.

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
- Next: Phase 1 brand complete: high-resolution Cubby image tier, animated pet
  policy, and RunHaven-authored rotating tooltips.
