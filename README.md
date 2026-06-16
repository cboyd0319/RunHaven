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

RunHaven is a Rust CLI for running AI coding agents inside Apple `container`
on macOS 26+. It gives every run a previewable local boundary: one selected
workspace, one isolated agent home volume, an explicit network mode, a pinned
agent image, and secret-free run records for review and recovery.

It is for people who want the power of Claude Code, Codex, Gemini,
Antigravity, Copilot, or a custom agent without casually handing an agent their
whole Mac. RunHaven does not replace those tools. It wraps them in a repeatable
container runtime that makes the safer path the default.

RunHaven only supports macOS 26+ on Apple silicon. Windows and Linux are not
supported runtimes or contributor verification targets.

[Installation](docs/INSTALLATION.md) |
[Capabilities](docs/CAPABILITIES.md) |
[Usage](docs/USAGE.md) |
[Security model](docs/SECURITY_MODEL.md) |
[Architecture](docs/ARCHITECTURE.md) |
[Research](docs/RESEARCH.md)

## What It Is

RunHaven is the outer runtime layer for local AI coding agents:

- a command planner that prints the Apple `container run` command before
  execution;
- a profile system for bundled Claude, Codex, Gemini, Antigravity, Copilot, and
  generic shell images;
- a narrow workspace mount policy that defaults to the current directory
  instead of a whole home directory or silent git-root expansion;
- a per-project/profile/session home volume so agent state stays isolated from
  the host and from other projects;
- a run ledger with secret-free metadata, active-run controls, logs, diffs, and
  recovery commands.

It is not a complete data-loss or exfiltration solution. The selected agent can
still read the mounted workspace and its own isolated home volume, and default
internet mode is not domain-restricted. RunHaven's job is to make the outer
container boundary explicit, narrow, inspectable, and recoverable.

## Why You Want It

AI coding agents are useful because they can inspect a project, edit files, run
commands, and iterate quickly. That same power is risky when the agent runs
directly on your Mac with ambient access to your home directory, dotfiles, SSH
keys, cloud credentials, browser profiles, unrelated repositories, and shell
environment.

RunHaven gives you a safer default workflow:

- preview the exact container command before the agent starts;
- mount only the project directory you intentionally selected;
- keep the agent's login/cache state in an isolated project volume;
- forward SSH through the macOS SSH agent without mounting raw private keys;
- pass environment variables only by reviewed name, never `NAME=value`;
- run local-only tasks with `--network internal`;
- use provider mode when you want a host allowlist proxy instead of broad
  internet egress;
- run risky edits in a RunHaven-owned git worktree and merge, keep, recover, or
  discard the result explicitly.

## What Makes It Cool

| Capability | Why it matters |
| --- | --- |
| Plan-first execution | `runhaven plan` shows the workspace, state volume, network, egress status, preflight, and full Apple `container run` command before anything mutates. |
| Beginner-safe defaults | Bundled images run as a non-root `agent` user with a read-only root filesystem, dropped Linux capabilities, temporary scratch space, and no host home or credential folder mount. |
| Project-scoped memory | Each project/profile/session gets its own agent home volume, so logins and caches can be reused without mixing unrelated projects. |
| Provider egress mode | `--network provider` runs the agent on an internal Apple `container` network and routes HTTP CONNECT traffic through a host-side reviewed provider allowlist proxy. |
| Worktree isolation | `--worktree` runs the agent in a RunHaven-owned git worktree so the source checkout stays untouched until you explicitly merge. |
| Secret-free observability | `runhaven runs ...`, `runhaven egress log`, and `runhaven auth ...` expose run status, provider policy, broker status, and recovery paths without storing prompts, command lines, env values, request bodies, token values, or diffs. |
| Local resource repair | `image doctor`, `image rebuild`, `state reset`, `network list`, and `network prune` focus only on RunHaven-owned images, volumes, and networks. |
| Tauri-ready shape | The Rust CLI is organized around separate runtime, provider, image, records, and support modules so a future desktop UI can call the same core behavior. |

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

## Supported Agents

```bash
runhaven agents
```

| Profile | Default image | Use case |
| --- | --- | --- |
| `claude` | `runhaven/claude:0.1.0` | Claude Code with isolated project state |
| `codex` | `runhaven/codex:0.1.0` | Codex CLI with its own workspace sandbox enabled |
| `gemini` | `runhaven/gemini:0.1.0` | Gemini CLI with project-scoped home state |
| `antigravity` | `runhaven/antigravity:0.1.0` | Antigravity CLI in the same container boundary |
| `copilot` | `runhaven/copilot:0.1.0` | GitHub Copilot CLI with isolated state |
| `shell` | `runhaven/base:0.1.0` | Generic shell profile for custom agent images |

Use `shell` for another agent image:

```bash
runhaven plan shell --image my-agent:2026.06.14 -- my-agent --help
```

## Common Commands

| Goal | Command |
| --- | --- |
| Guided first-run check | `runhaven setup` |
| Host prerequisite check | `runhaven doctor` |
| Build a bundled image | `runhaven image build claude` |
| Rebuild a bundled image | `runhaven image rebuild claude` |
| Diagnose bundled images | `runhaven image doctor` |
| Preview a run | `runhaven plan claude` |
| Run an agent | `runhaven run claude` |
| Run with a named session | `runhaven run claude --session review` |
| Read-only review | `runhaven run codex --read-only-workspace` |
| Provider-restricted run | `runhaven run claude --network provider` |
| Local-only command | `runhaven run shell --network internal -- cargo test` |
| Worktree-isolated run | `runhaven run claude --worktree` |
| Merge worktree run | `runhaven runs merge <run-id>` |
| Recover worktree run | `runhaven runs recover <run-id>` |
| Recover worktree run as JSON | `runhaven runs recover <run-id> --json` |
| Keep worktree run | `runhaven runs keep <run-id>` |
| Discard worktree run | `runhaven runs discard <run-id>` |
| Recent runs | `runhaven runs list --limit 20` |
| Provider policy log | `runhaven egress log --limit 20` |
| Auth broker status | `runhaven auth status` |
| Isolated state volumes | `runhaven state list` |
| Reset one session | `runhaven state reset claude --session review --yes` |
| Managed networks | `runhaven network list` |
| Prune managed networks | `runhaven network prune --yes` |

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
- [Pinning policy](docs/PINNING.md): exact dependency and image pin rules.
- [Roadmap](docs/ROADMAP.md): planned product and codebase work.
- [Contributing](CONTRIBUTING.md): local checks and review expectations.
- [Security policy](SECURITY.md): supported security reporting scope.

## Development

Use the smallest relevant check for a change:

```bash
cargo fmt --check
cargo test --locked
cargo run --locked --bin runhaven-check-pins
git diff --check
```

Full local harness verification:

```bash
./init.sh
```

Docs-only changes should use the docs checks from
[the verification matrix](docs/harness/feedback/verification-matrix.md). Runtime,
security boundary, image, or install-flow changes need focused tests plus the
relevant Apple `container` smokes.

## License

[MIT](LICENSE)
