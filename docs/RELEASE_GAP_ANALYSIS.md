# RunHaven v0.5.0 And v1.0.0 Gap Analysis

Last updated: 2026-06-27

Status: active release-gap tracker.

## Purpose

This document turns the release ladder in
[`V1_RELEASE_PLAN.md`](V1_RELEASE_PLAN.md) into an actionable gap analysis.
It is not a second roadmap. Use it to decide whether work is:

- blocking `v0.5.0`;
- blocking `v1.0.0`;
- acceptable as a v1.x follow-up.

RunHaven remains alpha/pre-release. `v0.5.0` is the CLI-complete pre-release
already cut.

Sequencing update: the 2026-06-26 directive deferred GUI/UI work to the end of
the roadmap. On 2026-06-27, the terminal UI was pulled forward as the reference
TUI for sibling projects, and its build plan is complete in this checkout. The
v1 desktop gap rows below stay valid for the later desktop phase; they are
re-sequenced, not removed, and the desktop release version label is no longer
locked to `v1.0.0`. The sequence of record is `current-state.md` and
`docs/ROADMAP.md`.

Above all else, the secure path must be the easy path. Secure defaults should
be the shortest workflow. Supported lower-security choices should warn and
require explicit intent; unsupported, invalid, or hard-boundary violations
still fail closed.
Apple `container machine` follows the same rule: it is not the default
RunHaven boundary, but explicit or user-managed machine workflows should be
warned rather than blocked solely because they are less secure.

## Evidence Used

Observed from live repo state, last refreshed on 2026-06-27:

- `feature_list.json` and `current-state.md`.
- `README.md`, `docs/ROADMAP.md`, `docs/V1_RELEASE_PLAN.md`,
  `docs/NON_UI_BACKLOG.md`, `docs/SECURITY_MODEL.md`,
  `docs/ARCHITECTURE.md`, `docs/TAURI_UI_GUARDRAILS.md`, and
  `docs/TAURI_UI_RESEARCH_PLAN.md`.
- `cargo run --locked --bin runhaven -- --help`.
- `cargo run --locked --bin runhaven -- runs --help`.
- `cargo run --locked --bin runhaven -- image --help`.
- `cargo run --locked --bin runhaven -- network --help`.
- `cargo run --locked --bin runhaven -- state --help`.
- `cargo run --locked --bin runhaven -- egress --help`.
- `cargo run --locked --bin runhaven -- auth --help`.
- `cargo run --locked --bin runhaven -- why --help`.
- Tauri command and capability scan over `src-tauri/` and `ui/src/`.
- File-size scan over Rust, Tauri, frontend, and TUI source files.

Observed CLI command families:

- top-level: `agents`, `doctor`, `setup`, `plan`, `run`, `image`, `network`,
  `state`, `runs`, `egress`, `auth`, and `why`;
- image: `build`, `rebuild`, and `doctor`;
- network: `list` and `prune`;
- state: `list`, `prune`, and `reset`;
- runs: completed-run review, worktree review, active-run status, attach,
  logs-follow, stop, kill, and repair;
- egress: `log`;
- auth: `status`, `explain`, and `log`;
- why: `host`, `workspace`, `network`, and `state`.

Observed desktop command families:

- implemented alpha commands: setup status, agent list, dashboard status,
  image status, run status, plan run, launch run, bounded log snapshot, stop
  run, kill run, repair run, egress log, auth status, and auth log;
- active capabilities: `main-read` (read-only status, planning, and secret-free
  diagnostics), `folder-pick`, `launch-run`, and `run-control`
  (`allow-get-log-snapshot`, `allow-stop-run`, `allow-kill-run`,
  `allow-repair-run`);
- missing first-class desktop families: image
  build/rebuild, worktree review, state cleanup, network cleanup, `why`
  explanations and blocked-host review, auth explain, maintenance actions,
  profile support matrix, and release packaging.

Observed TUI command families:

- implemented development surfaces: bare-TTY launch, fresh-cache guide, `?`/F1
  guide, agent detail, workspace
  picker, plan review, type-confirm launch, active-run dashboard, bounded log
  snapshot viewer, stop, hard-stop, stale-marker repair, run history, per-run
  diff review, egress/auth diagnostics, terminal/render capability probe, and
  TUI doctor remediation;
