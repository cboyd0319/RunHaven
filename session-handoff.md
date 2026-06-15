# Session Handoff

Last Updated: 2026-06-14

## Current Objective

Surface provider-mode blocked hosts without weakening the egress boundary.

## Files

- `AGENTS.md`
- `.github/copilot-instructions.md`
- `README.md`
- `SECURITY.md`
- `docs/ARCHITECTURE.md`
- `docs/RESEARCH.md`
- `docs/ROADMAP.md`
- `docs/SECURITY_MODEL.md`
- `docs/USAGE.md`
- `feature_list.json`
- `progress.md`
- `session-handoff.md`
- `init.sh`
- `pins.toml`
- `pyproject.toml`
- `src/runhaven/`
- `scripts/check_pins.py`
- `scripts/provider_egress_smoke.py`
- `tests/`
- `docs/HARNESS_EVALUATION.md`
- `docs/assets/logo.png`
- `docs/harness/`

## Blockers

- None recorded.

## Verification Evidence

- `PYTHONPATH=src python3.14 -m unittest discover -s tests`
  ran 34 tests and passed.
- `PYTHONPATH=src python3.13 -m unittest discover -s tests`
  ran 34 tests and passed.
- `python3.14 -m compileall src tests scripts` passed.
- `python3.14 scripts/check_pins.py` passed.
- `python -m ruff check .` in a temporary hardening venv passed.
- `python -m mypy src scripts` in a temporary hardening venv
  passed.
- `python -m build` in a temporary hardening venv passed.
- `PYTHON=<temporary-venv-python> ./init.sh` passed.
- `PYTHONPATH=../HarnessForge/src python3.14 -m harnessforge audit --target . --min-score 85`
  passed with 100/100.
- `PYTHONPATH=src python3.14 -m runhaven plan shell --tty always -- /bin/true`
  passed and emitted a run command with `--interactive --tty`.
- `PYTHONPATH=src python3.14 -m runhaven doctor` passed
  on macOS 26.5.1 arm64 with Apple `container` 1.0.0.
- `PYTHONPATH=src python3.14 -m runhaven state list`
  passed and found no RunHaven state volumes.
- `PYTHONPATH=src python3.14 -c 'from runhaven.cli import ensure_internal_network; ensure_internal_network("runhaven-smoke-20260614-hardening-internal")'`
  passed, and `container network delete runhaven-smoke-20260614-hardening-internal`
  removed the temporary network.
- `PYTHONPATH=src python3.14 -m unittest discover -s tests`
  ran 39 tests and passed after the follow-up hardening pass.
- `python3.14 scripts/check_pins.py` passed after dynamic image template
  discovery was added.
- `PYTHON=<temporary-venv-python> ./init.sh` passed after the follow-up
  hardening pass.
- `PYTHONPATH=src python3.13 -m unittest discover -s tests` ran 39 tests and
  passed after the follow-up hardening pass.
- `PYTHONPATH=../HarnessForge/src python3.14 -m harnessforge audit --target . --min-score 85`
  reported 100/100 after the follow-up hardening pass.
- Cleanup pass removed stale local paths, stale local-venv evidence, and old
  HarnessForge predecessor references from tracked docs.
- `PYTHONPATH=../HarnessForge/src python3.14 -m harnessforge audit --target . --min-score 85`
  reported 100/100 after the cleanup pass.
- `python3.14 scripts/check_pins.py`, `git diff --check`, and
  `python3 -m json.tool feature_list.json` passed after the cleanup pass.
- Sandboxed Antigravity read-only audit identified additional hardening,
  pin-ledger, and CLI UX findings after the cleanup pass.
- `PYTHON=<temporary-venv-python> ./init.sh` passed after the second follow-up
  hardening pass; the unit suite ran 47 tests.
- `PYTHONPATH=src python3.13 -m unittest discover -s tests` ran 47 tests and
  passed after the second follow-up hardening pass.
- `PYTHONPATH=src python3.14 -m runhaven run --help`,
  `PYTHONPATH=src python3.14 -m runhaven plan shell --network internal --tty never -- /bin/true`,
  and `PYTHONPATH=src python3.14 -m runhaven doctor` passed after the second
  follow-up hardening pass.
