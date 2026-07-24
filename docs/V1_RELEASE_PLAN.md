# RunHaven v0.5.0/v1.0.0 Release Plan

Last updated: 2026-06-27

Status: historical durable release ladder plus release-quality gates. Current
sequence is owned by `PROGRESS.md` and `docs/ROADMAP.md`.

> Sequencing update: the 2026-06-26 directive deferred GUI/UI work behind
> runtime/security hardening, remaining non-UI scope, and a CLI-based public
> release. The comprehensive terminal UI remains a product surface, while its
> exact pinned Codex dependencies are isolated behind the active
> `runhaven-tui` workspace boundary. The desktop design content below remains the
> active first-class release direction; the
> `v1.0.0` label is no longer locked to that desktop release. Already-shipped
> desktop slices stay complete and verified.

## Problem

RunHaven exists to make the secure way the easy way for running AI coding
agents on personal Macs. A CLI-only v1 would be useful for technical users,
but it would not satisfy the product goal for less-technical users. If the
secure path still requires remembering flags, reading long docs, or switching
between terminal commands for routine recovery, the secure path is not easy.

v1.0.0 should therefore treat the Tauri desktop app as a first-class product
surface, backed by the Rust CLI and shared planner. The CLI remains stable and
important for automation, diagnostics, and power users, but the desktop app
must carry the primary safe workflow from setup through launch, observation,
review, recovery, and cleanup.

The release ladder is three-stage:

- `v0.5.0` is the CLI-complete release.
- `v0.7.0` is the limited CLI plus desktop coworker-test candidate. It does not
  become shareable until the release wrapper produces a signed, notarized
  artifact and that artifact passes clean-Mac verification.
- `v1.0.0` is the first-class desktop release built on that stable CLI
  foundation.

RunHaven remains alpha/pre-release. `v0.5.0` has been cut as the CLI-complete
pre-release; it establishes the CLI contract. `v0.7.0` adds the coworker-test
desktop candidate and is not the full public desktop release.

## Release Decision

`v0.5.0` should be the CLI-complete release. By `v0.5.0`, all intended CLI
product behavior should be done: command set, docs, JSON and data-contract
decisions, runtime smokes, profile support tiers, diagnostics, cleanup, and
security boundaries. After `v0.5.0`, CLI changes should be bug fixes, security
fixes, release pin updates, documentation corrections, and internal work needed
to support the GUI without changing CLI semantics.

The `v0.5.0` CLI surface is:

- `runhaven setup`, `doctor`, `agents`, `plan`, and `run`;
- bundled image build, rebuild, and read-only image doctor;
- `internet`, `internal`, and source-backed `provider` network modes;
- explicit workspace scope, read-only workspace, named sessions, and sensitive
  path/root-user fail-closed validation;
- active run discovery, status, attach, logs-follow, stop, kill, and repair;
- completed run records, logs, diffs, and guarded worktree keep, recover,
  merge, and discard;
- guarded state-volume and managed-network cleanup;
- provider egress logs, `why` diagnostics, auth status/explain/log, and the
  Claude and Codex host-side API-key brokers.

`v1.0.0` should be a first-class desktop release with that stable CLI
foundation.

Above all else, the secure path must be the easy path. For every v1 workflow,
the secure default should be the shortest and clearest route. Supported
less-secure choices should warn and require explicit intent, but should not be
hidden or blocked only because they are less secure. Unsupported, invalid, or
hard-boundary violations still fail closed.

The desktop app is the primary beginner path. For v1.0.0, it must let a user
complete the safe workflow without needing to know the CLI command set:

- see whether the Mac is ready and what exact prerequisite is missing;
- select an agent and project folder;
- understand whether the selected folder is narrow enough;
- choose a goal-based network mode with secure defaults;
- build or rebuild missing bundled images from an explicit UI action;
- review the run plan before launch;
- launch only after clear confirmation and warning acknowledgements;
- watch sanitized run status and bounded raw output when acknowledged;
- stop a run, hard-stop a stuck run, and repair stale active markers;
- inspect recent runs, provider denials, auth broker decisions, and common
  "why" explanations;
- review agent changes through a safe git/worktree flow before merging or
  discarding;
- clean up RunHaven-owned state, networks, and worktrees with exact target
  previews and explicit confirmations.

