# macos-container-agents

[![CI](https://github.com/cboyd0319/macos-container-agents/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/cboyd0319/macos-container-agents/actions/workflows/ci.yml)
![Python 3.13+](https://img.shields.io/badge/python-3.13%2B-blue)
![macOS 26+](https://img.shields.io/badge/macOS-26%2B-black)
![Apple container 1.0.0](https://img.shields.io/badge/apple%20container-1.0.0-555)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

Run Claude Code, Codex, Gemini, Antigravity, Copilot, or a custom AI agent
inside Apple `container` with beginner-safe local defaults.

This repo is for people who should not need to understand containers,
sandboxing, SSH agents, or credential leakage before using an AI coding agent on
their Mac. The default path mounts one project, gives the agent one isolated
home volume, avoids host secrets, and shows the exact container command before
anything runs.

[Quick start](#quick-start) |
[Supported agents](#supported-agents) |
[Security model](docs/SECURITY_MODEL.md) |
[Troubleshooting](#troubleshooting) |
[Development](#development) |
[Research](docs/RESEARCH.md)

## Status

Early foundation. The CLI is usable for local testing and image builds, but
network egress allowlisting is not complete yet.

Use `mca plan` before `mca run`. Treat internet-enabled runs as unrestricted
egress inside whatever Apple `container` and your host network allow.

## What It Protects By Default

`mca` generates Apple `container` commands with these defaults:

- one selected project mounted at `/workspace`
- one per-project agent home volume mounted at `/home/agent`
- no macOS home directory mount
- no raw SSH key mount
- no host cloud credential mount
- no arbitrary host environment passthrough
- read-only container root filesystem
- temporary `/tmp`
- dropped Linux capabilities
- non-root `agent` user in bundled images
- explicit command preview with `mca plan`

Useful opt-in controls:

- `--read-only-workspace` for review-only work
- `--network internal` for local-only commands
- `--ssh` for SSH agent forwarding without mounting `~/.ssh`
- `--env NAME` for passing a single host environment variable by name

## What It Does Not Solve Yet

This is not a complete data-loss or exfiltration solution.

- Internet mode does not yet restrict outbound domains.
- The selected agent can still read files inside `/workspace` and `/home/agent`.
- If a credential is available inside the agent home volume or passed with
  `--env NAME`, malicious repository content may try to misuse it.
- Agent-native approval systems are useful, but they are not a replacement for
  the outer container boundary.

See [Security model](docs/SECURITY_MODEL.md) and [Security policy](SECURITY.md)
for the full boundary.

## Requirements

- macOS 26+
- Apple silicon
- Python 3.13+
- Apple [`container`](https://github.com/apple/container) 1.0.0

The recommended Python runtime is 3.14.6. CI also tests Python 3.13.13 as the
minimum supported maintenance release.

This repo intentionally pins Apple `container` 1.0.0. If Apple ships a newer
runtime, `mca doctor` should fail until the repo updates and verifies the new
runtime pin.

## Quick Start

Install and start Apple `container` first:

```bash
container system start
```

Install this repo in a local virtual environment:

```bash
python3.14 -m venv .venv
source .venv/bin/activate
python -m pip install pip==26.1.2
python -m pip install --no-deps -e .
```

Check the Mac before running an agent:

```bash
mca doctor
```

Build and preview a bundled agent image:

```bash
mca image build claude
mca plan claude
```

Run the agent from the project directory you want it to work on:

```bash
mca run claude
```

## Plan Before Run

`mca plan` is the trust checkpoint. It prints the workspace, the isolated state
volume, preflight setup, network mode, and exact Apple `container run` command.

Example shape:

```text
Workspace: /Users/me/code/my-project
State volume: mca-claude-...-home
Network: default internet network
Preflight:
  container network create --internal mca-volume-prep-internal
  container run ... --no-dns --network mca-volume-prep-internal ...
Run:
  container run --rm --init --read-only --tmpfs /tmp --cap-drop ALL ...
```

If the plan shows a mount, environment variable, or network mode you do not
expect, stop before running it.

## Supported Agents

```bash
mca agents
```

Bundled profiles:

| Profile | Default image | Use case |
| --- | --- | --- |
| `claude` | `mca/claude:0.1.0` | Claude Code with isolated project state |
| `codex` | `mca/codex:0.1.0` | Codex CLI with its own workspace sandbox enabled |
| `gemini` | `mca/gemini:0.1.0` | Gemini CLI with project-scoped home state |
| `antigravity` | `mca/antigravity:0.1.0` | Antigravity CLI in the same container boundary |
| `copilot` | `mca/copilot:0.1.0` | GitHub Copilot CLI with isolated state |
| `shell` | `mca/base:0.1.0` | Generic shell profile for custom agent images |

Use `shell` for another agent image:

```bash
mca plan shell --image my-agent:2026.06.14 -- my-agent --help
```

## Common Workflows

Read-only review:

```bash
mca run codex --read-only-workspace
```

Private Git access without mounting raw SSH keys:

```bash
mca run claude --ssh
```

Local-only command:

```bash
mca run shell --network internal -- pytest
```

Pass a token by variable name only:

```bash
mca run codex --env OPENAI_API_KEY
```

`mca` rejects `NAME=value` so secrets do not get copied into shell history or
dry-run output.

## Troubleshooting

Run this first:

```bash
mca doctor
```

If a run fails, collect these commands before opening an issue:

```bash
mca doctor
mca plan <agent>
container system status
```

Do not paste secret values, API keys, SSH keys, or private repository contents
into issues.

## Development

```bash
python3.14 -m venv .venv
source .venv/bin/activate
python -m pip install pip==26.1.2
python -m pip install -r requirements-dev.txt
python -m pip install --no-deps -e .
python -m compileall src tests scripts
PYTHONPATH=src python -m unittest discover -s tests
python scripts/check_pins.py
```

Optional checks:

```bash
python -m ruff check .
python -m mypy src
python -m build
```

## Pinning Rule

All package, image, tool, and CI action dependencies must use the current stable
release and be hard-pinned. Do not commit floating version ranges, mutable
`latest` tags, major-only GitHub Action refs, unversioned installer scripts, or
unpinned package installs.

Current pins are recorded in [pins.toml](pins.toml). The source ledger is
[docs/RESEARCH.md](docs/RESEARCH.md).

## Documentation

- [Usage](docs/USAGE.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Security model](docs/SECURITY_MODEL.md)
- [Pinning policy](docs/PINNING.md)
- [Research and source ledger](docs/RESEARCH.md)
- [Roadmap](docs/ROADMAP.md)
- [Contributing](CONTRIBUTING.md)
- [Security policy](SECURITY.md)

## License

[MIT](LICENSE)
