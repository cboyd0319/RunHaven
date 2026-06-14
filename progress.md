# Progress

Last Updated: 2026-06-14

## Current Objective

Prove provider egress enforcement with a host allowlist proxy on an internal
Apple `container` network while keeping normal `--network provider` runs
fail-closed.

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
- `runhaven plan` now prints explicit egress status for the selected network.
- `--network provider` is reserved and fails closed until RunHaven integrates
  the verified provider egress proxy lifecycle into normal runs.
- A standard-library CONNECT allowlist proxy now exists in `src/runhaven/egress.py`.
- `scripts/provider_egress_smoke.py` proves the proxy pattern with a temporary
  internal Apple `container` network.
- Live smokes passed for the default public host and for `api.openai.com`:
  allowed proxied HTTPS succeeded, while denied proxied host, proxied IP
  literal, direct DNS, and direct IP paths failed.
- `--network provider` still fails closed for normal runs because the proxy
  lifecycle is not yet integrated into `runhaven run`.
- CLI help and roadmap wording now match that state: provider mode is
  smoke-proven but unavailable for normal runs until lifecycle integration
  lands.

## Recommended Next Step

Integrate the smoke-proven proxy lifecycle into `runhaven run --network
provider`: start the proxy, attach the agent to an internal network, inject
proxy environment variables, clean up the network, and keep clear failure
messages for non-technical users.

## Verification Evidence

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
  exited 2 with the fail-closed provider egress message.
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