The release artifact must match the first-class desktop claim. A source tag
alone is not enough for v1.0.0 if the target audience includes less-technical
users. The release should provide a signed and notarized macOS desktop app
artifact, or the release should not be called v1.0.0.

## Scope And Non-Goals

In scope for v1.0.0:

- A CLI-complete `v0.5.0` release before broad v1 desktop expansion.
- A public desktop release boundary replacing the alpha contract in
  user-facing docs after `v0.5.0`.
- A first-class Tauri desktop app for the safe workflow listed above.
- The comprehensive terminal UI remains an active workspace product surface;
  linking it from the default release binary remains a separate audited product
  decision.
- A stable CLI that shares the same planner, validators, run records, provider
  policy, auth broker metadata, and cleanup rules as the desktop app.
- Narrow Tauri capabilities for every privileged operation.
- Exact version alignment across `Cargo.toml`, `src-tauri/Cargo.toml`,
  `ui/package.json`, `src-tauri/tauri.conf.json`, `pins.toml`, and bundled
  image tags when the release commit is cut.
- Fresh primary-source and live-runtime evidence for Rust, npm, Debian,
  bundled agent CLIs, Apple `container`, Apple helper images, and the Kata
  kernel.
- Real local Apple `container` smokes for the stable CLI runtime path,
  provider mode, SSH fail-closed behavior, and desktop launch/control paths.
- A profile support matrix that distinguishes "bundled image builds" from
  "verified provider/auth workflow".
- A data lifecycle statement for RunHaven cache records, active markers, state
  volumes, worktrees, image contexts, and provider/auth logs.
- Release artifacts with provenance, checksums, signing status, notarization
  status, and SBOM status.

Out of scope for v1.0.0:

- Runtime or contributor verification outside macOS 26+ on Apple silicon.
- Docker, Docker Desktop, Docker Compose, devcontainers as an execution
  backend, or Apple `container machine` as the default execution boundary.
- Managed Apple `container machine` workflows are not required for v1.0.0. Do
  not block explicit or user-managed machine workflows solely because they are
  less secure. If RunHaven integrates with them, warn and require explicit
  intent.
- Raw SSH key mounts, host home mounts, browser profile mounts, cloud
  credential folder mounts, or arbitrary environment passthrough.
- Re-enabling `--ssh` without no-secret non-root runtime proof.
- TLS interception or path-aware HTTPS inspection.
- Bundling broad path-sensitive hosts such as `github.com` or `api.github.com`
  for Copilot without a provider-specific broker or another enforceable
  policy.
- New Claude, Copilot, or Antigravity credential brokers. Each is
  credential-handling product work and should be a v1.x feature only after
  provider-specific design, tests, and disposable-secret smokes.
- MCP, editor extension, plugin marketplace, host socket, registry credential,
  updater, installer, uninstall, or Apple service-management surfaces.
- A generic desktop terminal, shell bridge, filesystem browser, HTTP bridge, or
  Apple `container` bridge to JavaScript.
- Automatic deletion of workspaces, non-RunHaven networks, non-RunHaven
  volumes, arbitrary images, or user-created Apple `container` resources.
- Automatic Apple `container` install, update, uninstall, or service restart
  without a separate security and user-consent design.
- New CLI product features after `v0.5.0` unless they are required security
  fixes or internal GUI support that preserves the CLI contract.

## Engineering Quality Gates

These apply to every `v0.5.0` and `v1.0.0` slice.

- Make the secure path the default and easiest path. Supported lower-security
  choices need clear warning and explicit intent; hard-boundary violations still
  fail closed. This includes explicit or user-managed Apple `container machine`
  workflows: warn rather than policy-block solely because they are less secure.
- Define success criteria, affected files, verification, and meaningful
  tradeoffs before coding.
- Use the build necessity ladder: no change, deletion, docs or config, standard
  library, native macOS or Apple `container` behavior, installed dependency,
  one clear local change, then minimum new code.
- Keep files, modules, crates, Tauri commands, Svelte components, and harness
  docs cohesive. If a touched file is already hard to review or would become
  hard to review, split or delete in the same slice unless the deferral is
  explicit and small.
- Eliminate meaningful duplication. Prefer deletion, narrower helpers, or
  existing local APIs over copy/paste, but do not create speculative
  abstractions for one implementation.
