# RunHaven Ratatui Brand Graphics and TUI Experience Plan

## Goal

Define a terminal UI for RunHaven that goes well past a plain widget grid: a
terminal experience that feels premium and trustworthy, with the RunHaven brand
present without ever getting in the way of the work. The static logo badge is the
floor, not the ceiling. The ambition is a TUI whose defining screen is the
security boundary made visible, with motion and graphics used only where they
communicate real state.

This plan lives in the RunHaven repo at `docs/plans/ratatui-brand-graphics.md`.
It started as a vision document and now records the design target and follow-up
idea bank for the active TUI implementation. The build sequence and completion
status live in `docs/plans/tui-build-plan.md`; this document stays focused on
experience, brand, motion, and guardrails.

Capability facts below (crate versions, terminal support) were verified on
2026-06-26 and are point-in-time. Re-verify against current sources before any
implementation, and treat the support matrix as volatile.

RunHaven runs only on macOS 26+ on Apple silicon (`AGENTS.md`: no Windows or
Linux runtime targets), so its TUI only ever runs in a macOS terminal. Unlike a
cross-platform tool, Windows terminal support is out of scope; the target set is
macOS Terminal.app, iTerm2, Ghostty, Kitty, WezTerm, Alacritty, and Rio, plus
tmux, Zellij, and SSH sessions from those.

## What the TUI Answers at a Glance

Frame the TUI as an operator surface, not a logo with widgets around it. Every
screen exists to answer a specific question without the user reading a wall of
command output:

- Is my Mac ready, and what exact prerequisite is missing? (doctor)
- What agent and folder am I about to run, and is the folder narrow enough?
- What will this run mount and reach, and what is it explicitly not allowed to
  touch? (the security boundary, made visible)
- Is anything running right now, and how much is it using?
- What did the agent change in my workspace?
- Was any network access blocked, and why?
- What is the next safe action?

If a screen does not help answer one of these, it does not earn its place.

Implemented Home direction: the header uses the RunHaven logo for brand
identity and an at-a-glance context panel for the selected agent, workspace,
network mode, default boundary, next safe action, and launch wizard stepper.
Cubby is the compact ambient pet, not the header hero. Logo and pet terminal
image behavior should stay on the Codex-derived overlay and ambient-placement
contracts wherever possible.

Source-first rule: before adding custom TUI behavior, check the local official
Codex TUI source at `/Users/c/Documents/GitHub/codex/codex-rs/tui` and vendor or
adapt the native implementation unless RunHaven's data model, logo asset, or
security boundary requires a local adapter.

## Default Terminal Entry Point

When RunHaven is used interactively from a terminal, the TUI is the default. A
bare `runhaven` on an interactive TTY launches the TUI, so a user can set up,
plan, launch, watch, and recover runs without memorizing the command set. This is
the terminal expression of the product rule that the secure path is the easiest
path: the guided surface, not a flag the user must discover, is what greets them.

The CLI stays complete and is never demoted. The rules that keep automation and
power use first-class:

- Explicit subcommands bypass the TUI. `runhaven run ...`, `runhaven plan ...`,
  `runhaven doctor`, and every other subcommand run exactly as today and print to
  stdout. Knowing the command stays faster than the menu.
- Non-interactive never launches the TUI. If stdout is not a TTY (pipes, scripts,
  CI, `runhaven ... | ...`), RunHaven runs the CLI and the existing printers, so
  no automation changes behavior. A `--no-tui` flag and a `RUNHAVEN_NO_TUI` env
  var also force the CLI path on a TTY.
- Every TUI action maps to a named CLI command shown in the UI (the inspectable
  principle). The TUI teaches the CLI rather than hiding it; a user can graduate
  to the commands at any time, and the command the TUI would run is always
  visible before it runs.
- Opt-out, not lock-in. The TUI is the friendly front door; the CLI is the
  always-open side door.

Timing: this behavior decision applies to the active TUI build. The CLI remains
the terminal interface for subcommands, pipes, redirection, and non-interactive
automation; the bare interactive `runhaven` default changes as the TUI ships.

