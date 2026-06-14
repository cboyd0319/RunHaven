# Harness Operations

Status: live

This directory is the operating layer for agent-assisted work in macos-container-agents.
It keeps instructions, state, verification, scope, and lifecycle handoff visible
in repo files instead of hidden in chat history.

Product runtime contract: macOS 26+ on Apple silicon, Python 3.13+, and Apple
`container` 1.0.0.

Contributor verification contract: Python import, docs, pin, and
command-construction checks should remain reviewable on Windows 11 and Ubuntu
22.04+ where Apple `container` runtime checks are unavailable. This does not
change the product runtime floor.

## Purpose

Make each coding session restartable, scoped, and verifiable. Agents should
find the current objective, understand what they may change, run the right
checks, and leave evidence for the next session.

## Practical Harness Map

| Domain | Artifact | Purpose |
| --- | --- | --- |
| Instructions | `AGENTS.md` | Startup path, invariants, definition of done |
| Tools | `init.sh`, `init.ps1` | Local verification entrypoints for POSIX and Windows |
| Environment | package/runtime manifests, `component-inventory.md`, `dependency-change-policy.md` | Versions, dependency managers, setup facts, project boundaries, and pin policy |
| State | `feature_list.json`, `progress.md`, `evidence-log.md` | Current objective, feature status, and evidence |
| Feedback | `verification-matrix.md`, `evaluator-rubric.md`, local checks | Deterministic signals before claiming completion |
| Scope | `change-contract.md`, `security-boundary-map.md`, `feature-privacy-labels.json` | Problem, non-goals, acceptance, rollback, and data boundaries |
| Lifecycle | `session-handoff.md`, `clean-state-checklist.md`, `quality-document.md`, `release-controls.md`, `self-healing.md`, `entropy-control.md` | Restart, release, and recurring harness upkeep |

## Operating Loop

1. Start from `AGENTS.md`.
2. Read `feature_list.json`, `progress.md`, and relevant project docs.
3. Use `change-contract.md` for non-trivial work.
4. Implement the smallest coherent slice.
5. Run the relevant checks from `verification-matrix.md`.
6. Use `clean-state-checklist.md` before ending non-trivial sessions.
7. Record evidence, blockers, skipped checks, and next steps.
8. Update this harness when repeated failures show a missing guide or sensor.

Remote CI is a shared cost and trust boundary. Run local checks before push, and
use remote CI to confirm reviewed changes rather than as a trial-and-error loop.

## Assessment And Updates

Use repo-harness-creator for regular structural checks:

```bash
repo-harness audit --target .
repo-harness update --target .
```

Run `repo-harness update --target . --apply` only when you want safe missing-file
corrections. Existing files are preserved unless `--force` is passed.

## When To Add Harness

Add a doc, script, test, or manifest rule when:

- A setup or verification step is repeated.
- A failure would be expensive if rediscovered in the next session.
- A privacy, security, cost, data-loss, or release rule needs a hard gate.
- A reviewer needs evidence that a claim matches current code.

Do not add harness for style preference alone. Prefer the smallest durable guide
or sensor that prevents the observed failure.