- Keep direct dependencies, package manifests, runtime pins, and image package
  pins exact-pinned to current stable releases. Lock transitive dependencies
  through lockfiles. Verify volatile version claims from current official
  sources before changing pins.
- Touch only the files needed for the objective, and clean up generated files,
  dead code, stale docs, and stale harness state introduced by the work.
- If unsure, stop and surface the uncertainty, or research current official
  documentation before implementing.
- Keep `feature_list.json`, `PROGRESS.md`, and focused harness docs aligned
  when active state, verification routing, or release scope changes.

## Audiences

Primary users:

- Mac developers on macOS 26+ Apple silicon who want the secure path to be the
  obvious path.
- Less-technical users who should not need to understand Apple `container`,
  git worktrees, egress policy, or cache directories to use RunHaven safely.
- Security-aware developers who need visible plans, explicit egress choices,
  isolated state, and recoverable agent changes.

Secondary users:

- Teams evaluating whether RunHaven is safe enough for personal workstations.
- Security reviewers who need a clear boundary, known limits, and reproducible
  release evidence.
- Power users who want custom shell images, named sessions, internal-network
  local checks, CLI automation, worktree review flows, and explicit
  user-managed `container machine` workflows.

Unsupported or intentionally limited users:

- Linux, Intel Mac, cloud VM, and hosted CI users.
- Users who need host-home developer environments, remote editor VMs, or
  persistent `container machine` workflows as the default beginner-safe
  boundary.
- Users who need private Git over SSH from inside the guest before Apple
  `container` non-root SSH forwarding is proven.
- Users who need fully path-aware provider policy for broad web hosts.

## Known Gaps And Edge Cases

### Runtime And Platform

- Apple `container` drift is expected. `doctor` should keep failing closed on
  unreviewed version, commit, helper image, vminit, or Kata kernel drift.
- Apple `container` JSON shapes can change. Unit fixtures are useful, but
  release requires live `doctor`, image, network, inspect, logs, exec, stop,
  kill, and repair evidence.
- macOS Local Network privacy can affect provider proxy reachability. Provider
  troubleshooting and smoke evidence must stay current.
- The provider proxy and Codex broker can fall back to `0.0.0.0` when the
  Apple gateway address is not bindable. v1 must keep subnet rejection tests
  and live off-subnet denial evidence, and release notes must not claim the
  listener is bound only to the gateway.
- Tauri launch runs in a background thread today. Before v1, verify app quit,
  app crash, child `container run` lifetime, active marker cleanup, and
  recovery guidance. If that behavior is not closed, desktop launch is not
  first-class.
- Host sleep, reboot, force quit, and interrupted provider cleanup can leave
  containers, provider networks, active markers, or worktrees behind. v1 must
  document and smoke the recovery commands from both CLI and desktop.
- Each Apple container run uses VM resources. v1 should keep active-run and
  memory warnings, but must not claim host memory pressure is measured until it
  is.

### Security And Credentials

- The default network mode is profile-aware: provider for profiles with bundled
  hosts, internet for those without. Internet mode is unrestricted egress;
  provider mode is the constrained path, but only for hostnames it can enforce.
- Provider mode permits a bundled host and its subdomains. That subdomain rule
  must remain visible in plans, docs, and UI warnings.
- Copilot needs path-sensitive GitHub hosts for some flows. v1 should document
  Copilot provider-mode limits instead of bundling `github.com` or
  `api.github.com` broadly.
- Antigravity has no bundled runtime provider hosts because no source-backed
  minimal runtime host list has been identified.
- Non-Codex headless credentials still require explicit `--env NAME` or
  isolated in-agent login state. That is a known limitation, not a reason to
  rush new credential brokers into v1.
- Raw active-container output can contain secrets or workspace contents. v1
  should keep bounded, acknowledged snapshots and avoid persistent frontend
  storage. A dedicated non-UI security slice should add a shared sanitizer before
  any raw or semi-raw output reaches CLI, Tauri, TUI, JSONL records, or support
  output.
- Ordinary provider OAuth stays inside each agent's isolated container volume.
  If RunHaven later accepts an explicitly configured host API key, that
  advanced broker-owned secret should prefer macOS Keychain storage rather
  than a plaintext file. RunHaven must not copy provider-owned Keychain items,
  browser profiles, cloud credential stores, or arbitrary host credentials.
