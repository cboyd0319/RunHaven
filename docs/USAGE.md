# Usage

## Install for Development

```bash
python3.14 -m venv .venv
source .venv/bin/activate
python -m pip install -r requirements-dev.txt
python -m pip install --no-deps -e .
```

Development tools are exact-pinned in `pyproject.toml` and
`requirements-dev.txt`. When updating them, use the current stable release and
commit the exact new version.

## Check the Mac

```bash
mca doctor
```

`doctor` checks Python, macOS, Apple silicon, the pinned Apple `container`
version, and the Apple container system status when the CLI is installed. A
newer Apple `container` release should fail until this repo updates its reviewed
pin.

## Build an Agent Image

```bash
mca image build claude
mca image build codex
mca image build gemini
mca image build antigravity
mca image build copilot
```

Dry-run first if you want to inspect the exact build command:

```bash
mca image build claude --dry-run
```

## Preview a Run

```bash
cd /path/to/project
mca plan claude
```

The plan prints:

- the mounted workspace
- the per-project state volume
- the selected network mode
- any preflight command
- the exact `container run` command

## Run an Agent

```bash
mca run claude
```

`mca` allows one active run per project/profile state volume. If another run is
already using the same isolated home volume, `mca` fails before starting Apple
`container` and tells you to wait or use a different workspace/profile.

Pass a host environment variable by name only:

```bash
mca run claude --env ANTHROPIC_API_KEY
mca run codex --env OPENAI_API_KEY
mca run gemini --env GEMINI_API_KEY
mca run copilot --env COPILOT_GITHUB_TOKEN
```

`mca` intentionally rejects `NAME=value` so secrets do not get copied into shell
history or dry-run output.

## Read-Only Review

```bash
mca run codex --read-only-workspace
```

This lets an agent inspect the project without writing to the mounted
workspace.

## Local-Only Network

```bash
mca run shell --network internal -- pytest
```

`internal` creates a host-only Apple container network before the run. Hosted AI
agent CLIs usually need internet access for model traffic, so this mode is most
useful for local commands and custom images.

## Private Git

```bash
mca run claude --ssh
```

This forwards the macOS SSH agent socket using Apple `container --ssh`. It does
not mount `~/.ssh`.
