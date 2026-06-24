<p align="center">
  <img src="docs/assets/logo.png" alt="RunHaven logo" width="180">
</p>

# RunHaven

![Rust 1.96.0](https://img.shields.io/badge/rust-1.96.0-orange)
![macOS 26+](https://img.shields.io/badge/macOS-26%2B-black)
![Apple container 1.0.0](https://img.shields.io/badge/apple%20container-1.0.0-555)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

> [!CAUTION]
> # ALPHA / PRE-RELEASE PROJECT
>
> RunHaven has not been deployed and has no external users yet. CLI contracts,
> container and image layouts, run-record formats, provider allowlists, auth
> broker behavior, and docs may change without backward-compatibility
> guarantees. RunHaven remains alpha/pre-release through the `v0.5.0`
> CLI-complete milestone.

RunHaven is a Rust CLI, with an alpha Tauri/Svelte desktop shell, for running
Claude Code, Codex, Gemini, Antigravity, Copilot, or custom coding agents
inside Apple `container` on macOS 26+. It does not replace those tools. It
gives them a repeatable local boundary so the secure path is easier to choose.

RunHaven only supports macOS 26+ on Apple silicon. Windows and Linux are not
supported runtimes or contributor verification targets.

## At A Glance

| Area | Current state |
| --- | --- |
| Status | Alpha/pre-release through the `v0.5.0` CLI-complete milestone. No deployed release or external users yet. |
| Product surface | The CLI is the current working product surface. |
| Desktop app | The Tauri/Svelte shell can read setup, dashboard, profile, folder-pick, and run-plan state. It supports confirmed launch, image readiness, sanitized live status, and opt-in bounded raw output snapshots. |
| Still CLI-first | Stop, kill, repair, image build, state cleanup, and worktree review controls. |
| Runtime | Apple `container` 1.0.0 on macOS 26+ with Apple silicon. |
| Safety posture | Mount one selected workspace, isolate agent home state, avoid ambient host credentials, and keep hard security boundaries fail-closed. |

## Release Targets

| Target | Release boundary |
| --- | --- |
| `v0.5.0` | CLI complete: command set, docs, JSON and local data decisions, runtime smokes, profile support tiers, diagnostics, cleanup, and security boundaries are finished and verified. RunHaven remains alpha/pre-release until after this milestone. |
| `v1.0.0` | First-class desktop release: the Tauri app becomes the easiest safe path for setup, image readiness and rebuild, planning, launch, live status, bounded output, stop, kill, repair, diagnostics, worktree review, cleanup, accessibility, signing, notarization, and release provenance. |
| `v1.x` | Post-v1 expansion: broader provider policy, non-Codex brokers, updater or installer management, MCP or extension workflows, and other work that should not block a secure desktop v1. |

## Product Rule

The secure path must be the easy path. Secure defaults should be the shortest
workflow. Supported lower-security choices should warn and require explicit
intent. Unsupported, invalid, or hard-boundary violations still fail closed.
That includes Apple `container machine`: it is not the default RunHaven
boundary, but explicit or user-managed machine workflows should be warned, not
blocked solely because they are less secure.

Start with [Installation](docs/INSTALLATION.md), [Capabilities](docs/CAPABILITIES.md),
[Usage](docs/USAGE.md), or the [Security model](docs/SECURITY_MODEL.md).

## Why RunHaven

AI coding agents can inspect a project, edit files, run commands, and iterate
quickly. That same power is risky when the agent runs directly on your Mac with
ambient access to your home directory, shell environment, SSH keys, cloud
credentials, browser profiles, and unrelated repositories.

| Need | RunHaven answer |
| --- | --- |
| Know what will run | `runhaven plan` prints the workspace, state volume, network mode, egress status, preflight, and Apple `container run` command before execution. |
| Avoid broad host access | Runs mount one selected workspace, not your whole home directory or credential folders. |
| Keep agent state separated | Each project/profile/session gets an isolated agent home volume. |
| Choose network scope | Use default internet, local-only `internal`, or provider allowlist proxy mode. |
| Review risky edits | `--worktree` runs in a RunHaven-owned git worktree that you can merge, keep, recover, or discard. |
| Recover local resources | RunHaven-owned images, volumes, networks, runs, egress logs, and auth broker state have explicit inspection and cleanup commands. |

RunHaven is not a complete data-loss or exfiltration solution. The selected
agent can still read the mounted workspace and its isolated home volume, and
default internet mode is not domain-restricted. See the
[security model](docs/SECURITY_MODEL.md) for the full boundary.

## Quick Start

Install and start Apple `container` first:

```bash
container system start
```

Install RunHaven from this checkout:

```bash
cargo install --path . --locked
```

Run the non-mutating setup guide, build an image, inspect the plan, then run
from the project directory you want the agent to work on:

```bash
runhaven setup
runhaven image build claude
runhaven plan claude
runhaven run claude
```

Use the smallest project directory the agent needs. RunHaven mounts that
directory at `/workspace`, not your whole home directory.

See [Installation](docs/INSTALLATION.md) for requirements and development
setup. See [Usage](docs/USAGE.md) for command-level workflows.

## Documentation

Use the smallest doc that matches the question:

| Need | Start here |
| --- | --- |
| Install and run | [Installation](docs/INSTALLATION.md), [Capabilities](docs/CAPABILITIES.md), [Usage](docs/USAGE.md) |
| Security and runtime boundary | [Security model](docs/SECURITY_MODEL.md), [Provider endpoints](docs/PROVIDER_ENDPOINTS.md), [Apple Container gap analysis](docs/APPLE_CONTAINER_GAP_ANALYSIS.md) |
| Architecture and auth | [Architecture](docs/ARCHITECTURE.md), [Auth broker](docs/AUTH_BROKER.md), [Research](docs/RESEARCH.md) |
| Desktop UI | [Tauri UI guardrails](docs/TAURI_UI_GUARDRAILS.md), [Tauri log viewing design](docs/TAURI_LOG_VIEWING_DESIGN.md), [Tauri UI research plan](docs/TAURI_UI_RESEARCH_PLAN.md) |
| Release planning | [Roadmap](docs/ROADMAP.md), [v0.5.0/v1.0.0 release plan](docs/V1_RELEASE_PLAN.md), [Release gap analysis](docs/RELEASE_GAP_ANALYSIS.md), [Pinning policy](docs/PINNING.md) |
| Project operations | [Harness](docs/harness/README.md), [Contributing](CONTRIBUTING.md), [Security policy](SECURITY.md) |

## Development

Agent-assisted work starts from three files only: [AGENTS.md](AGENTS.md),
[feature_list.json](feature_list.json), and [current-state.md](current-state.md).
Use the harness docs only when a task touches that surface.

Use the smallest relevant check for a change:

```bash
cargo fmt --check
cargo test --locked
cargo clippy --all-targets -- -D warnings
cargo run --locked --bin runhaven-check-pins
npm --prefix ui run check
npm --prefix ui test
npm --prefix ui run test:e2e
git diff --check
```

Full local harness verification:

```bash
./init.sh
```

Opt-in Apple `container` runtime smoke:

```bash
scripts/apple_container_smoke.sh
scripts/apple_container_smoke.sh --with-provider
```

Docs-only changes should use the docs checks from
[the verification matrix](docs/harness/feedback/verification-matrix.md). Runtime,
security boundary, image, or install-flow changes need focused tests plus the
relevant Apple `container` smokes.

## License

[MIT](LICENSE)
