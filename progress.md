# Progress

Last Updated: 2026-06-15

## Current Objective

Add worktree lifecycle commands.

## Current State

- The project has been renamed to RunHaven.
- The Python package, import module, console command, image tags, resource
  prefixes, cache path, tests, docs, and harness metadata now use `runhaven`.
- The old project, module, CLI, env var, runtime path, and filename patterns are
  absent from all working tree files outside `.git`.
- Ignored local `.venv*` directories were removed because generated activation
  scripts and editable-install metadata encoded the old local checkout path.
- The GitHub repository remote has not been renamed because that is a
  credentialed vendor change requiring explicit approval.
- Harness state files now exist: `feature_list.json`, `progress.md`, and
  `session-handoff.md`.
- Verification entrypoint now exists: `init.sh`.
- Harness docs now exist under `docs/harness/`.
- `AGENTS.md` now includes Startup, Verification, Definition Of Done, state
  file routing, and End of Session instructions.
- `docs/HARNESS_EVALUATION.md` records the before and after audit result.
- `docs/assets/logo.png` is now the tracked project logo and is displayed by
  `README.md`.
- `README.md` is now a concise project entry point instead of the full user
  manual. Detailed setup and capability material moved into
  `docs/INSTALLATION.md` and `docs/CAPABILITIES.md`, while command workflows
  remain in `docs/USAGE.md`.
- RunHaven is now documented and checked as macOS 26+ only. Windows and Linux
  runtime or contributor-verification targets are intentionally unsupported.
- The non-macOS verification entrypoint was removed.
- Command construction now rejects unsafe image references, invalid resource
  values, broad or credential-bearing workspaces, comma-containing workspace
  paths, and root agent execution unless explicitly overridden.
- Internal network reuse now verifies Apple `container` reports `hostOnly`.
- `runhaven state list` and `runhaven state prune --yes` manage isolated agent
  home volumes.
- Dev dependencies now match the `unittest` suite and no longer include pytest.
- `scripts/check_pins.py` now enforces `pins.toml` against source files.
- Follow-up hardening now rejects root group identities such as `agent:0`
  unless `--allow-root-user` is explicit.
- CLI help no longer resolves the current working directory during parser
  construction.
- `scripts/check_pins.py` now discovers image template `Containerfile` and
  npm-backed package directories dynamically.
- Added tests for `runhaven run`, doctor command error paths, root group
  rejection, and help behavior with an unavailable current directory.
- Second follow-up hardening now rejects invalid programmatic network modes
  instead of silently using internet mode.
- Root user detection now treats leading-zero numeric identities such as `00`
  and `agent:00` as root.
- Sensitive macOS system paths such as `/System`, `/Library`, and `/etc` now
  require `--allow-sensitive-workspace`.
- `runhaven doctor` now prints concise remediation for failed checks.
- `runhaven plan` and `runhaven run` help now explain the `--` separator for
  agent flags.
- Pin policy now records the RunHaven package/image version in `pins.toml`,
  checks package and image version consistency from that ledger, and rejects
  non-macOS GitHub runner pins.
- Apple DocC documentation was rendered with Playwright and cross-checked
  through generated DocC JSON endpoints because the raw HTML page is a
  JavaScript shell.
- The complete user-supplied DocC snapshot was reviewed: 1,022 rendered
  Markdown pages plus raw DocC JSON, zero fetch failures, and no exact hits for
  egress or allowlist control terms.
- Apple `container` 1.0.0 exposes NAT networking, DNS selection, subnet
  settings, and host-only networks, but no reviewed domain egress allowlist
  surface was found in rendered docs, generated JSON, local CLI help, or the
  pinned command reference.
- `runhaven plan` now prints explicit egress status for the selected network,
  provider hosts, and the runtime proxy injection note for provider mode.
- A standard-library CONNECT allowlist proxy now exists in `src/runhaven/egress.py`.
- `scripts/provider_egress_smoke.py` proves the proxy pattern with a temporary
  internal Apple `container` network.
- Live smokes passed for the default public host and for `api.openai.com`:
  allowed proxied HTTPS succeeded, while denied proxied host, proxied IP
  literal, direct DNS, and direct IP paths failed.
- `runhaven run --network provider` now creates a managed internal network,
  inspects its gateway and subnet, starts the host-side allowlist proxy, injects
  proxy environment variables, runs the agent, and deletes the managed provider
  network in cleanup.
- Bundled provider host allowlists now exist for Claude, Codex, Gemini, and
  Copilot. `--provider-host HOST` adds reviewed fully qualified extra hosts for
  provider mode.
- Provider host additions reject IP literals and single-label hosts, so entries
  like `com` cannot accidentally allow broad suffixes. A listed host permits
  that host and its subdomains.
- Provider mode now records blocked CONNECT host/port pairs in memory, caps the
  list, and prints a stderr summary after the run with review guidance for
  fully qualified host additions.
- Provider mode now resolves allowed hosts before connecting and denies
  loopback, private, link-local, multicast, and otherwise non-public resolved
  addresses.
- Provider mode now records allowed and denied proxy policy decisions and
  persists them to a JSONL cache log after each run.
- `runhaven egress log` now shows recent provider proxy decisions, with JSON
  output available for automation.
- `runhaven why host ...` now explains provider-host matching, IP literal
  rejection, bundled profile matches, and the next review step before a user
  adds a new host.
- `src/runhaven/provider_endpoints.py` is now the structured provider endpoint
  ledger for bundled, candidate, optional, and build-time hosts.
- `docs/PROVIDER_ENDPOINTS.md` records the source-backed provider endpoint
  matrix and explains why broad path-sensitive, telemetry, update, reporting,
  and weakly sourced hosts stay explicit.
- Source-backed bundled provider defaults now include Claude auth hosts,
  Codex ChatGPT auth, and Copilot-specific routing hosts. Antigravity still has
  no bundled runtime hosts because no minimal source-backed endpoint set has
  been identified.
- `runhaven why host ...` now surfaces known explicit-review endpoints, such as
  Copilot `api.github.com`, with the reason they are not bundled.
- Provider runs now print grouped blocked-host reviews with run id, count,
  denial reason, matched rule, and suggested next action.
- `runhaven egress log` now includes the run id in text output, matching the
  JSONL log field.
- `scripts/provider_egress_smoke.py --agent AGENT` now checks all bundled
  provider hosts for a selected profile through the same host-side proxy
  pattern, without requiring provider credentials.
- A live normal-run smoke passed for `runhaven run shell --network provider
  --provider-host example.com`: allowed proxied HTTPS succeeded, while denied
  proxied host, proxied IP literal, direct DNS, and direct IP paths failed.
- `container machine` remains out of scope for the default product boundary
  because Apple's docs say it maps the host username and home directory into the
  Linux environment.
- Supplemental Apple `container` sources supplied by the user were reviewed and
  recorded in `docs/RESEARCH.md`: Apple Open Source, DeepWiki, Wikipedia, The
  Register, HowToUseLinux, Apidog, and Suraj Deshmukh.
- The new source review reinforced current RunHaven decisions: macOS 26+ only,
  task-scoped `container run`, no `container machine` default, no host home
  mount, and no DNS-as-egress-control claim.
- Sibling repos `awman`, `aspec`, and `maki` were manually mined for
  transferable ideas. AGY/Antigravity was intentionally not used for this pass.
- The complete source-mined recommendation set is recorded in
  `docs/harness/source-mined-ideas.md` and summarized in
  `docs/harness/research-inbox.md`.
- High-value promoted candidates include provider endpoint matrix, provider
  proxy DNS/private-address guard, worktree isolation, workspace scope
  selection, run observability, typed run options, strict workflow files,
  context overlays, generated docs drift checks, JSON/headless output, and
  deny-by-default MCP or extension boundaries.
- The pass does not discard ideas merely because they are large. It stages them
  by dependency and risk while preserving the hard product boundary: macOS 26+
  only, Apple `container` only, no Docker fallback, no Windows/Linux runtime
  support, no host home or credential mounts by default, and no
  `container machine` default.
- Current external open source projects were manually researched for adjacent
  ideas. The full result is recorded in
  `docs/harness/external-project-ideas.md`.
- External source-backed candidates now in the backlog include `runhaven why`
  diagnostics, provider proxy policy logs, empty-allowlist regression tests,
  host-side provider credential brokering, agent profile investigation docs,
  devcontainer metadata import for image planning, warm reusable project
  sessions, and explicit extension/MCP boundary policy.
- A UX-focused research pass is recorded in
  `docs/harness/ux-research-ideas.md`.
- UX candidates now in the backlog include guided `runhaven setup`, goal-based
  network selection, `runhaven why`, provider policy logs, grouped
  blocked-host review, `runs list/show/log/diff/attach/stop`, worktree review
  flows, image/state/network repair commands, `auth status`, and
  task-language docs recipes.
- The "Development On Apple Silicon with Apple Container Machine" field report
  was reviewed and recorded in the UX backlog. It reinforced that RunHaven
  should keep `container machine` out of the beginner-safe default path while
  considering future host-service/DNS diagnostics, explicit advanced
  remote-editor workflows, and inspect-before-run bootstrap recommendations.
- The pre-release backlog now includes considering a major large-file refactor
  and modularization pass, especially around the CLI and broad test modules,
  before release.
- The pre-release modularization plan is now tracked in
  `docs/harness/modularization-plan.md`. The first behavior-preserving
  extraction moved setup guide output, active-run marker persistence, cache
  paths, and shared validators out of `src/runhaven/cli.py`.
- The second behavior-preserving modularization extraction moved run-record
  persistence, git metadata capture, `runs list/show/log/diff`, and run-record
  readers into `src/runhaven/run_history.py`. `src/runhaven/cli.py` now
  measures 1,874 lines, down from 2,440 after the first extraction.
