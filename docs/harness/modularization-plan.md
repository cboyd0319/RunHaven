# Modularization Plan

Status: paused

This plan tracks the pre-release large-file refactor. Keep each slice
behavior-preserving unless a separate feature change is explicitly selected.
The current cleanup can stay paused after the completed extractions below
unless a concrete readability, ownership, or test-maintenance issue reappears.

## Current Size Snapshot

Measured on 2026-06-15 after the image doctor source-metadata slice and docs
accuracy pass:

| File | Lines | Notes |
| --- | ---: | --- |
| `src/runhaven/cli.py` | 605 | Owns command dispatch, standard run flow, state commands, image/network dispatch, and thin provider-runtime compatibility wrappers. |
| `tests/test_plans.py` | 533 | Owns run-plan validation, workspace scope, network mode, provider host, broker, and session planning coverage. |
| `tests/test_cli_standard_run.py` | 509 | Owns standard run record, active-marker lifecycle, session metadata, and worktree-run setup coverage. |
| `src/runhaven/plans.py` | 483 | Cohesive planner and validation module. |
| `tests/test_cli_worktree_lifecycle.py` | 445 | Owns `runs keep`, `runs recover`, `runs merge`, and `runs discard` worktree lifecycle coverage. |
| `src/runhaven/worktree_lifecycle.py` | 431 | Owns worktree run-record validation, keep/recover output, merge application, and discard cleanup. |
| `src/runhaven/cli_parser.py` | 411 | Owns argparse construction for all RunHaven CLI commands. |
| `src/runhaven/egress.py` | 404 | Cohesive provider proxy implementation. |
| `tests/test_cli_active_repair.py` | 401 | Owns active-run stale-marker repair coverage; reviewed as cohesive after helper cleanup. |
| `src/runhaven/run_history.py` | 399 | Owns run-record persistence, provider/auth summaries, and `runs list/show/log/diff` output. |
| `scripts/check_pins.py` | 380 | Owns text target discovery, pin ledger orchestration, Python/dev deps, CI, Containerfile, and Debian package/source checks. |
| `src/runhaven/provider_runtime.py` | 379 | Owns provider run lifecycle, proxy/broker startup, active marker cleanup, internal network inspection, and runtime command injection. |
| `src/runhaven/image_commands.py` | 375 | Owns image build, rebuild, doctor, source-digest labeling, stale detection, and state-volume review. |
| `src/runhaven/auth_broker.py` | 374 | Owns the live Codex API-key broker proxy, upstream forwarding, request validation, and broker decision aggregation. |
| `tests/test_cli_active_attach_logs.py` | 369 | Owns active attach and logs-follow coverage. |
| `scripts/provider_egress_smoke.py` | 360 | Owns live provider proxy smoke setup, allowed/denied path checks, and profile host smoke coverage. |
| `tests/test_cli_provider_codex_broker.py` | 359 | Owns Codex API-key broker run, auth log, no-request, run-record, and missing-env coverage. |
| `src/runhaven/active_commands.py` | 343 | Owns active-run listing, attach/log-follow, sanitized status output, stop, and kill. |
| `src/runhaven/provider_endpoints.py` | 296 | Owns the structured bundled, candidate, optional, and build-time provider endpoint ledger. |
| `tests/test_cli.py` | 279 | Owns core CLI, setup, doctor, and plan smoke coverage. |
| `tests/test_cli_image.py` | 277 | Owns image build, rebuild, image doctor, source metadata, and state-volume review coverage. |
| `tests/test_cli_diagnostics.py` | 273 | Owns `auth`, `egress log`, and `why host` CLI coverage. |
| `tests/test_cli_runs_log.py` | 269 | Owns `runs log` text and JSON coverage. |
| `src/runhaven/diagnostic_commands.py` | 249 | Owns `auth status/explain/log`, `egress log`, `why host`, and diagnostic log readers. |
| `tests/test_egress.py` | 245 | Owns provider proxy allow/deny, DNS guard, and empty-allowlist policy coverage. |
| `src/runhaven/active_repair.py` | 243 | Owns stale active-marker repair, JSON payloads, and inspect-missing validation. |
| `tests/test_cli_provider_proxy.py` | 242 | Owns provider plan, proxy injection, blocked-host summary, and policy-log coverage. |
| `src/runhaven/git_metadata.py` | 235 | Owns git discovery, status parsing, run git summary construction, and live diff helpers. |
| `tests/test_cli_runs_diff.py` | 233 | Owns `runs diff` git validation and output coverage. |
| `tests/test_cli_active_status.py` | 233 | Owns active status coverage. |
| `tests/test_cli_active_stop_kill.py` | 216 | Owns active stop and kill coverage. |
| `tests/test_cli_state.py` | 208 | Owns state list, prune, reset, session filtering, and state lock coverage. |
| `tests/test_cli_runs_list_show.py` | 195 | Owns `runs list` and `runs show` coverage. |
| `src/runhaven/auth_profiles.py` | 183 | Owns static auth broker profile metadata and status output data. |
| `scripts/codex_broker_smoke.py` | 170 | Owns optional live Codex broker smoke behavior with disposable-key gating. |
| `src/runhaven/worktrees.py` | 151 | Owns RunHaven-owned branch/worktree creation, cleanup, and validation helpers. |
| `tests/test_auth_broker.py` | 140 | Owns Codex broker request, subnet, method/path, logging, and upstream behavior. |
| `src/runhaven/doctor.py` | 139 | Owns host prerequisite checks. |
| `src/runhaven/active_records.py` | 138 | Owns active-run marker persistence and status updates. |
| `src/runhaven/provider_observability.py` | 137 | Owns provider policy log writes, auth broker log writes, blocked-host review text, and UTC timestamps. |
| `tests/test_cli_active_list.py` | 133 | Owns active-run list coverage. |
| `src/runhaven/project_checks.py` | 122 | Owns advisory project-check detection for worktree review flows. |
| `tests/test_codex_broker_smoke.py` | 121 | Owns optional Codex broker smoke skip, missing-key, and command behavior. |
| `tests/test_cli_network.py` | 118 | Owns RunHaven-managed network list and prune CLI coverage. |
| `scripts/npm_pin_policy.py` | 108 | Owns package.json and package-lock pin policy checks. |
| `tests/cli_test_helpers.py` | 107 | Shared git, run-record, and active-marker helpers for split CLI tests. |
| `src/runhaven/images.py` | 100 | Owns bundled image profile paths, template source hashing, and expected image metadata. |
| `src/runhaven/setup_guide.py` | 94 | Owns guided setup output. |
| `src/runhaven/network_commands.py` | 86 | Owns managed-network list and prune command behavior. |

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
- `tests/test_cli_active_list.py`, `tests/test_cli_active_attach_logs.py`,
  `tests/test_cli_active_status.py`, and
  `tests/test_cli_active_stop_kill.py`: active listing, attach, logs-follow,
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

