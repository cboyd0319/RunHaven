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

## Source

Primary source:

```text
/Users/c/Documents/GitHub/codex/codex-rs/tui/src/
```

The intent is to fully replace the current custom `src/runhaven/cli/tui/` tree
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

After `src/runhaven/cli/tui/` is replaced with the vendored source, review the
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

Decision: remove copied `*.snap` files under `src/runhaven/cli/tui/`.

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

### Earlier RunHaven Zork Implementation

Decision: leave `src/runhaven/cli/tui/zork/` absent from the raw Codex vendor
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
  `src/runhaven/cli/tui/mod.rs` now keeps the crate buildable and fails closed
  for interactive TUI launch while the vendored Codex entrypoint is adapted.
- The copied Codex source still has crate-root assumptions from the upstream
  `codex-tui` crate. The next integration work is to adapt those assumptions
  into RunHaven product adapters without culling useful Codex surfaces early.

## Release Target

Do not publish a release from the interim vendor-reset state. After the TUI is
fully integrated, verified, and confirmed, do a full release bump to `v0.6.0`.
