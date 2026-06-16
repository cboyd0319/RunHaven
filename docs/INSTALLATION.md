# Installation

RunHaven is a macOS-only project. Runtime and contributor verification require
macOS 26+ on Apple silicon, Rust 1.96.0, and Apple `container` 1.0.0.
Windows and Linux are not supported.

## Requirements

- macOS 26+
- Apple silicon
- Rust 1.96.0
- Apple [`container`](https://github.com/apple/container) 1.0.0
- Git

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

Install the CLI from this checkout:

```bash
cargo install --path . --locked
```

For development without installing, run the binary through Cargo:

```bash
cargo run --locked --bin runhaven -- plan shell
```

Development tools and dependencies are exact-pinned in `rust-toolchain.toml`,
`Cargo.toml`, `Cargo.lock`, and `pins.toml`. When updating them, use the
current stable release and commit the exact new version.

Confirm the host before running an agent:

```bash
runhaven doctor
```

`doctor` checks Rust, macOS, Apple silicon, the pinned Apple `container`
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
cargo fmt --check
cargo test --locked
cargo run --locked --bin runhaven-check-pins
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
