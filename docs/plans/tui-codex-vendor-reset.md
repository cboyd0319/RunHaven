# TUI Codex Vendor Wishlist

Last updated: 2026-06-27

## Goal

RunHaven should use the Codex TUI source as the baseline for its terminal UI,
then shape that baseline into the RunHaven product.

This document is only a wishlist. It records what we want from the Codex TUI
source before we decide what to change, remove, or keep.

## Source

Primary source:

```text
/Users/c/Documents/GitHub/codex/codex-rs/tui/src/
```

The intent is to fully replace the current custom `src/runhaven/cli/tui/` tree
with vendored Codex TUI source, then make RunHaven changes from that baseline.

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

## Desired RunHaven Shape

After the vendored baseline is in place, we want the TUI to feel like
RunHaven:

- RunHaven name, logo, and product language
- Cubby as the default pet
- pets and animation that stay true to Codex source behavior
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

## First Milestone

The first milestone is a clean vendor baseline:

- current custom TUI code removed
- Codex TUI source copied into place
- attribution preserved
- local changes clearly marked
- compile gaps visible and tracked
- no product-shaping or culling decisions made before the baseline exists
