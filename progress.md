# Progress

Last Updated: 2026-06-14

## Current Objective

Rename the project to RunHaven across the package, CLI, runtime identifiers,
tests, docs, and harness state.

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
- Verification entrypoints now exist: `init.sh` and `init.ps1`.
- Harness docs now exist under `docs/harness/`.
- `AGENTS.md` now includes Startup, Verification, Definition Of Done, state
  file routing, and End of Session instructions.
- `docs/HARNESS_EVALUATION.md` records the before and after audit result.
- `docs/assets/logo.png` is now the tracked project logo and is displayed by
  `README.md`.

## Recommended Next Step

If the hosted GitHub repository should also be renamed, explicitly approve that
credentialed vendor change. After that, the next product feature is provider
egress allowlisting, which is tracked as planned in `feature_list.json`.

## Verification Evidence

- 2026-06-14: `PYTHON=.venv314/bin/python ./init.sh` passed.
- 2026-06-14: `PYTHON=.venv314/bin/python pwsh -NoProfile -File ./init.ps1`
  passed.
- 2026-06-14: `repo_harness_creator audit --target . --min-score 85` reported
  100/100.
- 2026-06-14: `magick identify docs/assets/logo.png` reported PNG 512x512.
- 2026-06-14: latest harness audit reported 100/100 after logo, release
  control, and agent threat-boundary updates.
- 2026-06-14: no-ignore old-name text scan across working tree files outside
  `.git` returned no matches.
- 2026-06-14: old-name filename scan across working tree files outside `.git`
  returned no matches.
- 2026-06-14: `PYTHONPATH=src python3 -m unittest discover -s tests` passed.
- 2026-06-14: `python3 scripts/check_pins.py` passed.
- 2026-06-14: temporary external venv installed pinned dev requirements; ruff,
  mypy, build, wheel install, and `runhaven agents` passed.
- 2026-06-14: `PYTHONPATH=../repo-harness-creator/src python3 -m harnessforge audit --target . --min-score 85`
  reported 100/100 after the RunHaven rename.
