# Session Handoff

Last Updated: 2026-06-14

## Current Objective

Harden RunHaven after whole-repo audit findings and keep the project clearly
macOS 26+ only.

## Files

- `AGENTS.md`
- `.github/copilot-instructions.md`
- `feature_list.json`
- `progress.md`
- `session-handoff.md`
- `init.sh`
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

## Next Session

1. Read `AGENTS.md`, `feature_list.json`, and `progress.md`.
2. Check `git status --short --branch`.
3. Use `docs/harness/verification-matrix.md` to choose checks for the requested
   change.
4. Ask for explicit approval before renaming the hosted GitHub repository or
   changing other credentialed vendor state.
5. Preserve the macOS 26+ only runtime and contributor-verification contract.
