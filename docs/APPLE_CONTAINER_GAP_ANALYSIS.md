# Apple Container Gap Analysis

Last updated: 2026-07-12

Status: historical pre-Tauri gap analysis plus recurring runtime evidence
reference.

RunHaven's core product boundary is Apple `container`. This document records
what was covered before the Tauri scaffold started, what remains intentionally
out of scope, and which runtime evidence gates still apply before broadening
CLI or desktop behavior.

Current release context: RunHaven remains pre-release after the `v0.5.0`
CLI-only release. The native Tauri desktop app is the active v0.7.0 coworker
test target; the comprehensive TUI remains a separate product surface. Runtime,
provider, image, SSH, and cleanup
claims still require focused Apple `container` evidence before release.

The repository now pins Apple `container` 1.1.0 at verified tag commit
`5973b9cc626a3e7a499bb316a958237ebe14e2ed`. Its signed, notarized installer
has SHA-256
`0ca1c42a2269c2557efb1d82b1b38ac553e6a3a3da1b1179c439bcee1e7d6714`
and Apple Team ID `UPBK2H6LZM`. Exact-tag source review found no removed
RunHaven command or JSON contract; builder and Kata pins stay unchanged, while
vminit advances to 0.35.0. The signed update and full local smoke matrix passed
on 2026-07-12.

## Evidence Used

- Host evidence: macOS 27.0, arm64, `/usr/local/bin/container`, Apple
  `container` CLI 1.1.0 build `release` commit `5973b9c`.
- Runtime service evidence: `container system status` reported running
  `container-apiserver` 1.1.0 commit `5973b9c`.
- Live Rust smoke evidence: `scripts/apple_container_smoke.sh --with-provider`
  passed on 2026-06-16, covering `doctor`, shell image readiness, internal
  read-only workspace behavior, active-run status/logs-follow/stop cleanup,
  provider allowlist behavior, denied provider/direct egress, and exact
  temporary state/network cleanup. A later 2026-06-16 rerun also passed without
  a macOS Local Network privacy prompt or guest-to-host proxy reachability
  failure on the current host.
- SSH smoke evidence: the no-secret `ssh-add -l` connectivity check now passes
  for the non-root agent user. On macOS 26+ with Apple `container` 1.1.0, the
  default macOS launchd SSH agent socket is world-accessible (`srw-rw-rw-`), so
  the forwarded socket is usable by the non-root `agent` user: a no-identities
  `container run --ssh --user agent runhaven/base sh -c 'ssh-add -l'` returns
  "The agent has no identities" (rc=1), not permission denied. `runhaven plan
  --ssh` and `runhaven run --ssh` are re-enabled and forward only the agent
  socket (never raw keys, non-root) with an explicit lower-security warning. A
  private `0600` agent socket is permission-denied inside the guest and fails
  safe.
- Runtime property evidence: `container system version --format json` and
  `container system property list --format json` matched the reviewed Apple
  `container` commit, builder image
  `ghcr.io/apple/container-builder-shim/builder:0.12.0`, vminit image
  `ghcr.io/apple/containerization/vminit:0.35.0`, and Kata kernel `3.28.0` /
  `vmlinux-6.18.15-186`.
- Local CLI help reviewed: `container run`, `container network`,
  `container image list`, `container volume list`, `container logs`,
  `container exec`, `container machine`, `container registry`, and
  `container system`.
- Local Apple source checkout evidence: `README.md`, `BUILDING.md`,
  `docs/technical-overview.md`, `docs/how-to.md`,
  `docs/container-system-config.md`, `docs/container-machine.md`, and source
  files under `Sources/ContainerCommands/`, `Sources/ContainerResource/`, and
  `Sources/Services/NetworkVmnet/`, plus
  `Sources/ContainerOS/LocalNetworkPrivacy.swift`.
- RunHaven implementation evidence: `crates/runhaven-core/src/runtime/plans/`,
  `crates/runhaven-core/src/provider/runtime.rs`, `crates/runhaven-core/src/runtime/network.rs`,
  `crates/runhaven-core/src/runtime/active/`, `crates/runhaven-core/src/image/`,
  `crates/runhaven-core/src/doctor.rs`, and product docs under `docs/`.

The exact 1.1.0 tag and installed signed CLI are release-specific evidence.

## Covered Today

