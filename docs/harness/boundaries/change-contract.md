# Change Contract

Use this for non-trivial work before editing. Keep one active objective unless
`multi-agent-orchestration.md` names separate owners and files.

## Problem

State the user-visible problem, security risk, documentation gap, or maintenance
failure. Include the source that proves the problem exists.

## Scope

In scope:

- Target-relative files or components to change.
- User-visible behavior or harness behavior to improve.
- Verification commands or evidence expected before handoff.

Non-goals:

- Unrelated cleanup, refactors, dependency changes, or workflow automation.
- Unsupported Windows or Linux runtime/contributor verification.
- Host home, raw SSH key, browser profile, cloud credential, or arbitrary env
  passthrough by default.
- Credentialed vendor changes such as repository rename, release publishing,
  secret rotation, or cloud cost actions without explicit approval.

## Build Necessity Gate

Before implementation, stop at the first rung that satisfies the problem:

1. No change.
2. Deletion or simplification.
3. Documentation or configuration.
4. Standard library.
5. Native macOS or Apple `container` behavior.
6. Existing project dependency.
7. One clear local change.
8. Minimum new code or harness surface.

Do not use this gate to cut input validation at trust boundaries, data-loss
prevention, security, privacy, accessibility, platform contract, or explicit
user requirements.

## Acceptance Criteria

- The requested behavior or harness improvement is visible in repo files.
- Security-sensitive changes preserve fail-closed defaults.
- macOS 26+ on Apple silicon remains the only runtime and contributor
  verification target.
- Project-owned instructions remain compact and route durable detail into
  focused docs.
- Relevant feature, current-state, evidence, and roadmap state agree.

## Verification

Choose checks from `docs/harness/feedback/verification-matrix.md`; that file
owns command routing and escalation rules.

Required evidence:

- Command names.
- Pass or fail result.
- Any skipped checks, reason, risk, and next best check.
- Any optional structural-review recommendation adopted into RunHaven must be
  backed by repo-owned docs, tests, policy, or maintainer decision.
- Runtime smoke evidence for Apple `container`, provider, image, auth, or
  worktree boundary changes.

## Rollback

Record the smallest safe rollback:

- revert the commit;
- restore a previous doc or generated file;
- disable a feature flag or command path;
- remove a generated artifact;
- run the explicit RunHaven cleanup command for state, network, image, or
  worktree changes.

## Platform Impact

Record whether the change affects Rust 1.96.0, Cargo dependency resolution,
macOS 26+ runtime behavior, Apple silicon, Apple `container` 1.0.0, future
hosted CI, or unsupported-platform guardrails.

Before changing platform floors, interpreter versions, future hosted-CI runner
labels, Apple `container` assumptions, package pins, or image pins, record
current primary-source evidence and the review date in `docs/RESEARCH.md` or
`docs/harness/evidence/evidence-log.md`.
