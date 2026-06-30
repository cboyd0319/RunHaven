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

`satnaing/shadcn-admin` evidence:

- The local checkout is `gui-ref-satnaing-shadcn-admin` at commit `e16c87f`.
- It is a React 19, Vite 8, Tailwind 4, and shadcn-oriented admin shell under
  the MIT license.
- Its useful ideas are a polished sidebar layout, command-search pattern,
  responsive shell behavior, data-table examples, settings surfaces, and
  explicit notes about customized shadcn components.
- It includes broad app dependencies such as Clerk, TanStack Router, TanStack
  Query, TanStack Table, Recharts, Sonner, Zustand, Zod, and Axios. Those are
  not needed for the first RunHaven GUI slice unless a concrete feature needs
  them.

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

## Completed Spike

Two throwaway prototypes were built outside committed product source on
2026-06-30.

### React + shadcn/ui

Prototype path: `/private/tmp/runhaven-gui-spike-react`.

Setup:

- `npx shadcn@latest init --template vite --base radix --preset nova --name
  runhaven-gui-spike-react --cwd /private/tmp --yes --no-rtl --pointer`
- `npx shadcn@latest add sidebar alert-dialog dialog field input select
  checkbox table tabs empty alert sheet badge separator --yes`
- Added a static RunHaven launch shell using `docs/assets/logo.png`.

Result:

- Screenshot: [React 1080 x 720](../assets/gui-stack-spike/react-1080.png)
- Screenshot: [React 720 x 560](../assets/gui-stack-spike/react-720.png)
- Dependencies: React 19.2.6, Vite 8.1.2, Tailwind 4, shadcn 4.12.0,
  `radix-ui`, `lucide-react`, `class-variance-authority`,
  `tailwind-merge`, `tw-animate-css`, and Geist font assets.
- Check: `npm run build` passed.
- Setup was smooth. Official shadcn init and component install worked without
  manual registry fixes.
- The first-screen prototype looked polished with minimal custom layout code.
- The typed Tauri command boundary can stay unchanged because React only
  replaces the WebView frontend layer.

### Svelte + shadcn-svelte

Prototype path: `/private/tmp/runhaven-gui-spike-svelte`.

Setup:

- Created a Vite Svelte TypeScript app.
- Installed Tailwind 4 and `@tailwindcss/vite`.
- Generated the shadcn-svelte default preset code with
  `shadcn-svelte/preset`, then ran `npx shadcn-svelte@latest init --preset b0`.
- Added sidebar, alert dialog, dialog, field, input, select, checkbox, table,
  tabs, empty, alert, sheet, badge, separator, and button components.
- Added a static RunHaven launch shell using `docs/assets/logo.png`.

Result:

- Screenshot: [Svelte 1080 x 720](../assets/gui-stack-spike/svelte-1080.png)
- Screenshot: [Svelte 720 x 560](../assets/gui-stack-spike/svelte-720.png)
- Dependencies: Svelte 5.56.4, Vite 8.1.2, Tailwind 4.3.2,
  `shadcn-svelte` 1.3.0, `@lucide/svelte`, `bits-ui`, `tailwind-variants`,
  `tailwind-merge`, `tw-animate-css`, and Inter font assets.
- Check: `npm run check` passed with one TypeScript 6 `baseUrl` deprecation
  warning. `npm run build` passed.
- Setup was workable but more brittle than React. It needed a non-interactive
  preset code, Tailwind import repair, explicit Vite aliasing, and Svelte
  snippet-oriented component wiring.
- The visual result can match React, but it took more local framework setup
  work and has less direct block/template leverage.
- The typed Tauri command boundary can stay unchanged because Svelte also
  remains only a WebView frontend layer.

## Decision Rule

Pick the stack that produces the best RunHaven-specific first screen with the
least custom UI code while preserving the Tauri/Rust security boundary.

Do not prefer Svelte merely because it exists. Do not prefer React merely
because shadcn is popular. Prefer the path that visibly gets RunHaven to a
polished, secure, maintainable GUI fastest.

## Decision

Use React + shadcn/ui + Tauri for the first real GUI slice.

React is not the security decision. The security decision remains the existing
typed `src-tauri` boundary over `runhaven-core`, narrow capabilities, bounded
payloads, and no generic host bridge.

React is the product and implementation decision because the spike produced the
best RunHaven-specific screen with the least brittle setup, official shadcn
component support, better template leverage, and a cleaner path to polished
macOS utility screens.

Keep Svelte + shadcn-svelte as a fallback only if the first real React slice
hits a concrete blocker in Tauri integration, tests, accessibility, or bundle
maintenance. Do not keep Svelte solely because the alpha shell currently uses
it.