- The third behavior-preserving modularization extraction moved active-run
  command handlers, sanitized status output, attach/log-follow command
  construction, stop/kill, and repair into `src/runhaven/active_commands.py`.
  `src/runhaven/cli.py` now measures 1,411 lines, down from 1,874 after the
  run-history extraction.
- The fourth behavior-preserving modularization extraction moved provider run
  orchestration, provider proxy and Codex broker startup, proxy/broker command
  injection, provider policy and auth decision logging, provider network
  cleanup, and internal-network inspection helpers into
  `src/runhaven/provider_runtime.py`. `src/runhaven/cli.py` now measures 1,005
  lines, down from 1,411 after the active-command extraction.
- The fifth behavior-preserving modularization extraction moved read-only
  diagnostic command handlers, provider/auth JSONL log readers, auth broker
  status/explain output, provider egress log output, and `why host` provider
  endpoint explanations into `src/runhaven/diagnostic_commands.py`.
  `src/runhaven/cli.py` now measures 767 lines, down from 1,005 after the
  provider-runtime extraction.
- The sixth behavior-preserving modularization extraction split the 90 CLI
  tests in `tests/test_cli.py` into focused files for core/setup, provider
  runtime, standard runs, active commands, active repair, run history,
  diagnostics, and state. `tests/test_cli.py` now measures 228 lines, down from
  3,515 lines; the largest split CLI test file is
  `tests/test_cli_active_commands.py` at 900 lines.
- The seventh behavior-preserving modularization extraction split the 33
  active-command CLI tests in `tests/test_cli_active_commands.py` into focused
  files for active listing, attach/logs-follow, status, and stop/kill.
- The eighth behavior-preserving modularization extraction split the 12
  run-history CLI tests in `tests/test_cli_run_history.py` into focused files
  for run list/show, run diff, and joined run logs.
- The ninth behavior-preserving modularization extraction split the 12
  provider-runtime CLI tests in `tests/test_cli_provider_runtime.py` into
  focused files for provider proxy behavior, Codex broker behavior, and
  internal-network handling.
- The tenth behavior-preserving modularization extraction moved git discovery,
  status parsing, run metadata summaries, and live diff helpers from
  `src/runhaven/run_history.py` into `src/runhaven/git_metadata.py`.
  `src/runhaven/run_history.py` now measures 383 lines, down from 604 lines.
- The eleventh behavior-preserving modularization extraction moved
  stale-marker repair, repair JSON payloads, exit-code rules, and
  inspect-missing validation from `src/runhaven/active_commands.py` into
  `src/runhaven/active_repair.py`. `src/runhaven/active_commands.py` now
  measures 342 lines, down from 569 lines.
- The twelfth behavior-preserving modularization extraction moved NPM package
  and package-lock pin policy from `scripts/check_pins.py` into
  `scripts/npm_pin_policy.py`. `scripts/check_pins.py` now measures 380 lines,
  down from 497 lines.
- The thirteenth behavior-preserving modularization extraction moved static
  auth broker profile metadata from `src/runhaven/auth_broker.py` into
  `src/runhaven/auth_profiles.py`. `src/runhaven/auth_broker.py` now measures
  374 lines, down from 520 lines.
- The fourteenth behavior-preserving modularization extraction moved provider
  policy log writes, auth broker log writes, blocked-host review text, denial
  next-action text, and UTC timestamp formatting from
  `src/runhaven/provider_runtime.py` into
  `src/runhaven/provider_observability.py`. `src/runhaven/provider_runtime.py`
  now measures 379 lines, down from 501 lines.
- The fifteenth behavior-preserving modularization extraction moved argparse
  construction from `src/runhaven/cli.py` into `src/runhaven/cli_parser.py`.
  `src/runhaven/cli.py` now measures 472 lines, down from 767 lines.
- The active-repair CLI test file was reviewed and kept as one focused repair
  command surface. Repeated hand-written active marker JSON setup now uses the
  existing `write_active_marker` helper, reducing
  `tests/test_cli_active_repair.py` to 401 lines.
- `runhaven plan` and `runhaven run` now support explicit workspace scope
  selection with `--workspace-scope current|git-root`. Default current scope
  keeps selected git subdirectories mounted without silently broadening to the
  repo root; explicit git-root scope expands only inside a git worktree.
- `runhaven run AGENT --worktree` now creates a RunHaven-owned git branch and
  worktree for clean source repositories, mounts that worktree for the agent,
  keeps it after the run, and records exact recovery commands in the run
  record. Dirty source repositories fail before worktree creation.
- `runhaven runs keep RUN_ID`, `runhaven runs merge RUN_ID`, and
  `runhaven runs discard RUN_ID` now provide guarded worktree lifecycle
  actions. They validate the recorded RunHaven-owned source repository,
  worktree, branch, and base metadata before acting. Merge refuses dirty or
  moved source checkouts, applies committed, dirty, and untracked worktree
  changes back to the source checkout, then removes the worktree and branch.
  Discard removes only the recorded RunHaven worktree and branch.
- `src/runhaven/auth_profiles.py` now records per-profile auth broker metadata,
  and `src/runhaven/auth_broker.py` implements the first Codex API-key broker
  prototype.
- `runhaven run codex --network provider --codex-api-key-broker-env NAME` reads
  the named host environment variable only during real runs, starts a
  subnet-restricted host broker on the Apple `container` provider network, and
  injects temporary Codex custom-provider config plus a placeholder token into
  the guest.
- The Codex broker accepts only Responses API create requests and injects the
  raw host API key into the host-side upstream request to `api.openai.com`; the
  raw key is not placed in the planned command or guest environment.
- `runhaven plan` and `runhaven run --dry-run` show broker status but do not
  read the named API-key environment variable.
- Missing Codex broker environment variables fail before Apple `container`
  runtime startup.
- `runhaven auth status` and `runhaven auth explain AGENT` now describe the
  current host-side broker boundary without reading Keychain, browser profiles,
  cloud credential files, provider login caches, or environment values.
- `runhaven auth status --json` and `runhaven auth explain AGENT --json` expose
  broker metadata for automation without secret values.
- `docs/AUTH_BROKER.md` records the Codex prototype status, remaining
  design-only provider status, trust boundary, provider auth notes, non-goals,
  and remaining acceptance criteria.
- `docs/RESEARCH.md` now records the current provider auth references and the
  official Codex configuration references used for this broker boundary.
- The Codex broker now records in-memory decisions with method, sanitized path,
  allow/deny outcome, reason, upstream status, and count.
- Brokered Codex runs now write secret-free entries to
  `auth-broker.jsonl` under the RunHaven cache root. The log omits token
  values, request bodies, and environment variable names.
- `runhaven auth log` and `runhaven auth log --json` now display recent auth
  broker decisions without inspecting credential stores or environment values.
- `scripts/codex_broker_smoke.py` can run a real non-interactive Codex request
  through the broker when `RUNHAVEN_CODEX_BROKER_SMOKE_API_KEY` or another
  named disposable-key env var is set. Without the key it prints `SKIP` and
  exits successfully unless `--require-api-key` is passed.
- Actual `runhaven run` executions now append secret-free records to
  `runs.jsonl` under the RunHaven cache root.
- `runhaven runs list` and `runhaven runs show RUN_ID` now display run id,
  profile, workspace, network mode, return code, provider policy summary, auth
  broker summary, and cleanup outcome.
- `runhaven runs log RUN_ID` and `runhaven runs log RUN_ID --json` now join the
  selected run record with matching provider policy and auth broker entries for
  the same run id.
- Git workspaces now add a run metadata summary with repo root, before and
  after `HEAD`, dirty state, changed file count, and a capped list of relative
  paths scoped to the selected workspace. Non-git workspaces are recorded as
  git unavailable.
- `runhaven runs show RUN_ID` and `runhaven runs log RUN_ID` now print a
  compact git summary line when metadata is present.
- `runhaven runs diff RUN_ID` now validates recorded git metadata against the
  live workspace and prints a live git diff for committed changes or dirty
  workspace changes.
- `runhaven runs diff RUN_ID` refuses when git metadata is unavailable, the
  recorded repo or workspace is gone, `HEAD` no longer matches the recorded
  run, the recorded path list was truncated, or the current dirty path set
  differs from the run record.
- Dirty working-tree diffs print a warning because RunHaven can verify the
  recorded `HEAD` and path set, but not exact file contents since the run.
- Planned and actual agent runs now include a RunHaven-owned Apple `container`
  name derived from profile and workspace.
- Active runs now write a temporary secret-free marker under
  `active-runs/<run-id>.json` in the RunHaven cache root. The marker records run
  id, profile, workspace, network mode, state volume, host pid, and container
  name, but not command lines, agent arguments, environment variable names,
  environment values, request bodies, prompts, or token values.
- `runhaven runs stop RUN_ID` now reads the active marker, verifies the
  container name is RunHaven-owned, marks stop requested, and calls Apple
  `container stop` for that container.
- `runhaven runs active` now lists current active-run markers in text or JSON
  without requiring Apple `container` access. It skips invalid or
  non-actionable marker files.
- `runhaven runs attach RUN_ID` now reads the active marker, verifies the
  container name is RunHaven-owned, rejects root attach without
  `--allow-root-user`, and calls Apple `container exec` to start a guarded
  shell or command in the active container.
- `runhaven runs logs-follow RUN_ID` now reads the active marker, verifies the
  container name is RunHaven-owned, and calls Apple `container logs --follow`
  with a configurable recent-line cap.
- `runhaven runs status RUN_ID` now reads the active marker, verifies the
  container name is RunHaven-owned, calls Apple `container inspect`, and prints
  only curated marker, state, image, resource, and network fields.
- `runhaven runs kill RUN_ID` now reads the active marker, verifies the
  container name is RunHaven-owned, marks kill requested, calls Apple
  `container kill`, rolls the marker back if the kill command fails, and
  records killed runs as `killed`.
- `runhaven runs repair RUN_ID` now reads the active marker, verifies the
  container name is RunHaven-owned, calls Apple `container inspect`, and removes
  the marker only when Apple reports that the recorded container is not found.
  It keeps the marker if the container still exists or if inspection fails for
  any other reason.
- `runhaven runs repair --all` now applies the same confirmed-missing guard to
  all valid active markers, removes only confirmed-stale markers, keeps live or
  unverified markers, and returns nonzero when any marker cannot be verified.
- `runhaven runs repair RUN_ID --json` and
  `runhaven runs repair --all --json` now emit secret-free repair results,
  counts, and exit codes for scripts without raw Apple inspect output or
  active-marker contents.
- `runhaven setup` now runs the same prerequisite checks as `doctor`, prints
  exact remedies when the host is not ready, and shows profile-specific image
  build, plan, and run commands when prerequisites pass.
- `runhaven setup --agent AGENT` selects the profile used in the suggested
  first-run commands. The command is intentionally non-mutating: it does not
  install Apple `container`, start services, build images, run agents, write
  state, or mount a workspace.
- `runhaven setup` now prints goal-based network guidance for local-only,
  provider-only, package install, and unrestricted internet runs. The guidance
  keeps provider egress framed as stricter and potentially review-heavy for
  login, telemetry, package registry, or feature-path hosts.
- `runhaven setup` now prints workspace-scope and credential-path guidance:
  run from the smallest project directory, confirm the `/workspace` mount with
  `runhaven plan`, avoid home directories and credential folders, use `--ssh`
  for SSH agent forwarding, and pass only reviewed environment variables with
  `--env NAME`.
- Active markers are removed after run completion. If a run exits after a stop
  or kill request, the completed run record is marked `stopped` or `killed`.
- Run records omit diffs, file contents, prompts, command lines, agent
  arguments, environment variable names, environment values, request bodies,
  and token values.
- Empty provider allowlist behavior is now covered at both the planner and
  proxy-policy layers.
- `internet` mode is regression-tested as no provider allowlist and
  unrestricted internet egress.
- `internal` mode is regression-tested as no provider allowlist and internet
  egress disabled.
- `provider` mode is regression-tested as fail-closed when the selected
  profile and explicit `--provider-host` values produce an empty allowlist.
- Profiles without bundled provider hosts, such as `shell` and `antigravity`,
  are regression-tested so they require an explicit fully qualified
  `--provider-host` before provider mode can plan.
- `EgressPolicy` is regression-tested so an empty allowlist cannot become an
  allow-all policy.

## Recommended Next Step

Run the optional Codex broker smoke with a disposable OpenAI API key when one
is available. Continue worktree polish with conflict-recovery guidance only
after the first guarded lifecycle commands have settled.

## Verification Evidence

- 2026-06-15: Focused provider-runtime extraction tests passed:
  `PYTHONPATH=src python3 -m unittest` with 11 selected provider runtime,
  Codex broker, provider run record, and internal-network tests.
- 2026-06-15: Focused diagnostic-command extraction tests passed:
  `PYTHONPATH=src python3 -m unittest` with 11 selected `egress log`,
  `auth log`, `auth status`, `auth explain`, `why host`, and
  `runs log --json` tests.
- 2026-06-15: Focused CLI test split checks passed:
  `python3 -m compileall tests`, `uvx --from ruff==0.15.17 ruff check` on the
  split CLI test files, and
  `PYTHONPATH=src python3 -m unittest discover -s tests -p 'test_cli*.py'` with
  90 tests.
- 2026-06-15: Focused active-command CLI test split checks passed:
  `python3 -m compileall` on the active-command CLI test files,
  `uvx --from ruff==0.15.17 ruff check` on those files, and
  `PYTHONPATH=src python3 -m unittest discover -s tests -p 'test_cli_active*.py'`
  with 33 tests.
- 2026-06-15: Focused run-history CLI test split checks passed:
  `python3 -m compileall` on the run-history CLI test files,
  `uvx --from ruff==0.15.17 ruff check` on those files, and
  `PYTHONPATH=src python3 -m unittest discover -s tests -p 'test_cli_runs*.py'`
  with 12 tests.
- 2026-06-15: Focused provider-runtime CLI test split checks passed:
  `python3 -m compileall` on the provider-runtime CLI test files,
  `uvx --from ruff==0.15.17 ruff check` on those files, and
  `PYTHONPATH=src python3 -m unittest discover -s tests -p 'test_cli_provider*.py'`
  with 12 tests.
- 2026-06-15: Focused git-metadata extraction checks passed:
  `python3 -m compileall` on `src/runhaven/git_metadata.py`,
  `src/runhaven/run_history.py`, `src/runhaven/cli.py`, and
  `src/runhaven/provider_runtime.py`;
  `uvx --from ruff==0.15.17 ruff check` on those files;
  `PYTHONPATH=src python3 -m unittest discover -s tests -p 'test_cli*.py'`
  with 90 tests; and `uvx --from mypy==2.1.0 mypy src`.
- 2026-06-15: Focused active-repair extraction checks passed:
  `python3 -m compileall` on `src/runhaven/active_commands.py`,
  `src/runhaven/active_repair.py`, and `src/runhaven/cli.py`;
  `uvx --from ruff==0.15.17 ruff check` on those files;
  `PYTHONPATH=src python3 -m unittest discover -s tests -p 'test_cli_active*.py'`
  with 33 tests; and `uvx --from mypy==2.1.0 mypy src`.
- 2026-06-15: Focused NPM pin-policy extraction checks passed:
  `python3 -m compileall scripts/check_pins.py scripts/npm_pin_policy.py`,
  `uvx --from ruff==0.15.17 ruff check --fix` plus
  `uvx --from ruff==0.15.17 ruff format` on those files,
  `python3 scripts/check_pins.py`,
  `PYTHONPATH=src python3 -m unittest tests.test_repo_policy` with 5 tests,
  and `uvx --from mypy==2.1.0 mypy scripts/check_pins.py scripts/npm_pin_policy.py`.
- 2026-06-15: Full verification passed after the NPM pin-policy extraction:
  `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 156 tests,
  `python3 scripts/check_pins.py`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`,
  `uvx --from mypy==2.1.0 mypy scripts/check_pins.py scripts/npm_pin_policy.py`,
  `python3 -m json.tool feature_list.json`, `git diff --check`, Markdown
  local link check, platform wording scan, and
  `PYTHON=<temporary-venv-python> ./init.sh` with compileall, 156 unit tests,
  pin check, ruff, mypy, and build.
- 2026-06-15: Focused auth-profile extraction checks passed:
  `python3 -m compileall src/runhaven/auth_broker.py src/runhaven/auth_profiles.py src/runhaven/diagnostic_commands.py`,
  `uvx --from ruff==0.15.17 ruff check` on those files,
  `uvx --from mypy==2.1.0 mypy` on those files,
  `PYTHONPATH=src python3 -m unittest tests.test_auth_broker tests.test_cli_diagnostics tests.test_cli_provider_codex_broker`
  with 18 tests, and JSON smokes for `runhaven auth status` plus
  `runhaven auth explain codex`.
- 2026-06-15: Full verification passed after the auth-profile extraction:
  `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 156 tests,
  `python3 scripts/check_pins.py`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 -m json.tool feature_list.json`,
  `git diff --check`, Markdown local link check, platform wording scan, and
  `PYTHON=<temporary-venv-python> ./init.sh` with compileall, 156 unit tests,
  pin check, ruff, mypy, and build.
- 2026-06-15: Focused provider-observability extraction checks passed:
  `python3 -m compileall src/runhaven/cli.py src/runhaven/provider_runtime.py src/runhaven/provider_observability.py`,
  `uvx --from ruff==0.15.17 ruff check` on those files,
  `uvx --from mypy==2.1.0 mypy` on those files,
  `PYTHONPATH=src python3 -m unittest discover -s tests -p 'test_cli_provider*.py'`
  with 12 tests, and
  `PYTHONPATH=src python3 -m unittest tests.test_cli_diagnostics tests.test_cli_runs_log`
  with 12 tests.
- 2026-06-15: Full verification passed after the provider-observability
  extraction: `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 156 tests,
  `python3 scripts/check_pins.py`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 -m json.tool feature_list.json`,
  `git diff --check`, Markdown local link check, platform wording scan, and
  `PYTHON=<temporary-venv-python> ./init.sh` with compileall, 156 unit tests,
  pin check, ruff, mypy, and build.
- 2026-06-15: Focused CLI parser extraction checks passed:
  `python3 -m compileall src/runhaven/cli.py src/runhaven/cli_parser.py`,
  `uvx --from ruff==0.15.17 ruff check src/runhaven/cli.py src/runhaven/cli_parser.py`,
  `uvx --from mypy==2.1.0 mypy src/runhaven/cli.py src/runhaven/cli_parser.py`,
  and `PYTHONPATH=src python3 -m unittest discover -s tests -p 'test_cli*.py'`
  with 90 tests.
- 2026-06-15: Full verification passed after the CLI parser extraction:
  `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 156 tests,
  `python3 scripts/check_pins.py`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 -m json.tool feature_list.json`,
  `git diff --check`, Markdown local link check, platform wording scan with
  the expected existing macOS-only acceptance-criteria line, and
  `PYTHON=<temporary-venv-python> ./init.sh` with compileall, 156 unit tests,
  pin check, ruff, mypy, and build.
- 2026-06-15: Focused active-repair test cleanup checks passed:
  `python3 -m compileall tests/test_cli_active_repair.py`,
  `uvx --from ruff==0.15.17 ruff check tests/test_cli_active_repair.py`, and
  `PYTHONPATH=src python3 -m unittest discover -s tests -p 'test_cli_active_repair.py'`
  with 11 tests.
