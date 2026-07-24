# Harness operations

RunHaven uses the system harness contract: instructions, tools, environment,
state, and feedback. This directory is on-demand reference material, not startup
context.

## Canonical pack

| Subsystem | Owner | Contract |
| --- | --- | --- |
| Instructions | `AGENTS.md` | Concise root router, safety boundary, fixed startup, and finish loop |
| Tools | `init.sh`, `scripts/` | Safe local setup, verification, semantic state checks, file budgets, and topology checks |
| Environment | manifests, lockfiles, runtime-version files, `rust-toolchain.toml`, `pins.toml` | Exact versions and reproducible local setup; harness scripts use the pinned Python runtime and standard library |
| State | `PROGRESS.md`, `feature_list.json` | Compact current objective plus observable feature registry |
| Feedback | `feedback/verification-matrix.md`, tests, smokes | Deterministic local proof before completion claims |

Always-read canonical startup context is exactly `AGENTS.md`, `PROGRESS.md`, and
`feature_list.json`. The measured startup aggregate also includes the thin
`CLAUDE.md` and Copilot compatibility loaders, even though one runtime does not
load both. Its enforced ceiling is 48 KiB and 700 physical lines, with warnings
at 90%. Per-file budgets are tighter.

## State contract

`feature_list.json` uses only `not_started`, `active`, `blocked`, and `passing`.
Exactly one feature is active. Dependencies resolve and remain acyclic. Every
feature has an observable behavior and exact runnable verification commands.
Passing features have durable evidence pointers and cannot be demoted or
removed. Blocked features name both the blocker and the event that allows work
to resume.

`PROGRESS.md` agrees with the active feature, status, branch, and update date.
It contains only the current objective, trusted baseline, blockers or unknowns,
and exact next action. Narrative history belongs in `evidence/evidence-log.md`
or Git.

`scripts/check_harness_state.py` measures the two state files together at a
32,768-byte and 500-line hard ceiling, permits at most 40 feature records, and
caps one compact record at 2,048 bytes. Warnings begin at 90% of each ceiling.
These limits retain headroom over the reviewed 2026-07-14 baseline while
preventing evidence narrative from returning to startup state.

Run the state gate and its negative fixtures with:

```bash
scripts/python.sh scripts/check_harness_state.py
scripts/python.sh scripts/test_harness_state.py
```

## File budgets

`scripts/check_file_limits.py` is the portable policy owner. It enumerates
tracked and untracked nonignored files through Git, rejects symlinks and path
escapes, reads files in bounded chunks, and measures:

- bytes for every file, including binary files;
- physical lines for text files, including an unterminated final line;
- maximum line bytes as a stable diagnostic alongside the canonical physical-line and total-byte limits;
- aggregate startup bytes and lines.

Each budget names its ownership class. A 90% warning provides review lead time;
the hard ceiling fails. Exact-path exceptions require an owner, reason, measured
baseline, and retirement condition. There are currently no exceptions.

The enforced entrypoint and fixtures are:

```bash
scripts/check-repo-structure.sh
scripts/python.sh scripts/test_file_limits.py
```

`scripts/check_repo_layout.py` semantically parses Cargo manifests, rejects
unsupported OS and shell projections from maintained text, and enforces
the exact default workspace, crate dependency direction, binary-only entrypoint
crate, shared-runtime ownership, pinned TUI and Codex placement, direct Tauri
runtime tripwire, absence of repo-local skills, and no-hosted-CI state. Its
valid and violating fixtures run with:

```bash
scripts/python.sh scripts/test_repo_layout.py
```

The shell entrypoint stays thin and invokes the file and layout policy owners.

`scripts/check_environment.py` is read-only. Its isolated standard-library
fixtures cover the supported platform floor, exact runtime projections,
symlinked version owners, missing commands, and repository paths with spaces,
quotes, Unicode, and shell metacharacters.

`./init.sh --bootstrap` is the only dependency synchronization path. It runs
locked `npm ci --ignore-scripts`, writes only ignored root and workspace
`node_modules/` content, starts no service, and changes no user-level state.
Repeating the command is safe. If it is interrupted or fails, rerun the same
command; `npm ci` replaces partial dependency content. Cleanup, when actually
needed, is limited to removing this repository's root and `ui/node_modules/`
directories before rerunning the bootstrap.

## Local verification

`./init.sh` is the standard session baseline and representative smoke;
`scripts/verify-all.sh` is the full manually invoked gate for this private,
single-maintainer project. Both run harness checks before broader Rust and
frontend checks. No CI or GitHub Actions workflow exists. That owner-directed
choice is a named canonical enforcement gap, not a skill-defined exception or
evidence of compliance.

No local hook is required. A hook may be added only as optional convenience; it
cannot replace the checked-in local gate.

The audit cadence is change-driven: rerun the installed harness audit after an
instruction, state, architecture, environment, or feedback-owner change; after
repeated developer friction; and before a release-readiness review. Each run
must produce a correction, retirement, or explicit no-change decision rather
than a calendar-only report.

## On-demand routing

| Change | Read first |
| --- | --- |
| Verification choice | `feedback/verification-matrix.md` |
| Environment tools and services | `environment-tool-contracts.md` |
| Repeated checks and retirement | `feedback/sensor-registry.md` |
| Component ownership | `boundaries/component-inventory.md` |
| Security or privacy boundary | `boundaries/security-boundary-map.md` |
| Dependencies and pins | `boundaries/dependency-change-policy.md` |
| Release | `release/release-controls.md` |
| Apple `container` update | `release/apple-container-update-playbook.md` |
| Clean handoff | `state/clean-state-checklist.md` |
| Historical proof | `evidence/evidence-log.md` |

## Retained controls and evidence

| Control | Observed failure or risk | False-positive cost | Coverage | Review or retirement trigger |
| --- | --- | --- | --- | --- |
| Semantic state gate | Structural checks accepted invalid statuses, missing dependencies, prose verification, and a missing canonical progress file | One focused fixture update when the state schema intentionally changes | Standard-library Python fixtures and live macOS verification | Replace only with an equally strict checked-in validator |
| Multidimensional file budgets | Line-only limits were loose, trimmed-to-pass, and blind to maximum-line growth and symlinks | Warning review near calibrated ceilings | Streaming fixtures cover bytes, lines, long lines, warnings, ignores, and symlinks | Recalibrate from a reviewed baseline when an ownership class changes |
| Semantic repository layout | Shell regexes had no violation fixtures and could drift from Cargo ownership | One focused fixture update when an intentional crate or directory boundary changes | TOML parsing and fixtures cover valid and invalid workspace, dependency, topology, no-hosted-CI, and Tauri boundary cases | Replace only with compiler-backed or equally strict semantic architecture tests |
| Full local gate | Separate commands could create false completion claims | Local runtime cost; manually invoked only | macOS 26+ Apple silicon, Rust, React, Tauri, and deterministic E2E | Split only if measured startup time prevents routine use without reducing coverage |

## Effectiveness check

A fresh session passes when the three startup files identify the project safety
boundary, one active feature, blockers, exact next action, and first verification
command without reading this directory. Record measured startup bytes, loading
hops, repeated command errors, and state-reconstruction time when evaluating a
future harness change. Do not convert those outcomes into a composite grade.