- Run records intentionally omit secrets and contents, but workspace paths,
  changed file names, provider hostnames, and run metadata can still reveal
  private project structure.
- `RUNHAVEN_CACHE_HOME` is a powerful local override. If it remains public or
  semi-public, v1 docs should describe it as advanced and warn against shared
  or world-readable paths.
- Frontend IPC input is untrusted. New desktop commands must stay typed,
  bounded, and capability-scoped.

### Data, Storage, And Recovery

- `runs.jsonl`, `egress-policy.jsonl`, and `auth-broker.jsonl` are append-only
  and currently read into memory. Long-term use can make listing slow and cache
  growth unbounded.
- JSON and local data lifecycle (decision, 2026-06-26): through `v0.5.0`, every
  CLI `--json` output (`runs list`, `show`, `log`, `diff`, `status`, `repair`;
  `auth status`, `explain`, `log`; `egress log`; `image doctor`) and every local
  record file (`runs.jsonl`, `egress-policy.jsonl`, `auth-broker.jsonl`, and
  active-run markers `active-runs/<id>.json`) is best-effort and unversioned.
  RunHaven owns these, fields may be added, renamed, or removed between versions,
  and external tools should not depend on them yet. OAuth token files, state lock
  files, the login workspace, and state and home volumes are internal
  implementation details, never a documented format. Before any output is
  declared stable it gains an explicit `schema_version` field and a documented
  shape; the first candidates are the audit and log outputs (`runs log`,
  `auth log`, `egress log`). No output is a stable integration contract before
  that step.
- State volumes can contain provider sessions, local caches, and generated
  code. `state reset` and `state prune --yes` are irreversible and need clear
  preview text in both CLI and desktop.
- Worktrees under the cache root contain real project files. The desktop app
  must make keep, recover, merge, and discard understandable before v1.
- Image contexts are materialized under the RunHaven cache by content digest.
  Repeated source changes can leave old contexts behind unless cleanup is
  documented or implemented.
- Active markers are secret-free but mutable local files. Commands must keep
  validating run ids and RunHaven-owned container names before mutation.

### Networking And Provider Behavior

- Provider mode is IPv4-gateway based for the pinned Apple `container` 1.1.0
  implementation. Missing or changed IPv4 gateway/subnet fields must fail
  closed.
- Browser sign-in flows can need localhost callbacks, broad auth hosts, or
  browser state. v1 should prefer isolated in-agent login or API-key flows and
  keep host browser profiles out of scope.
- Package managers, registries, CDNs, and updater hosts generally require
  `internet` mode. Provider mode is not a package-install mode.
- Extra provider hosts should stay fully qualified and explicit. IP literals,
  single-label hosts, and unsafe resolved addresses remain blocked.

### UX And User Failure Modes

- New users need to know "which folder," "which network," "where credentials
  go," and "how to undo" inside the app, not only in docs.
- If a path is rejected as sensitive, the user needs the reason and the narrow
  intentional override, not a generic failure.
- If a provider run blocks a host, the user needs `why host`, egress logs, and
  the endpoint matrix route before adding hosts.
- If a run hangs, the user needs active run listing, status, logs, stop, kill,
  repair, and cleanup guidance in the GUI.
- If a repository is dirty, the desktop app must explain the choices: run
  in-place, use read-only review, clean the repo, or use a worktree when safe.
- If the image is missing or stale, the desktop app must offer the safe rebuild
  path directly instead of sending users to a terminal.

### Accessibility, Packaging, And Trust

- A first-class desktop release must be keyboard navigable, readable at the
  minimum supported window size, and clear with macOS accessibility tooling.
- Dialogs and confirmations must name the exact resource being changed.
- The app artifact must be signed and notarized for v1.0.0, or the desktop
  surface should remain pre-v1.
- The release must state whether updates are manual or automatic. Automatic
  updater support is not required for v1.0.0, but silent update behavior is out
  of scope without a separate design.

### Performance And Maintainability

- JSONL reading should be streamed or tailed before the project expects heavy
  long-term run history.
- Regexes in hot list/prune paths can use standard-library lazy statics if
  they become measurable, but this is not a release blocker.
- `doctor`, `image doctor`, and provider setup call multiple Apple `container`
  commands. Avoid caching safety-critical drift checks across releases; prefer
  clearer progress output over stale cache.
