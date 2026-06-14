# Session Handoff

Last Updated: 2026-06-14

## Current Objective

RunHaven rename is complete for the local repository. The hosted GitHub
repository remote has not been renamed because that requires explicit approval
for a credentialed vendor change.

## Files

- `AGENTS.md`
- `.github/copilot-instructions.md`
- `feature_list.json`
- `progress.md`
- `session-handoff.md`
- `init.sh`
- `init.ps1`
- `pyproject.toml`
- `src/runhaven/`
- `tests/`
- `docs/HARNESS_EVALUATION.md`
- `docs/assets/logo.png`
- `docs/harness/`

## Blockers

- None recorded.

## Verification Evidence

- Historical harness setup: `PYTHON=.venv314/bin/python ./init.sh` passed.
- Historical harness setup: `PYTHON=.venv314/bin/python pwsh -NoProfile -File ./init.ps1`
  passed.
- `PYTHONPATH=../repo-harness-creator/src python3 -m harnessforge audit --target . --min-score 85`
  passed with 100/100.
- `magick identify docs/assets/logo.png` reported PNG 512x512.
- Latest harness audit passed with 100/100 after logo, release control, and
  agent threat-boundary updates.
- No-ignore old-name text scan across working tree files outside `.git`
  returned no matches.
- Old-name filename scan across working tree files outside `.git` returned no
  matches.
- `PYTHONPATH=src python3 -m unittest discover -s tests` passed.
- `python3 scripts/check_pins.py` passed.
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
5. Preserve the macOS 26+ product runtime contract unless the user explicitly
   asks for a supported-platform change.
