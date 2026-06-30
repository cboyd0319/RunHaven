# Native GUI Stack Spike

Last updated: 2026-06-30 UTC

## Purpose

Choose the GUI frontend stack before writing the first real RunHaven GUI slice.
The existing Svelte app is not a constraint. It is only one candidate.

The part to preserve is the secure Tauri/Rust boundary: typed commands,
bounded payloads, narrow capabilities, and no generic host bridge. The web UI
layer can change if another stack gives RunHaven a better, faster, more
polished product.

## Non-Negotiables

- Keep `src-tauri` as the trusted boundary over `runhaven-core`.
- Do not add broad shell, filesystem, PTY, process, HTTP, opener, log, or
  updater permissions as part of a visual-stack decision.
- Do not require a paid Apple Developer Program membership.
- Do not cut a release, notarize, add updater flow, or publish artifacts.
- Keep the first screen focused on safe launch, not a generic dashboard.
- The selected stack must support screenshot verification at `1080 x 720` and
  `720 x 560`.

## Evidence So Far

Local RunHaven UI state:

- `ui/` uses npm, Vite, Svelte 5, and plain CSS.
- `npx shadcn@latest info --json` detects the project as Vite, but reports no
  Tailwind config, no shadcn config, no preset, and no installed components.
- Therefore shadcn adoption is a deliberate stack/design-system change, not a
  small component install.

React shadcn evidence:

- The current shadcn CLI is available as `shadcn@4.12.0` through `npx`.
- `npx shadcn@latest docs button sidebar alert dialog field tabs table`
  returns docs and examples for the primitives RunHaven needs.
- `npx shadcn@latest search @shadcn --query sidebar --type ui,block --json`
  returns the `sidebar` UI component plus many sidebar blocks.
- `npx shadcn@latest search @shadcn --query dashboard --type block --json`
  returns `dashboard-01`.
- `npx shadcn@latest search @shadcn --query empty --type ui --json` returns
  `empty`.
- `npx shadcn@latest search @shadcn --query dialog --type ui,block --json`
  returns `dialog`, `alert-dialog`, and a sidebar-in-dialog block.
- Official shadcn docs describe it as open component code, a code distribution
  platform, composable, and styled with strong defaults. Its sidebar blocks are
  React-focused and work with React frameworks.

Svelte shadcn evidence:

- `shadcn-svelte` exists and supports npm through
  `npx shadcn-svelte@latest`.
- It has Svelte components for sidebar, alert, alert dialog, button, checkbox,
  dialog, empty, field, input, select, sheet, table, tabs, tooltip, and other
  RunHaven-relevant primitives.
- The sidebar documentation says the component is composable, themeable,
  customizable, and extracted into Svelte files.
- It is an unofficial community-led Svelte port of shadcn/ui. That does not
  disqualify it, but it makes maturity and registry stability part of the
  decision.

`agmmnn/tauri-ui` evidence:

- The local checkout is `gui-ref-agmmnn-tauri-ui` at commit `0cfd2a7`.
- It is a scaffold and docs project, not an Apple `container` GUI.
- Its useful ideas are desktop shell defaults: hidden-until-first-paint,
  dedicated scroll shell, no rubber-band overscroll, desktop-like selection
  behavior, sensible window defaults, and optional debug tooling.
- It is React, Tailwind, shadcn, and Bun-oriented. That is a viable stack
  candidate, not a reason to copy the whole scaffold blindly.

## Candidate Paths

### 1. React + shadcn/ui + Tauri

This is the strongest template/component path.

Pros:

- Best access to official shadcn components and blocks.
- Sidebar, alert-dialog, dialog, field, table, tabs, empty, and dashboard
  blocks are first-class.
- More likely to reduce custom UI code and improve visual polish quickly.
- Tauri/Rust backend can stay intact while `ui/` changes.

Risks:

- Frontend rewrite from Svelte to React.
- Package and test scripts must change.
- Existing Svelte tests and components must be replaced.
- Need to keep generated code lean, accessible, and not dashboard-heavy.

### 2. Svelte + shadcn-svelte + Tauri

This keeps the existing framework while adopting a shadcn-like design system.

Pros:

- Less frontend migration than React.
- Svelte 5 compatible path exists.
- Sidebar and needed primitives exist.
- Still gives copied component source and Tailwind tokens.

Risks:

- Unofficial community-led port.
- Blocks/template ecosystem appears smaller and less direct than React
  shadcn/ui.
- May still require more custom layout work.

### 3. Existing Svelte + Plain CSS

This is the current app path.

Pros:

- Lowest immediate migration work.
- No new UI dependency stack.
- Current tests and command wrappers remain usable.

Risks:

- Highest custom design burden.
- More likely to look homegrown.
- Less reuse from existing polished templates.

### 4. SwiftUI Native Shell

This remains the strongest native-macOS path, but it does not solve the
shadcn/template problem.

Pros:

- Best native feel if implemented well.
- Better alignment with macOS conventions.

Risks:

- New Rust-to-Swift or CLI sidecar bridge.
- More packaging, signing, testing, and IPC work.
- No direct shadcn/template leverage.

## Recommended Spike

Before real GUI code, build two throwaway prototypes outside committed product
source:

1. React + shadcn prototype:
   - Use Vite + React + Tailwind + shadcn/ui.
   - Try `@shadcn/sidebar`, `@shadcn/alert-dialog`, `@shadcn/dialog`,
     `@shadcn/field`, `@shadcn/table`, `@shadcn/tabs`, `@shadcn/empty`, and
     one sidebar block.
   - Recreate the RunHaven start screen using static mock data and
     `docs/assets/logo.png`.

2. Svelte + shadcn-svelte prototype:
   - Use the existing Svelte/Vite shape or a temp Svelte app.
   - Try sidebar, alert dialog, field/input/select, table, tabs, empty, and
     sheet/dialog.
   - Recreate the same RunHaven start screen with the same mock data.

For each prototype, record:

- Screenshot at `1080 x 720`.
- Screenshot at `720 x 560`.
- Dependency list.
- Approximate custom component count.
- Whether setup was smooth or brittle.
- Whether the result looks polished without heavy custom CSS.
- Whether it can call the existing typed Tauri command client without a broad
  permission change.

## Decision Rule

Pick the stack that produces the best RunHaven-specific first screen with the
least custom UI code while preserving the Tauri/Rust security boundary.

Do not prefer Svelte merely because it exists. Do not prefer React merely
because shadcn is popular. Prefer the path that visibly gets RunHaven to a
polished, secure, maintainable GUI fastest.