## Design Principles

RunHaven is a security tool first. The experience principles follow from that.

- Restraint and trust over spectacle. Operational and destructive screens stay
  calm, legible, and static. Delight lives on idle, branding, empty, and
  transition surfaces only.
- The secure path stays the shortest path, including in the UI. A less-secure
  choice should require more deliberate input (for example type-to-confirm), not
  less.
- Motion must mean something. Animation is allowed when it signals a real async
  state (a probe running, a connection pending), and banned as decoration on
  confirmation dialogs or security notices.
- Degrade gracefully and never hard-require a graphics protocol. The guaranteed
  visual floor is half-block rendering plus truecolor, which works almost
  everywhere; richer tiers are enhancements.
- Accessibility is a requirement, not a setting. Honor `NO_COLOR`, ship a
  reduced-motion switch, never encode state in color alone, and provide a
  line-mode fallback for assistive technology.
- Inspectable, never opaque. Every visual summary drills down to the exact path,
  command, host, or log line behind it. The UI never hides the command it would
  run or the path it would touch; the TUI extends the CLI's transparency rather
  than masking it.
- Plain language is the default. User-facing copy, menus, warnings, setup help,
  and docs are written for non-technical users at roughly an 8th grade reading
  level. Keep exact commands, paths, hosts, and safety facts when they matter,
  but explain them with short sentences and clear next actions.
- Built on the shared library, not shelled subcommands. The TUI calls the same
  planner and policy objects as the CLI (the shared planner and policy objects
  tracked in `docs/NON_UI_BACKLOG.md`), so plan, run, egress, and record
  semantics are identical across the CLI, the desktop app, and the TUI. Any
  structured output the TUI needs that the CLI does not yet expose is a CLI gap
  to close, not a place to re-parse text.
- Every dependency earns its place. Apply the `AGENTS.md` build-necessity ladder
  before adding a crate: does a ratatui built-in cover it, does a native
  construct, is it one small widget. Visual flourish alone never justifies a
  dependency or a new transitive C build.

## Source Asset and Brand Expansion

Primary source: `docs/assets/logo.png` (verified present): PNG, 512 x 512,
8-bit/color RGBA, non-interlaced. This is the best first-use asset; no SVG
conversion or normalization is needed before prototyping.

The logo has glassy gradients and a square app-icon composition, so it reads
better as a compact badge or an ambient mark than as a large illustration.
Brand expansion ideas, beyond a fixed badge:

- Reveal on launch: a one-shot logo reveal (dissolve or sweep) that yields to the
  UI in under a second and never loops.
- Breathing mark: a very subtle pulse or parallax on the idle dashboard badge,
  disabled under reduced motion, removed entirely from operational screens.
- Sampled brand palette: derive semantic accent colors (a blue and a teal) from
  the logo and use them as the theme's brand tokens, distinct from the danger and
  warning tokens.
- Stacked-square glyph: a two or three layer mark for the fallback tier and for
  inline status, so the brand survives even where images cannot render.

## Rendering Capability Tiers

Pick the highest tier the terminal proves it supports, and fall back cleanly.

Tier order, most to least capable:

1. Kitty graphics protocol: placements, Unicode-placeholder placement, z-index
   layering (negative z draws under text), animation frames, transmit-once and
   display-many, explicit delete-by-id. The terminal does not reflow images on
   scroll or resize, so a TUI re-emits or deletes placements per change.
2. iTerm2 inline images: OSC 1337 base64 PNG/GIF. Simple, no layering or
   animation control. Good fallback.
3. Sixel: DEC bitmap, wider but older support, palette-limited, no layering.
4. Unicode block rendering: octant (2x4, Unicode 16, newest, weakest font
   coverage), sextant (2x3), quadrant (2x2).
5. Half-block (2x1, two independently colored truecolor pixels per cell): the
   guaranteed floor for color imagery.
6. Braille (2x4, highest geometric density but one color per cell): best for line
   plots, not color images.
7. ASCII or solid block: universal last resort.