- Tauri command code, the frontend command adapter, and `App.svelte` are among
  the largest maintained files. First-class desktop work should split them by
  domain before adding many more controls.
- Release evidence comes from recorded local macOS 26+ Apple silicon gates;
  this private project has no hosted CI.

## Milestones

### M0: Freeze The Release Ladder And Desktop Contract

- Update `README.md`, `docs/CAPABILITIES.md`, `docs/SECURITY_MODEL.md`,
  `docs/INSTALLATION.md`, `docs/USAGE.md`, `SECURITY.md`, and
  `docs/ROADMAP.md` to replace alpha language with a desktop-first-class v1
  contract.
- State clearly that `v0.5.0` is CLI-complete and that v1 work should not add
  new CLI product scope.
- Add a support matrix for desktop stable, CLI stable, profile support tiers,
  network modes, auth modes, and unsupported surfaces.
- Decide JSON stability. Either add schema versions to stable JSON surfaces or
  document JSON as pre-stable.
- Decide update policy. Default: manual updates with signed/notarized release
  artifacts and checksums; no automatic updater in v1.0.0 unless separately
  designed.

Verification:

```bash
scripts/cargo.sh run --locked --bin runhaven-check-pins
git diff --check
```

Also run JSON validation and a local Markdown link check.

### M1: Cut CLI-Complete v0.5.0

- Finish any remaining CLI-only product behavior before the tag.
- Lock the command set, help text, user docs, and known-limits docs.
- Confirm CLI crate and module organization remains maintainable, with no known
  large-file or duplication debt intentionally pushed into v1 desktop work.
- Decide which JSON outputs and local record formats are stable, schema
  versioned, or explicitly best-effort.
- Verify CLI planner, run, image, provider, auth, worktree, diagnostics,
  cleanup, and fail-closed security behavior.
- Record profile support tiers from CLI evidence.
- After the tag, treat the CLI as the desktop backend contract rather than a
  place for new v1 product expansion.

Verification:

```bash
scripts/cargo.sh fmt --check
scripts/cargo.sh test --locked
scripts/cargo.sh clippy --all-targets --locked -- -D warnings
scripts/cargo.sh run --locked --bin runhaven-check-pins
scripts/cargo.sh build --locked
scripts/apple_container_smoke.sh
scripts/apple_container_smoke.sh --with-provider
scripts/apple_container_smoke.sh --with-ssh
git diff --check
```

Also run focused CLI profile dry-runs and live disposable-credential smokes
only where the support matrix claims verified provider/auth behavior.

### M2: Build The Desktop Core Workflow

- Keep the implemented first-run prerequisite recovery, signed-installer handoff,
  service start, visible agent installation, and launch-readiness checks aligned.
- Keep native folder picking and path validation through typed Rust commands.
- Add goal-based network selection with secure defaults and visible egress
  summaries.
- Add bundled image build/rebuild controls with explicit confirmation and
  builder status.
- Keep plan review and launch confirmation as the only launch path.
- Ensure the desktop app never parses CLI prose for privileged decisions.
- Keep frontend, Tauri, and shared planner modules cohesive; handle any
  large-file split in the slice that creates the pressure.

Verification:

```bash
scripts/cargo.sh fmt --check
scripts/cargo.sh test --workspace --locked
scripts/cargo.sh clippy --workspace --all-targets --locked -- -D warnings
npm run check
npm test
npm run test:e2e
npm run build
git diff --check
```

### M3: Add Desktop Run Control And Recovery

- Implement `stop_run` as the next slice.
- Add `kill_run` and stale-marker repair with separate confirmations and
  capability scopes.
- Keep raw logs bounded, acknowledged, and not durably stored in frontend
  state.
- Verify Tauri app quit/crash behavior after a launched run. Document or
  implement recovery for any child container that remains active.
- Add active-run list, status refresh, run history summaries, and recovery
  guidance in the GUI.

Verification:

```bash
scripts/cargo.sh fmt --check
scripts/cargo.sh test --workspace --locked
scripts/cargo.sh clippy --workspace --all-targets --locked -- -D warnings
npm run check
npm test
npm run test:e2e
npm run build
scripts/apple_container_smoke.sh
git diff --check
```

### M4: Make Worktree Review The Easy Safe Path