- 2026-06-15: Full verification passed after the active-repair test cleanup:
  `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 156 tests,
  `python3 scripts/check_pins.py`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 -m json.tool feature_list.json`,
  `git diff --check`, Markdown local link check, platform wording scan with
  only the expected existing macOS-only evidence lines, and
  `PYTHON=<temporary-venv-python> ./init.sh` with compileall, 156 unit tests,
  pin check, ruff, mypy, and build.
- 2026-06-15: Focused workspace-scope selection checks passed:
  `python3 -m compileall` on touched source and tests,
  `PYTHONPATH=src python3 -m unittest discover -s tests -p 'test_plans.py'`
  with 35 tests,
  `PYTHONPATH=src python3 -m unittest discover -s tests -p 'test_cli.py'`
  with 14 tests,
  `PYTHONPATH=src python3 -m unittest discover -s tests -p 'test_cli_standard_run.py'`
  with 6 tests,
  `uvx --from ruff==0.15.17 ruff check` on touched Python files, and
  `uvx --from mypy==2.1.0 mypy` on touched source files.
- 2026-06-15: Full verification passed after workspace-scope selection:
  `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 160 tests,
  `python3 scripts/check_pins.py`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 -m json.tool feature_list.json`,
  `git diff --check`, Markdown local link check, platform wording scan, manual
  `runhaven plan` smokes for default current scope and explicit git-root scope,
  and `PYTHON=<temporary-venv-python> ./init.sh` with compileall, 160 unit
  tests, pin check, ruff, mypy, and build.
- 2026-06-15: Focused worktree run isolation checks passed:
  `python3 -m compileall src/runhaven tests/test_cli_standard_run.py`,
  `PYTHONPATH=src python3 -m unittest discover -s tests -p 'test_cli_standard_run.py'`
  with 9 tests, `uvx --from ruff==0.15.17 ruff check` on touched Python
  files, and `uvx --from mypy==2.1.0 mypy` on touched source files.
- 2026-06-15: Full verification passed after worktree run isolation:
  `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 163 tests,
  `python3 scripts/check_pins.py`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src` with 24 source files,
  `python3 -m json.tool feature_list.json`, `git diff --check`, Markdown
  local link check, platform wording scan, manual
  `runhaven run shell --worktree --dry-run` smoke, and
  `PYTHON=<temporary-venv-python> ./init.sh` with compileall, 163 unit tests,
  pin check, ruff, mypy, and build.
- 2026-06-15: Full verification passed after the active-repair extraction:
  `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 156 tests,
  `python3 scripts/check_pins.py`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 -m json.tool feature_list.json`,
  `git diff --check`, Markdown local link check, generated artifact cleanup
  scan, platform wording scan, and `PYTHON=<temporary-venv-python> ./init.sh`
  with compileall, 156 unit tests, pin check, ruff, mypy, and build.
- 2026-06-15: Full verification passed after the git-metadata extraction:
  `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 156 tests,
  `python3 scripts/check_pins.py`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 -m json.tool feature_list.json`,
  `git diff --check`, Markdown local link check, generated artifact cleanup
  scan, platform wording scan, and `PYTHON=<temporary-venv-python> ./init.sh`
  with compileall, 156 unit tests, pin check, ruff, mypy, and build.
- 2026-06-15: Full verification passed after the provider-runtime CLI test
  split: `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 156 tests,
  `python3 scripts/check_pins.py`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 -m json.tool feature_list.json`,
  `git diff --check`, Markdown local link check, generated artifact cleanup
  scan, platform wording scan, and `PYTHON=<temporary-venv-python> ./init.sh`
  with compileall, 156 unit tests, pin check, ruff, mypy, and build.
- 2026-06-15: Full verification passed after the run-history CLI test split:
  `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 156 tests,
  `python3 scripts/check_pins.py`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 -m json.tool feature_list.json`,
  `git diff --check`, Markdown local link check, generated artifact cleanup
  scan, platform wording scan, and `PYTHON=<temporary-venv-python> ./init.sh`
  with compileall, 156 unit tests, pin check, ruff, mypy, and build.
- 2026-06-15: Full verification passed after the active-command CLI test split:
  `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 156 tests,
  `python3 scripts/check_pins.py`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 -m json.tool feature_list.json`,
  `git diff --check`, Markdown local link check, generated artifact cleanup
  scan, platform wording scan, and `PYTHON=<temporary-venv-python> ./init.sh`
  with compileall, 156 unit tests, pin check, ruff, mypy, and build.
- 2026-06-15: Full verification passed after the CLI test split:
  `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 156 tests,
  `python3 scripts/check_pins.py`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 -m json.tool feature_list.json`,
  `git diff --check`, Markdown local link check, generated artifact cleanup
  scan, platform wording scan, and `PYTHON=<temporary-venv-python> ./init.sh`
  with compileall, 156 unit tests, pin check, ruff, mypy, and build.
- 2026-06-15: Full verification passed after the diagnostic-command
  extraction: `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 156 tests,
  `python3 scripts/check_pins.py`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 -m json.tool feature_list.json`,
  `git diff --check`, Markdown local link check, generated artifact cleanup
  scan, platform wording scan, and `PYTHON=<temporary-venv-python> ./init.sh`
  with compileall, 156 unit tests, pin check, ruff, mypy, and build.
- 2026-06-15: Full verification passed after the provider-runtime extraction:
  `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 156 tests,
  `python3 scripts/check_pins.py`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 -m json.tool feature_list.json`,
  `git diff --check`, Markdown local link check, generated artifact cleanup
  scan, platform wording scan, and `PYTHON=<temporary-venv-python> ./init.sh`
  with compileall, 156 unit tests, pin check, ruff, mypy, and build.
- 2026-06-15: Focused active-command extraction tests passed:
  `PYTHONPATH=src python3 -m unittest` with 33 selected `runs active`,
  `runs status`, `runs attach`, `runs logs-follow`, `runs stop`, `runs kill`,
  and `runs repair` tests.
- 2026-06-15: Full verification passed after the active-command extraction:
  `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 156 tests,
  `python3 scripts/check_pins.py`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, and
  `PYTHON=<temporary-venv-python> ./init.sh` with compileall, 156 unit tests,
  pin check, ruff, mypy, and build.
- 2026-06-15: Focused run-history extraction tests passed:
  `PYTHONPATH=src python3 -m unittest tests.test_cli.CliTests.test_standard_run_writes_secret_free_run_record tests.test_cli.CliTests.test_provider_run_writes_run_record_with_policy_auth_and_cleanup_summary tests.test_cli.CliTests.test_runs_list_prints_recent_records tests.test_cli.CliTests.test_runs_show_json_is_secret_free tests.test_cli.CliTests.test_runs_show_prints_git_metadata_summary tests.test_cli.CliTests.test_runs_diff_prints_live_committed_git_diff tests.test_cli.CliTests.test_runs_diff_prints_live_dirty_git_diff_with_warning tests.test_cli.CliTests.test_runs_diff_prints_live_untracked_git_diff tests.test_cli.CliTests.test_runs_diff_includes_committed_and_dirty_changes tests.test_cli.CliTests.test_runs_diff_refuses_unavailable_git_metadata tests.test_cli.CliTests.test_runs_diff_refuses_when_recorded_head_is_stale tests.test_cli.CliTests.test_runs_diff_refuses_when_dirty_path_set_changed tests.test_cli.CliTests.test_runs_log_prints_joined_secret_free_run_events tests.test_cli.CliTests.test_runs_log_json_is_secret_free`.
- 2026-06-15: Full verification passed after the run-history extraction:
  `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 156 tests,
  `python3 scripts/check_pins.py`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, and
  `PYTHON=<temporary-venv-python> ./init.sh` with compileall, 156 unit tests,
  pin check, ruff, mypy, and build.
- 2026-06-15: `container --help`, `container exec --help`, and
  `container attach --help` were checked. The pinned local Apple `container`
  CLI exposes `exec`; `attach` reports plugin `container-attach` is not
  installed.
- 2026-06-15: Local `container logs --help` shows
  `container logs [--boot] [--follow] [-n <n>] <container-id>`.
- 2026-06-15: Local `container inspect --help` shows
  `container inspect [--debug] <container-ids> ...`; local
  `container inspect buildkit` confirmed JSON output with raw process
  arguments, environment, and mounts.
- 2026-06-15: Local `container kill --help` shows
  `container kill [--all] [--signal <signal>] [--debug] [<container-ids> ...]`;
  the default signal is `KILL`.
- 2026-06-15: Local
  `container inspect runhaven-nonexistent-repair-smoke` exits 1 with
  `Error: container not found: runhaven-nonexistent-repair-smoke`.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli.CliTests.test_runs_repair_removes_marker_when_container_is_missing tests.test_cli.CliTests.test_runs_repair_refuses_when_container_still_exists tests.test_cli.CliTests.test_runs_repair_leaves_marker_on_unverified_inspect_failure tests.test_cli.CliTests.test_runs_repair_refuses_unowned_container_name`
  first failed because `repair` was not a valid `runs` subcommand, then passed
  after adding fail-closed stale-marker repair.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli.CliTests.test_runs_repair_all_removes_confirmed_stale_markers tests.test_cli.CliTests.test_runs_repair_all_returns_nonzero_when_any_marker_unverified tests.test_cli.CliTests.test_runs_repair_requires_run_id_or_all tests.test_cli.CliTests.test_runs_repair_refuses_run_id_with_all`
  first failed because `repair` still required a positional run id and did not
  accept `--all`, then passed after adding guarded bulk repair.
