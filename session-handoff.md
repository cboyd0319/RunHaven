# Session Handoff

Last Updated: 2026-06-14

## Current Objective

Reserve provider egress mode without claiming allowlisting is enforced, and
record verified Apple `container` networking evidence.

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
- `PYTHONPATH=src python3.14 -m unittest tests.test_plans.RunPlanTests.test_provider_network_mode_fails_closed_until_enforced tests.test_cli.CliTests.test_provider_network_mode_fails_closed_with_clear_message tests.test_cli.CliTests.test_plan_prints_dry_run_command`
  ran 3 focused tests and passed.
- `PYTHONPATH=src python3.14 -m runhaven plan shell --network provider`
  exited 2 with the fail-closed provider egress message.
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
- `--network provider` is now reserved and fails closed until RunHaven has a
  verified provider egress enforcement mechanism.
- `runhaven plan` now prints explicit egress status for the selected network.

## Next Session

1. Read `AGENTS.md`, `feature_list.json`, and `progress.md`.
2. Check `git status --short --branch`.
3. Use `docs/harness/verification-matrix.md` to choose checks for the requested
   change.
4. Continue provider egress design only after choosing an enforcement mechanism
   that can prove both allowed and denied paths with Apple `container` smokes.
5. Ask for explicit approval before renaming the hosted GitHub repository or
   changing other credentialed vendor state.
6. Preserve the macOS 26+ only runtime and contributor-verification contract.
