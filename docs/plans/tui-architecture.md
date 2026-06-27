# TUI Architecture Patterns

Reference guidance for the RunHaven terminal UI (`crates/runhaven-tui/src/tui/` and its
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

## Product Contract Seam

RunHaven should use the same high-level move that made dbt-wizard's Codex TUI
integration production-ready: domain truth becomes a stable product payload,
then terminal and desktop renderers draw that payload.

For RunHaven, the first seam is:

```text
RunHaven planner/state/records
  -> crates/runhaven-core/src/ui_contracts.rs
  -> Ratatui widgets in crates/runhaven-tui/src/tui/
  -> optional Tauri or React renderers from the same payloads
```

The contract layer is RunHaven-owned and presentation-neutral. It must not
depend on Ratatui, Codex protocol types, Codex app-server types, or CLI prose
parsing. It may expose only data the CLI already treats as safe to show, such
as the selected agent, workspace path, state volume name, network posture,
provider hosts, safety notes, and the exact command the TUI will run.

Start with `LaunchPlanData` built from `AgentRunPlan`, then add active-run,
history, diagnostics, and diff payloads as those screens are reattached to the
vendored shell.

## Codex Source First

The TUI implementation vendors first from the official local Codex source at
`/Users/c/Documents/GitHub/codex/codex-rs/tui`. Before adding custom TUI code,
check that source for an equivalent behavior and vendor or adapt it with
attribution. This is mandatory for pet behavior, animation timing, ambient
placement, terminal image protocols, welcome/header layout patterns, terminal
wrapping/truncation, selection lists, clipboard behavior, and other generic TUI
primitives.

RunHaven-owned code is limited to data adaptation and product identity: mapping
RunHaven profiles, plans, records, diagnostics, and security-boundary state into
the vendored widgets; swapping the RunHaven logo asset; and small glue where
Codex has no equivalent. Each exception should be documented here or in
`docs/plans/tui-build-plan.md`.

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

The pre-reset custom TUI split remains useful design history, but it is not the
active source shape during the Codex vendor reset. The active source under
`crates/runhaven-tui/src/tui/` is a Codex source snapshot plus a staged `mod.rs`
adapter that keeps the crate buildable while integration proceeds.

Target ownership for the rebuilt source remains:

| Module | Ownership |
| --- | --- |
| `mod.rs` | Temporary RunHaven entrypoint during vendor integration; replace staged contracts with adapted Codex app-shell pieces as they come online. |
| `app_shell.rs` | Temporary read-only launch preview over `LaunchPlanData`; remove or shrink when the full Codex app shell is adapted. |
| `ui_contracts.rs` | Presentation-neutral RunHaven payloads shared by TUI widgets and any future desktop renderer. |
| `input.rs` | Keyboard navigation and action routing. Keep key behavior testable here instead of scattering it through draw code. |
| `theme.rs`, `color.rs`, `event_loop.rs` | Domain-agnostic settings, palettes, color math, and deterministic tick timing. |
| `widgets.rs`, `tooltips.rs` | Shared draw helpers and RunHaven-authored footer tips. Widgets draw data; they do not query the domain. |
| `launcher.rs` | Workspace picker, plan review, confirm state, and launch-plan construction over the shared planner. |
| `runs.rs`, `run_views.rs` | Active-run state, egress/log/control adapters, dashboard notices, and dashboard/log/control rendering. |
| `history.rs`, `history_views.rs` | Run history, diff review, diagnostics, terminal capability, doctor state, and their views. |
| `guide_views.rs` | First-run and help guide. It routes users to existing workflows; it does not own product logic. |
| `brand.rs`, `pet.rs`, `codex/` | RunHaven logo/Cubby asset adapters over attributed Codex-derived welcome, pet, and terminal graphics primitives. |
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

## User-facing language

Write TUI text, menus, warnings, setup help, and docs for non-technical users at
roughly an 8th grade reading level. Use short sentences, plain verbs, concrete
nouns, and one clear next action. Keep exact commands, paths, hosts, and safety
facts when they matter, but explain why they matter in plain language.
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
- Keep the logo and ambient pet compact and identity-oriented. They should help
  recognition, not push the workflow below the fold.
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

The brand graphics, startup chrome, and hidden Zork easter egg (see
`ratatui-brand-graphics.md`) solve a different problem than the functional cards.
They share design direction but not data plumbing; keep them in separate modules.

The active Home header uses `brand.rs` to load the RunHaven logo from
`docs/assets/logo.png`, render a terminal-safe half-block fallback, and emit the
same Codex-derived terminal image overlay used by pets when the terminal
supports it. Cubby is not the header hero; `pet.rs` adapts the validated Cubby
Codex pet package into Codex's ambient pet placement and overlay contract.
RunHaven supplies the available pane and the Cubby asset, while
`codex/ambient.rs` owns the target size, composer gap, right anchor, clear area,
and image protocol lifecycle.

Legacy terminal-mascot assets remain under `docs/assets/terminal-mascot/` as
historical QA/source evidence from the earlier Cubby hero experiment. They are
not active TUI code.

Source-first candidates to evaluate before adding custom TUI behavior:

- `chatwidget/status_surfaces.rs`: status line and terminal-title model.
- `status/card.rs`: `/status` card structure.
- `theme_picker.rs` and `render/highlight`: syntax/highlighting themes.
- `keymap.rs` and `chatwidget/keymap_picker.rs`: shortcut/accessibility model.
- `chatwidget/session_flow.rs`: thread naming.
- `session_archive_commands.rs` and `resume_picker.rs`: session resume flows.
- `chatwidget/slash_dispatch.rs`: `/status` command routing.
- `terminal_title.rs`: terminal title cleanup and updates.
- `tooltips.rs`: richer tooltip/announcement timing and suppression.

The Zork easter egg lives under `tui/zork/`. Its RunHaven-owned wrapper handles
the screen state, keyboard input, save-file boundary, bundled-story hash check,
and Quetzal validation. The vendored Ferrif-derived engine stays isolated under
`tui/zork/zmachine/` and is used only by this hidden screen. It must not call the
runtime planner, Apple `container`, provider/auth code, subprocess APIs, network
APIs, workspace paths, or arbitrary save-file paths.

## Parity and tests

For each card or screen, keep a fixture and a test that renders it with
`TestBackend` without panicking, and assert the data mapping. The current VT100
snapshot set covers the guide, home, detail, workspace, plan, confirm,
dashboard, logs, control, history, history detail, diagnostics, and doctor
screens, plus the hidden Zork screen. Keep snapshots deterministic: inject
settings, workspace paths, records, and tick state instead of depending on local
machine state.
