# Harness Operations

Status: live

This directory is the operating layer for agent-assisted work in RunHaven.
It keeps instructions, state, verification, scope, and lifecycle handoff visible
in repo files instead of hidden in chat history.

Product runtime contract: macOS 26+ on Apple silicon, Python 3.13+, and Apple
`container` 1.0.0.

Contributor verification contract: all local and CI verification runs target
macOS 26+. Windows and Linux are not supported runtime or contributor
verification targets for this project.

## Purpose

Make each coding session restartable, scoped, and verifiable. Agents should
find the current objective, understand what they may change, run the right
checks, and leave evidence for the next session.

## Five Core Subsystems

Harness = instructions + tools + environment + state + feedback. Missing any
one of these makes the harness incomplete.

| Subsystem | This Harness Provides | Review Question |
| --- | --- | --- |
| Instructions | `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `.github/copilot-instructions.md` | Does the agent see purpose, stack, startup commands, hard constraints, and links to detail? |
| Tools | `init.sh`, local shell commands, HarnessForge reports, CI | Can the agent do useful work with least privilege instead of blanket-disabled shell access or unrestricted access? |
| Environment | `pyproject.toml`, `.python-version`, `pins.toml`, image templates, component inventory | Are versions, dependencies, setup facts, and reproducible environment choices self-describing? |
| State | `feature_list.json`, `progress.md`, `session-handoff.md`, `docs/ROADMAP.md`, `docs/harness/roadmap.md` | Can a new session see what is done, current, blocked, accepted, and next? |
| Feedback | `verification-matrix.md`, `sensor-registry.md`, `evidence-log.md`, local checks | Are verification commands explicit, runnable, and prioritized before broader process? |

Feedback is the highest-return subsystem. When agent output is weak, first fix
missing, stale, or vague verification commands before adding more instructions.

## Effective Agent Boundary

The model is the LLM. The effective coding agent is the model plus the harness:
system prompts, instruction files, shell and file tools, git access, local
filesystem scope, startup scripts, verification commands, stop hooks,
lint/sensor checks, workflow permissions, Apple `container` defaults, provider
allowlists, and evaluator loops. Changing any of these changes effective agent
behavior. Treat those changes as product changes with scope, verification, and
rollback.

## Practical Harness Map

| Domain | Artifact | Purpose |
| --- | --- | --- |
| Instructions | `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, `.github/copilot-instructions.md` | Startup path, invariants, definition of done, and platform routing |
| Tools | `init.sh`, `scripts/check_pins.py`, smoke scripts, HarnessForge commands | Local macOS verification and review entrypoints |
| Environment | `pyproject.toml`, `.python-version`, `pins.toml`, image package locks, `component-inventory.md`, `dependency-change-policy.md` | Versions, package managers, setup facts, image boundaries, and pin policy |
| State | `feature_list.json`, `progress.md`, `session-handoff.md`, `docs/ROADMAP.md`, `roadmap.md` | Current objective, accepted roadmap, feature status, and evidence |
| Feedback | `verification-matrix.md`, `sensor-registry.md`, `evaluator-rubric.md`, `evidence-log.md` | Deterministic signals, ownership, and lifecycle before claiming completion |
| Research | `docs/RESEARCH.md`, `sources.md`, `research-sources.json`, `source-record.schema.json`, `source-record-example.json` | Reviewed provenance, public source ledger, and project-owned source records |
| Scope | `change-contract.md`, `security-boundary-map.md`, `feature-privacy-labels.json` | Problem, non-goals, acceptance, rollback, data, security, permission, and cost boundaries |
| Lifecycle | `first-agent-task.md`, `clean-state-checklist.md`, `quality-document.md`, `release-controls.md`, `entropy-control.md`, `modularization-plan.md` | First-session harness improvement, restart, release readiness, code-health sequencing, and recurring upkeep |

## Operating Loop

1. Start from `AGENTS.md`.
2. Read `feature_list.json`, `progress.md`, `session-handoff.md`, and relevant
   project docs.
3. If `first-agent-task.md` still exists, complete or retire it before
   unrelated feature work.
4. Check `docs/ROADMAP.md` for product direction and
   `docs/harness/roadmap.md` for harness/backlog operating boundaries.
5. Use `change-contract.md` for non-trivial work.
6. Implement the smallest coherent slice.
7. Run the relevant checks from `verification-matrix.md`.
8. Review `sensor-registry.md` when adding, deleting, or promoting checks.
9. Use `clean-state-checklist.md` before ending non-trivial sessions.
10. Record evidence, blockers, skipped checks, and next steps.
11. Update this harness when repeated failures show a missing guide or sensor.

Remote CI is a shared cost and trust boundary. Run local checks before push,
and use remote CI to confirm reviewed changes rather than as a trial-and-error
loop.

## Assessment And Updates

Use HarnessForge for regular structural checks after installing it in the
current development environment:

```bash
harnessforge index --target . --json
harnessforge session --target .
harnessforge report --target .
harnessforge plan --target . --since HEAD
harnessforge audit --target . --min-score 85
harnessforge update --target .
```

`harnessforge report --target .` is the preferred periodic harness status
artifact. It composes readiness, audit, generated drift, structural index,
verify evidence, effectiveness evidence, first-agent task status, and platform
contract without running target commands. Use target-relative `--json-report`
or `--markdown-report` paths only when intentionally recording evidence.

Before enhancing project-owned instruction files, run:

```bash
harnessforge enhance --target .
harnessforge enhance --target . --json
```

The enhancement plan reports proposed addenda, section coverage,
review-required cleanup, patch previews, duplicate instructions, local absolute
paths, user-specific tool mandates, and verification conflicts before files are
changed. Patch previews are review-only.

Run `harnessforge update --target . --apply` only when you want safe missing
artifact corrections. Existing files are preserved unless a force option is
passed.

## When To Add Harness

Add a doc, script, test, manifest rule, or sensor when:

- A setup or verification step is repeated.
- A failure would be expensive if rediscovered in the next session.
- A privacy, security, cost, data-loss, or release rule needs a hard gate.
- A reviewer needs evidence that a claim matches current code.

Do not add harness for style preference alone. Prefer the smallest durable guide
or sensor that prevents the observed failure.

## Bottleneck And Harness Debt

Harness rots like code. Audit it before releases, large refactors, platform
contract changes, provider endpoint changes, and repeated agent failures.

When evaluating a harness change, controlled-variable exclusion tests can help:
hold the model and task fixed, remove one subsystem at a time, and observe the
performance drop. Use that as supporting evidence only. Locate the real
bottleneck from failure records and attribution: unclear task, missing context,
unreproducible environment, missing feedback, broken state, or tool access that
is too narrow or too broad.
