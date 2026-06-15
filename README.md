<p align="center">
  <img src="docs/assets/logo.png" alt="RunHaven logo" width="180">
</p>

# RunHaven

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

Early foundation. RunHaven is usable for local testing and image builds.
Provider egress mode now runs agents on an internal Apple `container` network
through a host-side allowlist proxy.

Use `runhaven plan` before `runhaven run`. Treat internet-enabled runs as
unrestricted egress inside whatever Apple `container` and your host network
allow.

RunHaven only supports macOS 26+ on Apple silicon. Windows and Linux are not
supported runtimes or contributor verification targets for this project.

Use `--network provider` to restrict normal agent runs to the bundled provider
host allowlist plus any explicit fully qualified `--provider-host HOST`
additions. A listed host permits that host and its subdomains. The provider
proxy resolves allowed hosts before connecting and rejects non-public resolved
addresses.

## What It Protects By Default

`runhaven` generates Apple `container` commands with these defaults:

- one selected project mounted at `/workspace`
- one per-project agent home volume mounted at the container agent home path
- no macOS home directory mount
- no raw SSH key mount
- no host cloud credential mount
- no arbitrary host environment passthrough
- read-only container root filesystem
- temporary container scratch directory
- dropped Linux capabilities
- non-root `agent` user in bundled images
- explicit command preview with `runhaven plan`

Useful opt-in controls:

- `--read-only-workspace` for review-only work
- `--network internal` for local-only commands
- `--network provider` for provider allowlisting through a runtime proxy
- `--provider-host HOST` to add an explicit fully qualified HTTPS host to
  provider mode
- `--ssh` for SSH agent forwarding without mounting `~/.ssh`
- `--env NAME` for passing a single host environment variable by name
- `--tty never` for non-interactive automation
- `--allow-sensitive-workspace` only when you intentionally want to mount a
  broad or credential-bearing host path
- `--allow-root-user` only when you intentionally want the agent process to run
  as root inside the container

## What It Does Not Solve Yet

This is not a complete data-loss or exfiltration solution.

- Internet mode does not yet restrict outbound domains.
- Provider mode uses conservative host allowlists; login, telemetry, or
  provider-side feature paths may need additional reviewed fully qualified
  `--provider-host` entries. Blocked hosts are grouped after provider runs with
  counts, denial reasons, run id, and suggested next actions, then recorded in
  the provider egress policy log.
- The selected agent can still read files inside the mounted workspace and its
  isolated agent home volume.
- If a credential is available inside the agent home volume or passed with
  `--env NAME`, malicious repository content may try to misuse it.
- Host-side auth brokering has an opt-in Codex API-key prototype. Other agent
  auth brokers remain design-only. `runhaven auth status` and `runhaven auth
  explain AGENT` explain the boundary without reading or printing secrets.
- Agent-native approval systems are useful, but they are not a replacement for
  the outer container boundary.

See [Security model](docs/SECURITY_MODEL.md) and [Security policy](SECURITY.md)
for the full boundary.

## Requirements

- macOS 26+
- Apple silicon
- Python 3.13+
- Apple [`container`](https://github.com/apple/container) 1.0.0

The recommended Python runtime is 3.14.6. CI also tests Python 3.13.14 as the
minimum supported maintenance release.

RunHaven does not support Windows or Linux. Use a macOS 26+ Apple silicon host
for development, verification, image builds, and runtime checks.

This repo intentionally pins Apple `container` 1.0.0. If Apple ships a newer
runtime, `runhaven doctor` should fail until the repo updates and verifies the new
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
runhaven doctor
```

Build and preview a bundled agent image:

```bash
runhaven image build claude
runhaven plan claude
```

Run the agent from the project directory you want it to work on:

```bash
runhaven run claude
```

## Plan Before Run

`runhaven plan` is the trust checkpoint. It prints the workspace, the isolated
state volume, preflight setup, network mode, and Apple `container run` command.
For provider mode, RunHaven injects proxy environment variables at runtime
after discovering the internal-network gateway.

Example shape:

```text
Workspace: selected project directory
State volume: runhaven-claude-...-home
Network: default internet network
Egress: unrestricted internet egress; domain allowlisting is not enforced
Preflight:
  container network create --internal runhaven-volume-prep-internal
  container run ... --no-dns --network runhaven-volume-prep-internal ...
Run:
  container run --rm --init --read-only --tmpfs <container-temp> --cap-drop ALL ...
```

If the plan shows a mount, environment variable, or network mode you do not
expect, stop before running it.

## Supported Agents

```bash
runhaven agents
```

Bundled profiles:

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

## Common Workflows

Read-only review:

```bash
runhaven run codex --read-only-workspace
```

Private Git access without mounting raw SSH keys:

```bash
runhaven run claude --ssh
```

Local-only command:

```bash
runhaven run shell --network internal -- python -m unittest discover -s tests
```

Provider-only mode:

```bash
runhaven plan claude --network provider
runhaven run claude --network provider
```

Bundled profiles include conservative provider hosts. A listed host permits
that host and its subdomains. See the reviewed
[provider endpoint matrix](docs/PROVIDER_ENDPOINTS.md) before adding custom
image hosts or extra provider endpoints. Add reviewed fully qualified hosts
explicitly:

```bash
runhaven run shell --network provider --provider-host api.example.com
```

Before adding a host, ask RunHaven why it would or would not be allowed:

```bash
runhaven why host api.openai.com --agent codex
runhaven why host api.example.com
```

After a provider run, inspect recent allowed and denied CONNECT decisions:

```bash
runhaven egress log --limit 20
runhaven egress log --json
```

Inspect recent agent runs without exposing command lines, agent arguments, or
secrets:

```bash
runhaven runs list --limit 20
runhaven runs show <run-id>
runhaven runs log <run-id>
runhaven runs show <run-id> --json
runhaven runs log <run-id> --json
```

Broker a Codex API key without placing the raw value in the guest:

```bash
runhaven run codex --network provider --codex-api-key-broker-env OPENAI_API_KEY
```

Or pass a token by variable name only when you deliberately want that value
inside the guest:

```bash
runhaven run codex --env OPENAI_API_KEY
```

`runhaven` rejects `NAME=value` so secrets do not get copied into shell history or
dry-run output.

Inspect the auth broker boundary:

```bash
runhaven auth status
runhaven auth explain codex
runhaven auth log --limit 20
```

These commands do not read credential stores or environment values. The auth log
records broker decisions without request bodies, token values, or environment
variable names.

List or remove isolated agent home volumes:

```bash
runhaven state list
runhaven state prune --yes
```

## Troubleshooting

Run this first:

```bash
runhaven doctor
```

If a run fails, collect these commands before opening an issue:

```bash
runhaven doctor
runhaven plan <agent>
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

Full local harness verification:

```bash
./init.sh
```

Optional individual checks:

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
- [Auth broker](docs/AUTH_BROKER.md)
- [Pinning policy](docs/PINNING.md)
- [Research and source ledger](docs/RESEARCH.md)
- [Roadmap](docs/ROADMAP.md)
- [Contributing](CONTRIBUTING.md)
- [Security policy](SECURITY.md)

## License

[MIT](LICENSE)