- `PYTHONPATH=../HarnessForge/src python3.14 -m harnessforge audit --target . --min-score 85`
  reported 100/100 after the second follow-up hardening pass.
- `python3 -m json.tool feature_list.json`, `git diff --check`,
  generated-artifact checks, and stale-reference scans passed after the second
  follow-up hardening pass.
- Rendered Apple DocC networking docs with Playwright and checked generated
  DocC JSON endpoints for `ContainerNetworkService`.
- Complete user-supplied DocC snapshot review covered 1,022 rendered Markdown
  pages plus raw DocC JSON with zero fetch failures and no exact hits for
  egress or allowlist control terms.
- `PYTHONPATH=src python3.14 -m unittest tests.test_plans.RunPlanTests.test_provider_network_mode_fails_closed_until_enforced tests.test_cli.CliTests.test_provider_network_mode_fails_closed_with_clear_message tests.test_cli.CliTests.test_plan_prints_dry_run_command`
  ran 3 focused tests and passed.
- `PYTHONPATH=src python3.14 -m runhaven plan shell --network provider`
  exited 2 with the fail-closed provider egress message during the
  reserved-mode stage.
- `PYTHON=<temporary-venv-python> ./init.sh` passed after the provider egress
  preparation pass; the unit suite ran 49 tests.
- `PYTHONPATH=src python3.13 -m unittest discover -s tests` ran 49 tests and
  passed after the provider egress preparation pass.
- `PYTHONPATH=src python3.14 -m runhaven doctor` passed on macOS 26.5.1 arm64
  with Apple `container` 1.0.0.
- `PYTHONPATH=../HarnessForge/src python3.14 -m harnessforge audit --target . --min-score 85`
  reported 100/100 after the provider egress preparation pass.
- `git diff --check` and `python3 -m json.tool feature_list.json` passed after
  the provider egress preparation pass.
- `python3 -m json.tool feature_list.json`, `python3 scripts/check_pins.py`,
  `git diff --check`, local absolute-path leak scan, and
  `PYTHONPATH=<temporary-HarnessForge-copy>/src python3.14 -m harnessforge audit --target . --min-score 85`
  passed after the complete DocC snapshot evidence update.
- `PYTHONPATH=src python3.14 -m unittest tests.test_egress` ran 7 tests and
  passed after adding the allowlist proxy.
- `PYTHON=<temporary-venv-python> ./init.sh` passed after the provider egress
  proxy smoke pass; the unit suite ran 56 tests.
- `PYTHONPATH=src python3.13 -m unittest discover -s tests` ran 56 tests and
  passed after the provider egress proxy smoke pass.
- `PYTHONPATH=src python3.14 scripts/provider_egress_smoke.py --timeout 8`
  passed with allowed proxied HTTPS and denied proxied host, proxied IP
  literal, direct DNS, and direct IP paths.
- `PYTHONPATH=src python3.14 scripts/provider_egress_smoke.py --timeout 8 --allowed-host api.openai.com --allowed-url https://api.openai.com/ --denied-host example.com`
  passed with the same allowed and denied path checks.
- `python3.14 -m json.tool feature_list.json` and `git diff --check` passed
  after the provider egress proxy smoke pass.
- HarnessForge audit was intentionally skipped for this pass by user
  instruction because the sibling HarnessForge repo is being worked on.
- `PYTHONPATH=src python3.14 -m unittest tests.test_cli`,
  `python -m ruff check src/runhaven/cli.py tests/test_cli.py`,
  `python -m mypy src`, and `git diff --check` passed after the provider
  wording cleanup.
- `PYTHONPATH=src python3.14 -m unittest tests.test_plans tests.test_cli tests.test_egress`
  ran 47 tests and passed after provider lifecycle integration.
- `python -m ruff check src tests scripts` and
  `python -m mypy src scripts` passed after provider lifecycle integration.
- `PYTHON=<temporary-venv-python> ./init.sh` passed after provider lifecycle
  integration, including compileall, 59 unit tests, pin checks, ruff, mypy, and
  build.
- `PYTHONPATH=src python3.13 -m unittest discover -s tests` ran 59 tests and
  passed after provider lifecycle integration.
- Live `runhaven run shell --network provider --provider-host example.com`
  smoke passed with allowed proxied HTTPS and denied proxied host, proxied IP
  literal, direct DNS, and direct IP paths; follow-up checks found no leftover
  provider network or test state volume.
