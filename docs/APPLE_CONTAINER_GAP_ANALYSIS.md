# Apple Container Gap Analysis

Last updated: 2026-06-27

Status: historical pre-Tauri gap analysis plus recurring runtime evidence
reference.

RunHaven's core product boundary is Apple `container`. This document records
what was covered before the Tauri scaffold started, what remains intentionally
out of scope, and which runtime evidence gates still apply before broadening
CLI or desktop behavior.

Current release context: RunHaven remains alpha/pre-release after the `v0.5.0`
CLI-only pre-release. The TUI is active and the Tauri scaffold has started, but
runtime, provider, image, SSH, and cleanup claims still require focused Apple
`container` evidence before any release or runtime-sensitive UI expansion.

## Evidence Used

- Host evidence: macOS 26.5.1, arm64, `/usr/local/bin/container`, Apple
  `container` CLI 1.0.0 build `release` commit `ee848e3`.
- Runtime service evidence: `container system status` reported running
  `container-apiserver` 1.0.0 commit `ee848e3`.
- Live Rust smoke evidence: `scripts/apple_container_smoke.sh --with-provider`
  passed on 2026-06-16, covering `doctor`, shell image readiness, internal
  read-only workspace behavior, active-run status/logs-follow/stop cleanup,
  provider allowlist behavior, denied provider/direct egress, and exact
  temporary state/network cleanup. A later 2026-06-16 rerun also passed without
  a macOS Local Network privacy prompt or guest-to-host proxy reachability
  failure on the current host.
- SSH smoke evidence: a path-only `--with-ssh` smoke false-passed on
  2026-06-16 and was rejected by Apple Container expert review. The stricter
  no-secret check must prove `ssh-add -l` connectivity against a disposable
  empty `ssh-agent`; on the current pinned Apple `container` 1.0.0 runtime,
  non-root RunHaven agent runs see the socket path but `ssh-add -l` fails with
  permission denied. `runhaven plan --ssh` and `runhaven run --ssh` now fail
  closed until that runtime boundary is fixed and verified.
- Runtime property evidence: `container system version --format json` and
  `container system property list --format json` matched the reviewed Apple
  `container` commit, builder image
  `ghcr.io/apple/container-builder-shim/builder:0.12.0`, vminit image
  `ghcr.io/apple/containerization/vminit:0.33.3`, and Kata kernel `3.28.0` /
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

The local Apple source checkout was treated as source-map evidence. The
installed CLI help and RunHaven pins are the release-specific evidence for
Apple `container` 1.0.0 behavior.

## Covered Today

