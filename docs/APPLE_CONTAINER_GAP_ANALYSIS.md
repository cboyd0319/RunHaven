# Apple Container Gap Analysis

Last updated: 2026-06-16

Status: pre-Tauri action ledger.

RunHaven's core product boundary is Apple `container`. This document records
what is covered today, what is intentionally out of scope, and what should be
closed before Tauri/UI work starts.

## Evidence Used

- Host evidence: macOS 26.5.1, arm64, `/usr/local/bin/container`, Apple
  `container` CLI 1.0.0 build `release` commit `ee848e3`.
- Runtime service evidence: `container system status` reported running
  `container-apiserver` 1.0.0 commit `ee848e3`.
- Live Rust smoke evidence: `scripts/apple_container_smoke.sh --with-provider`
  passed on 2026-06-16, covering `doctor`, shell image readiness, internal
  read-only workspace behavior, active-run status/logs-follow/stop cleanup,
  provider allowlist behavior, denied provider/direct egress, and exact
  temporary state/network cleanup.
- Runtime property evidence: `container system version` and
  `container system property list` matched the reviewed Apple `container`
  commit, builder image `ghcr.io/apple/container-builder-shim/builder:0.12.0`,
  vminit image `ghcr.io/apple/containerization/vminit:0.33.3`, and Kata
  kernel `3.28.0` / `vmlinux-6.18.15-186`.
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
- RunHaven implementation evidence: `src/runhaven/runtime/plans/`,
  `src/runhaven/provider/runtime.rs`, `src/runhaven/runtime/network.rs`,
  `src/runhaven/runtime/active/`, `src/runhaven/image/`,
  `src/runhaven/cli/doctor.rs`, and product docs under `docs/`.

The local Apple source checkout was treated as source-map evidence. The
installed CLI help and RunHaven pins are the release-specific evidence for
Apple `container` 1.0.0 behavior.

## Covered Today

| Surface | Current Coverage | Primary Evidence |
| --- | --- | --- |
| Host prerequisites | `runhaven doctor` checks macOS, Apple silicon, installed CLI, pinned Apple `container` version, and service status. | `src/runhaven/cli/doctor.rs` |
| Task-scoped runs | `runhaven plan` builds a `container run` command with `--rm`, `--init`, `--read-only`, `--tmpfs /tmp`, `--cap-drop ALL`, CPU/memory limits, one workspace mount, one state volume, explicit env passthrough, and non-root bundled image defaults. | `src/runhaven/runtime/plans/mod.rs` |
| Sensitive mounts | Home directories, cloud credential folders, browser profiles, raw SSH keys, and broad system paths are rejected unless explicitly allowed. | `src/runhaven/runtime/plans/validation.rs`, `docs/SECURITY_MODEL.md` |
| State volume preparation | Non-root home volume ownership is prepared by a short-lived root container on an internal network with DNS disabled. | `src/runhaven/runtime/plans/mod.rs`, `src/runhaven/provider/runtime.rs` |
| Internal networks | Local-only runs create or reuse host-only Apple `container` networks and reject existing non-host-only networks. | `src/runhaven/provider/runtime.rs` |
| Provider egress | Provider mode creates a managed host-only network, inspects gateway/subnet, starts a host-side CONNECT proxy, injects proxy env vars, logs decisions, and deletes the managed provider network after the run. | `src/runhaven/provider/runtime.rs`, `src/runhaven/provider/egress.rs` |
| Active run control | `runs status`, `attach`, `logs-follow`, `stop`, `kill`, and `repair` route through Apple `container inspect`, `exec`, `logs`, `stop`, and `kill` with RunHaven-owned container-name validation. | `src/runhaven/runtime/active/` |
| Image lifecycle | `image build` uses Apple `container build` with source-digest labels. `image doctor` reads Apple image and volume listings without mutating resources. | `src/runhaven/image/build.rs`, `src/runhaven/image/doctor.rs` |
| Managed cleanup | `network list/prune`, `state list/prune/reset`, and repair commands operate on RunHaven-owned names only. | `src/runhaven/runtime/network.rs`, `src/runhaven/runtime/state.rs` |
| Opt-in live smoke | `scripts/apple_container_smoke.sh` proves the internal runtime path by default and adds provider egress coverage with `--with-provider`; it is intentionally local-only while alpha CI is disabled. | `scripts/apple_container_smoke.sh`, `docs/harness/feedback/verification-matrix.md` |
| JSON parser fixtures | Trimmed fixtures cover Apple `container` 1.0.0 image list, network inspect, container inspect, source-backed legacy inspect attachment aliases, and missing-container stderr classification without requiring live Apple `container` in unit tests. | `tests/fixtures/apple_container/`, `src/runhaven/image/doctor.rs`, `src/runhaven/provider/runtime.rs`, `src/runhaven/runtime/active/` |
| Machine avoidance | RunHaven uses task-scoped `container run`, not `container machine`, because machine defaults can mount the host home directory read-write. | `docs/ARCHITECTURE.md`, Apple `docs/container-machine.md` |

## Intentional Decisions

- Keep exact Apple `container` pinning. A newer runtime should fail `doctor`
  until this repo updates pins, source evidence, docs, and smokes.
- Do not replace managed cleanup with native broad prune commands. Native
  `container network prune` and `container volume prune` are useful Apple
  commands, but RunHaven must only delete resources it owns.
- Do not use `container machine` for default agent runs. The home-mount model
  is useful for development machines and wrong for beginner-safe AI agent
  boundaries.