Terminal support matrix for raster graphics and truecolor (observed 2026-06-26,
volatile, verify before relying):

| Terminal         | Kitty gfx | Sixel    | iTerm2 img | TrueColor |
| ---------------- | --------- | -------- | ---------- | --------- |
| Kitty            | Yes       | No       | No         | Yes       |
| Ghostty          | Yes       | Unverified | No       | Yes       |
| WezTerm          | Yes       | Yes (exp.) | Yes      | Yes       |
| iTerm2           | No        | Yes      | Yes        | Yes       |
| Rio              | Yes       | No       | Yes        | Yes       |
| Alacritty        | No        | No       | No         | Yes       |
| macOS Terminal   | No        | No       | No         | partial   |

Notes: WezTerm is the only emulator supporting all three raster protocols.
Ghostty sixel and iTerm2 sixel had conflicting sources and need a live check.
macOS Terminal.app lacks graphics protocols and full truecolor, so the half-block
plus 256-color path must look good there because it is a primary host on macOS.

Capability detection (the practical sequence):

1. Environment guess first (`TERM`, `TERM_PROGRAM`, `KITTY_WINDOW_ID`).
2. DA1 (`CSI c`): attribute 4 indicates sixel.
3. Kitty graphics query (an `APC _G` probe) and wait for an `OK`.
4. Font cell pixel size query, needed to size graphics; half-block works even
   when this fails.

RunHaven previously tested `ratatui-image` for this tier, but it selected
iTerm2's OSC 1337 path and rendered blank in the full-screen TUI. The active
path is Codex source-first: use the local official Codex TUI graphics protocol
code and overlay lifecycle from `/Users/c/Documents/GitHub/codex/codex-rs/tui`,
with half-block rendering as the portable floor.

tmux and SSH caveats: tmux needs `set -g allow-passthrough on` (off by default);
sixel passes through tmux 3.4 or newer; Kitty graphics over tmux is fragile and
best paired with Unicode-placeholder placement. Over SSH the escape codes tunnel
fine but latency hurts animation. Under tmux or SSH, assume graphics may silently
fail and keep the half-block tier fully functional.

## Crate Ecosystem

Verified on 2026-06-26 (versions and status are volatile):

| Crate                  | Version  | Status            | Use                                                        |
| ---------------------- | -------- | ----------------- | ---------------------------------------------------------- |
| ratatui                | 0.30.x   | active, core      | Core framework; modularized core/widgets, `ratatui::run()` |
| ratatui-image          | 11.0.6   | retired for RunHaven | Evaluated and reverted after blank iTerm2 full-screen rendering |
| tachyonfx              | 0.25.0   | active (rat org)  | 50+ shader-like effects: dissolves, fades, transitions     |
| tui-big-text           | 0.8.8    | active            | Oversized headline text; Full/Half/Quadrant pixel sizes    |
| throbber-widgets-tui   | 0.11.1   | active            | Spinners and loading throbbers                             |
| ansi-to-tui            | 8.0.1    | active            | Convert raw ANSI agent output into ratatui `Text`          |
| tui-popup              | current  | active            | Centered, scrollable popups and modals                     |
| tui-scrollview         | current  | active            | Scrollable content container                               |
| tui-rain               | 1.0.1    | stale (2024)      | Matrix-rain effect; usable but unmaintained                |
| Canvas/Sparkline/Chart | built-in | active            | Braille and block plots, shapes, meters                    |

There is no canonical toast or gradient crate. Build toasts from `tui-popup` plus
a timer, and do truecolor gradient interpolation by hand or via tachyonfx color
effects. `tui-widgets` also bundles bar-graph, cards, qrcode, and prompts widgets
worth a look.

Supply-chain note: prefer pure-Rust crates and avoid pulling new C builds (see
Engineering Discipline). For raster graphics, prefer the already-vendored Codex
protocol and Sixel code with attribution instead of adding a new graphics crate.
Add new UI crates only when a built-in ratatui primitive or vendored Codex
module cannot cover the behavior. Evaluate `tui-tree-widget` or `tui-nodes` only
if the run history or trust boundary view needs real tree or graph interaction
beyond the built-in `Canvas`.