- Phase 5 polish completed dashboard notices, accessibility switches, light/dark
  palette selection, final snapshot coverage, and architecture finalization.

Observed maintainability pressure:

- The old custom TUI size list was superseded by the Codex vendor reset and the
  workspace crate split.
- Current source-size and crate-boundary guidance lives in
  `docs/harness/state/modularization-plan.md`.

Vendored Codex TUI files are allowed to exceed the local soft size guard while
the baseline is being integrated, but culling or splitting decisions must be
recorded. Provider/runtime files remain security
boundaries and should be split only by clear ownership.

## Release Definitions

### v0.5.0

`v0.5.0` is complete when the CLI product surface is closed, verified, and
documented. After `v0.5.0`, CLI changes should be bug fixes, security fixes,
pin updates, documentation corrections, and internal support for the desktop
that preserves CLI semantics.

`v0.5.0` does not need to be the full public desktop release. The project
remains alpha/pre-release after this milestone.

### v1.0.0

`v1.0.0` is complete only when the desktop app is the first-class safe path for
less-technical users. A source tag is not enough. The release needs a signed
and notarized macOS desktop artifact, checksums, provenance, release notes, and
clear known limits.

## v0.5.0 Gap Analysis

### Summary

As of 2026-06-24 the v0.5.0 CLI contract gaps are closed: runtime evidence is
current on macOS 27.0 (G4); CLI help and docs agree, including the previously
undocumented `--user` and `runs attach` overrides (G1); the local JSONL records
are documented as best-effort/pre-stable (G2); the profile support matrix is
published (G3); the `--ssh` fail-closed posture is consistent across docs,
behavior, and tests (G5); the CLI now prints plain-language security notices for
every lower-security choice (G6); and touched modules stay under the size guard
with no new duplication (G7). The `internet`-default question is resolved: the
default network mode is now profile-aware (provider where the agent's own hosts
are bundled, otherwise internet), so the secure path is also the default. Tagged
release notes are deferred to the release-readiness cut.

### Blocking Gaps

