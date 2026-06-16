# Tauri UI Research Plan

Last updated: 2026-06-16

Status: research phase, no scaffold yet.

Goal: design the easiest safe desktop experience for people with little or no
technical background to run AI coding agents inside Apple `container`.

## Research Outcome

Before any Tauri scaffold is added, this phase should produce:

- a source-backed Tauri v2 architecture decision;
- a frontend framework recommendation;
- a first-pass information architecture;
- beginner-safe UX flows for setup, run launch, review, recovery, and cleanup;
- a typed Rust command contract for the WebView boundary;
- a short list of comparable products and reusable UI patterns;
- explicit "do not build" decisions for risky or confusing surfaces.

## Current Constraints

- RunHaven is macOS 26+ and Apple silicon only.
- The CLI is the source of truth for runtime behavior.
- Tauri WebViews must be treated as untrusted UI.
- No generic shell, filesystem, process, network, or Apple `container` bridge is
  allowed from JavaScript.
- Mutating operations need explicit confirmation.
- The UI must explain resource impact before launch: active runs, CPU, memory,
  workspace, state volume, image status, builder status, and network mode.
- `--ssh` remains fail-closed until Apple `container` non-root forwarding is
  proven by a no-secret smoke.

## Primary Sources To Use

Current source review should start with these sources and record the review date
in this file or a follow-on research note:

| Area | Source |
| --- | --- |
| Tauri project creation and official templates | <https://v2.tauri.app/start/create-project/> |
| Tauri frontend integration | <https://v2.tauri.app/start/frontend/> |
| Tauri capabilities | <https://v2.tauri.app/security/capabilities/> |
| Tauri permissions | <https://v2.tauri.app/security/permissions/> |
| Tauri Rust command IPC | <https://v2.tauri.app/develop/calling-rust/> |
| Vite templates and frontend tooling | <https://vite.dev/guide/> |
| React with Vite | <https://react.dev/learn/build-a-react-app-from-scratch> |
| Svelte and SvelteKit | <https://svelte.dev/docs/svelte/getting-started> |
| SvelteKit with Tauri | <https://v2.tauri.app/start/frontend/sveltekit/> |
| Docker Desktop UX baseline | <https://docs.docker.com/desktop/> |
| Podman Desktop UX baseline | <https://podman-desktop.io/docs/discover-podman-desktop> |
| DevPod workspace UX baseline | <https://devpod.sh/docs/developing-in-workspaces/create-a-workspace> |

## Framework Candidates

The research decision should compare at least these options:

| Candidate | Why Consider It | Main Risk To Evaluate |
| --- | --- | --- |
| SvelteKit SPA/static or Svelte + Vite | Small app code, simple component model, official Tauri/SvelteKit guidance, good fit for local desktop UI. | Need confirm routing, accessibility primitives, testing, and component-library choices do not add complexity. |
| React + Vite | Largest ecosystem, strongest accessible component options, easiest to hire for and maintain with common tooling. | More dependency gravity and more ways to overbuild a simple desktop tool. |
| Solid + Vite | Low overhead and fine-grained reactivity for desktop UI. | Smaller ecosystem and maintainability risk for future contributors. |
| Vue + Vite | Mature, approachable, official Vite path. | Must justify over Svelte or React for this product's team and component needs. |

Initial bias: shortlist Svelte + Vite or React + Vite. Choose Svelte if the
research values small, direct app code most. Choose React if the research values
accessible component ecosystem and long-term contributor familiarity most.

## Comparable UI Analysis

Study these products for patterns, not feature parity:

| Product | Pattern To Study | RunHaven Translation |
| --- | --- | --- |
| Docker Desktop | One-click local setup, container/image/resource dashboards, visible lifecycle actions. | RunHaven dashboard should show doctor status, images, active runs, state, and safe next actions without terminal commands. |
| Podman Desktop | Environment visualization, container logs, terminal access, start/stop/delete actions, extension boundaries. | RunHaven should make active run status, logs, stop/kill/repair, and provider egress state visible with confirmation gates. |
| DevPod Desktop | Workspace creation from local path or repo, provider selection, recreate/reset confirmations, devcontainer guessing. | RunHaven launch flow should start with a project folder, explain inferred choices, and avoid running lifecycle hooks automatically. |
| GitHub Desktop | Review-centered workflows for branches, diffs, checks, and safe local changes. | RunHaven should make worktree results reviewable before merge/discard and keep undo paths obvious. |

## Beginner-Safe UX Questions

Answer these before scaffold work:

1. What is the first screen for a non-technical user with no images built and
   Apple `container` not started?
2. How does the UI explain "choose a project folder" without exposing the home
   directory or credential folders?
3. How does the UI choose between local-only, provider-only, and unrestricted
   internet without jargon?
4. How does the UI warn about VM resources before launching or rebuilding?
5. How does the UI show what the agent can touch, what it cannot touch, and how
   to undo work?
6. How does the UI explain blocked provider hosts and suggest the next safe
   action without encouraging broad egress?
7. What operations are read-only, confirmation-required, or blocked in the UI?
8. What should the UI do when Apple `container`, image build, provider proxy,
   cleanup, or run repair fails?
9. Which data is safe to store in frontend state, and which data must stay only
   in Rust or the CLI cache?

## Proposed Research Milestones

1. Current Tauri v2 source review:
   - Verify scaffold, config, command, capability, permission, state, sidecar,
     build, and test guidance from official Tauri docs.
   - Output: source notes and implementation constraints.
2. Frontend framework comparison:
   - Compare Svelte, React, Solid, and Vue against accessibility, maintainable
     component patterns, test tooling, Tauri friction, bundle simplicity, and
     available UI primitives.
   - Output: recommendation and rejected alternatives.
3. Comparable product UX review:
   - Review Docker Desktop, Podman Desktop, DevPod Desktop, and GitHub Desktop.
   - Output: pattern matrix and anti-pattern list.
4. RunHaven information architecture:
   - Define first-run setup, dashboard, launch flow, run detail, review changes,
     provider-host review, image/state/network maintenance, and settings.
   - Output: navigation map and screen responsibilities.
5. Command contract planning:
   - Map each screen to typed Rust commands and capability files from
     `docs/TAURI_UI_GUARDRAILS.md`.
   - Output: command/request/response inventory with read-only and mutating
     gates.
6. UX copy and failure-state research:
   - Draft plain-language labels, warnings, and recovery states for
     non-technical users.
   - Output: copy deck or annotated wireframe notes.

## Do Not Start Yet

- Do not scaffold `src-tauri/`.
- Do not add JavaScript dependencies.
- Do not add a generic command bridge.
- Do not wire mutating operations before the read-only dashboard contract is
  reviewed.
- Do not enable updater, signing, registry, machine-management, or install
  flows in the first UI pass.
