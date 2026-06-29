# TUI Codex Vendor Wishlist

Last updated: 2026-06-27

## Goal

RunHaven should use the Codex TUI source as the baseline for its terminal UI,
then shape that baseline into the RunHaven product.

This document is only a wishlist. It records what we want from the Codex TUI
source before we decide what to change, remove, or keep.

The RunHaven TUI setup is the reference implementation for several sibling
projects. Keep the architecture clear enough to reuse: source-first Codex
vendoring, thin RunHaven product adapters, shared data contracts, documented
culling decisions, and user-facing copy that non-technical users can understand.

## Comparison Evidence

The dbt-wizard comparison note at
`/Users/c/Downloads/runhaven-dbt-wizard-codex-tui-direction.md` is useful
evidence, not a product direction to copy wholesale.

The Codex capability map at
[`codex-tui-capabilities.md`](codex-tui-capabilities.md) is the repo-owned
source map for what the Codex TUI already provides. Use it before deciding to
write custom TUI code, especially for the bottom pane, composer, keymaps,
terminal runtime, streaming output, history cells, approvals, sessions, status,
pets, and VT100/snapshot tests.

The lesson to carry forward is:

- Copy the architecture move: domain truth becomes a stable UI payload, then
  shared renderers draw it.
- Do not copy the dbt or Codex chat product shape unless RunHaven intentionally
  becomes a Codex chat fork.
- Keep Codex source as the base for terminal infrastructure: render lifecycle,
  status line patterns, bottom pane behavior, key handling, terminal image
  support, pets, text wrapping, styling, and related generic TUI behavior.
- Keep RunHaven as the product model: profiles, launch plans, active runs,
  run history, diagnostics, egress policy, auth posture, and safety boundaries.
- Build a small RunHaven UI contract seam before wiring product screens into
  the vendored app shell.

That means the next integration layer is presentation-neutral RunHaven payloads,
starting with launch-plan data from `AgentRunPlan`. Ratatui widgets and any
future Tauri/React surface should consume the same payload shape.

## Source

Primary source:

```text
/Users/c/Documents/GitHub/codex/codex-rs/tui/src/
```

The intent is to fully replace the current custom `crates/runhaven-tui/src/tui/` tree
with vendored Codex TUI source, then make RunHaven changes from that baseline.

Local Codex configuration evidence:

```toml
[tui]
status_line = [
  "model-with-reasoning",
  "current-dir",
  "git-branch",
  "run-state",
  "context-remaining",
  "five-hour-limit",
  "weekly-limit",
]
status_line_use_colors = true
pet = "custom:cubby"
```

Observed local custom pet packages:

```text
/Users/c/.codex/pets/cubby/pet.json
/Users/c/.codex/pets/cubby/spritesheet.webp
/Users/c/.codex/pets/ginger/pet.json
/Users/c/.codex/pets/ginger/spritesheet.webp
/Users/c/.codex/pets/prism-pip/pet.json
/Users/c/.codex/pets/prism-pip/spritesheet.webp
```

Each observed spritesheet is a `1536x1872` WebP, matching the Codex custom pet
contract. Treat `/Users/c/.codex/config.toml` and `/Users/c/.codex/pets/` as
local evidence for Codex behavior, not as repo source to copy wholesale. Do not
commit auth files, history, logs, SQLite state, or private user config.

## Desired Foundation

We want the Codex TUI foundation wherever possible:

- app shell and render lifecycle
- bottom pane
- event stream
- frame scheduling
- history cells
- exec cells
- status cells
- status line
- key mapping and help
- onboarding and startup chrome
- notifications
- terminal rendering helpers
- terminal image protocol and ambient pet support
- pets and pet picker
- streaming output handling
- terminal title behavior
- status slash command patterns
- session resume patterns
- light and dark terminal theme handling
- Codex TUI configuration shape for status line and pets

## Desired RunHaven Shape

After the vendored baseline is in place, we want the TUI to feel like
RunHaven:

- RunHaven name, logo, and product language
- Cubby as the default pet
- pets and animation that stay true to Codex source behavior
- native custom pet selection using the Codex `custom:<pet-id>` selector and
  `$CODEX_HOME/pets/<pet-id>/pet.json` package layout
