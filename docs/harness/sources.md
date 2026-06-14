# Harness Sources

Reviewed: 2026-06-14

This file records the source basis for the generated harness. Refresh it before
major harness redesigns because agent tooling, platform support, and packaging
practice change over time.

## Reviewed Source Set

- OpenAI, "Harness engineering: leveraging Codex in an agent-first world":
  <https://openai.com/index/harness-engineering/>
- Walking Labs, Learn Harness Engineering:
  <https://walkinglabs.github.io/learn-harness-engineering/en/>
- GitHub Docs, secure use reference for GitHub Actions:
  <https://docs.github.com/en/actions/reference/security/secure-use>
- AGENTS.md open format:
  <https://agents.md/>
- Project research ledger:
  [`../RESEARCH.md`](../RESEARCH.md)

Do not add unreviewed secondary links here. Put candidates in
`research-inbox.md` until they are checked and relevant.

## Local Adaptation

- Keep the root instruction file short.
- Put durable detail in `docs/harness/`.
- Generate both POSIX and PowerShell entrypoints.
- Preserve existing files unless forced.
- Track feature behavior, verification, status, and evidence in
  `feature_list.json`.
- Include clean-state, evaluator, and quality-review artifacts for lifecycle
  control.
- Check local Markdown links so harness docs remain navigable and portable.
- Keep dependency and workflow changes on latest stable supported releases with
  hard pins and verification evidence.
- Use reviewed self-healing pull requests instead of silent default-branch
  mutation.
- Treat structural scores as guidance, not proof of real task success.
