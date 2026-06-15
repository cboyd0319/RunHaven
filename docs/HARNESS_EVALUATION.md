# Harness Evaluation

Evaluated: 2026-06-14

Target: RunHaven

Evaluator: `HarnessForge` structural audit plus manual review of root
instructions, state files, local verification entrypoints, harness docs,
security boundaries, pinning policy, research ledger, CI, source entry points,
and tests.

## Executive Summary

The repo now satisfies the structural harness threshold while preserving the
macOS-only product contract.

Before this pass, the audit result was:

```text
Overall: 6/100
Bottleneck: state
```

After the macOS-only correction, the audit result is:

```text
Overall: 100/100
Bottleneck: instructions
```

The score means the required harness surfaces are present, internally
navigable, and aligned with RunHaven's macOS 26+ only support boundary. It does
not replace product security testing, Apple `container` runtime smokes, or
representative agent-workload evaluation.

## What Changed

- Added durable state files: `feature_list.json`, `progress.md`, and
  `session-handoff.md`.
- Added local macOS verification entrypoint: `init.sh`.
- Added the harness operating layer under `docs/harness/`.
- Updated `AGENTS.md` with Startup, Verification, Definition Of Done, state
  file routing, and End of Session instructions.
- Replaced generic harness source records with a small reviewed source list and
  a pointer to `docs/RESEARCH.md`.
- Corrected generated platform language so product runtime support remains
  macOS 26+ on Apple silicon with Python 3.13+ and Apple `container` 1.0.0.
- Removed non-macOS verification surfaces after clarifying that RunHaven is
  macOS 26+ only.

## Verification Evidence

```bash
PYTHON=<temporary-venv-python> ./init.sh
```

Result: passed.

Covered:

- `python -m compileall src tests scripts`
- `PYTHONPATH=src python -m unittest discover -s tests`
- `python scripts/check_pins.py`
- `python -m ruff check .`
- `python -m mypy src`
- `python -m build`

```bash
python3 -m harnessforge audit --target . --min-score 85
```

Result: passed with 100/100.

## Score Breakdown

| Domain | Score | Result |
| --- | ---: | --- |
| Instructions | 5/5 | Startup, Verification, Definition Of Done, state routing, and size checks passed |
| Tools | 5/5 | macOS entrypoint, pin check, fail-fast behavior, and tool-safety docs passed |
| Environment | 5/5 | Runtime manifest, Python floor, OS-floor language, component inventory, and manifest passed |
| State | 5/5 | Feature state, progress, privacy labels, and handoff checks passed |
| Feedback | 5/5 | Verification matrix, evidence log, evaluator rubric, audit loop, links, and entrypoints passed |
| Scope | 5/5 | Change contract, security map, dependency policy, acceptance, verification, and rollback passed |
| Lifecycle | 5/5 | Handoff, restart path, clean-state checklist, quality doc, sources, entropy control, roadmap, and update loop passed |

## Remaining Product Work

The harness is structurally complete. Product work still needs runtime-specific
security validation as features expand, especially:

- provider egress allowlisting
- custom profile file support
- one-command beginner installer flow
- periodic agent-effectiveness evaluations
- image build and runtime smokes after dependency or runtime pin updates
