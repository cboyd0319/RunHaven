<p align="center">
  <img src="docs/assets/logo.png" alt="RunHaven logo" width="180">
</p>

# RunHaven

![Rust 1.96.0](https://img.shields.io/badge/rust-1.96.0-orange)
![macOS 26+](https://img.shields.io/badge/macOS-26%2B-black)
![Apple container 1.0.0](https://img.shields.io/badge/apple%20container-1.0.0-555)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

> [!CAUTION]
> **Alpha / pre-release.** RunHaven has no deployed release and no external
> users yet. CLI contracts, container and image layouts, run-record formats,
> provider allowlists, auth-broker behavior, and docs can change without
> backward-compatibility guarantees through the `v0.5.0` CLI-complete milestone.

RunHaven is a Rust CLI (with an alpha Tauri/Svelte desktop shell) for running
Claude Code, Codex, Gemini, Antigravity, Copilot, or custom coding agents inside
Apple `container` on macOS 26+ (Apple silicon only). It does not replace those
tools. It gives them a repeatable local boundary so the secure way to run an
agent is also the easy way.

## Why RunHaven

AI coding agents can inspect a project, edit files, run commands, and iterate
fast. That same power is risky when the agent runs directly on your Mac with
ambient access to your home directory, shell environment, SSH keys, cloud
credentials, browser profiles, and unrelated repositories. RunHaven puts each
run inside a bounded sandbox:

| Need | RunHaven answer |
| --- | --- |
| Know what will run | `runhaven plan` prints the workspace, state volume, network mode, egress status, preflight, and Apple `container run` command before execution. |
| Avoid broad host access | Runs mount one selected workspace, not your whole home directory or credential folders. |
| Keep agent state separated | Each agent gets an isolated home volume; log in once and reuse it, or isolate per project. |
| Choose network scope | Secure default per profile (provider allowlist where the agent's hosts are bundled, otherwise internet), plus local-only `internal` and an explicit `--network` override. |
| Review risky edits | `--worktree` runs in a RunHaven-owned git worktree you can merge, keep, recover, or discard. |
| Recover local resources | RunHaven-owned images, volumes, networks, runs, egress logs, and auth-broker state have explicit inspection and cleanup commands. |

RunHaven is not a complete data-loss or exfiltration solution. The selected
agent can still read the mounted workspace and its isolated home volume, and
`internet` mode is not domain-restricted. See the
[security model](docs/SECURITY_MODEL.md) for the full boundary.

## Quick start

Start Apple `container`, then install RunHaven from this checkout:

```bash
container system start
cargo install --path . --locked
```

Run the non-mutating setup guide, build an agent image, inspect the plan, then
run from the project directory you want the agent to work on:

```bash
runhaven setup
runhaven image build claude
runhaven plan claude
runhaven run claude
```

Use the smallest project directory the agent needs. RunHaven mounts that
directory at `/workspace`, not your whole home directory. See
[Installation](docs/INSTALLATION.md) for requirements and
[Usage](docs/USAGE.md) for command-level workflows.

## Status and roadmap

RunHaven is alpha/pre-release. The CLI is the working product surface today. The
Tauri/Svelte desktop shell can read setup, dashboard, profile, folder-pick, and
run-plan state and supports confirmed launch, image readiness, sanitized live
status, and bounded output snapshots; stop, kill, repair, image build, state
cleanup, and worktree review remain CLI-first.

Release boundaries (full detail in [ROADMAP.md](docs/ROADMAP.md) and the
[release plan](docs/V1_RELEASE_PLAN.md)):

- **`v0.5.0`, CLI complete:** the command set, docs, JSON and local-data
  decisions, runtime smokes, profile support tiers, diagnostics, cleanup, and
  security boundaries finished and verified. Alpha ends after this milestone.
- **`v1.0.0`, first-class desktop:** the Tauri app becomes the easiest safe path
  for the full workflow, signed, notarized, and backed by release provenance.
- **`v1.x` and later:** broader provider policy, more credential brokers,
  updater or installer management, MCP or extension workflows, and a terminal
  UI.

## Product rule

The secure path must be the easy path: secure defaults are the shortest
workflow, supported lower-security choices warn and require explicit intent, and
unsupported, invalid, or hard-boundary violations fail closed. Apple
`container machine` is not the default boundary; explicit or user-managed machine
workflows are warned, not blocked solely for being less secure.

## Documentation

Use the smallest doc that matches the question:

| Need | Start here |
| --- | --- |
| Install and run | [Installation](docs/INSTALLATION.md), [Capabilities](docs/CAPABILITIES.md), [Usage](docs/USAGE.md) |
| Security and runtime boundary | [Security model](docs/SECURITY_MODEL.md), [Provider endpoints](docs/PROVIDER_ENDPOINTS.md), [Apple Container gap analysis](docs/APPLE_CONTAINER_GAP_ANALYSIS.md) |
| Architecture and auth | [Architecture](docs/ARCHITECTURE.md), [Auth broker](docs/AUTH_BROKER.md), [Research](docs/RESEARCH.md) |
| Desktop UI | [Tauri UI guardrails](docs/TAURI_UI_GUARDRAILS.md), [Tauri log viewing design](docs/TAURI_LOG_VIEWING_DESIGN.md), [Tauri UI research plan](docs/TAURI_UI_RESEARCH_PLAN.md) |
| Release planning | [Roadmap](docs/ROADMAP.md), [v0.5.0/v1.0.0 release plan](docs/V1_RELEASE_PLAN.md), [Release gap analysis](docs/RELEASE_GAP_ANALYSIS.md), [CLI surface coverage](docs/CLI_SURFACE_COVERAGE.md), [Pinning policy](docs/PINNING.md) |
| Project operations | [Harness](docs/harness/README.md), [Contributing](CONTRIBUTING.md), [Security policy](SECURITY.md) |

## Development

Agent-assisted work starts from three files only: [AGENTS.md](AGENTS.md),
[feature_list.json](feature_list.json), and
[current-state.md](current-state.md). Load the harness docs only when a task
touches that surface.

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

Full local harness verification is `./init.sh`. Opt-in Apple `container` runtime
smokes are `scripts/apple_container_smoke.sh` (add `--with-provider` for the
egress checks). Docs-only changes use the docs checks from
[the verification matrix](docs/harness/feedback/verification-matrix.md); runtime,
security-boundary, image, or install-flow changes need focused tests plus the
relevant Apple `container` smokes.

## License

[MIT](LICENSE)
