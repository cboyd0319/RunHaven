# V1 Decisions And Retrospective

This file holds release decision history and the release-cut retrospective so
`docs/V1_RELEASE_PLAN.md` stays focused on the current plan.

## Decision Log

- Desktop-first-class v1: the project goal is to make the secure way easy for
  less-technical users, and that requires a first-class GUI.
- CLI-complete v0.5.0: all CLI product work should be done by `v0.5.0`; after
  that, v1 iteration focuses on desktop, packaging, accessibility, and release
  trust.
- Maintainability is release scope: file size, modularity, duplication, crate
  organization, dependency discipline, and harness state are part of every
  slice, not cleanup for later.
- Secure easy path is the product rule: warning and explicit intent are enough
  for supported lower-security choices, while unsupported or hard-boundary
  violations fail closed.
- CLI remains stable after v0.5.0: the CLI is the shared backend, automation
  surface, and power-user escape hatch.
- No desktop generic shell: arbitrary attach/terminal behavior stays CLI-only
  for v1 because it widens the desktop trust and UX surface.
- Stop, kill, and repair are v1 desktop requirements: a GUI that can launch a
  VM must let the user recover from it.
- Worktree review is v1 desktop scope: safe change isolation is part of making
  agent work understandable and reversible.
- Exact Apple `container` pinning stays: fail-closed drift is better than
  silently trusting a changed runtime boundary.
- SSH stays disabled: a visible forwarded socket is not sufficient proof for
  non-root guest use.
- No new credential brokers for v1: broker work is security-sensitive and must
  be provider-specific with disposable-secret smokes.
- No TLS interception: path-aware HTTPS policy through interception would add
  certificate, trust, privacy, and maintenance risk.
- `container machine` is not the default boundary: task-scoped `container run`
  remains the secure-easy path, while explicit or user-managed machine
  workflows should warn and require intent rather than be policy-blocked.
- Internet mode remains unrestricted: do not market it as egress controlled.
- Provider mode is host allowlist control, not complete provider safety.
- Signed and notarized desktop artifact is required for v1.0.0.
- Automatic updater can wait: manual updates are acceptable if release notes
  make that clear.

## Outcomes And Retrospective

This section should be filled only when the release is cut.

- Release commit:
- Release tag:
- Artifact type:
- Checks passed:
- Apple `container` runtime evidence:
- Desktop artifact signing/notarization evidence:
- Known limits shipped:
- Rollback path:
- Follow-up for v1.0.1 or v1.1.0:
