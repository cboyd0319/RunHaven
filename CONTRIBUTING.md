# Contributing

This repo optimizes for safe defaults and beginner usability.

Users are trusting this project with personal machines, personal files, and
developer credentials. Security-sensitive behavior must fail closed when it
cannot verify a boundary. Do not hide risk behind friendly wording.

RunHaven remains alpha/pre-release until after the `v0.5.0` CLI-complete
milestone. Treat the CLI as the current product surface, and keep v1 work
focused on making the desktop app the easiest safe path.

RunHaven only supports macOS 26+ on Apple silicon. Do not add Windows or Linux
runtime or contributor-verification surfaces.

## Development Principles

All development is DRY and documentation-first:

- Walk the build-necessity ladder and stop at the first rung that fits: does it
  need to exist at all (YAGNI), does the standard library do it, does a native
  platform feature cover it, does an already-installed dependency solve it, can
  it be one clear line, then minimum custom code. Never add a dependency for
  what a few lines do.
- Documentation is product: an undocumented behavior does not exist, so docs
  ship in the same change as the behavior.
- Prefer boring over clever, and remove meaningful duplication instead of
  copy/paste or deferred large-file cleanup. Between two standard-library
  options of similar size, take the one correct on edge cases.

[AGENTS.md](AGENTS.md) Working Rules and
[the change contract](docs/harness/boundaries/change-contract.md) hold the full
gate, including the security and correctness carve-outs.

## Local Checks

Full local harness verification:

```bash
./init.sh
```

Focused checks:

```bash
cargo fmt --check
cargo test --locked
cargo run --locked --bin runhaven-check-pins
```

Additional Rust checks:

```bash
cargo clippy --all-targets -- -D warnings
cargo build --locked
```

## Security Review Expectations

- Show the exact `container` command with `runhaven plan` before changing runtime
  behavior.
- Make secure defaults the easiest path. Supported lower-security choices
  should warn and require explicit intent.
- Keep dependencies current stable and hard-pinned. Updating a package means
  changing the exact version or digest in source control.
- Keep files, modules, crates, Tauri commands, and frontend components cohesive
  so security-sensitive paths stay reviewable.
- Add or update tests for every change to command construction.
- Keep host secrets out of generated commands unless the user explicitly passes
  a variable name with `--env`.
- Do not add broad mounts for convenience. Add a narrow mount or a documented
  explicit option.
- Do not claim full isolation for a mode unless a focused runtime check proves
  the claimed boundary.
