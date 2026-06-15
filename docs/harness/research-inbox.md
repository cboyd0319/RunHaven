# Research Inbox

Generated: 2026-06-14

Use this file for compact source-change notes. Keep long excerpts out of the
repo and link to primary sources instead.

## Promotion checklist

- The source is current and relevant to this repo's harness.
- The finding improves security, portability, verification, restartability, or
  ease of use.
- The change can be represented as a template, audit check, verification
  command, policy file, or clear documentation.
- The proposed update preserves local-machine safety and reviewability.

## Notes

- 2026-06-15: Completed a manual source-mining pass across sibling local
  checkouts `awman`, `aspec`, and `maki`. AGY/Antigravity was not used.
- The full recommendation set is recorded in
  `docs/harness/source-mined-ideas.md`.
- High-value candidates promoted from the pass: provider endpoint matrix,
  provider proxy DNS/private-address guard, worktree isolation, workspace scope
  selection, run observability, strict workflow files, context overlays, MCP
  boundaries, generated docs drift checks, and JSON/headless output.
- Ideas explicitly not promoted as-is: host Keychain or credential extraction,
  default host home mounts, Docker fallback, Windows/Linux runtime support,
  `container machine` defaults, and unreviewed host-side dynamic provider
  scripts.
- 2026-06-15: Completed a manual external open source research pass. The full
  set is recorded in `docs/harness/external-project-ideas.md`.
- Additional external-project candidates promoted from that pass: `runhaven
  why` diagnostics, provider proxy policy logs, empty-allowlist regression
  tests, host-side provider credential brokering, agent profile investigation
  docs, devcontainer metadata import for image planning, warm reusable project
  sessions, and explicit extension/MCP boundary policy.
- 2026-06-15: Completed a UX-focused research pass around easier setup,
  clearer blocked-action explanations, recovery, and lower-friction safe
  autonomy. The full result is recorded in
  `docs/harness/ux-research-ideas.md`.
- UX candidates promoted from that pass: guided `runhaven setup`, goal-based
  network selection, `runhaven why`, provider policy logs, grouped
  blocked-host review, `runs list/show/log/diff/attach/stop`, worktree review
  flows, image/state/network repair commands, `auth status`, and task-language
  docs recipes.
- 2026-06-15: Implemented the git metadata slice for run observability. Actual
  runs now record before and after `HEAD`, dirty state, changed count, and
  capped relative paths for git workspaces without storing diffs, file
  contents, prompts, commands, or secrets.
- 2026-06-15: Implemented `runhaven runs diff RUN_ID` from the promoted run
  dashboard backlog. The command prints live git output only after recorded
  repo, `HEAD`, and path metadata still match the current workspace.
- 2026-06-15: Implemented `runhaven runs stop RUN_ID` from the promoted run
  recovery backlog. Active markers stay secret-free and stop only
  RunHaven-owned named Apple containers.
- 2026-06-15: Implemented `runhaven runs active` so users can recover active
  run ids from secret-free markers before using `runs stop`.
- 2026-06-15: Implemented `runhaven runs attach RUN_ID` from the promoted run
  recovery backlog. The command uses Apple `container exec` because the pinned
  local Apple `container` CLI exposes `exec` and has no installed `attach`
  plugin.
- 2026-06-15: Implemented `runhaven runs logs-follow RUN_ID` from the promoted
  run visibility backlog. The command uses Apple `container logs --follow`
  after the same active-marker and RunHaven-owned container-name checks.
- 2026-06-15: Implemented `runhaven runs status RUN_ID` from the promoted run
  visibility backlog. The command uses Apple `container inspect` but prints
  only curated state so raw inspect arguments, environment, and mounts are not
  exposed.
- 2026-06-15: Implemented `runhaven runs kill RUN_ID` from the promoted run
  recovery backlog. The command uses Apple `container kill` after the same
  active-marker and RunHaven-owned container-name checks.
- 2026-06-15: Implemented `runhaven runs repair RUN_ID` from the promoted run
  recovery backlog. The command removes a stale marker only after Apple
  `container inspect` reports that the recorded RunHaven-owned container is
  not found.
- 2026-06-15: Implemented `runhaven runs repair --all` from the promoted run
  recovery backlog. The command applies the same confirmed-missing guard to
  each valid active marker and keeps live or unverified markers.
- 2026-06-15: Implemented repair JSON summaries from the promoted automation
  backlog. `runhaven runs repair RUN_ID --json` and
  `runhaven runs repair --all --json` emit secret-free result lists, counts,
  and exit codes.
- 2026-06-15: Implemented the first guided setup slice from the promoted UX
  backlog. `runhaven setup` runs prerequisite checks, prints exact remedies
  when the host is not ready, and shows profile-specific first-run commands
  without installing, starting, building, running, or mounting anything.
- 2026-06-15: Implemented goal-based network selection copy in
  `runhaven setup`. The guide now distinguishes local-only, provider-only,
  package install, and unrestricted internet runs without changing runtime
  behavior.
- 2026-06-15: Implemented workspace-scope and credential-path guidance in
  `runhaven setup`. The guide now tells users to run from the smallest project
  directory, confirms `/workspace` semantics, names credential paths that are
  not mounted by default, and points to `--ssh` plus reviewed `--env NAME`
  usage.
- 2026-06-15: Implemented explicit workspace scope selection from the promoted
  backlog. `runhaven plan` and `runhaven run` default to current-directory
  workspace scope, warn when that directory sits inside a larger git repo, and
  expand to the repo root only with `--workspace-scope git-root`.
