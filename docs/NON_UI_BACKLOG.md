# Non-UI Backlog

Last updated: 2026-06-26

Status: durable backlog for CLI-complete, runtime, evidence, and product-scope
work that is not direct Tauri/UI implementation. Per the 2026-06-26 directive
that defers all GUI/UI work to the very end, this is now the primary near-term
backlog: runtime/security hardening leads, then promotion of design-first
non-UI product candidates one at a time, then a CLI-based public release.

RunHaven's Rust CLI is the current product core. This file keeps remaining
non-UI work explicit so the CLI product can be hardened and rounded out before
the deferred desktop/TUI surfaces.

RunHaven remains alpha/pre-release until after `v0.5.0`. The post-`v0.5.0`
contract-preserving guard (bug fixes, security fixes, pin updates, doc
corrections, internal GUI support only) is relaxed by this directive for
deliberately promoted non-UI product candidates below: promote one at a time
through its design gate, preserving CLI semantics and default safety, rather
than treating it as cleanup.

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
| Local proxy option for model credentials | done (API-key path) | Can model credentials stay host-owned while the guest receives only narrow provider access? | Built for API keys: the Codex/Claude/Gemini API-key broker keeps the key host-side (multi-provider-broker slice). |
| OAuth/subscription isolated-login UX | candidate (researched 2026-06-26) | Make in-container OAuth login smooth without a broker. | Decision: no host-side OAuth broker (provider terms forbid relaying subscription/OAuth credentials; subscription tokens are not drop-in bearers, different host or client impersonation; brokering would cross the read-host-creds boundary). Work: allowlist each provider's OAuth login and token-refresh hosts (verify each against the live CLI first), confirm a headless device-code or paste-code login completes in a container, and confirm token persistence in the `/home/agent` volume. Note: Gemini's free Code Assist OAuth path stopped serving 2026-06-18, so Gemini account login may need a paid key. See `docs/AUTH_BROKER.md`. |
| Strict workflow files | candidate | What schema allows repeatable setup/main/teardown inside Apple `container` without host-side surprises? | Reject unknown fields; persist workflow hash and state. |
| Read-only context overlays | candidate | What docs, skills, prompts, or project memory can be mounted read-only without exposing host secrets? | Prefer explicit overlays over host-home mounts. |
| Shared planner/policy objects | candidate | Which CLI planning data should become reusable by future Rust API and UI commands? | Avoid duplicating parser, docs, and UI state logic. |

## Borrowed Ideas From Competitive Scan (2026-06-26)

A landscape scan of similar agent-sandbox projects surfaced these candidates.
Each still requires the design gate above before implementation; provenance is
recorded for evidence, not endorsement. Source clones for inspection live under
`~/Documents/GitHub/` (`sand`, `container-use`, `agent-sandbox`).

| Idea | Source | Fit / Extends | Risk Note |
| --- | --- | --- | --- |
| Proxy-side credential injection (guest never holds the API key) | `agent-sandbox` (MIT) | Extends "Local proxy option for model credentials"; the host CONNECT proxy injects secrets into matched egress. | Highest-value security upgrade; verify current Codex broker key exposure first. |
| Profile / AgentRequirements intersection resolver | `sand` (Apache-2.0) | Extends "Custom profile file support"; agent declares needs, profile declares permits, resolver hands the minimum, profiles never persist secrets. | Must preserve pinned images and default mount/network/state boundaries. |
| In-box eBPF/DNS egress allowlist (default-on) | `sand` | Defense in depth under the host CONNECT proxy; the only real lever for the 2026-06-26 audit finding #1 (a hostOnly guest can raw-TCP host services on the gateway, since Apple `container` 1.0.0 has no per-port guest-to-host firewalling). | Keep default-on; `sand` ships it opt-in, which RunHaven should not copy. Heavyweight: needs `sand`'s custom init image plus a BTF-enabled kernel, incompatible with the pinned stock runtime, so design-first only. |
| Sanitized read-only policy file inside the box | `agent-sandbox` | Lets the agent read the effective allowlist/mounts and self-correct on a blocked host. | Must stay metadata-only: no secrets, no host paths. |
| APFS copy-on-write workspace clone | `sand` | Instant disposable workspace, host dir never mutated, free discard of a failed run. | Apple-silicon/APFS-native; verify interaction with the read-only workspace mount model. |
| Fine-grained PAT + git-askpass shim | `agent-sandbox` | Narrowly scoped token via askpass instead of mounting `~/.ssh` or full credentials. | Preserve the no-raw-SSH-key default. |

Anti-patterns confirmed out of scope (peers do these; RunHaven already refuses
them): privileged/nested containers, open egress by default, host-home or
credential-store mounts, and documenting an insecure escape hatch as the fix.

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
