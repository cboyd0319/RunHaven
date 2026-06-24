# Harness Roadmap

Status: live

This file tracks harness operating-model work only. Product direction lives in
`docs/ROADMAP.md`; current feature status lives in `feature_list.json`; current
handoff lives in `current-state.md`.

Do not read this file at startup unless the task is harness maintenance.

## Principles

- Keep `AGENTS.md` as a map, not a manual.
- Keep startup context to `AGENTS.md`, `feature_list.json`, and
  `current-state.md`.
- Load harness docs on demand by task surface.
- Prefer deletion, compression, or focused checks before adding new harness
  artifacts.
- Treat structural reports as advisory. Real confidence comes from repo-owned
  checks and representative task evidence.

## Items

| Item | Status | Outcome | Next Action |
| --- | --- | --- | --- |
| Lean startup contract | validated | Agents start from three compact files and load deeper docs only when needed. | Keep startup files short during future edits. |
| Harness historical evidence | validated | Long evidence remains available in `docs/harness/evidence/evidence-log.md` without being mandatory startup context. | Add only compact rows for meaningful verification. |
| Verification routing | validated | `docs/harness/feedback/verification-matrix.md` maps change type to focused checks. | Keep command lists current when tooling changes. |
| Real-agent effectiveness evidence | defined | A fixed representative task set now lives in `docs/harness/feedback/quality-document.md` and gates harness keep/remove decisions. | Run the task set before/after removing a component; record before public effectiveness claims. |

## Fresh-Session Check

A new session should be able to answer these from the three startup files:

| Question | Source |
| --- | --- |
| What is this project and what must not be weakened? | `AGENTS.md`, `current-state.md` |
| What is the current or next objective? | `feature_list.json`, `current-state.md` |
| What checks should run before completion? | `AGENTS.md`, then `docs/harness/feedback/verification-matrix.md` if needed |
| What is blocked? | `current-state.md` |

If the answer requires reading most of `docs/harness/`, the harness has drifted
back toward bloat.