## Provider-Observability Extraction Completed

- `src/runhaven/provider_observability.py`: provider proxy policy log writes,
  auth broker log writes, blocked-host review text, provider denial next-action
  text, and UTC timestamp formatting.
- `src/runhaven/provider_runtime.py`: provider run lifecycle, proxy and Codex
  broker startup, active marker lifecycle, command environment injection,
  provider network cleanup, and Apple `container network` inspection.

This separates provider run lifecycle control from provider observability while
preserving the existing policy log, auth log, run record, and blocked-host
review behavior.

## CLI Parser Extraction Completed

- `src/runhaven/cli_parser.py`: argparse construction for top-level commands,
  run/plan options, run-history commands, active-run commands, auth, egress,
  and `why host`.
- `src/runhaven/cli.py`: command dispatch, standard run flow, state commands,
  provider-runtime compatibility patch seams, and CLI-specific runtime helpers.

This separates parser construction from command execution while preserving the
existing `runhaven.cli.build_parser` import surface and the `Path` patch seam
used by help-path regression tests.

## Active-Repair Test Cleanup Completed

- `tests/test_cli_active_repair.py`: replaced repeated hand-written active
  marker JSON setup with the existing `write_active_marker` helper.

The file remains a single focused repair test surface. Splitting it now would
mostly move cohesive `runs repair` coverage around, so the large-file cleanup
can pause unless a concrete readability issue reappears.

## Recommended Sequence

1. Pause the large-file cleanup and return to the product
   backlog unless a concrete maintainability problem remains.

## Acceptance Criteria

- `cli.py` is primarily command dispatch, standard run flow, and small command
  wrappers.
- Tests are grouped by command surface, not by the historical single CLI file.
- Runtime subprocess patch seams are explicit and reviewable.
- No refactor weakens macOS 26+ only support, default isolation, egress
  behavior, active-run ownership checks, or secret-free logs.
- Each slice runs focused tests plus the full harness before merge.