## Motion and Effects

- Frame loop: render on a tick driven by an event-or-timer select, compute delta
  time from the last frame, and advance effects by that delta (the tachyonfx
  model). Ratatui's diff renderer repaints only changed cells, so animating a
  small region is cheap.
- Easing and transitions: tachyonfx ships interpolations, dissolves, sweeps,
  color shifts, glitch, and spatial timing patterns (radial, diagonal,
  checkerboard). Use ease-in-out by default.
- Effects to reach for: typewriter and reveal, progress shimmer, screen-to-screen
  transitions, spinners (throbber), and hand-rolled particle sparks for a
  completion moment.
- Frame budget: terminals realistically sustain roughly 30 to 60 FPS for small
  changes. Full-screen truecolor repaints every frame cause flicker and high CPU,
  worse over SSH and tmux. Cap the frame rate, mark only dirty regions, and never
  busy-loop.

## High-Resolution Techniques Without a Graphics Protocol

`ratatui`'s `Marker` enum exposes these; resolution is sub-cells per character:

| Technique     | Resolution | Tradeoff                                             |
| ------------- | ---------- | --------------------------------------------------- |
| Block or Dot  | 1x1        | Universal, lowest fidelity                          |
| Half-block    | 1x2        | Two independent truecolors per cell; best for color |
| Quadrant      | 2x2        | Very portable                                       |
| Sextant       | 2x3        | Unicode 13                                          |
| Octant        | 2x4        | Unicode 16, dense, newest, weakest font coverage    |
| Braille       | 2x4        | Highest density but one color per cell; line plots  |
| Dither + 24bit| varies     | Floyd-Steinberg over half-blocks approximates photos |

For colored imagery, half-block wins (two true colors per cell). For monochrome
line plots, braille wins. Nerd Font glyphs add icons but depend on the user's
font, so gate them behind detection and never assume.

## Signature Moments

Five hero ideas that could define the RunHaven TUI identity:

1. Reveal-to-readiness boot. The logo reveal dissolves directly into the live
   doctor prerequisite scan, so branding is the first useful screen rather than a
   gratuitous splash.
2. The egress ledger. A calm, streaming trust feed on the run dashboard: every
   network decision rendered in real time, allowed in a muted token, blocked in
   the danger token. This makes the security product visible and is RunHaven's
   defining screen.
3. The lifecycle mark. A restrained, Codex-pets-inspired corner mark that mirrors
   container state (booting, running, done) only on safe and idle screens, and
   vanishes on any confirmation or destructive surface. Delight that knows its
   place.
4. Run-as-node history graph. A serie-style lineage where each agent run is a
   commit-graph node showing branch and worktree relationships; Enter dives into
   that run's worktree diff.
5. Assemble-then-fire plan menu. A gitu-style transient where security flags are
   toggled and seen before a single deliberate confirm. The secure default sits
   one keystroke away; the less-secure choice demands type-to-confirm.
6. Trust boundary lens. A toggle that labels every surface a run touches:
   workspace mounted read-write or read-only, the state volume, host home and
   credentials explicitly not mounted, reachable provider hosts, blocked hosts,
   and where each provider credential lives (host-side broker versus passed into
   the guest). It makes the security model legible at a glance and is the
   inspectable companion to the egress ledger. Borrowed from Persona's trust
   boundary lens, narrowed to RunHaven's container boundary.

## Surface-by-Surface Design

Each RunHaven surface, with premium ideas drawn from named tools.

- Guided setup and doctor: an impala-style per-probe spinner that resolves to a
  pass or fail mark; a failed check expands inline with glow-rendered remediation
  text; the boot splash hosts the scan.
- Agent and folder selection: a yazi-style three-pane layout (agents, folders,
  live preview of the selected repo with git status and a last-run badge), with
  async neighbor preloading.
- Run-plan review: a gitu transient menu that assembles network mode, mounts, and
  egress flags; a jless-foldable view of the resolved config; security notices in
  a bordered, static callout.