- 2026-06-15: Focused `runs repair --all` tests, focused combined repair tests,
  full `PYTHONPATH=src python3 -m unittest discover -s tests` with 148 tests,
  `python3 -m compileall src tests scripts`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 scripts/check_pins.py`,
  `python3 -m json.tool feature_list.json`, `git diff --check`, Markdown
  link check, platform scan, and manual `runs repair --all` smoke passed.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 148 unit tests, pin check, ruff, mypy, and build after adding
  `runs repair --all`.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli.CliTests.test_runs_repair_json_reports_removed_marker tests.test_cli.CliTests.test_runs_repair_all_json_reports_mixed_outcomes tests.test_cli.CliTests.test_runs_repair_all_json_reports_empty_summary`
  first failed because `runs repair` did not accept `--json`, then passed
  after adding single-run and bulk repair JSON summaries.
- 2026-06-15: Focused combined repair tests, full
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 151 tests,
  `python3 -m compileall src tests scripts`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 scripts/check_pins.py`,
  `python3 -m json.tool feature_list.json`, `git diff --check`, Markdown
  link check, platform scan, and manual repair JSON smokes passed.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 151 unit tests, pin check, ruff, mypy, and build after adding
  repair JSON output.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli.CliTests.test_setup_prints_remedies_when_prerequisites_fail tests.test_cli.CliTests.test_setup_prints_first_run_commands_when_ready tests.test_cli.CliTests.test_setup_accepts_agent_profile`
  first failed because `setup` was not a valid subcommand, then passed after
  adding the non-mutating guided setup flow.
- 2026-06-15: Focused setup tests, full
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 154 tests,
  `python3 -m compileall src tests scripts`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 scripts/check_pins.py`,
  `python3 -m json.tool feature_list.json`, `git diff --check`, Markdown
  link check, platform scan, and manual `runhaven setup --agent shell` smoke
  passed.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 154 unit tests, pin check, ruff, mypy, and build after adding
  `runhaven setup`.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli.CliTests.test_setup_prints_goal_based_network_guidance`
  first failed because `setup` did not print a network-choice section, then
  passed after adding local-only, provider-only, package install, and
  unrestricted internet guidance.
- 2026-06-15: Focused setup/doctor tests, full
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 155 tests,
  `python3 -m compileall src tests scripts`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 scripts/check_pins.py`,
  `python3 -m json.tool feature_list.json`, `git diff --check`, Markdown
  link check, platform scan, and manual `runhaven setup --agent shell` smoke
  passed after adding setup network guidance.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 155 unit tests, pin check, ruff, mypy, and build after adding
  setup network guidance.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli.CliTests.test_setup_prints_workspace_and_credential_guidance`
  first failed because `setup` did not print a workspace and credential
  section, then passed after adding smallest-project workspace, avoided host
  credential paths, `--ssh`, and reviewed `--env NAME` guidance.
- 2026-06-15: Focused setup/doctor tests, full
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 156 tests,
  `python3 -m compileall src tests scripts`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 scripts/check_pins.py`,
  `python3 -m json.tool feature_list.json`, `git diff --check`, Markdown
  link check, platform scan, and manual `runhaven setup --agent shell` smoke
  passed after adding setup workspace and credential guidance.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 156 unit tests, pin check, ruff, mypy, and build after adding
  setup workspace and credential guidance.
- 2026-06-15: First modularization extraction moved setup guide output,
  active-run marker persistence, cache path helpers, and shared validators out
  of `src/runhaven/cli.py`. `src/runhaven/cli.py` measured 2,440 lines after
  extraction, down from 2,685 before the slice; `tests/test_cli.py` remains
  3,515 lines and is still a major pre-release split target.
- 2026-06-15: Focused setup and active-record CLI tests passed after the first
  modularization extraction:
  `PYTHONPATH=src python3 -m unittest tests.test_cli.CliTests.test_setup_prints_workspace_and_credential_guidance tests.test_cli.CliTests.test_standard_run_writes_and_removes_active_run_marker tests.test_cli.CliTests.test_runs_active_prints_active_run_markers tests.test_cli.CliTests.test_runs_repair_removes_marker_when_container_is_missing`.
- 2026-06-15: Full `PYTHONPATH=src python3 -m unittest discover -s tests`
  with 156 tests, `python3 -m compileall src tests scripts`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, and `python3 scripts/check_pins.py`
  passed after the first modularization extraction.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 156 unit tests, pin check, ruff, mypy, and build after the first
  modularization extraction. The build output included the new
  `active_records.py`, `cache_paths.py`, `setup_guide.py`, and
  `validators.py` modules.
- 2026-06-15: Focused `runs repair` tests, full
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 144 tests,
  `python3 -m compileall src tests scripts`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 scripts/check_pins.py`,
  `python3 -m json.tool feature_list.json`, `git diff --check`, Markdown
  link check, platform scan, and manual `runs repair` smoke passed.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 144 unit tests, pin check, ruff, mypy, and build after adding
  `runs repair`.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli.CliTests.test_standard_run_records_killed_status_when_kill_requested tests.test_cli.CliTests.test_runs_kill_kills_active_run_container tests.test_cli.CliTests.test_runs_kill_rolls_back_marker_when_container_kill_fails tests.test_cli.CliTests.test_runs_kill_refuses_unowned_container_name`
  first failed because `kill` was not a valid `runs` subcommand and
  kill-requested runs recorded as `failed`, then passed after adding guarded
  Apple `container kill` routing and killed run records.
- 2026-06-15: Focused `runs kill` tests, full
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 140 tests,
  `python3 -m compileall src tests scripts`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 scripts/check_pins.py`,
  `python3 -m json.tool feature_list.json`, `git diff --check`, Markdown
  link check, platform scan, and manual `runs kill` smoke passed.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 140 unit tests, pin check, ruff, mypy, and build after adding
  `runs kill`.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli.CliTests.test_runs_status_prints_sanitized_active_container_state tests.test_cli.CliTests.test_runs_status_json_is_sanitized tests.test_cli.CliTests.test_runs_status_refuses_unowned_container_name tests.test_cli.CliTests.test_runs_status_returns_container_inspect_failure`
  first failed because `status` was not a valid `runs` subcommand, then passed
  after adding sanitized Apple `container inspect` status output.
- 2026-06-15: Focused `runs status` tests, full `PYTHONPATH=src python3 -m unittest discover -s tests`
  with 136 tests, `python3 -m compileall src tests scripts`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 scripts/check_pins.py`,
  `python3 -m json.tool feature_list.json`, `git diff --check`, Markdown
  link check, platform scan, and manual `runs status` smoke passed.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 136 unit tests, pin check, ruff, mypy, and build after adding
  `runs status`.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli.CliTests.test_runs_logs_follow_streams_recent_active_container_logs tests.test_cli.CliTests.test_runs_logs_follow_accepts_line_count_override tests.test_cli.CliTests.test_runs_logs_follow_refuses_invalid_line_count tests.test_cli.CliTests.test_runs_logs_follow_refuses_unowned_container_name`
  first failed because `logs-follow` was not a valid `runs` subcommand, then
  passed after adding guarded Apple `container logs --follow` routing.
- 2026-06-15: Focused `runs logs-follow` tests, full `PYTHONPATH=src python3 -m unittest discover -s tests`
  with 132 tests, `python3 -m compileall src tests scripts`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 scripts/check_pins.py`,
  `python3 -m json.tool feature_list.json`, `git diff --check`, Markdown
  link check, platform scan, and manual `runs logs-follow` smoke passed.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 132 unit tests, pin check, ruff, mypy, and build after adding
  `runs logs-follow`.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli.CliTests.test_runs_attach_execs_shell_in_active_container tests.test_cli.CliTests.test_runs_attach_uses_custom_command_without_tty_when_requested tests.test_cli.CliTests.test_runs_attach_refuses_unowned_container_name tests.test_cli.CliTests.test_runs_attach_refuses_root_user_without_override tests.test_cli.CliTests.test_runs_attach_allows_root_user_with_override`
  first failed because `attach` was not a valid `runs` subcommand, then passed
  after adding guarded `container exec` attach.
- 2026-06-15: `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 128 tests,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 scripts/check_pins.py`,
  `python3 -m json.tool feature_list.json`, and `git diff --check` passed
  after adding `runs attach`.
- 2026-06-15: Local Markdown link check, macOS-only platform boundary scan,
  and manual `runs attach` command-construction smoke passed.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 128 unit tests, pin check, ruff, mypy, and build after adding
  `runs attach`.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli.CliTests.test_runs_active_prints_active_run_markers tests.test_cli.CliTests.test_runs_active_json_prints_active_run_markers tests.test_cli.CliTests.test_runs_active_prints_empty_message`
  first failed because `active` was not a valid `runs` subcommand, then passed
  after adding text and JSON active-marker listing.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli` ran 57 tests
  and passed after adding `runs active`.
- 2026-06-15: `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 123 tests,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 scripts/check_pins.py`,
  `python3 -m json.tool feature_list.json`, and `git diff --check` passed
  after adding `runs active`.
- 2026-06-15: Local Markdown link check, macOS-only platform boundary scan,
  and manual `runs active` text/JSON smoke passed.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 123 unit tests, pin check, ruff, mypy, and build after adding
  `runs active`.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_plans.RunPlanTests.test_default_plan_mounts_only_workspace_and_agent_home tests.test_cli.CliTests.test_standard_run_writes_and_removes_active_run_marker tests.test_cli.CliTests.test_standard_run_records_stopped_status_when_stop_requested tests.test_cli.CliTests.test_runs_stop_stops_active_run_container tests.test_cli.CliTests.test_runs_stop_refuses_missing_active_run tests.test_cli.CliTests.test_runs_stop_refuses_unowned_container_name`
  first failed because plans had no named container, runs wrote no active
  marker, and `runs stop` was not a valid subcommand. The focused set passed
  after adding named containers, active markers, stopped status, and guarded
  Apple `container stop` routing.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli tests.test_plans`
  ran 86 tests and passed after adding `runs stop`.
- 2026-06-15: `uvx --from ruff==0.15.17 ruff check src/runhaven/cli.py src/runhaven/plans.py tests/test_cli.py tests/test_plans.py`
  and `uvx --from mypy==2.1.0 mypy src/runhaven/cli.py src/runhaven/plans.py`
  passed after adding `runs stop`.
- 2026-06-15: `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 120 tests,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, and `python3 scripts/check_pins.py`
  passed after adding `runs stop`.