- a guided launch flow for agent, workspace, review, and confirm
- clear plan review before launch
- run dashboard and status
- run history and diff review
- diagnostics for checks, network log, auth log, and terminal support
- stop, hard stop, and repair controls with explicit confirmation
- hidden Zork I easter egg, ideally playable in the TUI if the final design can
  keep it small, attributed, and safely sandboxed
- simple user-facing text for non-technical users at about an 8th grade reading level

## Desired Safety Shape

The vendored TUI must still respect RunHaven's hard product boundary:

- no host home folder mount by default
- no credential folder mount by default
- no raw SSH key mount by default
- no browser profile access by default
- no arbitrary host environment passthrough by default
- secure path remains the easiest path
- lower-security paths stay explicit and warned
- user-loaded files must be reviewed before they become supported behavior

This section does not decide what to remove from Codex. It only states the
RunHaven boundary that any final TUI must keep.

## After Vendoring

After `crates/runhaven-tui/src/tui/` is replaced with the vendored source, review the
vendored baseline against this wishlist.

Then make decisions in this order:

1. What already fits.
2. What needs a small RunHaven tweak.
3. What does not match anything RunHaven wants right now.
4. What needs more design before it is exposed.

The goal is to make these decisions from a full Codex TUI baseline, not from the
current custom RunHaven TUI code.

Because this is the reference implementation for sibling projects, prefer
decisions that leave a clean reusable pattern. If a choice only works for
RunHaven, record why it belongs in the RunHaven adapter instead of the reusable
TUI layer.

## Culling Rule

Removal is not the default. For each removed item, record why removing it is
better than leaving it and adapting it.

Ask these questions before each removal:

1. Would keeping this be less work than rebuilding it later?
2. Can this be renamed, hidden, or adapted safely instead of deleted?
3. Does this help a RunHaven wishlist item now or soon?
4. Does this add a security, privacy, build, test, or user-confusion risk?

If the answer is not clear, keep the vendored code until the integration pass
has better evidence.

## Culling Decisions

### Upstream Codex Snapshot Goldens

Decision: remove copied `*.snap` files under `crates/runhaven-tui/src/tui/`.

Why removal is better than leaving and adapting:

- They are upstream Codex test goldens, not runtime code.
- The copied tests are not yet integrated into RunHaven's crate layout.
- Keeping them would make the vendor commit much larger and harder to review.
- Keeping them would imply those upstream snapshot tests already run in
  RunHaven, which is not true yet.
- If RunHaven keeps a snapshot-tested surface later, regenerate snapshots from
  the integrated RunHaven tests instead of carrying stale upstream goldens.

The old RunHaven custom TUI `*.snap` files are also removed with the custom TUI
tree. They are recoverable from git history if a future integrated RunHaven
snapshot suite needs them as reference material.

### Codex Pet Configuration And Picker

Decision: keep and adapt Codex's native pet configuration, custom pet loading,
pet picker, and ambient rendering path.

Why leaving and adapting is better than removing:

- `/Users/c/.codex/config.toml` already uses `pet = "custom:cubby"` under the
  native `[tui]` table.
- `/Users/c/.codex/pets/cubby/` already matches the Codex custom pet package
  contract and loads through the same picker path as other custom pets.
- Removing this path would force RunHaven to rebuild pet selection, package
  loading, preview, cache, animation, and terminal-image behavior.
- RunHaven's desired Cubby default can be a configuration/default-selection
  change instead of a custom asset subsystem.
- If Cubby assets are copied into a repo-owned export later, use the
  pet-mascot-studio export handoff and sanitize copied metadata first.

### Codex Pet Runtime And Terminal Detection

Decision: compile and test the lower Codex pet runtime before the picker and
bottom-pane UI.

Why leaving and adapting is better than removing:

- The pet image quality problem is controlled by Codex's native terminal image
  overlay path, not by Ratatui cell drawing.
- `pets/model.rs`, `pets/frames.rs`, `pets/image_protocol.rs`, `pets/sixel.rs`,
  `pets/ambient.rs`, and `pets/mod.rs` now compile in RunHaven.
- The Codex frame scheduler from `tui/frame_requester.rs` now compiles with
  Tokio, matching upstream behavior instead of replacing it with a custom loop.