- Do not automate Apple `container` installs, uninstalls, service restarts,
  registry pushes, registry credential changes, or machine deletion without an
  explicit user approval gate.

## Gaps To Close Before Tauri/UI

| Priority | Gap | Why It Matters | Suggested Action | Verification |
| --- | --- | --- | --- | --- |
| P0 | Rust-era live runtime smokes must stay easy to run before Tauri/UI work. | Unit tests prove command construction, but not installed Apple `container` JSON shapes or runtime behavior. | Use `scripts/apple_container_smoke.sh` as the opt-in local smoke. It runs `runhaven doctor`, `runhaven plan shell`, a minimal `runhaven run shell` smoke, active-run status/logs-follow/stop/repair cleanup, provider plan guidance, and exact resource cleanup checks. Keep it out of hosted CI while alpha CI is disabled. | `scripts/apple_container_smoke.sh` exits 0 on macOS 26+ with Apple `container` 1.0.0 and records cleanup evidence. |
| P0 | Provider-mode live smoke must stay covered in Rust. | Provider mode depends on host-only networking, network inspect schema, gateway binding, proxy env injection, and cleanup. | Use `scripts/apple_container_smoke.sh --with-provider` for release and pre-Tauri provider evidence. Keep the default smoke usable without live provider network/proxy dependencies. | Provider smoke passes with allowed proxied HTTPS, denied proxied host, denied proxied IP literal, denied direct DNS/IP egress, and no leftover provider network. |
| P1 | `doctor` does not yet enforce the full Apple `container` pin surface. | Repo policy records CLI version, commit, installer hash, builder image, vminit image, and kernel. `doctor` currently checks CLI semver and service status only, so config overrides can drift while version still says 1.0.0. | Extend `doctor` or add a read-only runtime-pin check that compares `container system version` and `container system property list` to `pins.toml`, or narrow docs if the repo only intends a softer advisory. | `runhaven doctor` or a focused check reports mismatched commit, builder image, vminit image, or kernel without mutating config. |
| P1 | Image builder lifecycle is not surfaced. | `container build` uses a builder VM with its own CPU, memory, image, and possible stale state. A GUI build button needs actionable diagnostics instead of a raw failure. | Extend `image doctor` or setup docs to read `container builder status` when available and explain builder start/delete/resource remedies without mutating by default. | `image doctor` smoke reports builder guidance; no builder mutation occurs without an explicit command. |
| P1 | Tauri needs resource guardrails for per-container VM behavior. | Apple `container` runs each container in a lightweight VM, and freed guest pages may not be returned promptly to macOS. Multiple UI-started agents could exhaust host memory. | Before UI work, define active-run count, memory-limit display, and warning rules. Prefer `container stats` or curated active-run metadata for UI status. | UI design spec or CLI JSON contract includes memory and concurrency warning behavior. |
| P1 | Service and install operations need UI approval gates. | `container system stop`, update, uninstall, registry login/logout, machine deletion, image deletion, and volume deletion can disrupt workloads or remove data. | Add a Tauri permission model note before implementing UI controls: read-only status by default, explicit confirmation for service mutation, credentials, deletion, and registry actions. | Tauri plan names approval gates before any service or credential command is wired. |
| P1 | Provider-mode local-network/privacy troubleshooting is not documented. | The proxy binds on the host side and must be reachable from a host-only Apple `container` network. Apple source includes local-network privacy handling, so failures may present as macOS permission or reachability issues rather than RunHaven policy errors. | During the provider smoke pass, capture any macOS local-network/privacy prompts or failures and add a troubleshooting note if observed. | Provider smoke proves guest-to-host proxy reachability on macOS 26.5.1, or docs record the observed prompt/failure mode and next action. |
| P2 | SSH forwarding lacks a live RunHaven smoke. | `--ssh` uses Apple `container --ssh`, which mounts an SSH agent socket rather than raw keys. This is safer but still a credential boundary. | Add a no-secret smoke that verifies the planned command shape and, when a disposable agent socket is available, confirms the guest sees only the forwarded socket path. | Plan test plus optional live smoke documented as skipped when no disposable socket exists. |
| P2 | Apple `container` release update playbook is manual. | Runtime pin updates require source review, helper version review, command help review, docs updates, and live smokes. | Add a small release-update checklist under docs or harness release controls. | Checklist names version, installer SHA, signing team ID, helper versions, command help diffs, and smoke commands. |

Current P0 and parser-fixture status: the opt-in Rust smoke harness exists and
passed with provider coverage on 2026-06-16, and Apple `container` JSON parser
fixtures are covered by `cargo test --locked`. Re-run the live smoke before
Tauri/UI work and before release hardening; the remaining known pre-Tauri gaps
are P1/P2 unless provider runtime behavior changes again.

## Backlog Additions

- Run the `apple-container-expert` agent with the Apple Container skill for all
  future Apple `container` runtime, networking, source, service, registry,
  machine, and security-boundary work.
- Run the `rust-expert` agent with the Rust skill across the whole repo before
  release hardening to find correctness, idiomatic Rust, safety, test,
  packaging, and maintainability issues.
- Keep alpha hosted CI disabled. Frequent commits and pushes should rely on
  local verification until a maintainer explicitly re-enables pinned macOS CI.

## Not Before Tauri

These are valuable, but they should not block the first UI planning pass after
the gaps above are handled:

- Registry push/pull/login UI.
- `container machine` management UI.
- Apple `container` source builds or local Containerization package editing.
- Automated Apple `container` install, update, uninstall, or service restart.
- Release signing, SBOM, and provenance automation.
