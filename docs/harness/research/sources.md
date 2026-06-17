# Harness Sources

Reviewed: 2026-06-17

Refresh these sources before major harness redesigns because agent tooling,
platform support, and packaging practice change over time.

## Reviewed Source Set

- OpenAI, "Harness engineering: leveraging Codex in an agent-first world":
  <https://openai.com/index/harness-engineering/>
- Walking Labs, Learn Harness Engineering:
  <https://walkinglabs.github.io/learn-harness-engineering/en/>
- AGENTS.md open format:
  <https://agents.md/>
- Project research ledger:
  [`../../RESEARCH.md`](../../RESEARCH.md)

## Local Adaptation

- Keep the root instruction file short.
- Keep startup context to `AGENTS.md`, `feature_list.json`, and
  `current-state.md`.
- Load harness docs on demand.
- Track current feature status and handoff in compact files.
- Keep verification commands explicit and runnable.
- Treat structural scores as guidance, not proof of real task success.
