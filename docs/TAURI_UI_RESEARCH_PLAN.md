# Tauri UI Research Plan

Last updated: 2026-06-27

Status: research complete. The first Tauri/Svelte scaffold started from this
plan with setup, dashboard, profile, folder-pick, and run-plan review surfaces.
The first mutating slice, `launch_run`, is now implemented behind explicit
plan and warning confirmation plus its own `launch-run` capability.

Current release context: RunHaven remains alpha/pre-release after the `v0.5.0`
CLI-only pre-release. This document records the accepted Tauri research decision
and alpha UI plan. The TUI is active in the current checkout; the desktop app
remains alpha and is deferred to a later first-class release phase.

Goal: build the easiest safe desktop experience for people with little or no
technical background to run AI coding agents inside Apple `container`.

## Decision Summary

| Decision | Outcome |
| --- | --- |
| Desktop shell | Use Tauri v2. |
| Frontend | Use Svelte + Vite + TypeScript as a static SPA. Do not start with SvelteKit. |
| Package manager | Use npm and a checked-in `package-lock.json` because npm is already installed locally and avoids introducing another package manager. |
| Rust integration | Add a separate `src-tauri` crate that depends on the existing root `runhaven` library by path. Do not invoke the `runhaven` CLI through a shell bridge for normal UI operations. |
| UI posture | The secure path is the default and shortest path. Supported advanced or risky paths warn clearly but stay available behind explicit confirmation. |
| WebView trust | Treat the WebView as untrusted. All privileged work stays in typed Rust commands. |
| Native plugins | Start with only the dialog plugin for folder selection. Avoid shell, filesystem, HTTP, updater, store, clipboard, notification, and opener plugins until a feature proves it needs one. |
| First implementation | Build a dashboard plus a launch-plan wizard first, then wire mutating controls one at a time behind typed Rust commands, explicit confirmation, and narrow capabilities. |

Hard blocks are reserved for invalid input, unsupported platform state, failed
doctor checks, nonfunctional features such as current `--ssh`, and operations
without an explicit confirmation. Risky but supported choices should warn, name
the tradeoff, and allow the user to proceed.

## Source Evidence

Reviewed on 2026-06-16.

| Area | Evidence |
| --- | --- |
| Tauri frontend fit | Tauri works with many frontend frameworks and officially lists Svelte, React, Solid, Vue, and others in `create-tauri-app`: <https://v2.tauri.app/start/create-project/> |
| Static frontend constraint | Tauri acts as a static web host, supports SPA/SSG/MPA, and does not natively support server-based frontend alternatives: <https://v2.tauri.app/start/frontend/> |
| Tauri command IPC | Tauri commands are the typed primitive for calling Rust from the frontend and can accept arguments, return values, errors, and async results: <https://v2.tauri.app/develop/calling-rust/> |
| Tauri capabilities | Capabilities grant or deny permissions per window or WebView, and overlapping capabilities merge their boundaries: <https://v2.tauri.app/security/capabilities/> |
| Tauri permissions | Permissions enable commands, scopes, allow lists, and deny lists, and must be referenced from capabilities: <https://v2.tauri.app/security/permissions/> |
| Tauri state | Tauri managed state is available through the Manager API and can be read from commands: <https://v2.tauri.app/develop/state-management/> |
| Tauri tests | Tauri supports mock-runtime tests and WebDriver, but macOS desktop WebDriver is not available because macOS lacks a WKWebView driver tool: <https://v2.tauri.app/develop/tests/> and <https://v2.tauri.app/develop/tests/webdriver/> |
| Tauri frontend mocks | `@tauri-apps/api/mocks` can mock IPC in frontend tests: <https://v2.tauri.app/develop/tests/mocking/> |
| Dialog plugin | Tauri dialog plugin provides native file and directory selection on macOS: <https://v2.tauri.app/plugin/dialog/> |
| Shell plugin risk | Shell plugin dangerous commands are blocked by default and require capability permissions: <https://v2.tauri.app/plugin/shell/> |
| Tauri dependency sync | Tauri npm and Cargo versions should stay synced by compatible major/minor version: <https://v2.tauri.app/develop/updating-dependencies/> |
| Vite | Vite provides a fast dev server and production static build, with templates for relevant frameworks: <https://vite.dev/guide/> |
| Svelte | Svelte can be used directly with Vite, and `npm run build` outputs static HTML, JS, and CSS in `dist`: <https://svelte.dev/docs/svelte/getting-started> |
| SvelteKit | Tauri SvelteKit requires static adapter and SPA or SSG mode because server-based solutions are unsupported: <https://v2.tauri.app/start/frontend/sveltekit/> |
| React | React's own docs name Vite as a build-tool option, but React SPA apps need additional choices for routing, data fetching, and styling: <https://react.dev/learn/build-a-react-app-from-scratch> |
| Solid | SolidStart uses Vinxi/Nitro under the hood, adding server-oriented machinery unnecessary for RunHaven's first desktop UI: <https://docs.solidjs.com/solid-start/getting-started> |
| Vue | Vue's recommended scaffold is Vite-backed and mature, but its optional router/state/testing prompts are broader than the first RunHaven UI needs: <https://vuejs.org/guide/quick-start> |
| Docker Desktop | Docker Desktop proves users expect one-click setup and a GUI for local containers, applications, and images: <https://docs.docker.com/desktop/> |
| Podman Desktop | Podman Desktop surfaces container lifecycle, logs, terminal access, and image tasks in a GUI: <https://podman-desktop.io/docs/discover-podman-desktop> |
| DevPod | DevPod Desktop starts workspace creation from a repo, local path, or image and hides provider setup behind guided modals: <https://devpod.sh/docs/developing-in-workspaces/create-a-workspace> |
| GitHub Desktop | GitHub Desktop centers branch changes, diffs, selective commits, and discard recovery: <https://docs.github.com/en/desktop/making-changes-in-a-branch/committing-and-reviewing-changes-to-your-project-in-github-desktop> |

