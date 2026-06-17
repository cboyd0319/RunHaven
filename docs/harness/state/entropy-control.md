# Entropy Control

Harnesses rot when project behavior changes but instructions, checks, or state
files do not.

## Review Triggers

Review the harness before releases, large refactors, platform contract changes,
provider endpoint changes, image pin changes, auth broker changes, and repeated
agent errors.

## Correction Loop

1. Identify the repeated failure or stale rule.
2. Confirm it from logs, reviews, missed checks, or runtime evidence.
3. Make the smallest deletion, compression, guide, sensor, test, or state
   update that would prevent the miss.
4. Run relevant repo-owned checks.
5. Record only the evidence that changes the next session's trusted state.

## Cleanup

- Remove stale instructions.
- Merge duplicate docs.
- Keep root instructions short.
- Keep startup state files current and compact.
- Delete generated reports unless intentionally tracked as evidence.
- Keep `first-agent-task.md` retired unless a maintainer explicitly resets it.
