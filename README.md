# macos-container-agents

Run AI coding agents inside Apple `container` on macOS with safer defaults and
plain-language controls.

This repo targets people who should not need to understand containers,
sandboxing, SSH agents, or credential leakage to use Claude Code, Codex, Gemini,
Antigravity, Copilot, or a custom agent more safely.

## Status

Early foundation. The current CLI builds and previews Apple `container` commands
with a narrow filesystem boundary:

- one project mounted at `/workspace`
- one per-project agent home volume at `/home/agent`
- no macOS home mount
- no raw SSH key mount
- no host cloud credential mount
- no arbitrary environment passthrough
- read-only container root filesystem
- dropped Linux capabilities
- optional SSH agent forwarding with `--ssh`

Network egress allowlisting is not fully implemented yet. Internet-enabled runs
use Apple container's default network. Use `--network internal` only for
local-only work that does not need model-provider access from inside the guest.

## Requirements

- macOS 26+
- Apple silicon
- Python 3.13+
- Apple [`container`](https://github.com/apple/container) 1.0.0

The current recommended Python runtime is 3.14.6. CI also tests 3.13.13 as the
minimum supported maintenance release.

This repo currently reviews and pins Apple `container` 1.0.0. If Apple ships a
newer runtime, `mca doctor` should fail until the repo updates its runtime pin.

Install and start Apple `container` first:

```bash
container system start
```

Then check your machine:

```bash
python3.14 -m venv .venv
source .venv/bin/activate
python -m pip install --no-deps -e .
mca doctor
```

## Quick Start

Preview exactly what will run:

```bash
mca plan claude
```

Build a bundled image:

```bash
mca image build claude
```

Run an agent:

```bash
mca run claude
```

Pass a token by variable name only when needed:

```bash
mca run codex --env OPENAI_API_KEY
```

The generated command inherits `OPENAI_API_KEY` by name. It does not print the
secret value in the plan.

## Supported Profiles

```bash
mca agents
```

Current bundled profiles:

- `claude`
- `codex`
- `gemini`
- `antigravity`
- `copilot`
- `shell`

Use `shell` for any custom image:

```bash
mca plan shell --image my-agent:2026.06.14 -- my-agent --help
```

## Common Workflows

Read-only review:

```bash
mca run codex --read-only-workspace
```

Private Git access without mounting `~/.ssh`:

```bash
mca run claude --ssh
```

Local-only network:

```bash
mca run shell --network internal -- pytest
```

## Development

```bash
python3.14 -m venv .venv
source .venv/bin/activate
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

All package, image, tool, and CI action dependencies must be current stable and
hard-pinned. Do not commit floating version ranges, mutable `latest` tags,
major-only GitHub Action refs, unversioned installer scripts, or unpinned
package installs.

## Docs

- [Usage](docs/USAGE.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Security model](docs/SECURITY_MODEL.md)
- [Pinning policy](docs/PINNING.md)
- [Research and source ledger](docs/RESEARCH.md)
- [Roadmap](docs/ROADMAP.md)

## References

- [Apple container](https://github.com/apple/container)
- [Claude Code sandbox environments](https://code.claude.com/docs/en/sandbox-environments)
- [Claude Code dev containers](https://code.claude.com/docs/en/devcontainer)
- [Codex sandboxing](https://developers.openai.com/codex/concepts/sandboxing)
- [Codex approvals and security](https://developers.openai.com/codex/agent-approvals-security)
- [Gemini CLI configuration](https://google-gemini.github.io/gemini-cli/docs/get-started/configuration.html)
- [Antigravity CLI](https://antigravity.google/docs/cli-getting-started)
- [GitHub Copilot CLI](https://docs.github.com/en/copilot/how-tos/copilot-cli/use-copilot-cli/overview)
