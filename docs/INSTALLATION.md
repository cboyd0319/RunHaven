# Installation

RunHaven is a macOS-only project. Runtime and contributor verification require
macOS 26+ on Apple silicon, Python 3.13+, and Apple `container` 1.0.0.
Windows and Linux are not supported.

## Requirements

- macOS 26+
- Apple silicon
- Python 3.13+
- Apple [`container`](https://github.com/apple/container) 1.0.0
- Git

The recommended Python runtime is 3.14.6. CI also tests Python 3.13.14 as the
minimum supported maintenance release.

RunHaven intentionally pins Apple `container` 1.0.0. If Apple ships a newer
runtime, `runhaven doctor` should fail until this repo updates and verifies the
new runtime pin.

## Apple Container

Install Apple `container` from Apple's project, then start the container
system:

```bash
container system start
```

## Install From This Checkout

Create a local virtual environment and install the CLI in editable mode:

```bash
python3.14 -m venv .venv
source .venv/bin/activate
python -m pip install pip==26.1.2
python -m pip install --no-deps -e .
```

For development checks, install the pinned development requirements:

```bash
python -m pip install -r requirements-dev.txt
```

Development tools are exact-pinned in `pyproject.toml`,
`requirements-dev.txt`, and `pins.toml`. When updating them, use the current
stable release and commit the exact new version.

Confirm the host before running an agent:

```bash
runhaven doctor
```

`doctor` checks Python, macOS, Apple silicon, the pinned Apple `container`
version, and Apple container system status.

## First Run

Run the non-mutating guided setup before running an agent:

```bash
runhaven setup
```

`setup` runs the same prerequisite checks as `doctor`, prints exact fixes when
the Mac is not ready, and shows the image build, plan, and run commands for
the selected agent. It does not install Apple `container`, start services,
build images, run agents, write state, or mount a workspace.

Build and preview a bundled image:

```bash
runhaven image build claude
runhaven plan claude
```

Run the agent from the project directory you want it to work on:

```bash
runhaven run claude
```

Use the smallest project directory the agent needs. Do not run from your home
directory, a cloud sync root, or a credential folder unless you intentionally
want that broader scope and have reviewed `runhaven plan`.

## Verification

Use focused checks for narrow changes:

```bash
python -m compileall src tests scripts
PYTHONPATH=src python -m unittest discover -s tests
python scripts/check_pins.py
git diff --check
```

Run full local harness verification before finishing broad code, runtime,
image, security-boundary, or install-flow changes:

```bash
./init.sh
```

Use `runhaven doctor` and Apple `container` runtime smokes when changes affect
the actual macOS container boundary, image templates, agent profiles, provider
networking, or install flow.

## Troubleshooting

Start with the guided setup:

```bash
runhaven setup
```

If a run fails, collect these commands before opening an issue:

```bash
runhaven setup
runhaven doctor
runhaven plan <agent>
container system status
```

Do not paste secret values, API keys, SSH keys, private repository contents, or
raw Apple `container inspect` output into issues.
