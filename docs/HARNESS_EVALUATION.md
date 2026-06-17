# Harness Evaluation

Evaluated: 2026-06-17

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
