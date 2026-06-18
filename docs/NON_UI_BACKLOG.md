# Non-UI Backlog

Last updated: 2026-06-18

Status: durable backlog for CLI-complete, runtime, evidence, and product-scope
work that is not direct Tauri/UI implementation.

RunHaven's Rust CLI is the current product core. This file keeps remaining
non-UI work explicit so the `v0.5.0` CLI-complete milestone can close before
broad `v1.0.0` desktop expansion.

RunHaven remains alpha/pre-release until after `v0.5.0`. After `v0.5.0`, new
CLI product features should be avoided unless they are bug fixes, security
fixes, release pin updates, documentation corrections, or internal GUI support
that preserves CLI semantics.

## Ongoing Runtime Evidence Gates

These started as pre-UI gates. UI work has started, so keep them as recurring
runtime evidence gates before broadening CLI claims or desktop launch,
run-control, image, state, cleanup, worktree, or network surfaces.

| Item | Status | Why It Matters | Action | Done When |
| --- | --- | --- | --- | --- |
| Fresh Apple `container` default smoke | recurring before broader UI runtime controls | Unit tests cannot prove installed Apple `container` runtime behavior or JSON shapes. | Run `scripts/apple_container_smoke.sh` on macOS 26+ with Apple `container` 1.0.0. | Smoke exits 0 and cleanup evidence shows no unexpected active runs, state volumes, or managed networks. |
| Fresh provider-mode smoke | recurring before provider-sensitive UI changes | Provider mode depends on host-only networking, gateway/subnet inspect output, proxy binding, egress denial, and cleanup. | Run `scripts/apple_container_smoke.sh --with-provider`. | Allowed provider HTTPS works, denied proxy/direct egress fails, and no provider network is left behind. |
| SSH forwarding decision | blocked | Apple `container --ssh` exposes a socket, but the default non-root guest cannot use it on the current pinned runtime. | Keep `--ssh` fail-closed unless a no-secret smoke proves `ssh-add -l` from the non-root guest. | Either documented as intentionally unsupported for the UI, or re-enabled with tests/docs after a passing no-secret runtime proof. |
| Final local verification pass | recurring before commits that broaden runtime control | Desktop work should build on a clean, verified CLI core. | Run `./init.sh` or the smallest equivalent complete check set for the changed surface, JSON validation, Markdown link check, maintainability review for touched files, and `git diff --check`. | All relevant checks pass and current-state evidence is updated. |

## v0.5.0 CLI-Complete Closure

| Item | Status | Scope | Smallest Next Step |
| --- | --- | --- | --- |
| CLI command and docs contract | planned | Confirm `setup`, `doctor`, `agents`, `plan`, `run`, image, network, state, auth, why, egress, runs, and worktree docs match current behavior. | Audit `docs/USAGE.md`, CLI help, and focused tests before tagging `v0.5.0`. |
| JSON and local data lifecycle decision | planned | Decide which CLI JSON outputs and local record files are stable, schema-versioned, or explicitly best-effort. | Record the decision in `docs/V1_RELEASE_PLAN.md`, `docs/USAGE.md`, or a focused data-lifecycle doc only if needed. |
| Profile support tiers | planned | Distinguish bundled image availability, basic CLI starts, provider mode support, interactive auth path, and brokered auth. | Add the support matrix to active docs before `v0.5.0`. |
| CLI maintainability check | planned | Avoid large-file, duplication, crate-organization, or dependency debt before desktop work scales. | Review touched CLI modules against `docs/harness/state/modularization-plan.md` and update state with findings. |

## Accepted Non-UI Polish

| Item | Status | Scope | Smallest Next Step |
| --- | --- | --- | --- |
| Image/state/network repair polish | accepted | Keep repair commands clear, exact, and limited to RunHaven-owned resources. | Review `docs/harness/research/ux-research-ideas.md` for unresolved repair UX gaps and either implement one focused gap or retire it with evidence. |

## Candidate Work Requiring Design First

Do not implement these as cleanup. Promote one item at a time only after the
problem, user outcome, security boundary, and verification are clear.

| Item | Status | Design Question | Notes |
| --- | --- | --- | --- |
| Real-agent effectiveness evidence | candidate | Which representative tasks prove RunHaven helps real users without overclaiming structural quality? | Define tasks and scoring before automation. |
| Path-aware provider host policy | candidate | Can broad hosts such as `github.com` be constrained by verified path policy or brokered credentials? | A plain CONNECT proxy cannot inspect TLS paths. Prefer brokered or source-backed designs. |
| Custom profile file support | candidate | What profile schema gives power users flexibility without bypassing default safety? | Must preserve pinned images, explicit env, workspace, network, and state boundaries. |
| Per-agent policy presets | candidate | Which defaults are safe for each agent without hiding risk? | Depends on profile schema and provider endpoint evidence. |
| MCP allowlists and extension support | candidate | Which MCP or extension surfaces are safe enough to expose? | Boundary policy exists in `docs/EXTENSION_MCP_BOUNDARY.md`; implementation is not started. |
| Import/export of project profiles | candidate | What portable profile data can be shared without secrets or machine paths? | Depends on custom profile schema. |
| Devcontainer metadata import | candidate | Can RunHaven recommend image/workspace settings from `devcontainer.json` without running host lifecycle hooks? | Host hooks must stay disabled unless explicitly approved. |
| Offline/package-install network modes | candidate | Is a separate mode clearer than current `internal`, `provider`, and `internet` choices? | Needs UX research and command semantics. |
| Additional provider auth-flow smokes | candidate | Which agent/provider login paths need source-backed live proof? | Keep optional and disposable; never require real user secrets. |
| Local proxy option for model credentials | candidate | Can model credentials stay host-owned while the guest receives only narrow provider access? | Current Codex API-key broker is the prototype. |
| Strict workflow files | candidate | What schema allows repeatable setup/main/teardown inside Apple `container` without host-side surprises? | Reject unknown fields; persist workflow hash and state. |
| Read-only context overlays | candidate | What docs, skills, prompts, or project memory can be mounted read-only without exposing host secrets? | Prefer explicit overlays over host-home mounts. |
| Shared planner/policy objects | candidate | Which CLI planning data should become reusable by future Rust API and UI commands? | Avoid duplicating parser, docs, and UI state logic. |

## Deferred Until v1 Desktop Packaging

These are not part of `v0.5.0` CLI-complete scope, but signed/notarized
desktop artifacts and provenance are required before calling the desktop release
`v1.0.0`.

| Item | Status | Notes |
| --- | --- | --- |
| Signing, notarization, SBOM, provenance, installer, and publication automation | v1 packaging | Required for the `v1.0.0` desktop release artifact, except automatic updater support can remain a v1.x feature if release notes state manual updates clearly. |

## Source Links

- Apple Container gap analysis: `docs/APPLE_CONTAINER_GAP_ANALYSIS.md`
- Product roadmap: `docs/ROADMAP.md`
- Release gap analysis: `docs/RELEASE_GAP_ANALYSIS.md`
- Harness operations: `docs/harness/README.md`
- Tauri UI guardrails: `docs/TAURI_UI_GUARDRAILS.md`
- Extension and MCP boundary: `docs/EXTENSION_MCP_BOUNDARY.md`
