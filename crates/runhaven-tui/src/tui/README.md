# RunHaven TUI Fork Boundary

This directory is RunHaven's pruned, locally adapted fork of the OpenAI Codex
terminal UI source at:

- repository: `https://github.com/openai/codex.git`
- commit: `5267e805fb830891c0b23376bcd9cbd382c3473c`
- upstream path: `codex-rs/tui/src`
- upstream license: Apache-2.0

The original copyright and license terms remain applicable. The repository
retains the upstream source separately under `crates/codex/` for the pinned
Codex crate dependency closure.

## Boundary

`crates/runhaven-tui/Cargo.toml` owns the workspace unit.
`crates/runhaven-tui/src/lib.rs` is its public entrypoint. This directory owns
the private terminal implementation used by that entrypoint.

The runtime entrypoint is `app_shell::run`. RunHaven's launch, recovery,
active-run, history, and diagnostic behavior lives under `runhaven/` and
delegates product truth to `runhaven-core`. Upstream-derived terminal,
rendering, and widget code supplies presentation infrastructure only. It is not
an alternate owner for container policy, launch plans, security defaults,
records, or diagnostics.

The fork targets Apple silicon on macOS 26 or newer. Unsupported platform
branches, shell launchers, and sandbox projections are intentionally absent
from the RunHaven fork. The separately isolated `crates/codex/` source remains
an upstream dependency record and is not RunHaven runtime policy.

## Source classification

`repository-structure-policy.json` records this exact directory as vendored
fork source with a named owner and reason. Exact RunHaven-owned adapter paths
are re-included as hand-authored source so the repository's physical-line and
byte limits apply to them. Lockfiles and upstream source do not create policy
exceptions for any RunHaven-owned adapter.

The local tree is deliberately smaller than upstream. Files absent from the
compiled RunHaven graph are deleted instead of being kept as speculative
future surface. Missing upstream files are therefore expected and do not imply
an incomplete checkout.

## Provenance check

Run:

```sh
scripts/compare-codex-tui.sh
```

The command fetches only the pinned upstream tree into a temporary directory,
builds deterministic SHA-256 manifests, and reports:

- files retained from upstream;
- retained files with local changes;
- RunHaven-only files;
- upstream files intentionally absent from the pruned fork.

Use `--write-manifests DIR` only with an output directory outside the
repository. The comparison is provenance evidence, not a compliance score and
not permission to restore deleted upstream behavior.

## Verification

The default fork checks are:

```sh
scripts/cargo.sh check -p runhaven-tui --locked
scripts/cargo.sh test -p runhaven-tui --locked
scripts/cargo.sh clippy -p runhaven-tui --all-targets --locked -- -D warnings
```

The opt-in upstream snapshot compile surface is:

```sh
scripts/cargo.sh test -p runhaven-tui --locked --features codex-vendored-tests --no-run
```

Repository layout validation rejects reintroduced unsupported-platform
projections in this fork. Product runtime smoke remains owned by the repository
verification matrix and `./init.sh`.