Package and tool evidence from local commands at research time on 2026-06-16:

| Package or tool | Observed stable on 2026-06-16 |
| --- | --- |
| Node | `v26.3.0` |
| npm | `11.16.0` |
| Rust | `rustc 1.96.0`, `cargo 1.96.0` |
| `create-tauri-app` | `4.6.2` |
| `create-vite` | `9.0.7` |
| `tauri` crate | `2.11.2` |
| `tauri-cli` crate | `2.11.2` |
| `tauri-build` crate | `2.6.2` |
| `tauri-driver` crate | `2.0.6` |
| `@tauri-apps/cli` | `2.11.2` |
| `@tauri-apps/api` | `2.11.0` |
| `tauri-plugin-dialog` and `@tauri-apps/plugin-dialog` | `2.7.1` |
| `vite` | `8.0.16` |
| `typescript` | `6.0.3` |
| `svelte` | `5.56.3` |
| `@sveltejs/vite-plugin-svelte` | `7.1.2` |
| `svelte-check` | `4.6.0` |
| `vitest` | `4.1.9` |
| `@lucide/svelte` | `1.20.0` |
| `eslint` | `10.5.0` |
| `prettier` | `3.8.4` |

At scaffold time and later pin-refresh passes, version checks were re-run and
exact stable versions were hard-pinned in `ui/package.json`,
`ui/package-lock.json`, `src-tauri/Cargo.toml`, and the workspace
`Cargo.lock`.
Treat the table above as historical research evidence. Current dependency
truth lives in the manifests and lockfiles, with source evidence routed through
`docs/RESEARCH.md` and `pins.toml`. Do not use caret, tilde, wildcard, or
unpinned plugin versions.

## Architecture Decision

Use Tauri v2 as a thin desktop shell around the existing Rust library.

Implemented first-scaffold layout:

```text
ui/
  package.json
  package-lock.json
  tsconfig.json
  vite.config.ts
  src/
    app/
    commands/
    components/
    styles/
    test/
src-tauri/
  Cargo.toml
  build.rs
  capabilities/
  src/
    commands/
    contracts.rs
    lib.rs
```

The Rust code now lives in workspace crates. The Tauri crate is a workspace
member and depends on `runhaven-core`, then exposes narrow typed commands to the
WebView. If a CLI-only path must be reused temporarily, the Rust backend may
call fixed internal functions or exact argument builders, but the WebView must
never receive a generic shell, process, filesystem, HTTP, or Apple `container`
bridge.

Do not package the CLI as a Tauri sidecar for the first pass. A sidecar would
duplicate command parsing, make typed errors harder, and pressure the project
toward broader shell permissions. Reconsider sidecars only if later packaging
requires a separate executable boundary with fixed arguments and structured
stdout/stderr handling.

