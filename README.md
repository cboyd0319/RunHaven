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
> guarantees until maintainers declare an explicit release boundary.

RunHaven is a Rust CLI, with an early Tauri/Svelte desktop shell, for running
Claude Code, Codex, Gemini, Antigravity, Copilot, or custom coding agents
inside Apple `container` on macOS 26+. It does not replace those tools; it
gives them a repeatable local boundary so the safer path is easier to choose.

RunHaven only supports macOS 26+ on Apple silicon. Windows and Linux are not
supported runtimes or contributor verification targets.

The CLI is the current working product surface. The desktop scaffold under
`ui/` and `src-tauri/` is alpha UI work: it reads setup, dashboard, profile,
folder-pick, and run-plan state, and it has a first explicitly confirmed
launch path with sanitized live run status plus opt-in bounded raw output
snapshots. Stop, kill, repair, image build, state cleanup, and worktree review
controls are still CLI-first.

[Installation](docs/INSTALLATION.md) |
[Capabilities](docs/CAPABILITIES.md) |
[Usage](docs/USAGE.md) |
[Security model](docs/SECURITY_MODEL.md) |
[Architecture](docs/ARCHITECTURE.md) |
[Research](docs/RESEARCH.md) |
[Harness](docs/harness/README.md)

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

- [Installation](docs/INSTALLATION.md): requirements, local install, first run,
  and verification.
- [Capabilities](docs/CAPABILITIES.md): feature overview, defaults, limits, and
  network modes.
- [Usage](docs/USAGE.md): command-level workflows and examples.
- [Security model](docs/SECURITY_MODEL.md): trust boundary, safe defaults, and
  current risks.
- [Provider endpoints](docs/PROVIDER_ENDPOINTS.md): reviewed provider host
  matrix.
- [Auth broker](docs/AUTH_BROKER.md): Codex API-key broker prototype and
  future broker criteria.
- [Architecture](docs/ARCHITECTURE.md): runtime pattern, profiles, networking,
  records, and broker model.
- [Apple Container gap analysis](docs/APPLE_CONTAINER_GAP_ANALYSIS.md):
  pre-Tauri runtime, security, and verification gaps.
- [Tauri UI guardrails](docs/TAURI_UI_GUARDRAILS.md):
  resource, approval, and permission boundaries for the desktop UI.
- [Pinning policy](docs/PINNING.md): exact dependency and image pin rules.
- [Roadmap](docs/ROADMAP.md): planned product and codebase work.
- [Harness](docs/harness/README.md): lightweight agent startup, state, and
  verification routing.
- [Contributing](CONTRIBUTING.md): local checks and review expectations.
- [Security policy](SECURITY.md): supported security reporting scope.

## Development

Agent-assisted work starts from three files only: [AGENTS.md](AGENTS.md),
[feature_list.json](feature_list.json), and [current-state.md](current-state.md).
Use the harness docs only when a task touches that surface.

Use the smallest relevant check for a change:

```bash
cargo fmt --check
cargo test --locked
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