- Add desktop controls for read-only review and worktree runs.
- Explain dirty repository choices in the app before launch.
- Add desktop run diff, keep, recover, merge, and discard flows for worktree
  runs.
- Require exact target previews and confirmation before merge or discard.
- Keep arbitrary shell attach out of the desktop v1 surface. CLI attach remains
  available for power users.
- Suggest detected project checks without running them automatically.

Verification:

```bash
scripts/cargo.sh test --locked worktree
scripts/cargo.sh test --workspace --locked
npm test
npm run test:e2e
git diff --check
```

### M5: Add Desktop Diagnostics And Maintenance

- Expose `why host`, `why workspace`, `why network`, and `why state` as typed
  desktop diagnostics.
- Show provider blocked-host reviews and egress policy logs without raw request
  bodies or token values.
- Show auth broker status/explain/log without reading secrets.
- Add state reset/prune and network prune controls only with exact target
  previews and explicit confirmations.
- Add image doctor and rebuild recovery guidance as a normal desktop path.

Verification:

```bash
scripts/cargo.sh test --locked
scripts/cargo.sh test --workspace --locked
npm run check
npm test
npm run test:e2e
git diff --check
```

### M6: Prove The Runtime Boundary

- Refresh Apple `container` 1.1.0 evidence through
  `docs/harness/release/apple-container-update-playbook.md`.
- Run the default Apple container smoke, provider smoke, and no-secret SSH
  forwarding smoke.
- Verify cleanup after smokes: active runs, state volumes, managed networks,
  native container list, volume list, and network list.
- Reconfirm provider wildcard-bind fallback is still subnet-restricted and
  off-subnet clients are denied.
- Reconfirm `--ssh` forwards only the agent socket after an explicit warning,
  and that no raw key mount or root default user workaround was introduced.

Verification:

```bash
scripts/cargo.sh run --locked --bin runhaven -- doctor
scripts/apple_container_smoke.sh
scripts/apple_container_smoke.sh --with-provider
scripts/apple_container_smoke.sh --with-ssh
scripts/cargo.sh run --locked --bin runhaven -- runs active
scripts/cargo.sh run --locked --bin runhaven -- network list
scripts/cargo.sh run --locked --bin runhaven -- state list
container list
container volume list
container network list
```

### M7: Define Profile Support Tiers

- For each bundled profile, verify image build or dry-run, image doctor,
  desktop image readiness, default plan, provider plan where applicable, and a
  no-secret `-- --version` or equivalent smoke when the agent supports it.
- Split profile claims into:
  - bundled image available;
  - basic CLI starts;
  - desktop workflow supported;
  - provider mode source-backed;
  - interactive auth known path;
  - headless brokered auth available.
- Keep Codex as the only brokered-auth profile unless a separate credential
  broker project completes before v1.
- Keep Copilot and Antigravity limitations explicit.

Example verification:

```bash
scripts/cargo.sh run --locked --bin runhaven -- image build claude --dry-run
scripts/cargo.sh run --locked --bin runhaven -- image build codex --dry-run
scripts/cargo.sh run --locked --bin runhaven -- image build antigravity --dry-run
scripts/cargo.sh run --locked --bin runhaven -- image build copilot --dry-run
scripts/cargo.sh run --locked --bin runhaven -- image doctor
```

Run live profile smokes only with disposable credentials and recorded scope.

### M8: Stabilize Data Lifecycle

- Decide whether v1 requires schema versions for run records, active markers,
  repair JSON, status JSON, egress logs, and auth logs. Prefer explicit
  `schema_version` before stable JSON claims.
- Add or document cache retention for run logs, provider/auth logs, worktrees,
  active markers, image contexts, locks, and state volumes.
- Decide whether to add a narrow `runs prune` or `cache doctor` command before
  v1. If not, document manual cache paths and privacy implications.
- Ensure destructive cleanup commands preview exact targets before `--yes` and
  state that agent home volumes may contain provider sessions.

Focused verification:

```bash
scripts/cargo.sh test --locked
scripts/cargo.sh clippy --all-targets -- -D warnings
git diff --check
```

### M9: Package The Desktop Release

- Bump release versions and image tags from `0.1.0` to `1.0.0` only when the
  release boundary is accepted.
- Refresh exact pins from primary sources and update `pins.toml`, lockfiles,
  image package locks, docs, and research evidence.