- `terminal_detection.rs` and `terminal_tests.rs` are copied from
  `/Users/c/Documents/GitHub/codex/codex-rs/terminal-detection/src/` because
  the pet protocol decision needs the same iTerm2, Kitty, Sixel, tmux, and
  Zellij behavior as Codex.
- `pets/picker.rs` and `pets/preview.rs` stay vendored but are staged until the
  bottom-pane adapter compiles. This is not a removal.
- The only behavior-preserving source adaptation in this slice is explicit
  SHA-256 lower-hex formatting in `pets/model.rs` for RunHaven's pinned
  `sha2` 0.11 dependency.

### Codex Pet Picker Selection Contract

Decision: compile and test `pets/picker.rs` and `pets/preview.rs` against a
staged bottom-pane selection contract before adapting the full Codex bottom
pane.

Why leaving and adapting is better than removing:

- The picker is the native `/pets` path that discovers built-in, custom, and
  legacy avatar packages.
- The full Codex bottom pane currently imports app-server, protocol, skills,
  plugin, file-search, and chat-composer surfaces that are not needed to verify
  pet discovery.
- The staged contract in `crates/runhaven-tui/src/tui/mod.rs` mirrors the Codex data
  types the picker returns: `SelectionItem`, `SelectionViewParams`, callbacks,
  side content sizing, event sender, and the standard popup hint.
- `pets/picker.rs` and `pets/preview.rs` stay compiled and tested. They are not
  removed or rewritten.
- The next bottom-pane pass should replace the staged contract with the full
  adapted Codex bottom-pane view once the wider app-shell dependencies are
  ready.

### Temporary Native Pet Image Smoke

Superseded decision: a temporary visual smoke path was added to the old
read-only `app_shell.rs` so image quality could be checked before the full app
shell and bottom pane were adapted. That hook has since been removed from the
live shell during core-completion cleanup.

Why this is a small RunHaven adapter instead of a custom renderer:

- The smoke path uses Codex's vendored `AmbientPet`, frame extraction,
  `FrameRequester`, and `render_ambient_pet_image` terminal-image writer.
- It materializes RunHaven's verified bundled Cubby package into the same
  Codex custom-pet package shape, using `custom:runhaven-cubby` under
  `$CODEX_HOME/pets/runhaven-cubby/`.
- It does not overwrite or depend on a user's own `$CODEX_HOME/pets/cubby/`.
- It gives the vendored `FrameRequester` a small Tokio runtime instead of
  replacing the scheduler with local code.
- It was off by default and only ran with `RUNHAVEN_TUI_IMAGE_SMOKE=1`.
- `RUNHAVEN_TUI_IMAGE_SMOKE_PET=<selector>` pointed the smoke at another Codex
  pet selector when needed.

Current state: the live shell has no active `RUNHAVEN_TUI_IMAGE_SMOKE` path.
The bundled Cubby package and lower pet/image modules remain parked as
source-first infrastructure. Reintroduce a bounded visual smoke only in a
final-pass pet slice or when a core terminal-image check explicitly requires it.

### Codex Renderable Contract

Decision: replace the temporary `Renderable` stand-in with Codex's vendored
`render/renderable.rs` while still staging the heavier syntax-highlight render
module.

Why leaving and adapting is better than removing:

- The pet preview, selection views, and future bottom-pane views all share the
  same renderable trait and layout helpers.
- `render/renderable.rs` and its tests now compile in RunHaven.
- The only source adaptation is a Ratatui 0.30 compatibility tweak for rendering
  `Line` through the borrowed `WidgetRef` implementation.
- Syntax highlighting remains staged because `render/highlight.rs` pulls in
  theme globals and terminal palette behavior that should be adapted with the
  app-shell theme pass.

### Codex Terminal Title Helper

Decision: compile and test Codex's `terminal_title.rs` as an app-shell helper
before the full app shell is runnable.

Why leaving and adapting is better than removing:

- Terminal title behavior was explicitly called out as a Codex source feature
  to evaluate.
- The helper is self-contained and protects OSC title output by stripping
  control, invisible, and bidi-formatting characters.
- Keeping it now avoids writing a custom title sanitizer later.
- The app-shell pass can decide when RunHaven sets or clears titles; this slice
  only preserves the safe low-level helper.

### Codex Motion, Palette, And Terminal Probe