| Surface | Current Coverage | Primary Evidence |
| --- | --- | --- |
| Host prerequisites | `runhaven doctor` checks macOS, Apple silicon, installed CLI, pinned Apple `container` version, service status, runtime commit, builder image, vminit image, and Kata kernel pin surface. | `crates/runhaven-core/src/doctor.rs`, `crates/runhaven-core/src/doctor/runtime_pins.rs` |
| Task-scoped runs | `runhaven plan` builds a `container run` command with `--rm`, `--init`, `--read-only`, `--tmpfs /tmp`, `--cap-drop ALL`, CPU/memory limits, one workspace mount, one state volume, explicit env passthrough, and non-root bundled image defaults. | `crates/runhaven-core/src/runtime/plans/mod.rs` |
| Sensitive mounts | Home directories, cloud credential folders, browser profiles, raw SSH keys, and broad system paths are rejected unless explicitly allowed. | `crates/runhaven-core/src/runtime/plans/validation.rs`, `docs/SECURITY_MODEL.md` |
| State volume preparation | Non-root home volume ownership is prepared by a short-lived root container on an internal network with DNS disabled. | `crates/runhaven-core/src/runtime/plans/mod.rs`, `crates/runhaven-core/src/provider/runtime.rs` |
| Internal networks | Local-only runs create or reuse host-only Apple `container` networks and reject existing non-host-only networks. | `crates/runhaven-core/src/provider/runtime.rs` |
| Provider egress | Provider mode creates a managed host-only network, inspects gateway/subnet, starts a subnet-restricted host-side CONNECT proxy, injects proxy env vars with the Apple gateway URL, logs decisions, and deletes the managed provider network after the run. On Apple `container` 1.1.0 the gateway is not a bindable macOS address, so tests and live smokes cover wildcard listener binding plus off-subnet client rejection. | `crates/runhaven-core/src/provider/runtime.rs`, `crates/runhaven-core/src/provider/egress.rs` |
| Active run control | `runs status`, `attach`, `logs-follow`, `stop`, `kill`, and `repair` route through Apple `container inspect`, `exec`, `logs`, `stop`, and `kill` with RunHaven-owned container-name validation. | `crates/runhaven-core/src/runtime/active/` |
| Image lifecycle | `image build` uses Apple `container build` with source-digest labels. `image doctor` reads Apple image, builder status, and volume listings without mutating resources. | `crates/runhaven-core/src/image/build.rs`, `crates/runhaven-core/src/image/doctor.rs` |
| Managed cleanup | `network list/prune`, `state list/prune/reset`, and repair commands operate on RunHaven-owned names only. | `crates/runhaven-core/src/runtime/network.rs`, `crates/runhaven-core/src/runtime/state.rs` |
| Tauri planning guardrails | Future WebView control is scoped to typed Rust commands, narrow Tauri capabilities, visible run resources, and explicit approval gates before mutating Apple `container` operations. | `docs/TAURI_UI_GUARDRAILS.md` |
| Provider troubleshooting | Usage docs distinguish policy denials from host-side proxy reachability failures and name the safe provider-smoke commands to collect before changing security posture. | `docs/USAGE.md`, `scripts/apple_container_smoke.sh` |
| SSH agent forwarding | `runhaven plan/run --ssh` forwards only the host SSH agent socket (never raw keys, non-root), with an explicit lower-security warning. Verified no-secret: the non-root agent user connects to the world-accessible default macOS launchd agent socket and `ssh-add -l` returns the no-identities status. | `crates/runhaven-core/src/runtime/plans/mod.rs`, `crates/runhaven/tests/cli.rs`, `scripts/apple_container_smoke.sh`, `docs/USAGE.md` |
| Apple `container` release-update playbook | Runtime pin updates have a repo-owned checklist for source review, installer signature and checksum evidence, helper images, Kata kernel, CLI help diffs, docs, tests, smokes, cleanup, and rollback. | `docs/harness/release/apple-container-update-playbook.md` |
| Opt-in live smoke | `scripts/apple_container_smoke.sh` proves the internal runtime path by default and adds provider egress coverage with `--with-provider`; it is intentionally local-only. | `scripts/apple_container_smoke.sh`, `docs/harness/feedback/verification-matrix.md` |
| JSON parser fixtures | Trimmed fixtures cover Apple `container` image list, network inspect, container inspect, source-backed legacy inspect attachment aliases, and missing-container stderr classification; live 1.1.0 smokes confirm the consumed shapes remain compatible. | `crates/runhaven-core/tests/fixtures/apple_container/`, `crates/runhaven-core/src/image/doctor.rs`, `crates/runhaven-core/src/provider/runtime.rs`, `crates/runhaven-core/src/runtime/active/` |
| Machine default avoidance | RunHaven defaults to task-scoped `container run`, not `container machine`, because machine defaults can mount the host home directory read-write. Explicit or user-managed machine workflows should warn and require intent, not be blocked solely because they are less secure. | `docs/ARCHITECTURE.md`, Apple `docs/container-machine.md` |

## Intentional Decisions

- Keep exact Apple `container` pinning. A newer runtime should fail `doctor`
  until this repo updates pins, source evidence, docs, and smokes.
- Do not replace managed cleanup with native broad prune commands. Native
  `container network prune` and `container volume prune` are useful Apple
  commands, but RunHaven must only delete resources it owns.
- Do not use `container machine` for default agent runs. The home-mount model
  is useful for development machines and wrong for beginner-safe AI agent
  boundaries.
- Do not block explicit or user-managed `container machine` workflows solely
  because they are less secure. If RunHaven manages them later, warn about
  host-home, credential, persistence, and cleanup tradeoffs and require
  explicit intent.
- Do not automate Apple `container` installs, uninstalls, service restarts,
  registry pushes, registry credential changes, or machine deletion without an
  explicit user approval gate.

