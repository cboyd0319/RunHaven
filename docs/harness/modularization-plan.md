# Modularization Plan

Status: active

This plan tracks the pre-release large-file refactor. Keep each slice
behavior-preserving unless a separate feature change is explicitly selected.

## Current Size Snapshot

Measured on 2026-06-15 after the provider-runtime extraction:

| File | Lines | Notes |
| --- | ---: | --- |
| `tests/test_cli.py` | 3515 | Broad integration-style CLI coverage. Useful, but too large for targeted review. |
| `src/runhaven/cli.py` | 1005 | Still owns parser, command routing, auth, egress logs, `why`, state commands, and thin provider-runtime compatibility wrappers. |
| `src/runhaven/run_history.py` | 604 | Owns run-record persistence, git metadata capture, and `runs list/show/log/diff`. |
| `src/runhaven/active_commands.py` | 569 | Owns active-run command handlers, sanitized status output, attach/log-follow command construction, stop/kill, and repair. |
| `src/runhaven/auth_broker.py` | 520 | Cohesive enough for now. |
| `src/runhaven/provider_runtime.py` | 500 | Owns provider run lifecycle, proxy/broker startup, policy/auth decision logging, active marker cleanup, and internal network inspection. |
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

## Run-History Extraction Completed

- `src/runhaven/run_history.py`: run-record persistence, provider/auth summary
  fields, git metadata capture, `runs list/show/log/diff`, and run-record
  readers.
- `src/runhaven/cli.py`: retains parser and command dispatch, and passes auth
  plus egress log readers into `runs log` to avoid circular imports.

This removes run observability from `cli.py` while preserving the existing
command output, git diff validation, and secret-free log behavior.

## Active-Command Extraction Completed

- `src/runhaven/active_commands.py`: `runs active/status/attach/logs-follow`,
  `runs stop/kill/repair`, sanitized container inspect summarization, attach
  validation, and repair result payloads.
- `src/runhaven/cli.py`: keeps parser and command dispatch, and passes
  `require_container_cli`, `subprocess.run`, `subprocess.call`, and TTY checks
  into active commands so runtime subprocess seams stay explicit.

This removes active-run command handlers from `cli.py` while preserving
RunHaven-owned container validation, non-root attach defaults, secret-free
status output, stale-marker repair behavior, and existing test patch seams.

## Provider-Runtime Extraction Completed

- `src/runhaven/provider_runtime.py`: provider run lifecycle, provider proxy
  startup, Codex broker startup, proxy environment injection, broker config
  injection, policy/auth decision logging, blocked-host review, provider
  network cleanup, and internal-network inspection helpers.
- `src/runhaven/cli.py`: keeps parser, command dispatch, standard run flow,
  and thin provider-runtime wrappers for `run_preflight`,
  `inspect_internal_network`, `create_provider_proxy`,
  `create_codex_api_key_broker`, `threading.Thread`, `subprocess.call`, and
  `delete_container_network`.

This removes provider orchestration from `cli.py` while preserving provider
egress behavior, Codex broker behavior, secret-free run records, active marker
cleanup, and existing test patch seams.

## Recommended Sequence

1. Split auth, egress log, and `why host` commands.
   These are moderate-risk read-only command surfaces and can move after run
   history is stable.

2. Split `tests/test_cli.py`.
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