- 2026-06-15: Manual `runs stop` smoke passed for a temporary active-run marker
  with mocked Apple `container stop`.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 120 unit tests, pin check, ruff, mypy, and build after adding
  `runs stop`.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli.CliTests.test_runs_diff_prints_live_committed_git_diff tests.test_cli.CliTests.test_runs_diff_prints_live_dirty_git_diff_with_warning tests.test_cli.CliTests.test_runs_diff_prints_live_untracked_git_diff tests.test_cli.CliTests.test_runs_diff_includes_committed_and_dirty_changes tests.test_cli.CliTests.test_runs_diff_refuses_unavailable_git_metadata tests.test_cli.CliTests.test_runs_diff_refuses_when_recorded_head_is_stale tests.test_cli.CliTests.test_runs_diff_refuses_when_dirty_path_set_changed`
  first failed because `runs diff` was not a valid subcommand. The mixed
  committed-and-dirty regression then failed until dirty diff assembly included
  committed changes. The focused set passed after adding live git diff,
  untracked-file diff, and refusal checks.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli` ran 49 tests
  and passed after adding `runs diff`.
- 2026-06-15: `uvx --from ruff==0.15.17 ruff check src/runhaven/cli.py tests/test_cli.py`
  and `uvx --from mypy==2.1.0 mypy src/runhaven/cli.py` passed after adding
  `runs diff`.
- 2026-06-15: `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 115 tests,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 scripts/check_pins.py`,
  `python3 -m json.tool feature_list.json`, and `git diff --check` passed
  after adding `runs diff`.
- 2026-06-15: Manual `runs diff` smoke passed for a committed git metadata run
  record.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 115 unit tests, pin check, ruff, mypy, and build after adding
  `runs diff`.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli.CliTests.test_standard_run_writes_secret_free_run_record tests.test_cli.CliTests.test_standard_run_records_git_change_metadata_without_file_contents tests.test_cli.CliTests.test_runs_show_prints_git_metadata_summary`
  first failed because run records had no git object and text output had no git
  summary, then passed after adding git metadata capture and display.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli` ran 42 tests
  and passed after adding git run metadata.
- 2026-06-15: `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 108 tests,
  `python3 scripts/check_pins.py`, `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 -m json.tool feature_list.json`,
  and `git diff --check` passed after adding git run metadata.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 108 unit tests, pin check, ruff, mypy, and build after adding git
  run metadata.
- 2026-06-15: Manual `runs show` and `runs log --json` reader smoke passed for
  a git metadata run record.
- 2026-06-15: Local Markdown link check, platform-boundary text scan, and
  generated-artifact cleanup scan passed after the git metadata docs update.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli.CliTests.test_runs_log_prints_joined_secret_free_run_events tests.test_cli.CliTests.test_runs_log_json_is_secret_free`
  first failed because `runs log` was not a valid subcommand, then passed after
  joining run, provider policy, and auth broker entries by run id.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli` ran 40 tests
  and passed after adding `runhaven runs log`.
- 2026-06-15: `uvx --from ruff==0.15.17 ruff check src/runhaven/cli.py tests/test_cli.py`
  and `uvx --from mypy==2.1.0 mypy src/runhaven/cli.py` passed after adding
  `runhaven runs log`.
- 2026-06-15: Manual reader smoke passed for
  `RUNHAVEN_CACHE_HOME=<temporary-dir> PYTHONPATH=src python3 -m runhaven runs log manual-run`
  and `RUNHAVEN_CACHE_HOME=<temporary-dir> PYTHONPATH=src python3 -m runhaven runs log manual-run --json`.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 106 unit tests, pin check, ruff, mypy, and build after adding
  `runhaven runs log`.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli.CliTests.test_standard_run_writes_secret_free_run_record tests.test_cli.CliTests.test_provider_run_writes_run_record_with_policy_auth_and_cleanup_summary tests.test_cli.CliTests.test_runs_list_prints_recent_records tests.test_cli.CliTests.test_runs_show_json_is_secret_free`
  first failed because `runs` and `runs.jsonl` did not exist, then passed after
  adding the run ledger.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli` ran 38 tests
  and passed after adding `runhaven runs list/show`.
- 2026-06-15: `uvx --from ruff==0.15.17 ruff check src/runhaven/cli.py tests/test_cli.py`
  and `uvx --from mypy==2.1.0 mypy src/runhaven/cli.py` passed after adding
  the run ledger.
- 2026-06-15: Manual reader smoke passed for
  `RUNHAVEN_CACHE_HOME=<temporary-dir> PYTHONPATH=src python3 -m runhaven runs list --limit 1`
  and `RUNHAVEN_CACHE_HOME=<temporary-dir> PYTHONPATH=src python3 -m runhaven runs show manual-run`.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 104 unit tests, pin check, ruff, mypy, and build after adding
  run observability.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_auth_broker tests.test_cli.CliTests.test_provider_run_with_codex_api_key_broker_writes_secret_free_auth_log tests.test_cli.CliTests.test_provider_run_with_codex_api_key_broker_logs_no_requests tests.test_cli.CliTests.test_auth_log_prints_recent_broker_entries tests.test_cli.CliTests.test_auth_log_json_is_secret_free tests.test_codex_broker_smoke`
  ran 10 focused broker observability and smoke harness tests and passed.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest discover -s tests` ran 100
  tests and passed after adding broker observability and the optional smoke
  harness.
- 2026-06-15: `python3 -m compileall src tests scripts`,
  `uvx --from ruff==0.15.17 ruff check .`, and
  `uvx --from mypy==2.1.0 mypy src scripts/codex_broker_smoke.py` passed.
- 2026-06-15: `python3 scripts/check_pins.py`,
  `python3 -m json.tool feature_list.json`, and `git diff --check` passed.
- 2026-06-15: Manual smokes passed:
  `PYTHONPATH=src python3 -m runhaven auth log --limit 1` and
  `PYTHONPATH=src python3 scripts/codex_broker_smoke.py`. The Codex broker
  smoke skipped because `RUNHAVEN_CODEX_BROKER_SMOKE_API_KEY` was not set.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 100 unit tests, pin check, ruff, mypy, and build after adding
  broker observability and the optional smoke harness.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest discover -s tests` ran 93
  tests and passed after adding the Codex API-key broker prototype.
- 2026-06-15: `python3 -m compileall src tests scripts`,
  `python3 -m json.tool feature_list.json`, and `git diff --check` passed
  after adding the Codex API-key broker prototype.
- 2026-06-15: `uvx --from ruff==0.15.17 ruff check src/runhaven/auth_broker.py src/runhaven/cli.py src/runhaven/plans.py tests/test_auth_broker.py tests/test_cli.py tests/test_plans.py`
  passed after adding the Codex API-key broker prototype.
- 2026-06-15: `uvx --from mypy==2.1.0 mypy src/runhaven/auth_broker.py src/runhaven/cli.py src/runhaven/plans.py`
  passed after adding the Codex API-key broker prototype.
- 2026-06-15: Manual CLI smokes passed:
  `PYTHONPATH=src python3 -m runhaven plan codex --workspace . --network provider --codex-api-key-broker-env OPENAI_API_KEY --tty never`,
  `env -u OPENAI_API_KEY PYTHONPATH=src python3 -m runhaven run codex --workspace . --network provider --codex-api-key-broker-env OPENAI_API_KEY --tty never --dry-run`,
  `env -u OPENAI_API_KEY PYTHONPATH=src python3 -m runhaven run codex --workspace . --network provider --codex-api-key-broker-env OPENAI_API_KEY --tty never`,
  and `PYTHONPATH=src python3 -m runhaven auth explain codex`. The real run
  without `OPENAI_API_KEY` exited 2 before container startup.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 93 unit tests, pin check, ruff, mypy, and build after adding the
  Codex API-key broker prototype.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_plans tests.test_egress`
  ran 39 focused planner and egress tests and passed after adding
  empty-allowlist regression coverage.
- 2026-06-15: `uvx --from ruff==0.15.17 ruff check tests/test_plans.py tests/test_egress.py`
  passed after adding empty-allowlist regression coverage.
- 2026-06-15: `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 84 tests,
  `python3 scripts/check_pins.py`, `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, and
  `python3 -m json.tool feature_list.json` passed after empty-allowlist
  regression coverage.
- 2026-06-15: `git diff --check`, local Markdown link check, and
  platform-boundary text scan passed after empty-allowlist regression coverage.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 84 unit tests, pin check, ruff, mypy, and build after
  empty-allowlist regression coverage.
- 2026-06-15: Generated build and cache artifacts were removed; cleanup scan
  found no `build`, `dist`, `src/runhaven.egg-info`, Python cache, ruff cache,
  or mypy cache directories after empty-allowlist regression coverage.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli.CliTests.test_auth_status_does_not_print_secret_values tests.test_cli.CliTests.test_auth_explain_prints_profile_boundary tests.test_cli.CliTests.test_auth_explain_json_is_static_and_secret_free`
  ran 3 focused auth CLI tests and passed.
- 2026-06-15: `PYTHONPATH=src python3 -m runhaven auth status` and
  `PYTHONPATH=src python3 -m runhaven auth explain codex` passed manual CLI
  smoke checks.
- 2026-06-15: `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src python3 -m unittest discover -s tests` with 79 tests,
  `python3 scripts/check_pins.py`, `uvx --from ruff==0.15.17 ruff check .`,
  and `uvx --from mypy==2.1.0 mypy src` passed.