Use one main window first. Avoid multi-window, tray, updater, custom protocol,
deep-link, notification, and background daemon work until a specific user flow
requires it.

## Frontend Framework Decision

Choose Svelte + Vite + TypeScript.

| Candidate | Decision | Reason |
| --- | --- | --- |
| Svelte + Vite | Choose | Best fit for a small local desktop tool: direct components, small dependency surface, static Vite output, readable state, and enough ecosystem for accessible custom controls. |
| SvelteKit SPA/static | Defer | Officially viable with Tauri, but it adds app-framework routing and adapter configuration before RunHaven needs it. Revisit if route complexity becomes real. |
| React + Vite | Defer | Strong ecosystem and accessible component options, but higher dependency gravity and more ways to overbuild the first UI. Keep as fallback if custom accessibility work becomes a bottleneck. |
| Solid + Vite | Reject for first pass | Technically strong, but the ecosystem and future maintainer familiarity are weaker for this repo. SolidStart adds Vinxi/Nitro complexity that does not fit the static desktop shell. |
| Vue + Vite | Reject for first pass | Mature and Vite-backed, but it does not beat Svelte on minimal first-pass code for this product. |

Initial UI dependencies should be minimal:

- Svelte, Vite, TypeScript, Tauri API, Tauri CLI, Tauri dialog plugin,
  `svelte-check`, Vitest, and `@lucide/svelte`.
- No Tailwind, shadcn, Radix, global state library, query library, router, CSS
  framework, chart library, data grid, animation library, or component kit in
  the first scaffold.
- Use semantic HTML, local Svelte components, CSS modules or scoped component
  styles, CSS custom properties, and lucide icons inside icon buttons.

## Product Principle: Secure Easy Path

The default and easiest path must be the secure path:

1. First screen runs read-only setup checks.
2. Launch flow defaults to the smallest selected project folder.
3. Network goal defaults to provider-only when the selected profile has reviewed
   provider hosts, otherwise local-only for offline tasks.
4. Worktree mode should be the recommended path for project-editing runs when
   the source checkout is clean.
5. No host home, credential folder, raw SSH key, browser profile, arbitrary
   environment variable, or root user path appears in the default flow.

Supported advanced paths should warn but not hide or block:

| Choice | UI handling |
| --- | --- |
| Full internet | Warn that egress is unrestricted and require confirmation. |
| Sensitive workspace override | Explain what folder is being exposed and require confirmation. |
| Root user override | Explain that the agent runs as root inside the container and require confirmation. |
| Environment passthrough | Show variable names only, never values, and require confirmation. |
| Custom image | Explain that the image is not a bundled RunHaven image and require confirmation. |
| Additional provider host | Explain host/subdomain scope and require confirmation. |
| State reset or prune | Show exact RunHaven-owned target and require confirmation. |
| Network prune | Show exact RunHaven-owned networks and require confirmation. |
| Worktree merge or discard | Show source/worktree paths and require confirmation. |
| Force kill | Explain it is a hard stop and require confirmation. |

Unavailable or invalid paths stay blocked until they work or validate:

- `--ssh` remains unavailable while Apple `container` non-root forwarding fails
  the no-secret smoke.
- Failed `runhaven doctor` blocks launch because the runtime is not ready.
- Invalid profile, session, resource, host, path, run id, or state target fails
  in Rust validation.
- Mutating operations cannot run without explicit confirmation.

## Comparable UI Patterns

| Product | Pattern to reuse | RunHaven translation |
| --- | --- | --- |
| Docker Desktop | One-click setup, direct management of local containers/images, visible local runtime status. | Dashboard starts with doctor, builder, image, active run, and safe next action status. |
| Podman Desktop | Containers page exposes lifecycle actions, logs, terminal, status, and cleanup. | Run detail shows status, logs, stop, kill, repair, and provider policy without raw Apple inspect output. |
| DevPod Desktop | Workspace creation begins with repository, local path, or image; provider setup is guided and modal. | Launch wizard starts with project folder, profile, network goal, image status, and clear explanations. |
| GitHub Desktop | Diff-centered review, selective changes, discard recovery, and visible branch state. | Worktree review makes AI changes inspectable before merge or discard. |

Anti-patterns to avoid:

- A landing page instead of the actual setup/dashboard experience.
- A terminal-first UI that asks beginners to paste commands before showing
  status.
