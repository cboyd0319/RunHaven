<p align="center">
  <img src="docs/assets/logo.png" alt="RunHaven logo" width="180">
</p>

# RunHaven

![Python 3.13+](https://img.shields.io/badge/python-3.13%2B-blue)
![macOS 26+](https://img.shields.io/badge/macOS-26%2B-black)
![Apple container 1.0.0](https://img.shields.io/badge/apple%20container-1.0.0-555)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

Run AI coding agents inside Apple `container` on macOS 26+ with a narrow,
previewable local boundary.

RunHaven is for people who should not need to understand containers,
sandboxing, SSH agents, or credential leakage before using Claude Code, Codex,
Gemini, Antigravity, Copilot, or a custom agent on their Mac. It mounts one
selected project, gives the agent one isolated home volume, avoids host
secrets, and shows the exact Apple `container` command before anything runs.

[Installation](docs/INSTALLATION.md) |
[Capabilities](docs/CAPABILITIES.md) |
[Usage](docs/USAGE.md) |
[Security model](docs/SECURITY_MODEL.md) |
[Architecture](docs/ARCHITECTURE.md) |
[Research](docs/RESEARCH.md)

## Status

Early foundation. RunHaven is usable for local testing, image builds, and
provider-restricted agent runs.

RunHaven only supports macOS 26+ on Apple silicon. Windows and Linux are not
supported runtimes or contributor verification targets.

Use `runhaven plan` before `runhaven run`. Treat the default internet network
as unrestricted egress inside whatever Apple `container` and your host network
allow. Use `--network provider` when a run should be restricted to the bundled
provider host allowlist plus any reviewed fully qualified `--provider-host`
entries.

## Quick Start

Install and start Apple `container` first:

```bash
container system start
```

Install RunHaven from this checkout:

```bash
python3.14 -m venv .venv
source .venv/bin/activate
python -m pip install pip==26.1.2
python -m pip install --no-deps -e .
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

## Core Capabilities

- `runhaven plan` prints the workspace, state volume, network mode, egress
  status, preflight setup, and Apple `container run` command before execution.
- `runhaven run` mounts one selected workspace and one isolated per-project
  agent home volume.
- Bundled images run as a non-root `agent` user with a read-only root
  filesystem, temporary scratch space, and dropped Linux capabilities.
- `--workspace-scope current|git-root` keeps the default mount narrow and makes
  repository-root expansion explicit.
- `--worktree` runs agents in a RunHaven-owned git worktree so the source
  checkout stays untouched.
- `--network internal` supports local-only commands.
- `--network provider` routes normal agent runs through a host-side provider
  host allowlist proxy on an internal Apple `container` network.
- `--ssh` forwards the macOS SSH agent without mounting raw SSH keys.
- `--env NAME` passes one reviewed host environment variable by name; `NAME=value`
  is rejected.
- `runhaven runs ...`, `runhaven egress log`, and `runhaven auth ...` expose
  secret-free run, provider policy, and auth broker diagnostics.

See [Capabilities](docs/CAPABILITIES.md) for the full feature and limitation
overview.

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
| Preview a run | `runhaven plan claude` |
| Run an agent | `runhaven run claude` |
| Read-only review | `runhaven run codex --read-only-workspace` |
| Provider-restricted run | `runhaven run claude --network provider` |
| Local-only command | `runhaven run shell --network internal -- python -m unittest discover -s tests` |
| Worktree-isolated run | `runhaven run claude --worktree` |
| Merge worktree run | `runhaven runs merge <run-id>` |
| Recover worktree run | `runhaven runs recover <run-id>` |
| Keep worktree run | `runhaven runs keep <run-id>` |
| Discard worktree run | `runhaven runs discard <run-id>` |
| Recent runs | `runhaven runs list --limit 20` |
| Provider policy log | `runhaven egress log --limit 20` |
| Auth broker status | `runhaven auth status` |
| Isolated state volumes | `runhaven state list` |

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
python -m compileall src tests scripts
PYTHONPATH=src python -m unittest discover -s tests
python scripts/check_pins.py
git diff --check
```

Full local harness verification:

```bash
./init.sh
```

Docs-only changes should use the docs checks from
[the verification matrix](docs/harness/verification-matrix.md). Runtime,
security boundary, image, or install-flow changes need focused tests plus the
relevant Apple `container` smokes.

## License

[MIT](LICENSE)