## Recurring Runtime Gaps And Gates

| Priority | Gap | Why It Matters | Suggested Action | Verification |
| --- | --- | --- | --- | --- |
| P0 | Rust-era live runtime smokes must stay easy to run before CLI release and broad desktop runtime controls. | Unit tests prove command construction, but not installed Apple `container` JSON shapes or runtime behavior. | Use `scripts/apple_container_smoke.sh` as the opt-in local smoke. It runs `runhaven doctor`, `runhaven plan shell`, a minimal `runhaven run shell` smoke, active-run status/logs-follow/stop/repair cleanup, provider plan guidance, and exact resource cleanup checks. | `scripts/apple_container_smoke.sh` exits 0 on macOS 26+ with Apple `container` 1.1.0 and records cleanup evidence. |
| P0 | Provider-mode live smoke must stay covered in Rust. | Provider mode depends on host-only networking, network inspect schema, Apple gateway reachability, subnet-restricted host proxy binding, proxy env injection, and cleanup. | Use `scripts/apple_container_smoke.sh --with-provider` for `v0.5.0`, release, and provider-sensitive desktop evidence. Keep the default smoke usable without live provider network/proxy dependencies. | Provider smoke passes with allowed proxied HTTPS, denied proxied host, denied proxied IP literal, denied direct DNS/IP egress, and no leftover provider network. |
| Resolved | SSH agent forwarding is re-enabled and verified no-secret for the non-root agent user. | `--ssh` maps to Apple `container --ssh`, which bind-mounts `SSH_AUTH_SOCK` into the guest with the host socket's mode bits. The default macOS launchd SSH agent socket is world-accessible (`srw-rw-rw-`), so the forwarded socket is usable by RunHaven's non-root `agent` user. A private `0600` agent socket is permission-denied inside the guest and fails safe; no raw keys are ever mounted and the agent stays non-root. | Re-enabled: `build_run_plan` adds `--ssh` and emits a lower-security warning; a raw key mount and root are still refused. | Verified end to end on macOS 26+ / Apple `container` 1.1.0: `container run --ssh --user agent runhaven/base:0.7.0 sh -c 'ssh-add -l'` returns the no-identities status, not permission denied, against the default launchd agent. `crates/runhaven-core/src/runtime/plans/{mod,tests}.rs`, `crates/runhaven/tests/cli.rs`, `scripts/apple_container_smoke.sh`. |

Current P0, parser-fixture, doctor-pin, and builder-diagnostic status: the
opt-in Rust smoke harness exists and passed with provider coverage on
2026-06-16, Apple `container` JSON parser fixtures are covered by
`cargo test --locked`, `runhaven doctor` now fails closed on mismatched Apple
`container` runtime commit, builder image, vminit image, or Kata kernel fields
from the structured JSON runtime probes, and `image doctor` reports sanitized
builder status/resource guidance from `container builder status --format json`.
`docs/TAURI_UI_GUARDRAILS.md` now owns UI resource warnings and approval gates
for the alpha Tauri scaffold and later desktop slices. `docs/USAGE.md`
now records provider reachability troubleshooting for host-side proxy and
Local Network privacy symptoms. `scripts/apple_container_smoke.sh --with-ssh`
now verifies the no-secret `--ssh` connectivity path: a non-root agent run with
`--ssh` connects to the forwarded default macOS launchd agent socket and
`ssh-add -l` returns the no-identities status. That live smoke passed on macOS
26+ with Apple `container` 1.1.0, so `runhaven plan/run --ssh` are re-enabled and
forward only the agent socket (never raw keys, non-root), with an explicit
lower-security warning; a private `0600` agent socket is permission-denied
inside the guest and fails safe. The Apple `container` release-update
playbook now covers runtime pin update evidence and rollback.
Re-run the live smoke before `v0.5.0`, before provider-sensitive desktop work,
and before release hardening; there is no remaining known runtime blocker.
The consolidated non-UI backlog is tracked in
[`NON_UI_BACKLOG.md`](NON_UI_BACKLOG.md).

## Backlog Additions

- Run the `apple-container-expert` agent with the Apple Container skill for all
  future Apple `container` runtime, networking, source, service, registry,
  machine, and security-boundary work.
- Run the `rust-expert` agent with the Rust skill across the whole repo before
  release hardening to find correctness, idiomatic Rust, safety, test,
  packaging, and maintainability issues.
- Frequent commits and pushes rely on local macOS verification; no hosted CI
  exists for this private project.

## Not Before Desktop Release Design

These are valuable, but they should not block the CLI-complete milestone or the
next focused desktop slices without a separate design:

- Registry push/pull/login UI.
- Managed `container machine` creation, mutation, deletion, or attachment UI.
  User-managed machine workflows remain possible outside RunHaven and any
  future integration should warn rather than policy-block.
- Apple `container` source builds or local Containerization package editing.
- Automated Apple `container` install, update, uninstall, or service restart.
- Release signing, SBOM, and provenance automation before the v1 desktop
  packaging path is selected.