Decision: compile and test Codex's motion helpers, shimmer renderer, terminal
palette helpers, and bounded terminal probe.

Why leaving and adapting is better than removing:

- Motion and reduced-motion behavior are part of the expected Codex TUI feel.
- Shimmer uses the terminal palette helpers so light and dark terminals get
  theme-aware color behavior instead of a RunHaven-only color path.
- `terminal_probe.rs` preserves Codex's short startup probes for default colors,
  cursor position, and keyboard enhancement support.
- RunHaven keeps the Codex motion primitive boundary test, but points it at
  `crates/runhaven-tui/src/tui/` and uses RunHaven's existing `regex` dependency
  instead of Codex's `codex_utils_cargo_bin` test helper.
- The Unix default-color requery path uses the vendored bounded terminal probe
  instead of Codex's fork-only crossterm color-query helpers. Revisit this only
  if RunHaven adopts Codex's pinned crossterm fork for the full app shell.

### Codex Style And Text Helpers

Decision: compile and test Codex's shared style, text formatting, line
truncation, wrapping, and render line utility helpers before the status widget
and bottom-pane views.

Why leaving and adapting is better than removing:

- These helpers are shared by the status row, markdown rendering, history
  cells, bottom-pane selection views, and app-shell chrome.
- Keeping them preserves Codex's URL-aware wrapping, styled line truncation,
  path truncation, compact JSON formatting, and terminal-palette-aware styles.
- RunHaven now enables `serde_json`'s `preserve_order` feature to match Codex
  TUI's dependency shape, which keeps the vendored compact JSON tests and UI
  formatting behavior intact.
- `render/line_utils.rs` is compiled as the low-level shared line-copy helper
  used by wrapping.

### Earlier RunHaven Zork Implementation

Decision: leave `crates/runhaven-tui/src/tui/zork/` absent from the raw Codex vendor
baseline.

Why removal is acceptable for the baseline:

- The old Zork code belonged to the custom RunHaven TUI tree that is being
  replaced.
- It can be recovered from git history if the final TUI design reintroduces it.
- Reintroducing it later should happen against the vendored Codex TUI baseline,
  with attribution and save/restore safety reviewed again.
- The wishlist still keeps the hidden Zork I easter egg.

## First Milestone

The first milestone is a clean vendor baseline:

- current custom TUI code removed
- Codex TUI source copied into place
- attribution preserved
- local changes clearly marked
- compile gaps visible and tracked
- no product-shaping or culling decisions made before the baseline exists
- local source-format exception recorded: `markdown_render_tests.rs` keeps the
  same Markdown hard-break input through `concat!` so RunHaven's whitespace
  check stays clean

## Known Integration Gaps

- The first compile gap after the reset was RunHaven's missing module entrypoint.
  `crates/runhaven-tui/src/tui/mod.rs` now keeps the crate buildable and dispatches to
  a temporary RunHaven-owned MVP launch flow while the vendored Codex entrypoint
  is adapted.
- The lower native pet runtime, terminal protocol detection, frame extraction,
  Sixel encoder, Kitty image writers, ambient draw request model, and Tokio
  frame scheduler now compile and pass their tests.
- The native pet picker and preview now compile and pass their tests against a
  staged bottom-pane selection contract.
- Codex's `render/renderable.rs` now compiles and passes its tests through the
  RunHaven adapter.
- Codex's terminal title helper now compiles and passes its tests.
- Codex's motion, shimmer, terminal palette, and terminal probe helpers now
  compile and pass their focused tests.
- Codex's shared style and text helpers now compile and pass their focused
  tests.
- `app_shell.rs` now restores a real bare interactive TUI path. It consumes
  `LaunchPlanData`, keeps the first chooser plain, shows dense launch detail in
  review and confirm, and hands confirmed plans to the foreground launch path
  after terminal restore. It is temporary glue until the Codex app shell is
  adapted.
- The copied Codex source still has crate-root assumptions from the upstream
  `codex-tui` crate. The next integration work is to adapt those assumptions
  into RunHaven product adapters without culling useful Codex surfaces early.

## Release Target

Do not publish a release from the interim vendor-reset state. After the TUI is
fully integrated, verified, and confirmed, do a full release bump to `v0.6.0`.
