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

Before implementation, stop at the first rung that satisfies the problem. This
gate is DRY: do not write what a higher rung already gives you.

1. No change. Does this need to exist at all (YAGNI)?
2. Deletion or simplification.
3. Documentation or configuration. Documentation is product: an undocumented
   behavior does not exist, so its doc ships in the same slice.
4. Standard library. Between two standard-library options of similar size, take
   the one correct on edge cases; lazy means writing less code, not picking the
   flimsier algorithm.
5. Native macOS, Apple `container`, web-platform, or schema behavior
   (`<input type="date">` over a picker library, CSS over JS, a DB or schema
   constraint over app code).
6. Already-installed project dependency. Never add a new dependency for what a
   few lines can do.
7. One clear local change; one line when that stays clear.
8. Minimum new code or harness surface. Boring over clever: clever is what
   someone has to decode at 3am.

Do not use this gate to cut input validation at trust boundaries, data-loss
prevention, security, privacy, accessibility, platform contract, or explicit
user requirements.

## Secure Easy Path Gate

Design the secure path as the default and easiest path. Supported lower-security
choices should show plain-language warnings and require explicit intent, but
should not be hidden or blocked only because they are less secure. Unsupported,
invalid, or hard-boundary violations still fail closed.

## Maintainability Gate

Before editing and before completion, check the touched surfaces:

- Files, modules, crates, Tauri commands, frontend components, and harness docs
  stay cohesive and reasonably reviewable.
- Meaningful duplication is deleted or collapsed into existing helpers where
  that improves clarity.
- New abstractions, configuration, dependencies, or files are justified by real
  repeated behavior, not speculation.
- Standard library, native platform behavior, and already-installed
  dependencies were considered before adding custom code.
- Direct dependency, package, runtime, and image pins stay exact-pinned to
  current stable sources; transitive dependencies stay locked.
- Unneeded code, generated files, docs, config, and harness surface are removed.
- Verify the target behavior before in-scope refactors of the path you just
  changed. A refactor moves the verified/unverified boundary and can silently
  break a path that only happened to work.

If a touched file is already too large or hard to review, include a local split
or deletion in the same scope unless deferral is explicit, small, and recorded.

## Acceptance Criteria

- The requested behavior or harness improvement is visible in repo files.
- Any changed behavior ships its documentation in the same slice; an
  undocumented behavior is treated as not shipped.
- Security-sensitive changes preserve fail-closed defaults.
- Secure defaults remain the easiest path; supported lower-security choices are
  explicit and warned.
- macOS 26+ on Apple silicon remains the only runtime and contributor
  verification target.
- New or changed custom failure messages name what failed, the boundary that
  matters, and where to repair it, per `docs/harness/feedback/sensor-registry.md`.
- Project-owned instructions remain compact and route durable detail into
  focused docs.
- File size, modularity, duplication, dependency use, and crate/component
  organization were considered for touched surfaces.
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
