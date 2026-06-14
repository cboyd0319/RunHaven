# Usage

## Install for Development

RunHaven development and runtime verification require macOS 26+ on Apple
silicon. Windows and Linux are not supported.

```bash
python3.14 -m venv .venv
source .venv/bin/activate
python -m pip install pip==26.1.2
python -m pip install -r requirements-dev.txt
python -m pip install --no-deps -e .
```

Development tools are exact-pinned in `pyproject.toml` and
`requirements-dev.txt`. When updating them, use the current stable release and
commit the exact new version.

## Check the Mac

```bash
runhaven doctor
```

`doctor` checks Python, macOS, Apple silicon, the pinned Apple `container`
version, and the Apple container system status when the CLI is installed. A
newer Apple `container` release should fail until this repo updates its reviewed
pin.

## Build an Agent Image

```bash
runhaven image build claude
runhaven image build codex
runhaven image build gemini
runhaven image build antigravity
runhaven image build copilot
```

Dry-run first if you want to inspect the exact build command:

```bash
runhaven image build claude --dry-run
```

## Preview a Run

```bash
cd /path/to/project
runhaven plan claude
```

The plan prints:

- the mounted workspace
- the per-project state volume
- the selected network mode
- the egress status for that network mode
- any preflight command
- the exact `container run` command

## Run an Agent

```bash
runhaven run claude
```

`runhaven` allows one active run per project/profile state volume. If another run is
already using the same isolated home volume, `runhaven` fails before starting Apple
`container` and tells you to wait or use a different workspace/profile.

RunHaven allocates an interactive TTY when attached to a terminal. Use
`--tty never` for non-interactive automation.

Pass a host environment variable by name only:

```bash
runhaven run claude --env ANTHROPIC_API_KEY
runhaven run codex --env OPENAI_API_KEY
runhaven run gemini --env GEMINI_API_KEY
runhaven run copilot --env COPILOT_GITHUB_TOKEN
```

`runhaven` intentionally rejects `NAME=value` so secrets do not get copied into shell
history or dry-run output.

## Read-Only Review

```bash
runhaven run codex --read-only-workspace
```

This lets an agent inspect the project without writing to the mounted
workspace.

## Local-Only Network

```bash
runhaven run shell --network internal -- python -m unittest discover -s tests
```

`internal` creates a host-only Apple container network before the run. Hosted AI
agent CLIs usually need internet access for model traffic, so this mode is most
useful for local commands and custom images.

## Provider Network

```bash
runhaven plan claude --network provider
runhaven run claude --network provider
```

`provider` creates a managed internal Apple `container` network and routes the
agent through RunHaven's host-side allowlist CONNECT proxy. Bundled profiles
include conservative provider hosts. Add reviewed extra hosts explicitly:

```bash
runhaven run shell --network provider --provider-host api.example.com
```

Run `runhaven plan` first. Provider plans show the managed provider network and
allowed hosts; the exact proxy URL is injected by `runhaven run` after the
internal-network gateway is inspected.

## Provider Egress Smoke

Build the base image and run the live smoke on macOS 26+:

```bash
runhaven image build shell
PYTHONPATH=src python3.14 scripts/provider_egress_smoke.py \
  --allowed-host api.openai.com \
  --allowed-url https://api.openai.com/ \
  --denied-host example.com
```

The smoke creates a temporary internal Apple `container` network and starts a
host-side allowlist CONNECT proxy. It passes only when the allowed proxied HTTPS
path succeeds and denied proxied host, proxied IP literal, direct DNS, and
direct IP paths fail. Normal provider runs use the same proxy enforcement
pattern through `runhaven run --network provider`.

## Private Git

```bash
runhaven run claude --ssh
```

This forwards the macOS SSH agent socket using Apple `container --ssh`. It does
not mount `~/.ssh`.

## State Volumes

```bash
runhaven state list
runhaven state prune --yes
```

`state list` shows RunHaven agent home volumes. `state prune --yes` deletes
those isolated agent home volumes and does not touch workspace files.