- Confirmed launch: a single modal naming the exact mounts, network mode, and
  egress posture, with type-to-confirm required only for less-secure choices.
- Live run dashboard: btop-style sparklines for CPU, memory, and network; a
  network-mode badge in the status line; the egress ledger streaming below; the
  lifecycle mark in a corner.
- Bounded log viewer: atuin-style search with smart tailing that pauses when the
  user scrolls up and shows a "N new lines" pill; a severity gutter; `ansi-to-tui`
  to preserve agent color output.
- Run history: a serie-style run graph with worktree lineage; Enter opens a
  lazygit or gitu style diff of that run's changes.
- Stop, kill, repair: a lazygit-style context menu of only the valid actions for
  the current container state, with a plain gpg-tui-style confirm modal and no
  motion.
- Provider egress and auth diagnostics: a slumber-style request tree leading to a
  response and headers detail; per-provider auth status with a copyable fix.
- Environment and capability panel: the negotiated render tier plus a terminal
  capability probe (truecolor, graphics protocol result, unicode width behavior,
  terminal size, tmux or Zellij or SSH caveat) shown in a visible panel, so a
  user or maintainer can debug rendering. This mirrors `runhaven doctor`'s
  transparency and is the rendering equivalent of it. Borrowed from Persona's
  capability probe and Environment tab.
- Run receipt: after a run, a copyable, reproducible summary (run id, agent,
  workspace, network mode, egress allowed and blocked counts, git change summary,
  logs path, and the exact CLI command that reproduces the run), exportable as
  text or Markdown. More useful than a decorative success screen, and it builds
  on the existing run records. Borrowed from Persona's receipt mode.

## Information Architecture

Concrete structure so the surfaces above compose into one app.

Tabs:

- `Overview`: live run dashboard and the next safe action.
- `Setup`: doctor prerequisites and exact fixes.
- `Plan`: agent and folder selection, run-plan review, confirmed launch.
- `Runs`: active runs plus the run-history graph and per-run diffs.
- `Diagnostics`: the egress ledger, provider auth, and the trust boundary lens.
- `Environment`: the negotiated render tier and the terminal capability probe.

Keyboard, with mouse optional but keyboard always complete:

- `Tab` and `Shift+Tab` move between tabs; arrows navigate lists.
- `Enter` opens details or runs the selected safe action.
- `/` opens a command palette; `?` opens a help overlay; `q` quits.

A persistent status line carries the run context: container name, network-mode
badge, blocked-egress count, and the active keymap hint. Every action is
discoverable on screen; there is no hidden keyboard-only workflow, and every
mutating or destructive action routes through an explicit confirm. Borrowed from
Persona's information architecture, narrowed to RunHaven's surfaces.

Codex-source candidates to evaluate before custom implementation: status line
and terminal-title surfaces, `/status` cards, syntax/highlighting themes,
keymap/accessibility picker behavior, thread naming, resume/session picker
patterns, slash status routing, and tooltip timing/suppression.

## Restraint and Trust

- Delight allowed: idle dashboard, first-run and empty states, the boot splash,
  a successful-completion moment, and the lifecycle mark during a running state.
- Calm required: every mutating or destructive action (kill, repair,
  allow-egress, mounting host paths), all security notices, doctor failures, and
  auth errors. Static layout, semantic danger color, plain language, explicit
  confirm.
- Anti-patterns to ban: spinners or animation on confirmation dialogs (they imply
  progress and mask risk); motion that softens a security tradeoff; the lifecycle
  mark reacting on a destructive screen; decorative color that collides with the
  danger hue; auto-dismiss toasts for anything irreversible; any path where the
  secure choice is harder than the insecure one.

## Easter Egg: Zork I

The desired signature easter egg remains a hidden Home-only `~` screen that
runs the original Zork I story in the TUI. It is nerdy, terminal-native, and
more substantial than a decorative animation while staying outside RunHaven's
runtime boundary.

Current reset state:

- Game data: `third_party/zork1/` vendors the MIT-licensed
  `historicalsource/zork1` collection, including the compiled Z-machine story.