- Raw JSON dumps from Apple `container`, provider logs, or run records.
- A generic command runner.
- Hidden lifecycle hooks or automatic repository scripts.
- Broad default internet or host-home access.
- Nested cards, decorative marketing layout, one-note purple/blue gradients,
  and large hero-style text in operational panels.

## Information Architecture

The first app should feel like a calm operational tool, not a developer portal
or marketing page.

| Screen | Purpose | Main actions |
| --- | --- | --- |
| Setup | Explain whether RunHaven can run now. | Read-only doctor, setup guidance, image status, Apple `container` status. |
| Dashboard | Show current safety and activity at a glance. | Launch run, open active run, open recent run, review maintenance warnings. |
| Launch | Turn one plain-language goal into a safe run plan. | Choose folder, choose agent, choose network goal, choose worktree/session/resources, review plan, launch. |
| Run Detail | Monitor and control one run. | View status, logs, provider policy, broker status, stop, kill, attach with warning, repair stale marker. |
| Review Changes | Decide what to do with a completed worktree run. | View changed files, run suggested checks manually later, merge, keep, discard. |
| Maintenance | Clean only RunHaven-owned resources. | Image doctor/rebuild, state reset/prune, network prune, active marker repair. |
| Settings | Configure UI-level preferences only. | Default profile, default network goal, resource warning threshold. No credentials in frontend storage. |

First-run path:

1. Show setup status immediately.
2. If doctor fails, show exact issue and plain-language next action.
3. If doctor passes but images are missing or stale, show image build/rebuild as
   the next explicit action.
4. When ready, offer "Start an Agent Run".
5. Ask for a project folder with a native folder picker.
6. Explain what will be mounted and what will not be mounted.
7. Recommend the safe network goal and worktree mode when available.
8. Show resource and access review before enabling launch.

## Beginner-Safe Copy

Use labels that describe outcomes, not implementation details:

| Technical concept | UI label |
| --- | --- |
| `runhaven doctor` | Setup check |
| Apple `container` runtime | Local container engine |
| profile | Agent |
| workspace | Project folder |
| `--workspace-scope current` | This folder only |
| `--workspace-scope git-root` | Whole repository |
| `--network internal` | Offline or local only |
| `--network provider` | AI provider only |
| `--network internet` | Full internet |
| state volume | Agent memory |
| `--session` | Named agent memory |
| worktree | Safe copy for edits |
| provider host | Allowed website for the agent |
| blocked host | Website the agent tried to reach |
| `runs kill` | Force stop |
| `runs repair` | Clean stale status |

Example warnings:

- Full internet: "This lets the agent reach the internet from inside the
  container. Use this for package installs or tools that need broad network
  access."
- Sensitive folder: "This folder may contain private files. The agent can read
  files inside the selected project folder."
- Environment variable: "RunHaven will pass the variable name you chose into
  the run. The UI never shows or stores its value."
- Root user: "The agent will run as root inside the container. This does not
  make it root on your Mac, but it weakens the container's normal guardrails."
- Worktree discard: "This removes the RunHaven-created worktree and branch. It
  does not delete your source project folder."
- SSH unavailable: "SSH forwarding is currently disabled because the verified
  Apple `container` path does not work for the non-root agent user."

## Command Contract

Use typed Tauri commands with request and response structs. Command names below
are planning names; exact Rust names can change during implementation if the
same contract remains explicit.

Read-only commands:

| Command | Request | Response | Backing RunHaven behavior |
| --- | --- | --- | --- |
| `get_setup_status` | none | `SetupStatus` | `doctor`, setup guidance, runtime pins, Apple `container` status. |
| `list_agents` | none | `AgentProfile[]` | bundled profile metadata. |
| `get_dashboard_status` | none | `DashboardStatus` | active runs, recent runs, image summaries, state/network summaries. |
| `explain_workspace` | `WorkspaceExplainRequest` | `WorkspaceExplanation` | `why workspace`. |
| `explain_network` | `NetworkExplainRequest` | `NetworkExplanation` | `why network`. |
| `explain_state` | `StateExplainRequest` | `StateExplanation` | `why state`. |
| `explain_provider_host` | `ProviderHostExplainRequest` | `ProviderHostExplanation` | `why host`. |
| `plan_run` | `RunPlanRequest` | `RunPlanResponse` | existing planner and validators. |
| `get_image_status` | `ImageStatusRequest` | `ImageStatus` | `image doctor`. |
| `list_runs` | `ListRunsRequest` | `RunSummary[]` | `runs list/show/log` data readers. |
| `get_run_detail` | `RunDetailRequest` | `RunDetail` | run record plus policy and auth events. |
| `get_active_run_status` | `RunIdRequest` | `ActiveRunStatus` | `runs status`. |
| `get_logs_snapshot` | `RunIdRequest` | `LogSnapshot` | bounded log snapshot. |
| `get_maintenance_status` | none | `MaintenanceStatus` | state list, network list, active markers, image status. |

