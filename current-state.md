# Current State

Last Updated: 2026-06-16 UTC

## Current Objective

RunHaven has been converted from a Python project to a fully functional Rust
CLI while preserving the macOS Apple `container` harness contract, exact pin
policy, and repo-owned verification route.

## State Contract

- `feature_list.json`: machine-readable feature state and durable product
  evidence.
- `docs/harness/evidence/evidence-log.md`: meaningful verification, source
  review, release, or harness evidence.
- `current-state.md`: current objective, trusted verification, touched
  surfaces, blockers, and next step.
- Do not recreate separate root `progress.md` or `session-handoff.md` files.

## Product State

- RunHaven is a Rust 1.96.0 CLI for running AI coding agents inside Apple
  `container` on macOS 26+ on Apple silicon.
- The application code is organized as a Cargo crate under `src/runhaven/` with
  CLI, runtime, provider, image, records, harness, and support modules. Bundled
  image templates live under top-level `images/`.
- The old Python package, Python tests, Python scripts, `pyproject.toml`,
  `.python-version`, and `requirements-dev.txt` have been removed.
- Windows and Linux are not supported runtime or contributor-verification
  targets.
- Default product safety boundaries remain: no host home mount, no cloud
  credential folder mount, no raw SSH key mount, no arbitrary environment
  passthrough, explicit workspace scope, non-root bundled images, and
  provider egress allowlisting only through reviewed provider mode.
- HarnessForge output is advisory unless a maintainer promotes a recommendation
  into repo-owned docs, tests, policy, code, or release checks.

## Latest Verified Work

- Rebuilt the CLI in Rust with exact-pinned Cargo dependencies and a checked-in
  `Cargo.lock`.
- Replaced the Python pin checker with `runhaven-check-pins`.
- Updated `init.sh`, CI, root docs, installation docs, usage docs, pinning docs,
  harness docs, component inventory, verification matrix, and manifest metadata
  for the Rust stack.
- Kept file organization nested by responsibility instead of flattening the
  Rust source tree.
- Split large Rust modules so every Rust source file is under 500 lines; the
  current largest file is `src/runhaven/cli/app.rs` at 494 lines.
- Updated `.gitignore` for Rust build output.
- Completed the final active-document accuracy sweep for the Rust conversion
  across product docs, GitHub instructions, harness boundaries, roadmap,
  release controls, and source-mined ideas.
- Removed ignored local cleanup artifacts from the working tree, including
  stale Python cache/build output and `.DS_Store` files.
- Deduped the main README after the overview refresh so the top-level page now
  keeps one product narrative and routes detailed feature coverage to
  `docs/CAPABILITIES.md`.
- Corrected the Cargo development command in installation docs to name the
  `runhaven` binary explicitly.

## Trusted Verification

- `cargo fmt --check`: passed.
- `cargo test --locked`: passed with 7 unit tests and 2 integration tests.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo run --locked --bin runhaven-check-pins`: passed.
- `cargo build --locked`: passed.
- `./init.sh`: passed. The full local harness ran Cargo format, tests, clippy,
  pin policy, and build.
- Rust source size scan: passed; no Rust source file is over 500 lines.
- Direct CLI smokes passed: `target/debug/runhaven agents`,
  `target/debug/runhaven plan shell --workspace . -- /bin/bash -lc pwd`,
  `target/debug/runhaven doctor`, and
  `target/debug/runhaven image build shell --dry-run`.
- Active-doc stale-reference scan: passed for old Python project paths,
  Python-package guidance, and pre-Rust source paths.
- Cleanup scan: passed; no Python project artifacts, Python caches, old Python
  packaging files, or `.DS_Store` files remain outside ignored build output.
- JSON validation, local Markdown link check, `git diff --check`, and Rust
  source size guard: passed.
- README docs checks: pin check, local Markdown link check, platform
  wording/stale-command scan, `git diff --check`, and CLI `agents`/`plan`
  smokes passed.

## Touched Surfaces

- `AGENTS.md`
- `.github/workflows/ci.yml`
- `.gitignore`
- `Cargo.toml`
- `Cargo.lock`
- `rust-toolchain.toml`
- `init.sh`
- `current-state.md`
- `pins.toml`
- `README.md`
- `CONTRIBUTING.md`
- `docs/`
- `docs/harness/`
- `feature_list.json`
- `images/`
- `src/`
- `tests/`

## Blockers

- None known.

## Next Step

Monitor main-branch CI after the README dedupe commit. Future work should
broaden live provider/container smokes and Tauri planning from the Rust module
boundaries now in place.