- Engine: the earlier Ferrif-derived TUI engine was removed with the discarded
  custom TUI and is recoverable from git history. If restored, it should live
  under `crates/runhaven-tui/src/tui/zork/` and remain TUI-local.
- Save/restore: Zork `save` and `restore` should use one private RunHaven cache
  slot by default, with any user-selected disk load path treated as untrusted
  input and validated before parsing.
- Attribution: `THIRD_PARTY_NOTICES.md` and `licenses/zork1-MIT.txt` already
  record the story-source attribution. Reintroducing a third-party engine needs
  matching attribution.

Security boundary:

- No new Cargo dependencies.
- No Apple `container` calls, subprocesses, network sockets, workspace reads,
  provider credential reads, or arbitrary file paths.
- The bundled story is checked by exact byte length and SHA-256 before VM start.
- Restore reads only the fixed RunHaven cache file and rejects oversized,
  non-Quetzal, unknown-chunk, duplicate-chunk, truncated-stack, or incomplete
  save data before the vendored parser sees it.
- The VM restore path still checks story release, serial, and checksum before
  accepting a save.

Other ideas worth keeping in the bank:

- Lifecycle mark pokes: press a key on a safe or idle screen and the corner mark
  reacts (a wave, a slow blink, a "zzz" after long idle, a small hop after a
  successful run). Emergent, low-cost personality on the Codex-pets contract.
- Clean-run streak reward: after several consecutive runs with no blocked egress
  and no errors, the run-complete screen earns a brief one-shot flourish and a
  hidden "smooth sailing" note. This makes the secure, well-behaved path the one
  that feels good, reinforcing the product thesis rather than fighting it.
- Hidden theme: a key sequence unlocks a "midnight harbor" or phosphor-CRT theme
  for the idle and branding surfaces only.
- Maiden voyage: the very first successful run ever shows a one-time, slightly
  larger reveal of a small boat reaching harbor, then never again.

Guardrails (firm; an easter egg that breaks one of these is a bug):

- Only on idle, splash, empty, and successful-completion surfaces. Never on
  setup, plan review, confirmation, run control, security notices, or any
  mutating or destructive screen.
- Never hide or delay information, a command, a path, or a security decision.
- Never trivialize the security boundary. There is no joke at the expense of a
  blocked host, a refused mount, or a kill confirmation; the wink is about the
  brand, never about a risk the user is taking.
- Honor reduced motion and `NO_COLOR`: the egg degrades to a static, quiet form
  or disappears, and an env switch (for example `RUNHAVEN_NO_EASTER_EGGS`) turns
  it off entirely.
- Deterministic and isolated: the trigger and any animation run off the injected
  clock so the snapshot harness can render the egg frame on demand. It pulls no
  new dependency and cannot touch a run.
- Cheap and findable: RunHaven-owned glue stays small and dependency-free;
  vendored material must be attributed, fenced to the easter-egg module, and
  recorded in product docs so the community can discover it without reverse
  engineering.

## Accessibility

- Honor `NO_COLOR` (disable all color) and ship a reduced-motion switch
  (`--no-animation` or an env var); terminals have no reduced-motion signal, so
  RunHaven must provide one.
- Use colorblind-safe palettes and never encode state in color alone; always pair
  a hue with a glyph or label.
- Provide a line-mode, non-fullscreen fallback for assistive technology.
  Full-screen alternate-buffer apps that move the cursor break VoiceOver and NVDA;
  a documented Claude Code VoiceOver regression in Terminal.app is the cautionary
  case, and its line-mode toggle is the lesson to copy.

## Feasibility Tiers

Feasible now, cross-terminal because half-block is the floor: half-block,
braille, and octant rendering; truecolor gradients; tachyonfx transitions;
big-text headers; throbbers; `ansi-to-tui` log panes; popups and scroll views;
charts and sparklines; Codex-derived graphics detection/overlays; `NO_COLOR`
and reduced-motion.