Mutating commands:

| Command | Request | Response | Confirmation field |
| --- | --- | --- | --- |
| `build_image` | `ImageBuildRequest` | `TaskStarted` or `ImageBuildResult` | `confirm_build` |
| `launch_run` | `LaunchRunRequest` | `RunStarted` | `confirm_launch` |
| `stop_run` | `RunControlRequest` | `RunControlResult` | `confirm_stop` |
| `kill_run` | `RunControlRequest` | `RunControlResult` | `confirm_force_stop` |
| `attach_run` | `AttachRequest` | `AttachStarted` | `confirm_attach` |
| `repair_run` | `RepairRunRequest` | `RepairResult` | `confirm_repair` |
| `merge_worktree` | `RunIdRequest` | `WorktreeResult` | `confirm_merge` |
| `discard_worktree` | `RunIdRequest` | `WorktreeResult` | `confirm_discard` |
| `reset_state` | `StateResetRequest` | `StateResetResult` | `confirm_reset` |
| `prune_state` | `StatePruneRequest` | `StatePruneResult` | `confirm_prune` |
| `prune_networks` | `NetworkPruneRequest` | `NetworkPruneResult` | `confirm_prune` |

Contract rules:

- Every request and response derives `Serialize` or `Deserialize` as needed.
- The Rust command layer reuses existing validators for profile, session,
  workspace, network, provider host, resource, run id, image, and confirmation
  fields.
- Frontend state may store display summaries, selected UI options, and run ids.
  It must not store token values, env values, prompts, command lines, raw Apple
  inspect payloads, raw logs beyond visible UI buffers, or workspace file
  contents.
- Log streaming should use bounded snapshots first. Add Tauri events or channels
  only after the run detail screen needs live streaming.
- Long-running operations need progress state, cancellation behavior, and a
  final result. Do not block the WebView event loop.

## Capability Map

Use individual capability files and explicitly reference them in `tauri.conf.*`.
Do not rely on every file in `src-tauri/capabilities/` being enabled by
default.

| Capability | Window | Allows | Does not allow |
| --- | --- | --- | --- |
| `main-read` | main | setup status, dashboard status, agents, plan, explanations, image status, run summaries, maintenance status | launch, stop, kill, attach, reset, prune, merge, discard |
| `folder-pick` | main launch step | dialog open for one directory selection | arbitrary filesystem read/write |
| `launch-run` | main launch step | `launch_run` after reviewed plan and confirmation | generic shell, generic process, arbitrary command |
| `run-control` | run detail | stop, kill, attach, repair for one validated run id | broad process control, arbitrary Apple `container` commands |
| `worktree-review` | review changes | merge, keep, discard for one validated RunHaven worktree run | raw git command bridge |
| `maintenance` | maintenance | image build/rebuild, state reset/prune, network prune with exact target confirmation | Apple `container system stop`, install/update/uninstall, registry credentials |

Initial plugin permissions:

- `dialog` permission scoped only to directory selection for the main window.
- No shell plugin.
- No filesystem plugin.
- No HTTP plugin.
- No store plugin.
- No updater plugin.
- No clipboard, notification, opener, tray, single-instance, SQL, stronghold, or
  websocket plugin in the first pass.

## First Scaffold And Launch Slice Acceptance Criteria

The first scaffold and first launch slice were acceptable only while all of
these remained true, and future desktop slices should keep the same guardrails
unless `docs/V1_RELEASE_PLAN.md` records a narrower release decision:

1. `docs/TAURI_UI_RESEARCH_PLAN.md` remains the accepted research source.
2. `docs/TAURI_UI_GUARDRAILS.md` is still accurate.
3. Version checks are rerun and exact stable pins are recorded.
4. `package.json` uses exact versions and `package-lock.json` is checked in.
5. `src-tauri/Cargo.toml` uses exact versions and `Cargo.lock` is updated.
6. Tauri capabilities are committed with the scaffold, and generated Tauri
   schema/permission artifacts stay out of source control.