| Surface | Current Coverage | Primary Evidence |
| --- | --- | --- |
| Host prerequisites | `runhaven doctor` checks macOS, Apple silicon, installed CLI, pinned Apple `container` version, service status, runtime commit, builder image, vminit image, and Kata kernel pin surface. | `crates/runhaven-core/src/doctor.rs`, `crates/runhaven-core/src/doctor/runtime_pins.rs` |
| Task-scoped runs | `runhaven plan` builds a `container run` command with `--rm`, `--init`, `--read-only`, `--tmpfs /tmp`, `--cap-drop ALL`, CPU/memory limits, one workspace mount, one state volume, explicit env passthrough, and non-root bundled image defaults. | `crates/runhaven-core/src/runtime/plans/mod.rs` |
| Sensitive mounts | Home directories, cloud credential folders, browser profiles, raw SSH keys, and broad system paths are rejected unless explicitly allowed. | `crates/runhaven-core/src/runtime/plans/validation.rs`, `docs/SECURITY_MODEL.md` |
| State volume preparation | Non-root home volume ownership is prepared by a short-lived root container on an internal network with DNS disabled. | `crates/runhaven-core/src/runtime/plans/mod.rs`, `crates/runhaven-core/src/provider/runtime.rs` |
| Internal networks | Local-only runs create or reuse host-only Apple `container` networks and reject existing non-host-only networks. | `crates/runhaven-core/src/provider/runtime.rs` |
| Provider egress | Provider mode creates a managed host-only network, inspects gateway/subnet, starts a subnet-restricted host-side CONNECT proxy, injects proxy env vars with the Apple gateway URL, logs decisions, and deletes the managed provider network after the run. On Apple `container` 1.0.0 the gateway is not a bindable macOS address, so tests and live smokes cover wildcard listener binding plus off-subnet client rejection. | `crates/runhaven-core/src/provider/runtime.rs`, `crates/runhaven-core/src/provider/egress.rs` |
| Active run control | `runs status`, `attach`, `logs-follow`, `stop`, `kill`, and `repair` route through Apple `container inspect`, `exec`, `logs`, `stop`, and `kill` with RunHaven-owned container-name validation. | `crates/runhaven-core/src/runtime/active/` |
| Image lifecycle | `image build` uses Apple `container build` with source-digest labels. `image doctor` reads Apple image, builder status, and volume listings without mutating resources. | `crates/runhaven-core/src/image/build.rs`, `crates/runhaven-core/src/image/doctor.rs` |
| Managed cleanup | `network list/prune`, `state list/prune/reset`, and repair commands operate on RunHaven-owned names only. | `crates/runhaven-core/src/runtime/network.rs`, `crates/runhaven-core/src/runtime/state.rs` |
| Tauri planning guardrails | Future WebView control is scoped to typed Rust commands, narrow Tauri capabilities, visible run resources, and explicit approval gates before mutating Apple `container` operations. | `docs/TAURI_UI_GUARDRAILS.md` |
| Provider troubleshooting | Usage docs distinguish policy denials from host-side proxy reachability failures and name the safe provider-smoke commands to collect before changing security posture. | `docs/USAGE.md`, `scripts/apple_container_smoke.sh` |
| SSH forwarding fail-closed guard | `runhaven plan --ssh` and `runhaven run --ssh` refuse before launch. The local smoke harness verifies the refusal without using real keys. Live no-secret connectivity remains blocked for the default non-root agent user on the current pinned Apple `container` runtime. | `crates/runhaven-core/src/runtime/plans/mod.rs`, `crates/runhaven/tests/cli.rs`, `scripts/apple_container_smoke.sh`, `docs/USAGE.md` |
| Apple `container` release-update playbook | Runtime pin updates have a repo-owned checklist for source review, installer signature and checksum evidence, helper images, Kata kernel, CLI help diffs, docs, tests, smokes, cleanup, and rollback. | `docs/harness/release/apple-container-update-playbook.md` |
| Opt-in live smoke | `scripts/apple_container_smoke.sh` proves the internal runtime path by default and adds provider egress coverage with `--with-provider`; it is intentionally local-only while alpha CI is disabled. | `scripts/apple_container_smoke.sh`, `docs/harness/feedback/verification-matrix.md` |
| JSON parser fixtures | Trimmed fixtures cover Apple `container` 1.0.0 image list, network inspect, container inspect, source-backed legacy inspect attachment aliases, and missing-container stderr classification without requiring live Apple `container` in unit tests. | `crates/runhaven-core/tests/fixtures/apple_container/`, `crates/runhaven-core/src/image/doctor.rs`, `crates/runhaven-core/src/provider/runtime.rs`, `crates/runhaven-core/src/runtime/active/` |
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
| P0 | Rust-era live runtime smokes must stay easy to run before CLI release and broad desktop runtime controls. | Unit tests prove command construction, but not installed Apple `container` JSON shapes or runtime behavior. | Use `scripts/apple_container_smoke.sh` as the opt-in local smoke. It runs `runhaven doctor`, `runhaven plan shell`, a minimal `runhaven run shell` smoke, active-run status/logs-follow/stop/repair cleanup, provider plan guidance, and exact resource cleanup checks. Keep it out of hosted CI while alpha CI is disabled. | `scripts/apple_container_smoke.sh` exits 0 on macOS 26+ with Apple `container` 1.0.0 and records cleanup evidence. |
| P0 | Provider-mode live smoke must stay covered in Rust. | Provider mode depends on host-only networking, network inspect schema, Apple gateway reachability, subnet-restricted host proxy binding, proxy env injection, and cleanup. | Use `scripts/apple_container_smoke.sh --with-provider` for `v0.5.0`, release, and provider-sensitive desktop evidence. Keep the default smoke usable without live provider network/proxy dependencies. | Provider smoke passes with allowed proxied HTTPS, denied proxied host, denied proxied IP literal, denied direct DNS/IP egress, and no leftover provider network. |
| P2 | SSH forwarding is not usable by the default non-root agent user on the current pinned runtime. | `--ssh` uses Apple `container --ssh`, which should forward an SSH agent socket rather than raw keys, but a visible socket is not enough if the `agent` user cannot connect to it. | Keep `--ssh` fail-closed in planner and run paths. Do not mount raw SSH keys or switch the default agent user to root. Investigate Apple `container` 1.0.0 non-root socket permissions or wait for an upstream runtime fix before treating `--ssh` as proven. | The gap closes only after a no-secret live smoke proves a disposable empty agent from the non-root guest and `ssh-add -l` returns the no-identities status, then `runhaven plan --ssh` and `runhaven run --ssh` are intentionally re-enabled with tests/docs updated. |

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
now verifies that `runhaven run --ssh` fails closed while the current
SSH-forwarding blocker remains unresolved. The Apple `container` release-update
playbook now covers runtime pin update evidence and rollback.
Re-run the live smoke before `v0.5.0`, before provider-sensitive desktop work,
and before release hardening; the remaining known runtime blocker is P2
SSH-forwarding unless provider or SSH runtime behavior changes again.
The consolidated non-UI backlog is tracked in
[`NON_UI_BACKLOG.md`](NON_UI_BACKLOG.md).

## Backlog Additions

- Run the `apple-container-expert` agent with the Apple Container skill for all
  future Apple `container` runtime, networking, source, service, registry,
  machine, and security-boundary work.
- Run the `rust-expert` agent with the Rust skill across the whole repo before
  release hardening to find correctness, idiomatic Rust, safety, test,
  packaging, and maintainability issues.
- Keep alpha hosted CI disabled. Frequent commits and pushes should rely on
  local verification until a maintainer explicitly re-enables pinned macOS CI.

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
