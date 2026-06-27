# Harness Evaluation

Evaluated: 2026-06-27

Target: RunHaven

This repo now uses the lightweight five-subsystem harness model:
instructions, tools, environment, state, and feedback.

## Current Result

The active startup path is intentionally small:

- `AGENTS.md`
- `feature_list.json`
- `current-state.md`

Historical evidence, component maps, verification routing, and release controls
remain available under `docs/harness/`, but they are on-demand reference
material.

## 2026-06-27 Resource Review

The local Learn Harness Engineering resource tree was reviewed from:

```text
/Users/c/Documents/GitHub/learn-harness-engineering/docs/en/resources/
```

The review covered the minimal templates, reference startup flow, failure-mode
map, prompt calibration notes, OpenAI advanced pack, SOPs, and repo-template
docs.

RunHaven should keep its current lightweight startup surface. The external
template reinforces the same shape: short root router, repo-local state,
standard verification, explicit boundaries, quality feedback, and restartable
handoff. The template filenames are not required when RunHaven already has a
clear owner for the same concept.

## Template Concept Map

| External Concept | RunHaven Artifact |
| --- | --- |
| `AGENTS.md` root router | `AGENTS.md` plus thin platform shims |
| `feature_list.json` | `feature_list.json` |
| `claude-progress.md` and `session-handoff.md` | `current-state.md` |
| `init.sh` | `init.sh` |
| `ARCHITECTURE.md` | `docs/ARCHITECTURE.md`, `docs/harness/boundaries/component-inventory.md`, `docs/harness/state/modularization-plan.md` |
| `docs/SECURITY.md` | `docs/SECURITY_MODEL.md`, `docs/harness/boundaries/security-boundary-map.md` |
| `docs/RELIABILITY.md` | `docs/harness/feedback/verification-matrix.md`, `docs/harness/feedback/sensor-registry.md`, `docs/harness/release/release-controls.md` |
| `docs/QUALITY_SCORE.md` | `docs/harness/feedback/quality-document.md` |
| `docs/PLANS.md` and `docs/exec-plans/` | `feature_list.json`, `current-state.md`, `docs/plans/`, release and roadmap docs |
| `docs/product-specs/` | `README.md`, `docs/USAGE.md`, `docs/CAPABILITIES.md`, `docs/ROADMAP.md`, focused plan docs |
| `docs/references/` and generated facts | `docs/RESEARCH.md`, `docs/harness/research/`, `docs/CLI_SURFACE_COVERAGE.md`, `pins.toml` |

## Current Decision

Do not create parallel `docs/QUALITY_SCORE.md`, `docs/RELIABILITY.md`,
`docs/PLANS.md`, `docs/product-specs/`, or root progress/handoff files right
now. That would duplicate working RunHaven owners and increase startup
confusion. If a repeated miss proves one current owner is overloaded, split that
owner then and update this map.

## What Changed

- Root instructions were reduced to a startup map and hard constraints.
- `feature_list.json` was compressed from a historical changelog into a compact
  active feature ledger.
- `current-state.md` was compressed into the current objective, trusted facts,
  blocker, touched surfaces, and next step.
- The harness manifest was changed from a generated snippet checklist into a
  compact map that is explicitly not startup context.
- Retired first-agent and quality artifacts were shortened so they do not pull
  agents back into the old generated structure.

## Acceptance Check

A new session should be able to identify project purpose, current work,
blocked work, and first verification options from the three startup files. If a
session needs to read most of `docs/harness/` before choosing a task, the
harness has regressed.

Structural tools such as HarnessForge are optional. They are not proof of
real-agent effectiveness and are not contributor prerequisites.
