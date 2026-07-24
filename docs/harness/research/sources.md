# Harness Sources

Reviewed: 2026-07-14

Refresh these sources before major harness redesigns because agent tooling,
platform support, and packaging practice change over time.

## Reviewed Source Set

- OpenAI, "Harness engineering: leveraging Codex in an agent-first world":
  <https://openai.com/index/harness-engineering/>
- Walking Labs, Learn Harness Engineering:
  <https://walkinglabs.github.io/learn-harness-engineering/en/>
- Walking Labs source repository:
  <https://github.com/walkinglabs/learn-harness-engineering>
- AGENTS.md open format:
  <https://agents.md/>
- Project research ledger:
  [`../../RESEARCH.md`](../../RESEARCH.md)

## Local Adaptation

- Keep the root instruction file short.
- Keep startup context to `AGENTS.md`, `PROGRESS.md`, and
  `feature_list.json`.
- Load harness docs on demand.
- Track current feature status and handoff in compact files.
- Keep verification commands explicit and runnable.
- Map external templates onto existing RunHaven owners before adding files.
- Reject composite structural scores as proof of real task success. Use
  deterministic contract checks and representative outcomes.