Aspirational or conditional, gorgeous where supported but never a baseline:
Kitty-protocol inline images, animation frames, and z-layering (great on
Ghostty, Kitty, WezTerm; unreliable under tmux and Zellij; absent on Alacritty
and macOS Terminal). Smooth full-screen effects over SSH should not be
promised.

## Implementation Path

Start with the universal floor, then layer enhancements behind detection.

```text
src/tui/
  brand/
    mod.rs       # logo asset adapter, fallback stacked-square mark
  render/
    image.rs     # Codex-derived graphics-protocol path (kitty/iterm2/sixel)
    blocks.rs    # half-block, braille, octant fallback renderers
  motion/
    effects.rs   # tachyonfx wiring, transitions, reduced-motion gate
  theme.rs       # semantic tokens: brand, danger, warn, safe, muted
```

Sequence:

1. Theme tokens and the half-block plus truecolor floor, working on macOS
   Terminal and Alacritty.
2. The static logo through the Codex-derived terminal image overlay with the
   stacked-square fallback.
3. The reveal-to-readiness boot and the doctor scan, the first signature moment.
4. The egress ledger on the dashboard, the defining screen.
5. Motion polish via tachyonfx, gated behind reduced-motion, on safe screens
   only.
6. Kitty-protocol enhancements as a top tier, never a requirement.

The active implementation now uses a Codex-style ambient overlay renderer: render
UI, save cursor, place image, restore cursor, and delete/clear on resize. Keep
future changes source-first against the local Codex TUI before adding custom
overlay logic.

## Engineering Discipline

Borrowed largely from the Tamworth plan and adapted to RunHaven. These are what
let an ambitious TUI be built and refactored safely rather than become a fragile
demo.

- Deterministic by construction. Drive every animation from an injected logical
  clock (a frame counter or accumulated `Duration`), never `Instant::now()`
  inside draw code, so frame N renders identically every run. Build the snapshot
  harness first: ratatui's `TestBackend` renders into an in-memory buffer with no
  real terminal or timing, and `insta` snapshots it at several sizes and chosen
  animation frames. This turns a flashy TUI into one we can change without fear,
  and the final-frame path doubles as the reduced-motion baseline. The half-block
  brand renderer is the golden snapshot; pixel-protocol tiers are excluded
  because they write out-of-band escapes the buffer never sees.
- Pure-Rust and supply-chain bounded. Keep widgets, layout, animation, and tests
  in safe Rust; quarantine the unavoidable platform `unsafe` in the terminal
  backend (`crossterm`). Prefer a pure-Rust Sixel encoder (`icy_sixel` or
  `sixel-image`), never `libsixel` or `sixel-sys` (C). Add a CI gate
  (`cargo tree -e normal`) that fails on any new `*-sys`, `cc`, or `cmake`
  dependency, so the no-new-C claim is enforced, not asserted. This matches
  RunHaven's exact-pin, minimal-dependency, lock-transitive posture.
- Resource bounded. The model holds the full set; widgets draw only the visible
  viewport (O(visible), not O(all)). Streaming history, the log pane and the
  egress ledger, lives in fixed-size ring buffers so TUI memory never grows with
  run length or log volume.
- Panic isolated. The TUI hosts the actual container run, so a render panic must
  never abort or corrupt the run, its cleanup, or its record. Isolate the render
  thread from the run orchestration, and on a rendering failure degrade to the
  plain CLI output rather than taking the run down with it.
- Worker-to-channel architecture. Run, egress, and log events flow over a channel
  from the orchestration into the UI thread, which drains per tick and rebuilds
  the frame from the model (ratatui is immediate mode and diffs cells, so
  frequent redraws are cheap). Idle means no redraw.
- Correct columns. Use `unicode-width` for wide-character and emoji column math
  so layout, carets, and truncation never misalign.

## Verification

- Render and tier-select correctly across Ghostty, Kitty, WezTerm, iTerm2, and
  macOS Terminal, with the half-block floor verified on Terminal.app and
  Alacritty.
- Verify the fallback path in a terminal with no graphics protocol.
- Verify resize and scroll behavior, including image deletion and re-placement.
- Verify no overlap of brand or motion with tables, forms, security notices, or
  command output.