7. The first UI renders setup status and dashboard state without mutating
   RunHaven resources.
8. Folder picker output is revalidated in Rust before being used.
9. The launch wizard cannot call `launch_run` until the plan is reviewed, the
   user confirms launch, and every warning is acknowledged.
10. `launch_run` uses existing Rust validators, blocks when setup checks fail,
    starts work through the shared runtime launch path, and has its own
    `launch-run` capability.
11. Frontend tests cover setup, dashboard, plan, warning, preview, and launch
    confirmation states.
12. Rust command tests cover invalid input, missing confirmation, and denied
    capability assumptions where testable.
13. Verification includes `cargo fmt --check`, `cargo test --locked`, frontend
    typecheck/unit/browser/build checks, Tauri Rust checks, local Markdown link
    check, pin policy, and `git diff --check`.

The scaffold used the then-current stable equivalent of:

```bash
npm create vite@latest ui -- --template svelte-ts
npm --prefix ui install --save-exact
```

Tauri was added manually so the repo keeps the planned `ui/` plus
`src-tauri/` layout.

## Deferred

- Do not add a generic shell, process, filesystem, HTTP, or Apple `container`
  bridge.
- Do not add a daemon. The existing state-volume lock is sufficient until
  concrete concurrency problems prove otherwise.
- Do not add updater, signing, notarization, release publication, registry
  login, or installer automation in the first UI phase.
- Do not store credentials, env values, prompts, command lines, request bodies,
  raw logs, raw inspect payloads, or workspace file contents in frontend
  storage.
- Do not expose raw SSH-key mounting as an alternative to current disabled SSH
  forwarding.
- Do not add remote content to a privileged WebView.
- Do not add extension or MCP configuration UI until the deny-by-default
  extension boundary moves from policy to implementation.
- Do not use a UI component framework or state library unless a concrete screen
  proves local Svelte state and small components are insufficient.

## Milestones For The UI Implementation Phase

The early scaffold and launch milestones below are now historical status for
the alpha desktop. The active v1 desktop milestone sequence is in
[`V1_RELEASE_PLAN.md`](V1_RELEASE_PLAN.md).

1. Scaffold only:
   - add `ui/`, `src-tauri/`, exact pins, lockfiles, and no mutating commands;
   - verify static app opens and reads a mocked setup state.
2. Read-only Rust command layer:
   - implement setup, dashboard, agents, plan, image status, run summaries, and
     maintenance status commands;
   - test invalid inputs and serialization contracts.
3. First usable app shell:
   - build setup, dashboard, launch wizard, and maintenance read-only views;
   - verify text fits on mobile-sized and desktop windows.
4. Launch planning:
   - wire native folder selection and `plan_run`;
   - show secure default choices and warning-required advanced choices.
5. Mutating controls:
   - launch is implemented as the first slice;
   - add stop, kill, repair, image build, state reset, network prune,
     worktree merge, and worktree discard one at a time with confirmation tests.
6. Visual and accessibility pass:
   - keyboard paths, focus states, labels, contrast, reduced motion, responsive
     widths, and non-overlapping text.

## Decision Log

| Decision | Why |
| --- | --- |
| Svelte + Vite over React | Smaller first-pass code and fewer dependencies matter more than React's ecosystem for this focused local tool. |
| npm over pnpm | npm is installed locally and avoids adding package-manager setup for contributors. |
| Svelte direct Vite over SvelteKit | Tauri needs static assets, and the first UI does not need app-framework routing or server-oriented features. |
| Root Rust library remains source of truth | Prevents two behavior surfaces and keeps CLI and UI policy aligned. |
| Dialog plugin only | Folder selection needs native UX; other plugins are unnecessary for first scaffold. |
| Warnings over hiding supported advanced paths | The user should be able to choose advanced behavior intentionally, while the easy path remains safe. |
| No daemon yet | The existing state-volume lock already protects the known concurrent-volume failure mode. |

## Research Outcome

This research phase is complete when the repo records:

- source-backed Tauri v2 architecture decision: complete;
- frontend framework recommendation: complete, Svelte + Vite + TypeScript;
- first-pass information architecture: complete;
- beginner-safe UX flows for setup, launch, review, recovery, and cleanup:
  complete;
- typed Rust command contract: complete;
- initial capability map: complete;
- comparable product patterns and anti-patterns: complete;
- first scaffold acceptance criteria: complete.
