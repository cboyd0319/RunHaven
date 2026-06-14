# Entropy Control

Harnesses rot when project behavior changes but instructions, checks, or state
files do not.

## Regular Assessment

Run at least before releases, large refactors, and after repeated agent errors:

```bash
PYTHONPATH=../repo-harness-creator/src python3 -m harnessforge audit --target .
```

## Correction Loop

1. Identify the lowest-scoring domain.
2. Confirm the failure from logs, review comments, or missed checks.
3. Add the smallest guide or sensor that would have prevented it.
4. Re-run the audit and relevant project checks.

## Cleanup

- Remove stale instructions.
- Merge duplicate docs.
- Keep root instructions short.
- Keep state files current.
- Delete generated reports unless intentionally tracked.
