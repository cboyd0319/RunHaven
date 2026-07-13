# Native GUI Reference Pass

Last updated: 2026-06-30 UTC

## Purpose

RunHaven is shifting product focus from terminal polish to a native-feeling
macOS GUI. This pass records what to borrow, what to avoid, and what to prove
first from the Apple `container` GUI references reviewed on 2026-06-30.

This is reference research, not a dependency decision. Do not import code from
these projects without a separate license, security, and architecture review.

## Local Reference Checkouts

The reference repositories were cloned under `<local-github-workspace>/`:

| Source | Local checkout | Commit reviewed |
| --- | --- | --- |
| `lcandy2/container-ui` | `gui-ref-lcandy2-container-ui` | `1cf6022` |
| `Container-Kit/ContainerKit` | `gui-ref-Container-Kit-ContainerKit` | `e6735b4` |
| `knowledge-forest/apple-container-ui` | `gui-ref-knowledge-forest-apple-container-ui` | `0b8c06b` |
| `andrew-waters/orchard` | `gui-ref-andrew-waters-orchard` | `298a064` |
| `SterbenSQ/container-ui` | `gui-ref-SterbenSQ-container-ui` | `ca15b26` |
| `locustave/keel` | `gui-ref-locustave-keel` | `db0c542` |
| `0Itsuki0/AppleContainerDesktop` | `gui-ref-0Itsuki0-AppleContainerDesktop` | `2595351` |
| `agmmnn/tauri-ui` | `gui-ref-agmmnn-tauri-ui` | `0cfd2a7` |
| `satnaing/shadcn-admin` | `gui-ref-satnaing-shadcn-admin` | `e16c87f` |

The `0Itsuki0/AppleContainerDesktop` checkout is the source project linked from
the Medium reference.

The `agmmnn/tauri-ui` checkout is a Tauri app scaffold and documentation
project, not an Apple `container` GUI.

The `satnaing/shadcn-admin` checkout is a polished React, Vite, Tailwind, and
shadcn admin shell reference. It is useful for layout and component patterns,
not as a wholesale product template.

## Product Direction

RunHaven should not become a general Apple `container` dashboard. The default
GUI job is narrower:

- Pick or confirm a project folder.
- Pick an agent.
- Show the secure run plan in plain language.
- Explain what is not shared with the agent.
- Launch only after clear confirmation.
- Show run status, bounded output, stop, hard stop, repair, history, and
  diagnostics without requiring CLI knowledge.

Container management can exist later as expert detail, but it should not be the
first screen or the mental model for nontechnical users.

The current identity and usage rules live in
`docs/design/runhaven-design-spec.md`. Use `docs/assets/app-icon.png` as a small
app-identity mark, not a hero, background, splash screen, or mascot-first
surface.

## Packaging Constraint

Do not require a paid Apple Developer Program membership for current GUI
prototype work. Local development should use Tauri dev, no-sign builds, or
ad-hoc signing as needed. A signed and notarized public artifact remains a
later release gate, because Apple requires Developer ID distribution to use
Apple Developer Program or Enterprise Program membership, a Developer ID
certificate, and notarization.

This means the first GUI slices should optimize for local build, test,
screenshot, and maintainer trial runs. Do not add updater, notarization,
Developer ID, App Store, certificate, keychain import, or release-publication
work until the maintainer explicitly asks for a distribution pass.

## Pre-Release Native Design Audit

Before any first external GUI release, run a deeper native macOS design and app
asset audit against current primary resources. This is intentionally deferred
until the CLI and GUI workflows are functionally complete, so implementation
time stays focused on the hard runtime, auth, recovery, and safety paths first.

Audit resources to review at that point:

- <https://developer.apple.com/design/human-interface-guidelines/>
- <https://www.figma.com/community/file/1651309434229735362>
- <https://www.sketch.com/s/57153a31-3379-4737-8ac6-dbfd6525f052>
- <https://developer.apple.com/icon-composer/>
- <https://developer.apple.com/sf-symbols/>

Expected output: one focused pass on macOS layout, sidebar behavior, controls,
copy density, iconography, app icon/export readiness, and SF Symbols usage.
Do not add release, signing, notarization, updater, or store work as part of
that audit unless the maintainer explicitly opens release work.

## Borrow

- Orchard: native macOS information architecture. The sidebar, list, detail,
  status, and action placement are the strongest visual reference. Its use of
  Apple `ContainerAPIClient` and CLI fallback is also the best architecture
  signal among the references.
- `0Itsuki0/AppleContainerDesktop`: typed Apple `container` package direction,
  settings and missing-executable states, and a concrete example of avoiding a
  web app shell for a native macOS surface.
- `knowledge-forest/apple-container-ui`: a small central command adapter, demo
  mode, simple SwiftUI layout, and bounded log stream behavior.
- `lcandy2/container-ui`: sandboxed app plus XPC/helper separation as a pattern
  to study if RunHaven moves to SwiftUI and needs privileged or unsandboxed
  host access.
- `SterbenSQ/container-ui`: small SwiftUI/MVVM structure, localization
  awareness, and actor-owned service state.
- `keel`: phase-gated workflow, audit, and rollback ideas. It is not a
  container GUI, but the "plan, gate, execute, recover" shape fits RunHaven.
