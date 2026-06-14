# Progress

Last Updated: 2026-06-14

## Current Objective

Harden RunHaven command construction, state handling, and macOS-only support
boundaries after a whole-repo audit.

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

## Recommended Next Step

Continue to provider egress allowlisting. Keep the current macOS 26+ only
runtime and verification boundary intact.

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
- 2026-06-14: `PYTHONPATH=../repo-harness-creator/src python3.14 -m harnessforge audit --target . --min-score 85`
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
