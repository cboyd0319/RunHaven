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
- Keep files, modules, crates, Tauri commands, and frontend components
  cohesive. Remove meaningful duplication instead of deferring large-file
  cleanup.
- Add or update tests for every change to command construction.
- Keep host secrets out of generated commands unless the user explicitly passes
  a variable name with `--env`.
- Do not add broad mounts for convenience. Add a narrow mount or a documented
  explicit option.
- Do not claim full isolation for a mode unless a focused runtime check proves
  the claimed boundary.
