# Native GUI Reference Pass

Last updated: 2026-06-30 UTC

## Purpose

RunHaven is shifting product focus from terminal polish to a native-feeling
macOS GUI. This pass records what to borrow, what to avoid, and what to prove
first from the Apple `container` GUI references reviewed on 2026-06-30.

This is reference research, not a dependency decision. Do not import code from
these projects without a separate license, security, and architecture review.

## Local Reference Checkouts

The reference repositories were cloned under `/Users/c/Documents/GitHub/`:

| Source | Local checkout | Commit reviewed |
| --- | --- | --- |
| `lcandy2/container-ui` | `gui-ref-lcandy2-container-ui` | `1cf6022` |
| `Container-Kit/ContainerKit` | `gui-ref-Container-Kit-ContainerKit` | `e6735b4` |
| `knowledge-forest/apple-container-ui` | `gui-ref-knowledge-forest-apple-container-ui` | `0b8c06b` |
| `andrew-waters/orchard` | `gui-ref-andrew-waters-orchard` | `298a064` |
| `SterbenSQ/container-ui` | `gui-ref-SterbenSQ-container-ui` | `ca15b26` |
| `locustave/keel` | `gui-ref-locustave-keel` | `db0c542` |
| `0Itsuki0/AppleContainerDesktop` | `gui-ref-0Itsuki0-AppleContainerDesktop` | `2595351` |

The `0Itsuki0/AppleContainerDesktop` checkout is the source project linked from
the Medium reference.

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

Two paths are viable:

1. Continue the existing Tauri/Svelte shell first.

   This is the lowest-risk path because RunHaven already owns typed Rust
   commands, focused Tauri capabilities, tests, and Svelte panels. The next
   value comes from deleting dashboard noise and reshaping the first screen
   into a native-feeling RunHaven workflow.

2. Build a SwiftUI shell over a narrow RunHaven backend.

   This is the strongest native-feel path, but it creates harder Rust to Swift,
   packaging, IPC, signing, and test seams. The backend must still be a typed
   RunHaven bridge, either through an exact JSON CLI sidecar or a reviewed
   local IPC/FFI layer. It must not become a generic Apple `container` GUI.

Working recommendation: start with a Tauri native-feel prototype over the
existing typed boundary, then make the SwiftUI switch only if that prototype
fails the native-feel acceptance bar. This avoids spending the first GUI slice
on packaging and bridge work before validating the product workflow.

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

1. Keep `src-tauri` as the narrow Rust command boundary.
2. Redesign the Svelte first screen around "Start a safe agent run":
   workspace, agent, plan review, launch, then status.
3. Move CPU, memory, diagnostics, image builder detail, raw logs, and history
   behind secondary detail views.
4. Add a visual acceptance pass using screenshots at common macOS window sizes.
5. Run focused frontend, Tauri, and capability-guard checks.
6. Decide whether Tauri clears the native-feel bar before committing to
   SwiftUI.
