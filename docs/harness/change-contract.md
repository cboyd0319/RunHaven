# Change Contract

Use this for non-trivial work before editing.

## Problem

What needs to change?

## Scope

In scope:

- TBD

Non-goals:

- TBD

## Acceptance Criteria

- TBD

## Verification

Detected commands:

- `python3 -m compileall src tests scripts`
- `PYTHONPATH=src python3 -m unittest discover -s tests`
- `python3 scripts/check_pins.py`
- `python3 -m ruff check .`
- `python3 -m mypy src`
- `python3 -m build`

Required evidence:

- Command names.
- Pass or fail result.
- Any skipped checks and risk.

## Rollback

How can this change be reverted or disabled?

## Platform Impact

Record whether the change affects Python 3.13+, macOS 26+ runtime behavior,
Apple silicon, Apple `container` 1.0.0, Windows 11 contributor checks, Ubuntu
22.04+ CI checks, or other Linux import/docs checks.
