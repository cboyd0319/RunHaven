# Non-UI Backlog

Last updated: 2026-06-27

Status: durable backlog for CLI-complete, runtime, evidence, and product-scope
work that is not direct Tauri/UI implementation. The 2026-06-26 directive made
this the near-term backlog ahead of broad UI work; the 2026-06-27 directive
pulled the TUI forward as the active reference implementation. Keep this file
for non-UI product/security candidates and recurring runtime evidence gates.

RunHaven's Rust CLI is the current product core. This file keeps remaining
non-UI work explicit so product/security slices can be promoted deliberately
without weakening CLI semantics or default safety.

RunHaven remains alpha/pre-release after the `v0.5.0` CLI-only pre-release. The
post-`v0.5.0` contract-preserving guard (bug fixes, security fixes, pin updates,
doc corrections, internal UI support only) may be relaxed only for deliberately
promoted non-UI product candidates below: promote one at a time through its
design gate, preserving CLI semantics and default safety, rather than treating
it as cleanup.

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
| CLI command and docs contract | done (2026-06-26) | Confirm `setup`, `doctor`, `agents`, `plan`, `run`, image, network, state, auth, why, egress, runs, and worktree docs match current behavior. | Audited the 14-command tree against clap help, `CLI_SURFACE_COVERAGE.md`, and `USAGE.md`: added the missing `agents` USAGE section, made `state reset`/`prune` deletion existence-aware (graceful on a missing volume) plus a retry for transiently-held volumes, and updated the surface check to exercise the session-volume reset path with `--auth-scope project`. `cli_surface_check.sh` 39/39. |
| JSON and local data lifecycle decision | done (2026-06-26) | Decide which CLI JSON outputs and local record files are stable, schema-versioned, or explicitly best-effort. | Decision recorded in `docs/V1_RELEASE_PLAN.md` (Data, Storage, And Recovery) with a pointer in `USAGE.md`: through `v0.5.0` all `--json` outputs and record files are best-effort and unversioned; stability requires an explicit `schema_version` field first. |
| Profile support tiers | done (2026-06-26) | Distinguish bundled image availability, basic CLI starts, provider mode support, interactive auth path, and brokered auth. | `runhaven agents` now prints the code-derived tier summary (sign-in path, default network, API-key broker) and the `CAPABILITIES.md` matrix carries the source-backed detail. |
| CLI maintainability check | done (2026-06-26) | Avoid large-file, duplication, crate-organization, or dependency debt before desktop work scales. | Reviewed the session's touched modules: deleted dead `ApiKeyBrokerProxy::bind`, deduped `logout_shared_volume` onto the shared existence-aware `delete_volume`. `egress.rs` (525), `auth_broker.rs` (512), `app.rs` (508) are slightly over the ~500 guard but cohesive command/boundary surfaces (leave whole); `egress.rs` has a policy-vs-proxy seam to split only if it grows. |

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
| Signed auto-updating provider policy | candidate | Can the per-agent egress allowlist ship as signed, auto-updating data (antivirus-definitions model) so new provider endpoints need no release or user action? | Builds on the shipped domain-family wildcard patterns (`*-name.domain.tld`, anchored to one registrable domain, default-deny preserved, used for Antigravity). Must keep default-deny and verify the policy signature; users still manage no hosts. |
| Custom profile file support | candidate | What profile schema gives power users flexibility without bypassing default safety? | Must preserve pinned images, explicit env, workspace, network, and state boundaries. |
| Per-agent policy presets | candidate | Which defaults are safe for each agent without hiding risk? | Depends on profile schema and provider endpoint evidence. |
| MCP allowlists and extension support | candidate | Which MCP or extension surfaces are safe enough to expose? | Boundary policy exists in `docs/EXTENSION_MCP_BOUNDARY.md`; implementation is not started. |
| Import/export of project profiles | candidate | What portable profile data can be shared without secrets or machine paths? | Depends on custom profile schema. |
| Devcontainer metadata import | candidate | Can RunHaven recommend image/workspace settings from `devcontainer.json` without running host lifecycle hooks? | Host hooks must stay disabled unless explicitly approved. |
| Offline/package-install network modes | candidate | Is a separate mode clearer than current `internal`, `provider`, and `internet` choices? | Needs UX research and command semantics. |
| Additional provider auth-flow smokes | candidate | Which agent/provider login paths need source-backed live proof? | Keep optional and disposable; never require real user secrets. |
| Central log sanitizer and Keychain-backed RunHaven secrets | planned | What shared sanitizer and macOS Keychain storage layer prevent token leakage without weakening the no-host-credential-read boundary? | User-requested 2026-06-27. Treat agent/container stdout/stderr and provider CLI output as untrusted. Redact Authorization headers, bearer/API/OAuth tokens, URL token parameters, provider config snippets, and accidental secret prints before CLI/Tauri/TUI/log records/support output. Because RunHaven is macOS 26+ only, prefer Keychain for host-held RunHaven secrets such as Claude setup tokens and future broker-owned secrets. Do not read provider-owned Keychain entries, browser profiles, cloud credential stores, or arbitrary host credentials. Tracked as `logs-secret-redaction` in feature_list.json. |
| Local proxy option for model credentials | done (API-key path) | Can model credentials stay host-owned while the guest receives only narrow provider access? | Built for API keys: the Codex/Claude/Gemini API-key broker keeps the key host-side (multi-provider-broker slice). |
| OAuth/subscription isolated-login UX | login done; auto-updating policy remaining (2026-06-26) | Make in-container OAuth login smooth without a broker, and keep provider endpoints current without a release. | Done: `runhaven login <agent>` is built and live-verified for Claude (host `claude setup-token`), Codex (`codex login --device-auth`), Copilot (`copilot login`), and Antigravity (agy first-run Google OAuth). `--clear` removes the stored or shared login. Once-per-agent reuse via `--auth-scope agent` (default). Gemini stays on the API-key broker (no OAuth login). Tracked as `oauth-isolated-login` in feature_list.json. Decision: no host-side OAuth broker (provider terms forbid relaying subscription/OAuth credentials; subscription tokens are not drop-in bearers, different host or client impersonation; brokering would cross the read-host-creds boundary). Login and token-refresh hosts are allowlisted per agent (`auth.openai.com` for Codex; `github.com` and `api.github.com` for Copilot, a documented egress widening). Remaining: ship the per-agent provider policy as signed, auto-updating data so new provider endpoints need no release or user action (see the candidate row below). See `docs/AUTH_BROKER.md`. |
| Strict workflow files | candidate | What schema allows repeatable setup/main/teardown inside Apple `container` without host-side surprises? | Reject unknown fields; persist workflow hash and state. |
| Read-only context overlays | candidate | What docs, skills, prompts, or project memory can be mounted read-only without exposing host secrets? | Prefer explicit overlays over host-home mounts. |
| Shared planner/policy objects | candidate | Which CLI planning data should become reusable by future Rust API and UI commands? | Avoid duplicating parser, docs, and UI state logic. |
| On-device natural-language help assistant | far backlog | Could RunHaven answer help questions inside the Codex-vendored TUI using a local Apple on-device model instead of shipping dense help screens? | Investigation only. Verify current Apple on-device model APIs, Codex model-provider extension points, privacy boundaries, offline behavior, and fallback docs before any build. Keep the answer path local by default if the design proves viable. Source note: see the Codex and Apple Foundation Models PDF in `docs/plans/`. |

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
