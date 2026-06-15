# Modularization Plan

Status: active

This plan tracks the pre-release large-file refactor. Keep each slice
behavior-preserving unless a separate feature change is explicitly selected.

## Current Size Snapshot

Measured on 2026-06-15 after the auth-profile extraction:

| File | Lines | Notes |
| --- | ---: | --- |
| `src/runhaven/cli.py` | 766 | Still owns parser, command routing, standard run flow, state commands, and thin provider-runtime compatibility wrappers. |
| `src/runhaven/provider_runtime.py` | 501 | Owns provider run lifecycle, proxy/broker startup, policy/auth decision logging, active marker cleanup, and internal network inspection. |
| `tests/test_cli_active_repair.py` | 452 | Owns active-run stale-marker repair coverage. |
| `src/runhaven/egress.py` | 404 | Cohesive provider proxy implementation. |
| `src/runhaven/plans.py` | 403 | Cohesive planner and validation module. |
| `src/runhaven/run_history.py` | 383 | Owns run-record persistence, provider/auth summaries, and `runs list/show/log/diff` output. |
| `scripts/check_pins.py` | 380 | Owns text target discovery, pin ledger orchestration, Python/dev deps, CI, Containerfile, and Debian package/source checks. |
| `src/runhaven/auth_broker.py` | 374 | Owns the live Codex API-key broker proxy, upstream forwarding, request validation, and broker decision aggregation. |
| `tests/test_cli_active_attach_logs.py` | 369 | Owns active attach and logs-follow coverage. |
| `tests/test_cli_provider_codex_broker.py` | 359 | Owns Codex API-key broker run, auth log, no-request, run-record, and missing-env coverage. |
| `src/runhaven/active_commands.py` | 342 | Owns active-run listing, attach/log-follow, sanitized status output, stop, and kill. |
| `tests/test_cli_standard_run.py` | 304 | Owns standard run record and active-marker lifecycle coverage. |
| `tests/test_cli_diagnostics.py` | 273 | Owns `auth`, `egress log`, and `why host` CLI coverage. |
| `tests/test_cli_runs_log.py` | 269 | Owns `runs log` text and JSON coverage. |
| `src/runhaven/diagnostic_commands.py` | 249 | Owns `auth status/explain/log`, `egress log`, `why host`, and diagnostic log readers. |
| `src/runhaven/active_repair.py` | 243 | Owns stale active-marker repair, JSON payloads, and inspect-missing validation. |
| `tests/test_cli_provider_proxy.py` | 242 | Owns provider plan, proxy injection, blocked-host summary, and policy-log coverage. |
| `src/runhaven/git_metadata.py` | 235 | Owns git discovery, status parsing, run git summary construction, and live diff helpers. |
| `tests/test_cli_runs_diff.py` | 233 | Owns `runs diff` git validation and output coverage. |
| `tests/test_cli_active_status.py` | 233 | Owns active status coverage. |
| `tests/test_cli.py` | 228 | Owns core CLI, setup, doctor, and plan smoke coverage. |
| `tests/test_cli_active_stop_kill.py` | 216 | Owns active stop and kill coverage. |
| `tests/test_cli_runs_list_show.py` | 195 | Owns `runs list` and `runs show` coverage. |
| `src/runhaven/auth_profiles.py` | 183 | Owns static auth broker profile metadata and status output data. |
| `tests/test_cli_active_list.py` | 133 | Owns active-run list coverage. |
| `scripts/npm_pin_policy.py` | 108 | Owns package.json and package-lock pin policy checks. |
| `tests/cli_test_helpers.py` | 107 | Shared git, run-record, and active-marker helpers for split CLI tests. |
| `tests/test_cli_state.py` | 80 | Owns state list, prune, and state lock coverage. |
| `tests/test_cli_provider_internal_network.py` | 52 | Owns existing, rejected, and newly created internal-network coverage. |

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
  fields, `runs list/show/log/diff`, and run-record readers. Git metadata
  helpers were split out later into `src/runhaven/git_metadata.py`.
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
- `tests/test_cli_provider_proxy.py`,
  `tests/test_cli_provider_codex_broker.py`, and
  `tests/test_cli_provider_internal_network.py`: provider runtime, Codex
  broker run, and internal-network CLI coverage.
- `tests/test_cli_standard_run.py`: standard run record and active-marker
  lifecycle coverage.
- `tests/test_cli_active_commands.py`: active listing, attach, logs-follow,
  status, stop, and kill coverage.
- `tests/test_cli_active_repair.py`: stale active-marker repair coverage.
- `tests/test_cli_runs_list_show.py`, `tests/test_cli_runs_diff.py`, and
  `tests/test_cli_runs_log.py`: `runs list/show/diff/log` coverage.
- `tests/test_cli_diagnostics.py`: `auth`, `egress log`, and `why host`
  diagnostic coverage.
- `tests/test_cli_state.py`: state list, prune, and state lock coverage.

