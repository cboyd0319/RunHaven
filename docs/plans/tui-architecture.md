# TUI Architecture Patterns

Reference guidance for the RunHaven terminal UI (`src/runhaven/cli/tui/` and its
submodules), drawn from studying the Codex `ratatui` TUI and adapting its
component approach to RunHaven's launcher and manager domain.

RunHaven's TUI is not an agent chat; the agent's own chat runs inside the
container. The TUI renders RunHaven's own data: profiles, run plans, run records,
egress policy, auth broker metadata, doctor checks, and active-run status. These
patterns keep that rendering clean as it grows and make the implementation a
reference for sibling projects.

## Single source of truth

The data model lives once, in RunHaven's existing planner and policy objects
(`profiles`, `RunOptions` / `AgentRunPlan`, the egress policy, diagnostics, and
run records). The TUI never re-derives or duplicates that logic; widgets are
pure functions of that data. This is already why the agent detail screen reuses
auth posture labels from `provider/auth_profiles.rs` and `default_network_mode`
instead of restating them.

## Adapters build, widgets draw

Keep the layers separate:

- planner and policy code build the data (a plan, a status, a profile),
- the TUI passes that data to a widget,
- the widget only draws it.

No container calls, planning, or policy decisions inside a widget. That keeps
widgets pure and testable with `TestBackend` (render every screen without
panic), which the current tests already do.

Shared data needed by the TUI belongs in presentation-neutral modules before a
screen consumes it. Examples: host readiness in `doctor.rs`, secret-free
diagnostics in `diagnostics.rs`, run records in `records/`, auth posture labels
in `provider/auth_profiles.rs`, and active-run control in `runtime/active/`.
Do not parse CLI prose or import shared data from `cli/app.rs`.

## Current module map

The current split is the reference shape:

| Module | Ownership |
| --- | --- |
| `mod.rs` | TUI entrypoint, app state, render dispatch, shared home/detail rendering, terminal overlay lifecycle. |
| `input.rs` | Keyboard navigation and action routing. Keep key behavior testable here instead of scattering it through draw code. |
| `theme.rs`, `color.rs`, `event_loop.rs` | Domain-agnostic settings, palettes, color math, and deterministic tick timing. |
| `widgets.rs`, `tooltips.rs` | Shared draw helpers and RunHaven-authored footer tips. Widgets draw data; they do not query the domain. |
| `launcher.rs` | Workspace picker, plan review, confirm state, and launch-plan construction over the shared planner. |
| `runs.rs`, `run_views.rs` | Active-run state, egress/log/control adapters, dashboard notices, and dashboard/log/control rendering. |
| `history.rs`, `history_views.rs` | Run history, diff review, diagnostics, terminal capability, doctor state, and their views. |
| `guide_views.rs` | First-run and help guide. It routes users to existing workflows; it does not own product logic. |
| `pet.rs`, `mascot.rs`, `mascot/`, `codex/` | Branding, Cubby pet rendering, and attributed Codex-derived terminal graphics primitives. |
| `snapshot.rs`, `test_backend.rs` | VT100 snapshot harness used by screen regression tests. |

If a new screen needs shared data, add the data API outside `cli/` first. If a
new draw helper has no RunHaven dependency, keep it in the framework modules so
it remains extractable later.

## Wizard and action model

RunHaven launch is a wizard, not a menu tree: choose agent, choose workspace,
review boundary, confirm launch. Keep that stepper visible on launch-path
screens, keep the next safe action visible on Home, and keep broad destinations
in the guide/actions surface.

Rules:

- Show the current task and step before listing actions.
- Keep footer actions local to the current screen. Do not make Home carry every
  global destination.
- Use task labels (`review plan`, `choose workspace`, `open dashboard`) instead
  of vague nouns.
- Group non-launch actions by job in the guide: prepare, run, review, diagnose,
  display.
- Keep destructive run controls inside their own screen with explicit typed
  confirmation.
- Keep `?`/F1 as the discoverable guide route and `q` as the consistent quit.

## Primary user flows

Design screens from flows, not from available commands:

| Flow | Entry | Exit |
| --- | --- | --- |
| Launch | Home or Guide | Confirm restores the terminal and launches through the shared runtime path. |
| Monitor | Home, Guide, or after a launch record exists | Dashboard, bounded logs, or a typed run-control result. |
| Review | Home, Guide, or Dashboard notice | History list and selected run diff. |
| Diagnose | Home, Guide, or History | Diagnostics and doctor checks with inline remediation. |
| Display/accessibility | Guide or environment variables | Cubby visibility, reduced motion, line mode, no-color, light/dark palette. |

When adding a screen, name its flow, entry point, success state, and escape path
before adding key bindings. If a destination does not serve one of these flows,
do not put it in the Home footer. If two flows need the same data, move the data
API outside `cli/` and let each screen render it through its own adapter.

## Agent CLI reference conventions

Stock agent CLIs use a few patterns RunHaven should keep, adapted to its
launcher role:

- Put product identity, version, selected agent, workspace, and ready state near
  the top, not hidden in help.
- Keep the mascot compact and identity-oriented. It should help recognition, not
  push the workflow below the fold.
- Keep the bottom strip for immediate commands and current context.
- Use contextual tips sparingly, and prefer facts the user can act on.
- Do not copy the chat prompt as RunHaven's primary model. RunHaven's primary
  model is launch, monitor, review, and diagnose over the shared runtime data.

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

Theme state lives in `theme.rs`: `TuiSettings`, `ColorMode`, `MotionMode`, and
`Palette`. `NO_COLOR`, `RUNHAVEN_TUI_REDUCED_MOTION=1`,
`RUNHAVEN_TUI_LINE_MODE=1`, `RUNHAVEN_TUI_PET=0`, and
`RUNHAVEN_TUI_COLOR_MODE=light|dark` are the supported environment switches.
Honor the selected mode; a `ColorMode::Light` that returns the dark palette is a
bug, not a feature.

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

For each card or screen, keep a fixture and a test that renders it with
`TestBackend` without panicking, and assert the data mapping. The current VT100
snapshot set covers the guide, home, detail, workspace, plan, confirm,
dashboard, logs, control, history, history detail, diagnostics, and doctor
screens. Keep snapshots deterministic: inject settings, workspace paths, records,
and tick state instead of depending on local machine state.
