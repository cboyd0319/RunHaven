---
name: harness
description: Use when maintaining, reviewing, or improving this repository's agent harness.
---

# Harness Skill

Use this skill for changes to agent instructions, startup files, state,
verification routing, harness docs, lifecycle handoff, or harness evidence.

## Model

RunHaven uses a lightweight five-part harness:

- Instructions: `AGENTS.md`
- Tools: shell, git, file edits, and `init.sh`
- Environment: manifests, lockfiles, `rust-toolchain.toml`, and `pins.toml`
- State: `feature_list.json` and `current-state.md`
- Feedback: focused checks plus `./init.sh` when the change needs it

## Startup

1. Read `references/repo-harness.md`.
2. Read only the startup files or focused harness docs named by the task.
3. Do not bulk-load `docs/harness/`.

## Rules

- Keep root instructions as a map, not a manual.
- Prefer deleting stale guidance, merging duplicates, or tightening a check
  before adding a new harness file.
- Do not require HarnessForge or any optional owner tool for ordinary
  contributor verification.
- Do not add machine-local paths, personal tool mandates, credential rotation,
  cloud-cost actions, pushes, PRs, or broad automation.
- Treat structural scores as advisory signals, not proof of agent
  effectiveness.
- Record only current state and meaningful verification. Keep raw logs and long
  history out of startup files.

## Verification

For harness-only edits, normally run:

```bash
cargo run --locked --bin runhaven-check-pins
python3 -m json.tool feature_list.json >/dev/null
git diff --check
```

Add a local Markdown link check when docs links change. Run broader checks only
when the harness change affects runtime commands, dependency pins, security
boundaries, release controls, or startup scripts.