This removes the 3,515-line CLI test file while preserving the existing 90 CLI
tests and the same production patch targets.

## Active-Command CLI Test Split Completed

- `tests/test_cli_active_list.py`: `runs active` text, JSON, and empty output
  coverage.
- `tests/test_cli_active_attach_logs.py`: `runs attach` and
  `runs logs-follow` coverage.
- `tests/test_cli_active_status.py`: `runs status` text, JSON, ownership, and
  inspect-failure coverage.
- `tests/test_cli_active_stop_kill.py`: `runs stop`, `runs kill`, rollback,
  missing-run, and ownership coverage.
- `tests/test_cli_active_repair.py`: unchanged stale active-marker repair
  coverage.

This removes the 900-line active-command CLI test file while preserving the
existing 33 active-command tests and the same production patch targets.

## Run-History CLI Test Split Completed

- `tests/test_cli_runs_list_show.py`: `runs list` text output,
  `runs show --json`, and git summary coverage.
- `tests/test_cli_runs_diff.py`: committed, dirty, untracked, mixed, and
  fail-closed `runs diff` coverage.
- `tests/test_cli_runs_log.py`: joined secret-free `runs log` text and JSON
  coverage.

This removes the 663-line run-history CLI test file while preserving the
existing 12 run-history tests and the same production patch targets.

## Provider-Runtime CLI Test Split Completed

- `tests/test_cli_provider_proxy.py`: provider allowlist planning, proxy
  injection and cleanup, blocked-host summary, and policy-log coverage.
- `tests/test_cli_provider_codex_broker.py`: Codex API-key broker config,
  auth log, no-request log, run record summary, and missing-env coverage.
- `tests/test_cli_provider_internal_network.py`: existing, rejected, and newly
  created internal-network coverage.

This removes the 622-line provider-runtime CLI test file while preserving the
existing 12 provider-runtime tests and the same production patch targets.

## Git-Metadata Extraction Completed

- `src/runhaven/git_metadata.py`: git worktree discovery, status parsing,
  run metadata snapshot summaries, diff validation helpers, and live git diff
  subprocess wrappers.
- `src/runhaven/run_history.py`: run-record persistence, provider/auth
  summaries, `runs list/show/log`, and the `runs diff` user-facing command
  output.

This removes git subprocess and parsing details from `run_history.py` while
preserving run metadata capture, live diff refusal behavior, and existing CLI
test coverage.

## Active-Repair Extraction Completed

- `src/runhaven/active_repair.py`: `runs repair`, `runs repair --all`, repair
  result payloads, exit-code rules, RunHaven-owned container validation, and
  confirmed-missing inspect checks.
- `src/runhaven/active_commands.py`: `runs active/status/attach/logs-follow`,
  `runs stop`, and `runs kill`, plus sanitized status output and attach/log
  command validation.

This removes stale-marker repair internals from `active_commands.py` while
preserving the existing CLI import surface, active-run ownership checks,
fail-closed repair behavior, and active-command test coverage.

## NPM Pin-Policy Extraction Completed

- `scripts/npm_pin_policy.py`: package.json and package-lock policy,
  install-script approvals, exact NPM version checks, registry checks, and
  lockfile integrity checks.
- `scripts/check_pins.py`: text target discovery, pin ledger loading,
  Python/dev dependency checks, CI action checks, Containerfile checks, Debian
  package/source checks, and orchestration of NPM package checks.

This keeps `python3 scripts/check_pins.py` as the pin-policy entrypoint while
moving package-lock-specific checks into a focused helper module.

## Auth-Profile Extraction Completed

- `src/runhaven/auth_profiles.py`: auth broker status constants, static
  per-profile metadata, JSON serialization, and profile lookup helpers.
- `src/runhaven/auth_broker.py`: live Codex API-key broker proxy, upstream
  forwarding, request validation, placeholder token constants, and broker
  decision aggregation. It re-exports the previous profile symbols to preserve
  the internal import surface.
- `src/runhaven/diagnostic_commands.py`: reads auth profile metadata directly
  from `auth_profiles.py` so read-only diagnostics do not import the live
  broker server implementation.

This removes static auth profile data from the live broker module while
preserving existing auth status and Codex broker behavior.

## Recommended Sequence

1. Review `src/runhaven/provider_runtime.py` for the same kind of
   complexity-only refactor. The next pass should be willing to stop at "no
   split needed" if the module is cohesive.

2. Consider splitting `tests/test_cli_active_repair.py` only if test
   readability becomes a blocker. The test file is large but currently maps to
   one focused command surface.

## Acceptance Criteria

- `cli.py` is primarily parser construction, command dispatch, and small
  command wrappers.
- Tests are grouped by command surface, not by the historical single CLI file.
- Runtime subprocess patch seams are explicit and reviewable.
- No refactor weakens macOS 26+ only support, default isolation, egress
  behavior, active-run ownership checks, or secret-free logs.
- Each slice runs focused tests plus the full harness before merge.