- 2026-06-15: `python3 -m json.tool feature_list.json`,
  `git diff --check`, local Markdown link check, and platform-boundary text
  scan passed.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 79 unit tests, pin check, ruff, mypy, and build.
- 2026-06-15: Generated build and cache artifacts were removed; cleanup scan
  found no `build`, `dist`, `src/runhaven.egg-info`, Python cache, ruff cache,
  or mypy cache directories.
- 2026-06-15: `git diff --check` passed after recording the source-mined
  recommendations.
- 2026-06-15: `python3 -m json.tool feature_list.json` passed after recording
  the source-mined recommendations.
- 2026-06-15: `python3 scripts/check_pins.py` passed after recording the
  source-mined recommendations.
- 2026-06-15: local Markdown link check passed for
  `docs/harness/source-mined-ideas.md`, `docs/harness/research-inbox.md`,
  `docs/ROADMAP.md`, `progress.md`, and `session-handoff.md`.
- 2026-06-15: `git diff --check` passed after recording the external open
  source research pass.
- 2026-06-15: `python3 -m json.tool feature_list.json` passed after recording
  the external open source research pass.
- 2026-06-15: `python3 scripts/check_pins.py` passed after recording the
  external open source research pass.
- 2026-06-15: local Markdown link check passed for
  `docs/harness/source-mined-ideas.md`,
  `docs/harness/external-project-ideas.md`,
  `docs/harness/research-inbox.md`, `docs/ROADMAP.md`, `progress.md`, and
  `session-handoff.md`.
- 2026-06-15: macOS-only boundary text check confirmed the edited docs still
  exclude Windows and Linux runtime support.
- 2026-06-15: `git diff --check` passed after recording the UX research pass.
- 2026-06-15: `python3 -m json.tool feature_list.json` passed after recording
  the UX research pass.
- 2026-06-15: `python3 scripts/check_pins.py` passed after recording the UX
  research pass.
- 2026-06-15: local Markdown link check passed for
  `docs/harness/source-mined-ideas.md`,
  `docs/harness/external-project-ideas.md`,
  `docs/harness/ux-research-ideas.md`,
  `docs/harness/research-inbox.md`, `docs/ROADMAP.md`, `progress.md`, and
  `session-handoff.md`.
- 2026-06-15: platform boundary text check confirmed the edited docs still
  state macOS 26+ only, Apple `container` only, and no Windows/Linux runtime
  support.
- 2026-06-15: `PYTHONPATH=src python3.14 -m unittest tests.test_egress.AllowlistProxyTests.test_proxy_rejects_allowed_host_resolving_to_private_address tests.test_egress.AllowlistProxyTests.test_proxy_records_allowed_and_denied_policy_decisions`
  ran 2 focused proxy tests and passed after provider DNS unsafe-address
  rejection and policy aggregation.
- 2026-06-15: `PYTHONPATH=src python3.14 -m unittest tests.test_cli.CliTests.test_provider_run_writes_policy_log tests.test_cli.CliTests.test_egress_log_prints_recent_policy_entries tests.test_cli.CliTests.test_why_host_explains_ip_literal_rejection tests.test_cli.CliTests.test_why_host_explains_profile_allowlist_match`
  ran 4 focused CLI tests and passed after provider policy logs and
  `runhaven why host ...`.
- 2026-06-15: `PYTHONPATH=src python3.14 -m unittest tests.test_egress tests.test_cli tests.test_plans`
  ran 56 focused integration tests and passed after provider diagnostics.
- 2026-06-15: `PYTHONPATH=src python3.14 -m unittest discover -s tests`
  ran 68 tests and passed after provider diagnostics.
- 2026-06-15: `python3.14 -m compileall src tests scripts` passed after
  provider diagnostics.
- 2026-06-15: `ruff check src/runhaven/egress.py src/runhaven/cli.py src/runhaven/plans.py tests/test_egress.py tests/test_cli.py`
  passed after provider diagnostics.
- 2026-06-15: `uvx --from mypy==2.1.0 mypy src` passed after provider
  diagnostics.
- 2026-06-15: `PYTHONPATH=src python3.14 -m runhaven why host api.openai.com --agent codex`
  and `PYTHONPATH=src python3.14 -m runhaven why host 1.1.1.1` passed manual
  CLI diagnostic checks.
- 2026-06-15: `RUNHAVEN_CACHE_HOME=<temporary-dir> PYTHONPATH=src python3.14 -m runhaven egress log --limit 1`
  passed a manual policy-log display check.
- 2026-06-15: `python3.14 -m json.tool feature_list.json`,
  `python3.14 scripts/check_pins.py`, `git diff --check`, and local Markdown
  link checks passed after documenting provider diagnostics.
- 2026-06-15: platform boundary scan confirmed live repo instructions and docs
  preserve macOS 26+ only runtime and contributor verification.
- 2026-06-15: final `ruff check .`, `uvx --from mypy==2.1.0 mypy src`, and
  `PYTHONPATH=src python3.14 -m unittest discover -s tests` passed after the
  provider diagnostics implementation.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 68 unit tests, pin check, ruff, mypy, and build after provider
  diagnostics.
- 2026-06-14: `PYTHONPATH=src python3.14 -m unittest discover -s tests`
  ran 34 tests and passed.
- 2026-06-14: `PYTHONPATH=src python3.13 -m unittest discover -s tests`
  ran 34 tests and passed.
- 2026-06-14: `python3.14 -m compileall src tests scripts`
  passed.
- 2026-06-14: `python3.14 scripts/check_pins.py`
  passed.
- 2026-06-14: `python -m ruff check .` in a temporary hardening venv
  passed.
- 2026-06-14: `python -m mypy src scripts` in a temporary hardening venv
  passed.
- 2026-06-14: `python -m build` in a temporary hardening venv
  passed.
- 2026-06-14: `PYTHON=<temporary-venv-python> ./init.sh`
  passed.
- 2026-06-14: `PYTHONPATH=../HarnessForge/src python3.14 -m harnessforge audit --target . --min-score 85`
  reported 100/100.
- 2026-06-14: `PYTHONPATH=src python3.14 -m runhaven plan shell --tty always -- /bin/true`
  passed and emitted a run command with `--interactive --tty`.
- 2026-06-14: `PYTHONPATH=src python3.14 -m runhaven doctor`
  passed on macOS 26.5.1 arm64 with Apple `container` 1.0.0.
- 2026-06-14: `PYTHONPATH=src python3.14 -m runhaven state list`
  passed and found no RunHaven state volumes.
- 2026-06-14: `PYTHONPATH=src python3.14 -c 'from runhaven.cli import ensure_internal_network; ensure_internal_network("runhaven-smoke-20260614-hardening-internal")'`
  passed, and `container network delete runhaven-smoke-20260614-hardening-internal`
  removed the temporary network.
- 2026-06-14: non-macOS verification entrypoint removed after clarifying
  macOS-only support.
- 2026-06-14: `magick identify docs/assets/logo.png` reported PNG 512x512.
- 2026-06-14: no-ignore old-name text scan across working tree files outside
  `.git` returned no matches.
- 2026-06-14: old-name filename scan across working tree files outside `.git`
  returned no matches.
- 2026-06-14: `PYTHONPATH=src python3 -m unittest discover -s tests` passed.
- 2026-06-14: `python3 scripts/check_pins.py` passed.
- 2026-06-14: temporary external venv installed pinned dev requirements; ruff,
  mypy, build, wheel install, and `runhaven agents` passed.
- 2026-06-14: `PYTHONPATH=src python3.14 -m unittest discover -s tests`
  ran 39 tests and passed after the follow-up hardening pass.
- 2026-06-14: `python3.14 scripts/check_pins.py` passed after dynamic image
  template discovery was added.
- 2026-06-14: `PYTHON=<temporary-venv-python> ./init.sh` passed after the
  follow-up hardening pass.
- 2026-06-14: `PYTHONPATH=src python3.13 -m unittest discover -s tests`
  ran 39 tests and passed after the follow-up hardening pass.
- 2026-06-14: `PYTHONPATH=../HarnessForge/src python3.14 -m harnessforge audit --target . --min-score 85`
  reported 100/100 after the follow-up hardening pass.
- 2026-06-14: cleanup pass removed stale local paths, stale local-venv
  evidence, and old HarnessForge predecessor references from tracked docs.
- 2026-06-14: `PYTHONPATH=../HarnessForge/src python3.14 -m harnessforge audit --target . --min-score 85`
  reported 100/100 after the cleanup pass.
- 2026-06-14: `python3.14 scripts/check_pins.py`, `git diff --check`, and
  `python3 -m json.tool feature_list.json` passed after the cleanup pass.
- 2026-06-14: sandboxed Antigravity read-only audit identified additional
  concrete hardening, pin-ledger, and CLI UX findings.
- 2026-06-14: `PYTHON=<temporary-venv-python> ./init.sh` passed after the
  second follow-up hardening pass; the unit suite ran 47 tests.
- 2026-06-14: `PYTHONPATH=src python3.13 -m unittest discover -s tests`
  ran 47 tests and passed after the second follow-up hardening pass.
- 2026-06-14: `PYTHONPATH=src python3.14 -m runhaven run --help`,
  `PYTHONPATH=src python3.14 -m runhaven plan shell --network internal --tty never -- /bin/true`,
  and `PYTHONPATH=src python3.14 -m runhaven doctor` passed after the second
  follow-up hardening pass.
- 2026-06-14: `PYTHONPATH=../HarnessForge/src python3.14 -m harnessforge audit --target . --min-score 85`
  reported 100/100 after the second follow-up hardening pass.
- 2026-06-14: `python3 -m json.tool feature_list.json`, `git diff --check`,
  generated-artifact checks, and stale-reference scans passed after the second
  follow-up hardening pass.
- 2026-06-14: rendered Apple DocC networking docs with Playwright and checked
  generated DocC JSON endpoints for `ContainerNetworkService`.