- Verify the 256-color half-block path on macOS Terminal.app, which lacks
  truecolor and graphics protocols, and force cell rendering under tmux and
  Zellij.
- Verify `NO_COLOR` and the reduced-motion switch fully disable color and motion.
- Verify the line-mode fallback is usable under VoiceOver or NVDA.
- Snapshot-test the fallback widgets and the theme token resolution.
- Confirm no animation appears on any mutating, destructive, or security screen.

## References

Terminal graphics protocols:

- Kitty graphics protocol: https://sw.kovidgoyal.net/kitty/graphics-protocol/
- Terminal graphics protocol overview: https://akmatori.com/blog/terminal-graphics-protocols
- Terminal compatibility matrix: https://tmuxai.dev/terminal-compatibility/

Ratatui ecosystem:

- ratatui v0.30 highlights: https://ratatui.rs/highlights/v030/
- ratatui-image: https://github.com/ratatui/ratatui-image
- tachyonfx: https://github.com/ratatui/tachyonfx
- tui-big-text: https://crates.io/crates/tui-big-text
- throbber-widgets-tui: https://crates.io/crates/throbber-widgets-tui
- ansi-to-tui: https://crates.io/crates/ansi-to-tui
- tui-widgets (popup, scrollview, and more): https://github.com/ratatui/tui-widgets
- ratatui Canvas: https://docs.rs/ratatui/latest/ratatui/widgets/canvas/struct.Canvas.html
- ratatui Marker (block resolution): https://docs.rs/ratatui/latest/ratatui/symbols/enum.Marker.html

Inspiring TUIs to model:

- yazi (async preview panes): https://github.com/yazi-rs/yazi
- lazygit (context action menus): https://github.com/jesseduffield/lazygit
- gitu (transient assemble-then-fire menus): https://github.com/altsem/gitu
- serie (graphics-protocol git graph): https://github.com/lusingander/serie
- atuin (inline search with metadata): https://github.com/atuinsh/atuin
- television (live async preview): https://github.com/alexpasmantier/television
- bottom (dense live dashboard): https://github.com/ClementTsang/bottom
- slumber (request tree and detail): https://github.com/LucasPickering/slumber
- jless (foldable data navigation): https://github.com/PaulJuliusMartinez/jless
- glow (calm markdown rendering): https://github.com/charmbracelet/glow
- gpg-tui (security-tool restraint): https://github.com/orhun/gpg-tui
- impala (animation as real status): https://github.com/pythops/impala
- Codex TUI and pets (ambient status mark): https://github.com/openai/codex/tree/main/codex-rs/tui

Sibling RunHaven-family plans this draws from:

- Tamworth brand and live-scan TUI plan (determinism, pure-Rust supply-chain
  gate, snapshot-first harness, panic isolation):
  `/Users/c/Documents/GitHub/Tamworth/docs/plans/tamworth-ratatui-brand-graphics-plan.md`
- Persona ratatui experience plan (command-center framing, capability panel,
  receipt mode, trust boundary lens, information architecture):
  `/Users/c/Documents/GitHub/persona/docs/plans/persona-ratatui-brand-graphics-plan.md`

## Recommendation

Keep the half-block plus truecolor floor universal so every terminal, including
macOS Terminal.app, looks deliberate. Treat graphics protocols as an enhancement,
never a requirement. Lead the identity with two moments: reveal-to-readiness boot
and the live egress ledger. The first makes the brand the first useful screen;
the second turns RunHaven's security model into the screen people remember. Hold
all motion to the rule that it must communicate real state, and keep every
mutating and security surface calm. Defer custom image protocols, the ambient
mark, and full animation until the floor, the theme, and those two signature
moments are solid. Build the deterministic snapshot harness before any of it so
the TUI stays refactorable, keep the dependency surface pure-Rust with no new C
build, and treat the trust boundary lens and the run receipt as the inspectable
backbone that keeps the experience honest. The brand is the entry point; the
security boundary made legible is the product.
