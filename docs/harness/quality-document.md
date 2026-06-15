# Quality Document

Last Updated: 2026-06-15

Use this as a periodic codebase health snapshot. It is separate from the
evaluator rubric: the rubric scores one work session, while this document scores
the repo over time.

## Domain Grades

| Domain | Grade | Verification Status | Agent Legibility | Key Gaps |
| --- | --- | --- | --- | --- |
| Harness | A | Repo-owned docs and state are current; HarnessForge audit reports 100/100 as an advisory structural signal | Strong startup path, state files, roadmap, retired first-agent task, and sensors | Keep evidence fresh and treat new HarnessForge suggestions as candidates until repo-owned contracts accept them |
| Product security boundary | A- | Unit tests, pin checks, docs, and runtime smokes cover core boundaries | Security model is explicit and repeated in product docs | Live Apple `container` smokes remain required for boundary changes |
| Provider egress and auth broker | B+ | Proxy, endpoint, auth broker, and smoke scripts exist | Good diagnostics through `why host`, `egress log`, and `auth` commands | Broader path-sensitive hosts and non-Codex brokers need explicit design and evidence |
| Worktree/session/run observability | A- | Focused tests and run-history docs cover lifecycle behavior | Recovery commands and secret-free records are discoverable | Keep data-loss checks tight as lifecycle commands evolve |
| Supply chain and release | B | Pin policy and package/image checks exist | Pin policy is documented | SBOM, provenance, signing, and release evidence automation still need release-prep work |
| Codebase modularity | B+ | Modularization plan and recent extraction evidence are recorded | Component map names current modules | Continue watching files that cross 400 lines or accumulate unrelated responsibilities |

Grades: A is strong, B is usable with known gaps, C needs targeted cleanup, D is
unsafe to rely on without repair.

## Harness Subsystem Health

| Subsystem | Current State | Review Trigger |
| --- | --- | --- |
| Instructions | Root instructions are compact and route to harness docs, product docs, roadmap, and state | Root file grows beyond router role, platform routers duplicate rules, or agents miss current work |
| Tools | `init.sh`, focused Python commands, smoke scripts, advisory HarnessForge reports, and CI are discoverable | New command surface, CI behavior, or runtime smoke changes |
| Environment | Python, macOS, Apple `container`, pins, image manifests, and component boundaries are documented | Version, runner, image, dependency, or platform contract changes |
| State | `feature_list.json`, `progress.md`, `session-handoff.md`, product roadmap, and harness roadmap exist | Current objective changes, release prep starts, or planning decisions move from chat to repo |
| Feedback | Verification matrix, sensor registry, evidence log, repo-policy tests, and pin checks exist | Repeated misses, vague failures, new release gates, or real-agent effectiveness claims |

## Architecture Layers

| Layer | Boundary Status | Verification | Notes |
| --- | --- | --- | --- |
| Repo harness | Documented | Repo-owned docs checks plus advisory `python3 -m harnessforge report --target .` and `python3 -m harnessforge audit --target . --min-score 85` when available | Keep root instructions short and detail in `docs/harness/` |
| CLI and planner | Tested | Compile, unit, ruff, mypy, pin checks | Command construction is security-sensitive |
| Apple `container` runtime | Requires live host evidence | `runhaven doctor`, `runhaven plan`, focused `runhaven run shell` smokes | Do not imply enforced boundaries without code, tests, or live runtime evidence |
| Provider proxy and auth broker | Tested plus selective live smokes | Egress/auth focused tests and smoke scripts | Keep logs secret-free and hosts explicit |
| Release surface | Draft controls | Pin checks, build, dirty-tree check, future SBOM/provenance | Do not publish from an unverified or dirty tree |

## Clean-State Dimensions

- Instructions: current, compact, no machine-local paths.
- State: current objective, blockers, evidence, and next step match.
- Feedback: checks are runnable and mapped to change types.
- Security: mounts, network, credentials, auth, worktrees, and cleanup remain
  least privilege.
- Release: pins, platform assumptions, and provenance evidence are current.

## Harness Simplification

At least monthly, review one harness component and decide whether it is still
useful. Keep it when it prevents repeated failures. Remove or merge it when it
adds upkeep without improving verification, restartability, scope control, or
security review.

## Benchmark Or Task Evidence

Structural harness scores are not proof of real-agent effectiveness. Before
claiming that RunHaven makes agents more effective, record representative task
evidence with baseline/candidate comparison, held-out tasks, worst-case quality,
cost, safety review, and rollback notes.
