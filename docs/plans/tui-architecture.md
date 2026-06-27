# TUI Architecture Patterns

Design guidance for the RunHaven terminal UI (`src/runhaven/cli/tui/` and its
submodules), drawn from studying the Codex `ratatui` TUI and adapting its
component approach to RunHaven's launcher and manager domain.

RunHaven's TUI is not an agent chat; the agent's own chat runs inside the
container. The TUI renders RunHaven's own data: profiles, run plans, egress
policy, and run status. These patterns keep that rendering clean as it grows.

## Single source of truth

The data model lives once, in RunHaven's existing planner and policy objects
(`profiles`, `RunOptions` / `AgentRunPlan`, the egress policy, run records). The
TUI never re-derives or duplicates that logic; widgets are pure functions of
that data. This is already why the agent detail screen reuses
`agent_sign_in` / `agent_broker` and `default_network_mode` instead of restating
them.

## Adapters build, widgets draw

Keep the layers separate:

- planner and policy code build the data (a plan, a status, a profile),
- the TUI passes that data to a widget,
- the widget only draws it.

No container calls, planning, or policy decisions inside a widget. That keeps
widgets pure and testable with `TestBackend` (render every screen without
panic), which the current tests already do.

## Cards

Render structured data as self-contained "cards" in two shapes:

- Fixed-size cards (constant width and height) for content that should stay
  stable in scrollback or a fixed pane, for example an agent summary.
- Variable-height, width-aware cards with `desired_height(data, width)` and
  `draw(area, data)` for content that grows, for example a run plan with a
  variable number of egress hosts or security notices.

Bound every list: cap the number of rows shown (with a "+N more" affordance)
rather than rendering unbounded content.

## Shared draw helpers

As screens multiply, factor small terminal helpers into one place (a
`tui/widgets` or `tui/layout` module): a cell or line writer, a divider, and a
pad-or-truncate that clips to the available width. The existing shared three-row
`layout()` helper is the start of this.

## Palette and color mode

When theming arrives, put it in one `tui` palette module: a `ColorMode`
(Dark or Light) detected from the terminal background, a `Palette`, status
colors, and progress thresholds. Honor the mode you detect; a `ColorMode::Light`
that returns the dark palette is a bug, not a feature.

## The TUI and the desktop app share data, not duplicated logic

RunHaven also has a Tauri and Svelte desktop app. Both surfaces should render the
same underlying data (plans, status, profiles) from the same Rust source of
truth, never two divergent models. If a structured payload is ever exchanged
between surfaces, use one general component seam, not a bespoke message per card.
If visual tokens are ever shared between the TUI and the web UI, generate both
from one source; hand-synced tokens drift.

## Branding stays separate from functional cards

The brand graphics, startup chrome, and the mascot easter egg (see
`ratatui-brand-graphics.md`) solve a different problem than the functional cards.
They share design direction but not data plumbing; keep them in separate modules.

This lives in `tui/mascot.rs` (renderer) plus `tui/mascot/sprites.rs` (generated
pixel data): the mascot is **Cubby**, a glass container cube with a tiny gold
agent spark inside, drawn as half-block pixel art (the guaranteed-portable
rendering floor, no image protocol). The sprites are xterm-256 indexed (indices
16-255, avoiding 0-15 so macOS Terminal.app stays stable) at several sizes;
`hero_for_banner` shows the largest one that fits the terminal, so detail scales
up on bigger windows and degrades cleanly on an 80x24 floor. The source renders
are in `docs/assets/terminal-mascot/` and the 1024px master is
`docs/assets/cubby-hero-1024.png`. It is pure branding with no
data plumbing, the static counterpart to the animated pet (the lifecycle mark in
`ratatui-brand-graphics.md`).

## Parity and tests

For each card, keep a fixture and a test that renders it with `TestBackend`
without panicking, and assert the data mapping. This is cheap and catches layout
regressions and bad data handling early.
