# Modularization Plan

Status: active

This plan tracks the pre-release large-file refactor. Keep each slice
behavior-preserving unless a separate feature change is explicitly selected.

## Current Size Snapshot

Measured on 2026-06-15 after the CLI test split:

| File | Lines | Notes |
| --- | ---: | --- |
| `tests/test_cli_active_commands.py` | 900 | Largest remaining split CLI test file; owns active listing, attach, logs-follow, status, stop, and kill coverage. |
| `src/runhaven/cli.py` | 767 | Still owns parser, command routing, standard run flow, state commands, and thin provider-runtime compatibility wrappers. |
| `tests/test_cli_run_history.py` | 663 | Owns `runs list/show/diff/log` CLI coverage. |
| `tests/test_cli_provider_runtime.py` | 622 | Owns provider runtime, Codex broker run, and internal-network CLI coverage. |
| `src/runhaven/run_history.py` | 604 | Owns run-record persistence, git metadata capture, and `runs list/show/log/diff`. |
| `src/runhaven/active_commands.py` | 569 | Owns active-run command handlers, sanitized status output, attach/log-follow command construction, stop/kill, and repair. |
| `src/runhaven/auth_broker.py` | 520 | Cohesive enough for now. |
| `src/runhaven/provider_runtime.py` | 500 | Owns provider run lifecycle, proxy/broker startup, policy/auth decision logging, active marker cleanup, and internal network inspection. |
| `scripts/check_pins.py` | 497 | Separate script; review after CLI/test split. |
| `tests/test_cli_active_repair.py` | 452 | Owns active-run stale-marker repair coverage. |
| `src/runhaven/egress.py` | 404 | Cohesive provider proxy implementation. |
| `src/runhaven/plans.py` | 403 | Cohesive planner and validation module. |
| `tests/test_cli_standard_run.py` | 304 | Owns standard run record and active-marker lifecycle coverage. |
| `tests/test_cli_diagnostics.py` | 273 | Owns `auth`, `egress log`, and `why host` CLI coverage. |
| `src/runhaven/diagnostic_commands.py` | 249 | Owns `auth status/explain/log`, `egress log`, `why host`, and diagnostic log readers. |
| `tests/test_cli.py` | 228 | Owns core CLI, setup, doctor, and plan smoke coverage. |
| `tests/cli_test_helpers.py` | 107 | Shared git, run-record, and active-marker helpers for split CLI tests. |
| `tests/test_cli_state.py` | 80 | Owns state list, prune, and state lock coverage. |

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

## Diagnostic-Command Extraction Completed

- `src/runhaven/diagnostic_commands.py`: `auth status`, `auth explain`,
  `auth log`, `egress log`, `why host`, provider/auth JSONL log readers, and
  provider endpoint explanation output.
- `src/runhaven/cli.py`: keeps parser and command dispatch, and passes
  `read_egress_policy_log(limit=0)` plus `read_auth_broker_log(limit=0)` into
  `runs log` so joined run-history output keeps explicit reader seams.

This removes read-only diagnostics from `cli.py` while preserving secret-free
auth output, provider policy log output, `why host` provider matching, and
`runs log` joins.

## CLI Test Split Completed

- `tests/cli_test_helpers.py`: existing shared git, run-record, and
  active-marker helpers moved out of the monolithic test file.
- `tests/test_cli.py`: core CLI, setup, doctor, and plan smoke coverage.
- `tests/test_cli_provider_runtime.py`: provider runtime, Codex broker run, and
  internal-network CLI coverage.
- `tests/test_cli_standard_run.py`: standard run record and active-marker
  lifecycle coverage.
- `tests/test_cli_active_commands.py`: active listing, attach, logs-follow,
  status, stop, and kill coverage.
- `tests/test_cli_active_repair.py`: stale active-marker repair coverage.
- `tests/test_cli_run_history.py`: `runs list/show/diff/log` coverage.
- `tests/test_cli_diagnostics.py`: `auth`, `egress log`, and `why host`
  diagnostic coverage.
- `tests/test_cli_state.py`: state list, prune, and state lock coverage.

This removes the 3,515-line CLI test file while preserving the existing 90 CLI
tests and the same production patch targets.

## Recommended Sequence

1. Split `tests/test_cli_active_commands.py` further if the next cleanup pass
   stays focused on large test files. Good seams are active listing, attach and
   logs-follow, status, stop/kill, and ownership refusal cases.

2. Review `scripts/check_pins.py`, `src/runhaven/auth_broker.py`, and
   `src/runhaven/provider_runtime.py` for complexity-only refactors. Keep them
   intact if a split would only move code without improving reviewability.

## Acceptance Criteria

- `cli.py` is primarily parser construction, command dispatch, and small
  command wrappers.
- Tests are grouped by command surface, not by the historical single CLI file.
- Runtime subprocess patch seams are explicit and reviewable.
- No refactor weakens macOS 26+ only support, default isolation, egress
  behavior, active-run ownership checks, or secret-free logs.
- Each slice runs focused tests plus the full harness before merge.