- Build all bundled images or record why a profile is not fully verified.
- Build the Tauri desktop app as a release artifact.
- Sign and notarize the macOS artifact.
- Produce release provenance: source commit, build host, commands, pins,
  checksums, signing status, notarization status, SBOM status, release
  operator, and date.
- Do not publish from a dirty worktree.

Verification:

```bash
./init.sh
scripts/cargo.sh run --locked --bin runhaven-check-pins
scripts/cargo.sh build --locked
npm run tauri:build
npm run tauri:bundle
git status --short --branch
git diff --check
```

`scripts/release-desktop.sh` owns this gate. It verifies a clean exact-tagged
source tree, full Xcode, the Developer ID identity and matching provisioning
profile, notary credentials, hardened runtime, signed entitlements, notarization
acceptance, stapled tickets, Gatekeeper acceptance, checksums, CycloneDX SBOMs,
and provenance. It publishes the artifact directory only after every check
passes. The remaining release proof is to run it with the project's real Apple
credentials and install the resulting DMG on a clean supported Mac. The wrapper
also mounts the finished DMG, checks its exact drag-to-Applications layout,
copies the app to an isolated Applications directory, and rechecks architecture,
bundle metadata, signature, stapled ticket, and Gatekeeper acceptance.

### M10: Final Public Readiness Review

- Run one adversarial security review of the release diff, focused on mounts,
  process execution, environment handling, logs, WebView IPC, provider proxy,
  auth broker, cleanup, and destructive commands.
- Run one maintainability review of large modules, test coverage, duplicate
  planner logic between CLI and Tauri, and docs/source drift.
- Run an accessibility and visual regression pass over the desktop app.
- Run a representative real-agent task set before making product effectiveness
  claims. Keep claims limited to observed outcomes.
- Record final evidence in `docs/harness/evidence/evidence-log.md`,
  `feature_list.json`, and `PROGRESS.md`.

## Release Gates

No v1.0.0 release unless all applicable gates are green or explicitly waived in
release notes with risk and follow-up:

- Worktree clean and aligned with intended release commit.
- `v0.5.0` has been cut as the CLI-complete release, or the v1 release notes
  explicitly include the CLI-complete evidence and contract.
- No planned CLI product work remains open except accepted v1.x follow-ups,
  bug fixes, security fixes, or internal GUI support that preserves semantics.
- Touched files, modules, crates, Tauri commands, frontend components, and
  harness docs have been reviewed for file size, cohesion, duplication,
  dependency use, and maintainability debt.
- The secure path is the easiest default for every shipped workflow, and any
  supported less-secure path is warned and explicit.
- `./init.sh` passes on macOS 26+ Apple silicon.
- `runhaven doctor` passes against the reviewed Apple `container` pin.
- `scripts/apple_container_smoke.sh` passes.
- `scripts/apple_container_smoke.sh --with-provider` passes.
- `scripts/apple_container_smoke.sh --with-ssh` proves fail-closed behavior, or
  SSH is intentionally re-enabled only after no-secret non-root proof.
- Tauri command tests, frontend tests, Playwright e2e, Svelte check, UI build,
  and Tauri build pass.
- Desktop app covers setup, image readiness/build, plan, launch, status,
  bounded output, stop, kill, repair, diagnostics, worktree review, and safe
  cleanup.
- Desktop app passes focused accessibility and minimum-window visual checks.
- All changed JSON files validate.
- Local Markdown link check passes.
- `scripts/cargo.sh run --locked --bin runhaven-check-pins` passes.
- Direct dependencies, package manifests, runtime pins, and image package pins
  are exact-pinned to current stable versions from official sources, with
  transitive dependencies locked.
- `git diff --check` passes.
- Release artifact is signed, notarized, checksummed, and backed by provenance.
- Release notes state stable, unsupported, known-limited, update, install, and
  security-reporting surfaces.
- Cleanup checks show no unexpected active runs, provider networks, state
  volumes, or native Apple `container` resources left by smokes.

## Decisions And Retrospective

Release decisions and the fill-in release retrospective moved to
[`release/v1-decisions-and-retrospective.md`](release/v1-decisions-and-retrospective.md).
Use [`RELEASE_GAP_ANALYSIS.md`](RELEASE_GAP_ANALYSIS.md) as the active tracker
for gaps before `v0.5.0` and `v1.0.0`.
