# Entropy Control

Harnesses rot when project behavior changes but instructions, checks, or state
files do not.

## Regular Assessment

Run at least before releases, large refactors, platform contract changes,
provider endpoint changes, image pin changes, auth broker changes, and after
repeated agent errors:

```bash
harnessforge report --target .
harnessforge audit --target . --min-score 85
```

## Correction Loop

1. Identify the weakest subsystem or repeated failure mode.
2. Confirm the failure from logs, review comments, missed checks, or real
   runtime evidence.
3. Add the smallest guide, sensor, test, or state update that would have
   prevented it.
4. Re-run the relevant project checks and harness report/audit.
5. Record evidence and the next review trigger.

## Cleanup

- Remove stale instructions.
- Merge duplicate docs.
- Keep root instructions short.
- Keep state files current.
- Delete generated reports unless intentionally tracked as evidence.
- Keep `first-agent-task.md` retired unless a maintainer explicitly resets it.
- Move completed roadmap behavior into durable docs, tests, schemas, templates,
  or code.
