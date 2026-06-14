# Multi-Agent Orchestration

Use this only when independent work can proceed without file ownership
conflicts.

## Coordinator Duties

- Define each slice with owned files, off-limits paths, acceptance criteria, and
  verification.
- Keep integration and final correctness with the coordinator.
- Do not claim broad completion from a narrow delegated check.

## Delegated Task Requirements

Every delegated task must include:

- Objective.
- Owned files.
- Off-limits paths.
- Required checks.
- Handoff format.

## Safety Rules

- Delegation does not relax repo rules.
- Child work must not overwrite user changes.
- Recursive delegation is not allowed unless the repo explicitly approves it.