- `agmmnn/tauri-ui`: desktop-shell polish checklist, especially
  hidden-until-first-paint startup, a dedicated app scroll shell, no page-level
  overscroll/rubber-band scrolling, desktop-like selection defaults with
  explicit selectable surfaces, and sensible default window sizing.
- `satnaing/shadcn-admin`: React shadcn sidebar composition, command-search
  pattern, responsive shell, data-table examples, settings layout, and explicit
  documentation of customized shadcn components.

## Avoid

- Do not expose a generic `container` argument bridge to the WebView or
  frontend.
- Do not grant broad Tauri shell, filesystem, PTY, OS, or process capabilities.
- Do not make an integrated terminal the primary workflow for nontechnical
  users.
- Do not make arbitrary volumes, host home mounts, raw environment passthrough,
  SSH, or full internet normal first-screen controls.
- Do not make generic image, network, or volume administration the default
  experience.
- Do not copy command-string execution patterns. Several references build
  shell strings or fall back to `/bin/sh -c`; RunHaven should keep exact
  subprocess argument lists and typed backend commands.
- Do not migrate RunHaven to React, Tailwind, shadcn/ui, Bun, or a scaffolded
  app shell blindly. A pivot is allowed if the stack spike shows a clear
  product-quality and implementation-speed win.
- Do not add opener, log, debug-panel, release-workflow, or size-optimization
  changes from `tauri-ui` by default. Each affects dependencies, capabilities,
  packaging, logs, or release behavior and needs its own RunHaven review.
- Do not copy `shadcn-admin` as a full admin dashboard. Avoid its auth,
  routing, charting, query, notification, and state dependencies unless a
  RunHaven feature explicitly needs them.

## Current RunHaven Advantage

RunHaven already has a safer desktop boundary than the broadest references:

- `src-tauri` exposes typed commands over `runhaven-core`, not arbitrary
  `container` execution.
- Tauri capabilities are split into read, launch, run-control, and folder-pick
  scopes.
- `capability_guard` fails closed if a capability grants an unvetted plugin
  bridge.
- The frontend plan request maps into `RunOptions` with secure defaults:
  provider or internal network, agent auth scope, non-root user, no SSH, no
  arbitrary host credential mounts, no arbitrary host environment values.

Keep that boundary even if the visual shell changes.

## Framework Decision

Do not treat the current Svelte frontend as a constraint. The Tauri/Rust
boundary is the valuable part to preserve; the web frontend can change.

`docs/plans/native-gui-stack-spike.md` records the completed stack comparison.
React + shadcn/ui + Tauri is the selected first implementation path. Svelte +
shadcn-svelte remains a fallback only if the first real React slice hits a
concrete blocker.

Four paths are viable:

1. React + shadcn/ui + Tauri.

   This is the strongest component/template path. It can use official shadcn
   primitives and blocks while keeping the existing typed Tauri commands.

2. Svelte + shadcn-svelte + Tauri.

   This is the lower-migration shadcn-like path. It should be considered, but
   its community-port maturity and block coverage need proof.

3. Existing Svelte + plain CSS + Tauri.

   This is the smallest code migration, but likely carries the highest custom
   design burden.

4. SwiftUI shell over a narrow RunHaven backend.

   This is the strongest native-feel path, but it creates harder Rust to Swift,
   packaging, IPC, signing, and test seams. The backend must still be a typed
   RunHaven bridge, either through an exact JSON CLI sidecar or a reviewed
   local IPC/FFI layer. It must not become a generic Apple `container` GUI.

Working recommendation: keep `src-tauri` and `runhaven-core` as the secure
backend boundary, replace the alpha Svelte frontend with a React + shadcn/ui
frontend, and implement the first real GUI slice from
`docs/plans/native-gui-wireframe.md`.

## Hardest Problems First

The next GUI work should prove these before visual polish:

- Trust boundary: the frontend remains untrusted, and all mutating operations
  go through typed RunHaven commands with explicit confirmation.
- No paid signing dependency: the prototype builds and runs locally without
  Apple Developer Program credentials.
- First-run path: a new user can reach "safe launch ready" without reading CLI
  docs.
- Missing setup: the app clearly explains missing Apple `container`, missing
  image, missing login, and blocked provider network state with one next action.
- Recovery: an active, stale, blocked, or failed run has obvious stop, hard
  stop, repair, and log-review paths.
- Native feel: the first screen looks like a macOS utility, not a terminal
  dashboard or generic container admin UI.
- No secret exposure: logs and diagnostics stay bounded and confirmation-gated;
  workspace paths and auth URLs are minimized where not needed.

## Immediate GUI Slice

Use `docs/plans/native-gui-wireframe.md` as the implementation contract.

1. Keep `src-tauri` as the narrow Rust command boundary.
2. Implement the first screen in React + shadcn/ui around "Start a safe agent run":
   workspace, agent, plan review, launch, then status.
3. Move CPU, memory, diagnostics, image builder detail, raw logs, and history
   behind secondary detail views.
4. Add a visual acceptance pass using screenshots at common macOS window sizes.
5. Run focused frontend, Tauri, and capability-guard checks.
6. Decide whether Tauri clears the native-feel bar before committing to
   SwiftUI.
