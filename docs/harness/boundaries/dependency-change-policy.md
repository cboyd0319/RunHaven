# Dependency Change Policy

Status: live

## Rule

Default to the latest stable or supported release. Do not guess. Verify the
current date and a current primary source before changing package, CLI, vendor
API, Rust, Cargo crate, Node, npm, or security-tool versions.

Use exact pins for direct package dependencies. If the newest stable release has a known reachable
security issue, pin the newest supported clean release and record the reason.
Use `pins.toml` for reviewed pin rationale when a dependency or image base is
governed by the project pin ledger.

## Decision Order

1. Use the standard library when it is enough.
2. Use a native platform feature when it covers the need.
3. Use an already-installed dependency when it removes real complexity.
4. Add the smallest new dependency only when the value is clear.

## Required Evidence

Record:

- Current date checked.
- Primary source used for the latest stable version.
- Why standard library, native platform, or installed dependencies were not
  enough.
- Pin or lockfile update.
- Focused test, build, audit, or smoke command.
- Rust 1.97.0, Cargo lockfile, and macOS 26+ runtime impact.
- macOS 26+ contributor-check impact. Keep dependency, script, and command
  construction surfaces on the supported host target.
