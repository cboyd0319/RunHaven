# Modularization Plan

Status: active

This plan tracks the pre-release large-file refactor. Keep each slice
behavior-preserving unless a separate feature change is explicitly selected.

## Current Size Snapshot

Measured on 2026-06-15 after the first extraction:

| File | Lines | Notes |
| --- | ---: | --- |
| `tests/test_cli.py` | 3515 | Broad integration-style CLI coverage. Useful, but too large for targeted review. |
| `src/runhaven/cli.py` | 2440 | Still owns parser, command routing, provider runtime, run records, active run commands, logs, auth, network helpers, and state commands. |
| `src/runhaven/auth_broker.py` | 520 | Cohesive enough for now. |
| `scripts/check_pins.py` | 497 | Separate script; review after CLI/test split. |
| `src/runhaven/egress.py` | 404 | Cohesive provider proxy implementation. |
| `src/runhaven/plans.py` | 403 | Cohesive planner and validation module. |

## First Extraction Completed

- `src/runhaven/setup_guide.py`: guided setup and doctor check output.
- `src/runhaven/active_records.py`: active-run marker persistence and status
  updates.
- `src/runhaven/cache_paths.py`: cache, log, active-run, and lock paths.
- `src/runhaven/validators.py`: shared string, run id, and RunHaven container
  name validation.

This removes setup copy and active-marker persistence from `cli.py` while
leaving command handlers and runtime subprocess calls in place.

## Recommended Sequence

1. Split run observability from `cli.py`.
   Move `runs list/show/log/diff`, git metadata helpers, and run-record readers
   into a `run_logs` or `run_history` module. Keep command-output tests first.

2. Split active run commands from `cli.py`.
   Move `runs active/status/attach/logs-follow/stop/kill/repair` after the
   subprocess patch seams are updated or wrapped explicitly.

3. Split provider runtime orchestration.
   Move provider proxy startup, broker wiring, cleanup, and decision logging
   into a provider runtime module. This is higher risk because tests patch
   runtime hooks heavily.

4. Split auth, egress log, and `why host` commands.
   These are moderate-risk read-only command surfaces and can move after run
   history is stable.

5. Split `tests/test_cli.py`.
   Mirror the production seams after they exist: setup, planning, provider
   runtime, run history, active runs, auth, egress, state, and repo policy.

## Acceptance Criteria

- `cli.py` is primarily parser construction, command dispatch, and small
  command wrappers.
- Tests are grouped by command surface, not by the historical single CLI file.
- Runtime subprocess patch seams are explicit and reviewable.
- No refactor weakens macOS 26+ only support, default isolation, egress
  behavior, active-run ownership checks, or secret-free logs.
- Each slice runs focused tests plus the full harness before merge.
