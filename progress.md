# Progress

Last Updated: 2026-06-15

## Current Objective

Add secret-free run observability with `runhaven runs list/show`.

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
- `src/runhaven/auth_broker.py` now records per-profile auth broker metadata
  and implements the first Codex API-key broker prototype.
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
- Run records omit command lines, agent arguments, environment variable names,
  environment values, request bodies, and token values.
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

Run the optional Codex broker smoke with a disposable OpenAI API key when one is
available, then add the next run-observability slice: `runhaven runs log RUN_ID`
or `runhaven runs diff RUN_ID` for deeper post-run review without exposing
secrets.

## Verification Evidence

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