| ID | Gap | Current State | Done When | Verification |
| --- | --- | --- | --- | --- |
| V05-G1 | CLI command contract audit | Command families exist, but the release contract has not been signed off against help text and docs. | Every CLI command family is checked against `docs/USAGE.md`, `docs/CAPABILITIES.md`, `README.md`, and help output; missing docs or stale examples are fixed. | `cargo run --locked --bin runhaven -- <command> --help` for each family; docs link check; `git diff --check`. |
| V05-G2 | JSON and local data contract decision | `runs.jsonl`, `egress-policy.jsonl`, and `auth-broker.jsonl` exist; release docs note missing explicit schema versions. | Stable JSON surfaces are either schema-versioned or explicitly documented as best-effort/pre-stable. Local record retention and privacy behavior are documented. | Focused tests for any schema changes; JSON validation; docs check. |
| V05-G3 | Profile support matrix | Bundled images and provider metadata exist, but support claims are not yet split into tiers. | Each profile states bundled image availability, basic CLI start, provider mode source backing, interactive auth path, headless broker availability, and known limits. | Image dry-runs or builds; `image doctor`; source-backed provider review; disposable credential smokes only where claimed. |
| V05-G4 | Runtime evidence refresh | Prior Apple `container` smokes exist, but release evidence must be current for the tag. | `doctor`, default runtime smoke, provider smoke, and SSH fail-closed smoke are current for the release candidate. Cleanup evidence is recorded. | `runhaven doctor`; `scripts/apple_container_smoke.sh`; `scripts/apple_container_smoke.sh --with-provider`; `scripts/apple_container_smoke.sh --with-ssh`; cleanup commands. |
| V05-G5 | SSH fail-closed release posture | `--ssh` is blocked because non-root Apple `container` forwarding is not proven. | Docs, CLI behavior, tests, and release notes all say SSH forwarding is unavailable until no-secret non-root proof exists. | Planner/run tests; `scripts/apple_container_smoke.sh --with-ssh`; docs scan for raw-key workaround language. |
| V05-G6 | Secure-easy CLI review | Closed: the default network mode is profile-aware (provider where the agent's hosts are bundled, otherwise internet), and `plan`/`run` print plain-language security notices for every lower-security choice. | CLI docs and help make the safe path clear; supported lower-security choices are visible, warned, and explicit. Hard-boundary violations still fail closed. | CLI help/docs audit; focused validation tests for sensitive workspace, env, root, provider host, and SSH paths. |
| V05-G7 | CLI maintainability gate | No major CLI file is far beyond the size guard, but provider egress and auth broker are near it. | Touched CLI/runtime/auth files remain cohesive; no known large-file or duplication debt is intentionally deferred into v1 desktop work. | File-size scan; focused module review; Rust checks. |
| V05-G8 | Release documentation and state closure | Release ladder exists, but v0.5-specific release notes and final state are not cut. | `README.md`, roadmap, `feature_list.json`, `current-state.md`, and release notes agree on alpha/pre-release status and v0.5 CLI-complete scope. | Docs link check; stale wording scan; JSON validation. |

### Important But Deferrable From v0.5.0

| Gap | Reason To Defer |
| --- | --- |
| First-class desktop stop/kill/repair controls | v1 desktop blocker, not CLI-complete blocker. CLI already has run-control commands. |
| Desktop image build/rebuild controls | v1 desktop blocker. CLI image commands exist. |
| Desktop worktree review | v1 desktop blocker. CLI worktree lifecycle commands exist. |
| Signed/notarized desktop artifact | Required for v1.0.0, not for v0.5.0 CLI-complete. |
| New non-Codex credential brokers | Security-sensitive provider work. Keep as v1.x unless separately designed and smoked before v1. |
| Path-aware provider policy for broad hosts | A CONNECT proxy cannot inspect TLS paths. Needs broker or other provider-specific design. |

### v0.5.0 Milestones

1. CLI contract audit.
   - Files: `README.md`, `docs/USAGE.md`, `docs/CAPABILITIES.md`,
     `docs/ARCHITECTURE.md`, and CLI help output.
   - Expected result: command docs and help agree.
   - Verification: help smokes, docs link check, `git diff --check`.

2. Data contract decision.
   - Files: `docs/V1_RELEASE_PLAN.md`, `docs/USAGE.md`, and possibly a new
     data-lifecycle doc only if needed.
   - Expected result: stable vs best-effort JSON and local record retention are
     explicit.
   - Verification: focused tests if schemas change; JSON validation.

3. Profile support matrix.
   - Files: `docs/CAPABILITIES.md`, `docs/PROVIDER_ENDPOINTS.md`,
     `docs/AUTH_BROKER.md`, and `docs/USAGE.md`.
   - Expected result: every bundled profile has a clear tier and known limits.
   - Verification: image dry-runs, `image doctor`, provider/source review.

4. Runtime evidence refresh.
   - Files: `docs/harness/evidence/evidence-log.md`, `current-state.md`.
   - Expected result: Apple `container` evidence is current for the release
     candidate.
   - Verification: `runhaven doctor`, Apple container smokes, cleanup checks.

5. Maintainability closeout.
   - Files: touched source modules and `docs/harness/state/modularization-plan.md`.
   - Expected result: no CLI large-file, duplication, or crate-organization
     debt is knowingly deferred into v1.
   - Verification: file-size scan, Rust checks, maintainability note in state.

## v1.0.0 Gap Analysis

### Summary

The v1 gap is much larger than the v0.5 gap. The current desktop app proves the
initial architecture and launch path, but it is not yet the first-class safe
path. Most recovery, cleanup, diagnostics, worktree review, packaging,
accessibility, and release-trust work remains.

### Blocking Gaps

| ID | Gap | Current State | Done When | Verification |
| --- | --- | --- | --- | --- |
| V1-G1 | Desktop setup and prerequisite path | Desktop can read setup/dashboard state. | A less-technical user can understand exact missing prerequisites, image readiness, builder status, and next action without using the CLI. | Frontend tests, Tauri tests, Playwright, minimum-window review. |
| V1-G2 | Desktop image build/rebuild | Desktop shows image status but does not build or rebuild images. | Missing/stale bundled images can be rebuilt from explicit UI action with builder status and confirmation. | Tauri command tests; frontend tests; image dry-run/build smoke where appropriate. |
| V1-G3 | Desktop run control | Done: the desktop app can stop, hard-stop (`kill_run`), and repair (`repair_run`) one validated RunHaven run behind `run-control`, each confirm-gated with exact target preview. | GUI can stop, hard-stop, and repair one validated RunHaven run with exact target preview and confirmation. | Rust/Tauri command tests, capability review, Playwright, Apple container smoke for run-control paths. |
| V1-G4 | Desktop recovery after app quit/crash/host interruption | Release plan names this as unproven. | App quit, force quit, crash, sleep, reboot, stale active marker, and leftover container guidance is verified or documented in the GUI. | Manual lifecycle smokes plus focused tests for stale-marker repair guidance. |
| V1-G5 | Desktop diagnostics | Partial: egress log and auth status/log are exposed read-only behind `main-read` (`get_egress_log`, `get_auth_log`, `get_auth_status`), secret-free and without workspace paths. `why` explanations, blocked-host review, and auth explain remain CLI-only. | GUI exposes `why host/workspace/network/state`, blocked-host review, egress log summaries, auth status/explain/log, and safe next actions without raw secrets. | Tauri command tests; frontend tests; secret-output review. |
| V1-G6 | Desktop state/network cleanup | CLI has state and network cleanup; desktop cleanup is missing. | GUI previews exact RunHaven-owned state volumes and networks, then resets/prunes only after explicit confirmation. | Tauri tests, frontend tests, destructive-target validation tests. |
| V1-G7 | Desktop worktree review | CLI has diff, keep, recover, merge, discard; desktop does not. | GUI makes dirty repo choices clear before launch and supports diff/review/keep/recover/merge/discard for RunHaven-owned worktrees. | Worktree tests, Tauri tests, frontend tests, data-loss review. |
| V1-G8 | Desktop profile support tiers | Not yet a first-class UI surface. | GUI clearly distinguishes image availability, CLI start support, provider mode support, auth path, and known limits per profile. | Profile matrix docs, frontend tests, profile dry-runs/smokes. |
| V1-G9 | Accessibility and responsive UI | Not yet release-gated. | The app is keyboard navigable, readable at minimum supported window size, has visible focus states, non-overlapping text, and accessible labels. | Playwright screenshots, keyboard review, Svelte checks, manual accessibility pass. |
| V1-G10 | Desktop maintainability before feature expansion | `src-tauri/src/commands/mod.rs`, `ui/src/commands/runhaven.ts`, and `ui/src/app/App.svelte` are already over the rough size guard. | These surfaces are split by domain before adding many more desktop controls. Duplication is removed during each slice. | File-size scan, focused tests after each split, frontend/Tauri checks. |
| V1-G11 | Data/history performance | Append-only JSONL records are currently read into memory. | Long-term run, egress, and auth history can be listed or tailed without unbounded frontend or CLI memory assumptions, or limits are explicitly documented. | Focused tests or documented limits; performance smoke with generated records if implemented. |
| V1-G12 | Packaging and release trust | Tauri build exists, but v1 release artifact path is not complete. | macOS app is built, signed, notarized, checksummed, and backed by provenance. SBOM status and update policy are stated. | `npm --prefix ui run tauri:build`, signing/notarization commands, checksum/SBOM/provenance record. |
| V1-G13 | Runtime release evidence | Prior smokes exist but v1 needs current release-candidate evidence. | `./init.sh`, doctor, provider smoke, SSH fail-closed smoke, profile evidence, desktop launch/control smokes, and cleanup checks are current for the release candidate. | Full release gate commands from `V1_RELEASE_PLAN.md`. |
| V1-G14 | Public docs alpha-to-release transition | Docs now say alpha through v0.5, but v1 release docs need final stable wording. | User docs replace alpha warnings with precise stable/unsupported/known-limited surfaces only when v1 gates pass. | Docs stale scan, link check, release notes review. |

### v1.x Follow-Ups, Not v1.0.0 Blockers

| Follow-Up | Reason |
| --- | --- |
| New Claude, Gemini, Copilot, or Antigravity credential brokers | Provider-specific secret handling needs separate design and disposable-secret smokes. |
| Path-aware HTTPS policy for `github.com` or `api.github.com` | Host-only CONNECT policy cannot inspect URL paths without TLS interception. |
| Generic desktop terminal or command runner | It widens the desktop trust surface and is not needed for first-class safe workflow. |
| Automatic updater | Manual updates are acceptable for v1.0.0 if release notes say so. |
| Installer or Apple `container` service-management automation | Needs separate user-consent and security design. |
| Managed Apple `container machine` integration | Not required for v1.0.0. User-managed machine workflows should not be blocked solely for a lower-security posture, but any RunHaven integration needs warning, explicit intent, target validation, and deletion approval. |
| MCP, extension, plugin marketplace, or host socket support | Deny-by-default policy exists; implementation is larger product work. |
| Windows, Linux, Intel Mac, hosted CI runtime support | Outside the project support boundary. |

### v1.0.0 Milestones

1. Desktop codebase split before expansion.
   - Files: `src-tauri/src/commands/mod.rs`, `ui/src/commands/runhaven.ts`,
     and `ui/src/app/App.svelte`.
   - Expected result: command families and UI state are split by domain.
   - Verification: Tauri tests, frontend tests, Playwright, file-size scan.

2. Desktop run-control slice.
   - Files: Tauri commands, capabilities, contracts, frontend run detail.
   - Expected result: stop, kill, and repair are first-class GUI operations.
   - Verification: Rust/Tauri/frontend/Playwright checks plus runtime smoke.

3. Desktop maintenance slice.
   - Files: image, state, network, and maintenance UI command families.
   - Expected result: image rebuild, state cleanup, and network cleanup are
     safe, previewed, and confirmed in the app.
   - Verification: Tauri command tests, frontend tests, cleanup smokes.

4. Desktop diagnostics slice.
   - Files: `why`, egress, auth, and blocked-host UI command families.
   - Expected result: users can understand denials and safe next actions
     without reading CLI docs.
   - Verification: Tauri tests, frontend tests, secret-output review.

5. Desktop worktree review slice.
   - Files: worktree command wrappers, review UI, dirty-repo guidance.
   - Expected result: isolated agent changes are easy to inspect, keep, merge,
     recover, or discard.
   - Verification: worktree tests, frontend tests, data-loss review.

6. Accessibility, visual, and packaging release slice.
   - Files: UI styles/components, release docs, packaging scripts if added.
   - Expected result: release artifact is usable by the target audience and
     backed by trust evidence.
   - Verification: UI checks, Playwright screenshots, Tauri build, signing,
     notarization, checksums, provenance, SBOM status.

## Cross-Release Rules

- Keep startup state compact. Update `feature_list.json` and
  `current-state.md` when release scope or active state changes.
- Do not add new broad dependencies, plugins, config, or harness surfaces
  unless they remove real complexity.
- Use standard library, native macOS or Apple `container` behavior, and already
  installed dependencies before custom code.
- Keep direct dependencies, package manifests, runtime pins, and image package
  pins exact-pinned to current stable sources, with lockfiles committed.
- Preserve the default product boundary: no host home mounts, raw SSH key mounts,
  browser profile mounts, cloud credential folder mounts, arbitrary env
  passthrough by default, generic desktop bridges, or root-by-default bundled
  images.
- Treat explicit or user-managed Apple `container machine` workflows as warned
  advanced paths, not hard policy violations solely because they are less
  secure.
- Every mutating UI operation needs a typed Rust command, narrow capability,
  exact target preview, explicit confirmation, and focused tests.

## Immediate Next Actions

1. Done (2026-06-25): split the largest desktop files
   (`src-tauri/src/commands/mod.rs` into validation/warnings; `runhaven.ts` into
   types/client/plan behind a barrel; `App.svelte` into panel components).
2. Done (2026-06-25): desktop run control (`stop_run`, `kill_run`, `repair_run`)
   behind `run-control` and partial read-only diagnostics behind `main-read`
   (V1-G5). Per the 2026-06-26 directive these shipped GUI slices are now
   re-sequenced to the roadmap end; remaining desktop gaps wait behind the
   non-UI work.
3. Near term (non-UI): runtime/security hardening (audit-and-fix slice is
   `passing`), then promote design-first non-UI product candidates one at a time
   through their design gates.
4. Cut tagged release notes for the CLI public release at the release-readiness
   step.

## Decision Log

- One combined gap-analysis doc is enough. Separate v0.5 and v1 documents would
  duplicate the release ladder and increase maintenance cost.
- `v0.5.0` gaps are mostly proof, docs, data, support-tier, and release-state
  gaps because the CLI command families already exist.
- `v1.0.0` gaps are mostly desktop product, accessibility, packaging, and trust
  gaps because the current Tauri app is still alpha.
- Historical evidence remains historical. This document should be updated with
  current facts when a gap closes, not by rewriting past evidence rows.