- 2026-06-14: complete user-supplied DocC snapshot review covered 1,022
  rendered Markdown pages plus raw DocC JSON with zero fetch failures and no
  exact hits for egress or allowlist control terms.
- 2026-06-14: `PYTHONPATH=src python3.14 -m unittest tests.test_plans.RunPlanTests.test_provider_network_mode_fails_closed_until_enforced tests.test_cli.CliTests.test_provider_network_mode_fails_closed_with_clear_message tests.test_cli.CliTests.test_plan_prints_dry_run_command`
  ran 3 focused tests and passed.
- 2026-06-14: `PYTHONPATH=src python3.14 -m runhaven plan shell --network provider`
  exited 2 with the fail-closed provider egress message during the
  reserved-mode stage.
- 2026-06-14: `PYTHON=<temporary-venv-python> ./init.sh`
  passed after the provider egress preparation pass; the unit suite ran 49
  tests.
- 2026-06-14: `PYTHONPATH=src python3.13 -m unittest discover -s tests`
  ran 49 tests and passed after the provider egress preparation pass.
- 2026-06-14: `PYTHONPATH=src python3.14 -m runhaven doctor` passed on
  macOS 26.5.1 arm64 with Apple `container` 1.0.0.
- 2026-06-14: `PYTHONPATH=../HarnessForge/src python3.14 -m harnessforge audit --target . --min-score 85`
  reported 100/100 after the provider egress preparation pass.
- 2026-06-14: `git diff --check` and
  `python3 -m json.tool feature_list.json` passed after the provider egress
  preparation pass.
- 2026-06-14: `python3 -m json.tool feature_list.json`,
  `python3 scripts/check_pins.py`, `git diff --check`, local absolute-path
  leak scan, and
  `PYTHONPATH=<temporary-HarnessForge-copy>/src python3.14 -m harnessforge audit --target . --min-score 85`
  passed after the complete DocC snapshot evidence update.
- 2026-06-14: `PYTHONPATH=src python3.14 -m unittest tests.test_egress`
  ran 7 tests and passed after adding the allowlist proxy.
- 2026-06-14: `PYTHON=<temporary-venv-python> ./init.sh` passed after the
  provider egress proxy smoke pass; the unit suite ran 56 tests.
- 2026-06-14: `PYTHONPATH=src python3.13 -m unittest discover -s tests`
  ran 56 tests and passed after the provider egress proxy smoke pass.
- 2026-06-14: `PYTHONPATH=src python3.14 scripts/provider_egress_smoke.py --timeout 8`
  passed with allowed proxied HTTPS and denied proxied host, proxied IP
  literal, direct DNS, and direct IP paths.
- 2026-06-14: `PYTHONPATH=src python3.14 scripts/provider_egress_smoke.py --timeout 8 --allowed-host api.openai.com --allowed-url https://api.openai.com/ --denied-host example.com`
  passed with the same allowed and denied path checks.
- 2026-06-14: `python3.14 -m json.tool feature_list.json` and
  `git diff --check` passed after the provider egress proxy smoke pass.
- 2026-06-14: HarnessForge audit was intentionally skipped for this pass by
  user instruction because the sibling HarnessForge repo is being worked on.
- 2026-06-14: `PYTHONPATH=src python3.14 -m unittest tests.test_cli`,
  `python -m ruff check src/runhaven/cli.py tests/test_cli.py`,
  `python -m mypy src`, and `git diff --check` passed after the provider
  wording cleanup.
- 2026-06-14: `PYTHONPATH=src python3.14 -m unittest tests.test_plans tests.test_cli tests.test_egress`
  ran 47 tests and passed after provider lifecycle integration.
- 2026-06-14: `python -m ruff check src tests scripts` and
  `python -m mypy src scripts` passed after provider lifecycle integration.
- 2026-06-14: `PYTHON=<temporary-venv-python> ./init.sh` passed after
  provider lifecycle integration, including compileall, 59 unit tests,
  pin checks, ruff, mypy, and build.
- 2026-06-14: `PYTHONPATH=src python3.13 -m unittest discover -s tests`
  ran 59 tests and passed after provider lifecycle integration.
- 2026-06-14: live `runhaven run shell --network provider --provider-host example.com`
  smoke passed with allowed proxied HTTPS and denied proxied host, proxied IP
  literal, direct DNS, and direct IP paths; follow-up checks found no leftover
  provider network or test state volume.
- 2026-06-14: reviewed local Apple `container-machine.md` and
  `container-system-config.md` docs from the sibling Apple container checkout;
  they did not change the provider proxy design.
- 2026-06-14: `PYTHONPATH=src python3.14 -m unittest tests.test_plans.RunPlanTests.test_provider_network_rejects_single_label_allowed_hosts tests.test_plans tests.test_cli`,
  `python -m ruff check src/runhaven/cli.py src/runhaven/plans.py tests/test_plans.py tests/test_cli.py`,
  and `python -m mypy src` passed after rejecting single-label provider hosts.
- 2026-06-14: `PYTHON=<temporary-venv-python> ./init.sh` and
  `PYTHONPATH=src python3.13 -m unittest discover -s tests` each ran 60 tests
  and passed after the provider-host guard cleanup.
- 2026-06-14: directly reviewed user-supplied supplemental Apple `container`
  sources and ran an Antigravity research pass over the same source list.
- 2026-06-14: `PYTHONPATH=src python3.14 -m unittest tests.test_egress tests.test_cli tests.test_plans`
  ran 51 tests and passed after adding provider blocked-host diagnostics.
- 2026-06-14: `python -m ruff check src/runhaven/cli.py src/runhaven/egress.py tests/test_cli.py tests/test_egress.py`
  and `python -m mypy src` passed after adding provider blocked-host
  diagnostics.
- 2026-06-14: live `runhaven run shell --network provider --provider-host example.com`
  diagnostic smoke reported denied `iana.org:443`; follow-up cleanup removed
  the test state volume and found no leftover provider network.
- 2026-06-14: `PYTHON=<temporary-venv-python> ./init.sh` and
  `PYTHONPATH=src python3.13 -m unittest discover -s tests` each ran 63 tests
  and passed after provider blocked-host diagnostics.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_cli tests.test_plans`
  ran 49 tests and passed after provider endpoint matrix integration.
- 2026-06-15: `uvx --from ruff==0.15.17 ruff check src/runhaven/provider_endpoints.py src/runhaven/profiles.py src/runhaven/cli.py tests/test_cli.py tests/test_plans.py`
  passed after provider endpoint matrix integration.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest discover -s tests` ran 71
  tests and passed after provider endpoint matrix integration.
- 2026-06-15: `python3 -m compileall src tests scripts`,
  `python3 -m json.tool feature_list.json`, and `git diff --check` passed.
- 2026-06-15: `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, and `python3 scripts/check_pins.py`
  passed.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 71 unit tests, pin check, ruff, mypy, and build.
- 2026-06-15: `python3 -m json.tool feature_list.json`, `git diff --check`,
  `python3 scripts/check_pins.py`, focused CLI/plan tests, and targeted local
  Markdown link checks passed after final harness-state updates.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest tests.test_provider_egress_smoke tests.test_cli`
  ran 30 focused tests and passed after grouped blocked-host review and
  provider-profile smoke support.
- 2026-06-15: `PYTHONPATH=src python3 -m unittest discover -s tests` ran 76
  tests and passed.
- 2026-06-15: `python3 -m compileall src tests scripts`,
  `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src scripts/provider_egress_smoke.py`,
  `python3 scripts/check_pins.py`, `python3 -m json.tool feature_list.json`,
  and `git diff --check` passed.
- 2026-06-15: `container --version` and `container system status` confirmed
  Apple `container` 1.0.0 was running before the live smoke.
- 2026-06-15: `PYTHONPATH=src python3 scripts/provider_egress_smoke.py --agent codex --timeout 8 --denied-host example.com`
  passed. Proxied HTTPS reached `api.openai.com` and `chatgpt.com`; denied
  host and proxied IP literal were blocked; direct DNS and direct IP paths were
  blocked.
- 2026-06-15: cleanup scan found no leftover `runhaven-egress-smoke` or
  provider network after the live smoke.
- 2026-06-15: `PYTHON=<temporary-venv-python> ./init.sh` passed with
  compileall, 76 unit tests, pin check, ruff, mypy, and build after grouped
  blocked-host review and provider-profile smoke support.
- 2026-06-15: README documentation split checks passed:
  `git diff --check`, `python3 scripts/check_pins.py`,
  `python3 -m json.tool feature_list.json`, local Markdown link check across
  41 Markdown files, and platform wording scan.
- 2026-06-15: Worktree lifecycle red/green focused tests passed:
  `PYTHONPATH=src:tests python3 -m unittest tests.test_cli_worktree_lifecycle`
  covered `runs keep`, dirty and committed `runs merge`, stale-source merge
  refusal, explicit `runs discard`, and non-worktree refusal. Adjacent
  run-history and worktree tests passed with
  `PYTHONPATH=src:tests python3 -m unittest tests.test_cli_worktree_lifecycle tests.test_cli_standard_run tests.test_cli_runs_diff tests.test_cli_runs_list_show`.
- 2026-06-15: Full worktree lifecycle verification passed:
  `python3 -m compileall src tests scripts`,
  `PYTHONPATH=src:tests python3 -m unittest discover -s tests` with 169
  tests, `uvx --from ruff==0.15.17 ruff check .`,
  `uvx --from mypy==2.1.0 mypy src`, `python3 scripts/check_pins.py`,
  `python3 -m json.tool feature_list.json`, local Markdown link check across
  41 Markdown files, platform wording scan, `PYTHONPATH=src python3 -m runhaven runs --help`,
  `git diff --check`, and `PYTHON=<temporary-venv-python> ./init.sh`.
