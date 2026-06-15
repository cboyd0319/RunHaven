# First-Agent Harness Improvement Task

REVIEW REQUIRED: use this during the first agent session after this harness
overhaul. Replace or retire this task after the maintainer accepts the improved
harness and any remaining actions are captured in `docs/harness/roadmap.md`.

## Goal

Deepen this repo's harness beyond what static generation can know. Use the
generated structure as a starting point, then inspect the real repository to
improve guidance, readiness signals, checks, ownership, and review boundaries.

## Inputs

- `AGENTS.md`
- `docs/harness/README.md`
- `docs/harness/component-inventory.md`
- `docs/harness/change-contract.md`
- `docs/harness/verification-matrix.md`
- `docs/harness/sensor-registry.md`
- `docs/harness/evidence-log.md`
- `docs/harness/security-boundary-map.md`
- `docs/harness/roadmap.md`
- `feature_list.json`
- `progress.md`
- `session-handoff.md`

## Work

1. Confirm the harness matches the real RunHaven repo shape.
2. Run a fresh-session test from repo files only: identify what the project is,
   how it is organized, how it starts, how it is verified, and what work is
   current.
3. Treat initialization as its own phase. Confirm startup, one trustworthy
   verification path, current progress visibility, and project-specific
   source-of-truth routing before unrelated feature work.
4. Improve component inventory entries when important boundaries, packages,
   apps, services, jobs, or docs roots were omitted.
5. Improve source-of-truth routing for specs, plans, work items, workflows,
   ADRs, release notes, and project docs.
6. If localized docs exist, identify the source-of-truth locale or canonical
   docs root before editing translated copies.
7. Improve verification guidance with repo-owned commands that maintainers can
   run locally and in macOS 26+ CI.
8. Improve the verification matrix, sensor registry, and evidence log so they
   name realistic checks, owners, review cadence, and retirement conditions.
9. Improve runtime and process observability notes. Runtime signals explain what
   happened; plans, contracts, rubrics, and evidence explain why a change should
   be accepted.
10. Improve security boundary notes for Apple `container`, mounts, worktrees,
   state volumes, provider egress, auth brokers, SSH, env passthrough, secrets,
   credentials, permissions, external services, and cost-incurring tools.
11. Add source, applicability, and retirement notes for durable rules that the
   project accepts into root instructions or harness docs.
12. Record uncertain project-specific assumptions as review-required notes until
   a maintainer accepts them.

## Boundaries

- Do not overwrite project-owned instructions unless the user explicitly asks
  for that change or passes force behavior.
- Do not run target commands until the repo has an approved verification route
  for that command.
- Do not add user-specific tool preferences, local research mandates, personal
  memory paths, MCP configs, or machine-specific absolute paths.
- Do not install dependencies, create autonomous workflows, push branches, open
  pull requests, rotate secrets, or change cloud resources unless explicitly
  requested.
- Do not treat structural audit score as proof of real-agent effectiveness.
- Do not add Windows or Linux runtime or contributor-verification targets.
- Do not mark features, roadmap items, or tasks passing from agent confidence
  alone. Behavior, verification, status, and evidence must agree.

## Acceptance

- Root instructions still stay compact and route durable detail into
  `docs/harness/`.
- `docs/harness/component-inventory.md` names the important project boundaries
  known after review.
- The fresh-session test has an answer source for project purpose,
  organization, startup, verification, and current work.
- `docs/harness/verification-matrix.md` identifies the smallest reliable
  checks for common change types.
- `docs/harness/sensor-registry.md` records project-relevant checks with owner,
  source, purpose, and retirement condition.
- Important sensor or validation failures include actionable repair guidance:
  what failed, why the boundary matters, and where to repair.
- `docs/harness/evidence-log.md` records what was reviewed, what changed, what
  checks ran, and what still needs maintainer review.
- `progress.md` and `session-handoff.md` capture the current objective,
  blockers, verification evidence, and recommended next step.