- 2026-06-15: Implemented the first worktree isolation slice from the promoted
  backlog. `runhaven run AGENT --worktree` now creates a RunHaven-owned branch
  and git worktree for clean source repositories, mounts that worktree for the
  agent, and records exact recovery commands; `runs merge`, `runs keep`, and
  `runs discard` remain backlog items.
- 2026-06-15: Added a pre-release backlog item to consider a major large-file
  refactor and modularization pass, especially around the CLI and broad test
  modules, before release.
- 2026-06-15: Started the pre-release modularization pass. Added
  `docs/harness/modularization-plan.md` and extracted setup guide output,
  active-run marker persistence, cache path helpers, and shared validators from
  `src/runhaven/cli.py`.
- 2026-06-15: Continued the pre-release modularization pass by extracting
  `src/runhaven/run_history.py` for run-record persistence, git metadata
  capture, `runs list/show/log/diff`, and run-record readers.
- 2026-06-15: Continued the pre-release modularization pass by extracting
  `src/runhaven/active_commands.py` for `runs active/status/attach/logs-follow`,
  `runs stop/kill/repair`, sanitized status output, and repair payloads.
- 2026-06-15: Continued the pre-release modularization pass by extracting
  `src/runhaven/diagnostic_commands.py` for `auth status/explain/log`,
  `egress log`, `why host`, provider/auth log readers, and provider endpoint
  explanation output.
- 2026-06-15: Continued the pre-release modularization pass by splitting the
  3,515-line `tests/test_cli.py` into focused CLI test files for core/setup,
  provider runtime, standard runs, active commands, active repair, run history,
  diagnostics, and state, plus `tests/cli_test_helpers.py`.
- 2026-06-15: Continued the pre-release modularization pass by splitting the
  900-line `tests/test_cli_active_commands.py` into focused CLI test files for
  active listing, attach/logs-follow, status, and stop/kill.
- 2026-06-15: Continued the pre-release modularization pass by splitting the
  663-line `tests/test_cli_run_history.py` into focused CLI test files for
  run list/show, run diff, and joined run logs.
- 2026-06-15: Continued the pre-release modularization pass by splitting the
  622-line `tests/test_cli_provider_runtime.py` into focused CLI test files for
  provider proxy behavior, Codex broker behavior, and internal-network
  handling.
- 2026-06-15: Continued the pre-release modularization pass by extracting
  `src/runhaven/git_metadata.py` from `src/runhaven/run_history.py` for git
  discovery, status parsing, run metadata summaries, and live diff helpers.
- 2026-06-15: Continued the pre-release modularization pass by extracting
  `src/runhaven/active_repair.py` from `src/runhaven/active_commands.py` for
  stale-marker repair, JSON payloads, exit-code rules, and inspect-missing
  validation.
- 2026-06-15: Reviewed "Development On Apple Silicon with Apple Container
  Machine" and recorded UX backlog items in
  `docs/harness/ux-research-ideas.md`: explain why RunHaven avoids
  `container machine` defaults, add future host-service/DNS diagnostics,
  treat remote-editor and persistent-dev-environment workflows as explicit
  advanced modes, and support inspect-before-run bootstrap recommendations.
- 2026-06-15: First implementation slice landed from the promoted backlog:
  provider proxy DNS/private-address rejection, provider policy decision logs,
  and `runhaven why host ...`.
- 2026-06-15: Promoted the provider endpoint matrix into
  `docs/PROVIDER_ENDPOINTS.md` and `src/runhaven/provider_endpoints.py`.
  Source-backed defaults now include Claude auth hosts, Codex ChatGPT auth, and
  Copilot-specific routing hosts. Broad path-sensitive, telemetry, update, and
  weakly sourced hosts remain explicit review items.
- 2026-06-15: Implemented grouped blocked-host review and profile-host smoke
  support. Provider run summaries now include run id, reason, count, rule, and
  suggested next action; `scripts/provider_egress_smoke.py --agent AGENT`
  validates bundled provider hosts without credentials.
- 2026-06-15: Implemented the auth broker boundary diagnostics slice.
  `runhaven auth status` and `runhaven auth explain AGENT` read static profile
  metadata only, with JSON output available for future automation. No
  credential store or environment value is read.
- 2026-06-15: Implemented empty-allowlist regression coverage for network
  policy modes. Internet mode stays explicitly unrestricted, internal mode
  stays local-only, provider mode fails closed without provider hosts, and the
  proxy policy rejects empty allowlists directly.
- 2026-06-15: Continued pre-release modularization by extracting
  `scripts/npm_pin_policy.py` from `scripts/check_pins.py` for NPM package and
  package-lock policy checks.
- 2026-06-15: Continued pre-release modularization by extracting
  `src/runhaven/auth_profiles.py` from `src/runhaven/auth_broker.py` for static
  auth broker profile metadata.
- 2026-06-15: Continued pre-release modularization by extracting
  `src/runhaven/provider_observability.py` from
  `src/runhaven/provider_runtime.py` for provider policy logs, auth broker log
  writes, and blocked-host review text.
- 2026-06-15: Continued pre-release modularization by extracting
  `src/runhaven/cli_parser.py` from `src/runhaven/cli.py` for argparse
  construction.
- 2026-06-15: Reviewed `tests/test_cli_active_repair.py`; kept it as one
  focused repair test surface and reduced repeated marker setup with the
  existing `write_active_marker` helper.
