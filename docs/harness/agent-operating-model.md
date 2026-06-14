# Agent Operating Model

Agents working in RunHaven should behave like maintainers with a narrow
task, explicit evidence, and a clean handoff.

## Start

- Read `AGENTS.md`.
- Read `feature_list.json`, `progress.md`, and any active plan.
- Identify the current objective and non-goals.
- Choose the verification path before editing.

## During Work

- Keep edits scoped to the current objective.
- Prefer existing project patterns.
- Use repo-relative paths in durable artifacts.
- Treat generated files as maintained source.

## Finish

- Run the relevant checks.
- Record evidence and skipped checks.
- Update state files when the objective, blockers, or next step changed.
- Leave the repo restartable for the next session.