- Local Apple `container-machine.md` and `container-system-config.md` docs from
  the sibling Apple container checkout were reviewed and did not change the
  provider proxy design.
- `PYTHONPATH=src python3.14 -m unittest tests.test_plans.RunPlanTests.test_provider_network_rejects_single_label_allowed_hosts tests.test_plans tests.test_cli`,
  `python -m ruff check src/runhaven/cli.py src/runhaven/plans.py tests/test_plans.py tests/test_cli.py`,
  and `python -m mypy src` passed after rejecting single-label provider hosts.
- `PYTHON=<temporary-venv-python> ./init.sh` and
  `PYTHONPATH=src python3.13 -m unittest discover -s tests` each ran 60 tests
  and passed after the provider-host guard cleanup.
- Directly reviewed user-supplied supplemental Apple `container` sources and
  ran an Antigravity research pass over the same source list.
- `PYTHONPATH=src python3.14 -m unittest tests.test_egress tests.test_cli tests.test_plans`
  ran 51 tests and passed after adding provider blocked-host diagnostics.
- `python -m ruff check src/runhaven/cli.py src/runhaven/egress.py tests/test_cli.py tests/test_egress.py`
  and `python -m mypy src` passed after adding provider blocked-host
  diagnostics.
- Live `runhaven run shell --network provider --provider-host example.com`
  diagnostic smoke reported denied `iana.org:443`; follow-up cleanup removed
  the test state volume and found no leftover provider network.
- `PYTHON=<temporary-venv-python> ./init.sh` and
  `PYTHONPATH=src python3.13 -m unittest discover -s tests` each ran 63 tests
  and passed after provider blocked-host diagnostics.
- `magick identify docs/assets/logo.png` reported PNG 512x512.
- No-ignore old-name text scan across working tree files outside `.git`
  returned no matches.
- Old-name filename scan across working tree files outside `.git` returned no
  matches.
- Temporary external venv installed pinned dev requirements; ruff, mypy, build,
  wheel install, and `runhaven agents` passed.
- Ignored local `.venv*` directories were removed after verification because
  generated activation scripts and editable-install metadata encoded stale
  checkout paths.
- Apple DocC documentation was rendered with Playwright and cross-checked
  through generated DocC JSON endpoints because the raw HTML page is a
  JavaScript shell.
- The complete user-supplied DocC snapshot was reviewed: 1,022 rendered
  Markdown pages plus raw DocC JSON, zero fetch failures, and no exact hits for
  egress or allowlist control terms.
- A standard-library CONNECT allowlist proxy now exists in
  `src/runhaven/egress.py`.
- `scripts/provider_egress_smoke.py` proves the proxy pattern with a temporary
  internal Apple `container` network.
- `runhaven plan` now prints explicit egress status, provider hosts, and the
  runtime proxy injection note for provider mode.
- `runhaven run --network provider` now creates a managed internal network,
  inspects its gateway and subnet, starts the host-side allowlist proxy,
  injects proxy environment variables, runs the agent, and deletes the managed
  provider network in cleanup.
- Bundled provider host allowlists exist for Claude, Codex, Gemini, and
  Copilot. `--provider-host HOST` adds reviewed fully qualified extra hosts for
  provider mode.
- Provider host additions reject IP literals and single-label hosts, so entries
  like `com` cannot accidentally allow broad suffixes. A listed host permits
  that host and its subdomains.
- Provider mode records blocked CONNECT host/port pairs in memory, caps the
  list, and prints a stderr summary after the run with review guidance for
  fully qualified host additions.
- Supplemental Apple `container` source review is recorded in
  `docs/RESEARCH.md`. It reinforced the current `container run` boundary and
  the decision not to use `container machine` defaults for beginner-safe agent
  runs.

## Next Session

1. Read `AGENTS.md`, `feature_list.json`, and `progress.md`.
2. Check `git status --short --branch`.
3. Use `docs/harness/verification-matrix.md` to choose checks for the requested
   change.
4. Continue by building the provider endpoint matrix for authentication,
   telemetry, and optional provider feature paths.
5. Ask for explicit approval before renaming the hosted GitHub repository or
   changing other credentialed vendor state.
6. Preserve the macOS 26+ only runtime and contributor-verification contract.
