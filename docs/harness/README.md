# Harness Operations

Status: live

RunHaven's harness exists to let a new agent start, stay in scope, verify work,
and leave a restartable handoff without loading the whole repository history.

## Minimal Model

Harness = instructions + tools + environment + state + feedback.

| Subsystem | RunHaven Artifact | Purpose |
| --- | --- | --- |
| Instructions | `AGENTS.md` | Startup path, hard rules, and routing |
| Tools | shell, git, file edits, `init.sh` | Do useful work and verify it locally |
| Environment | `Cargo.toml`, locks, `rust-toolchain.toml`, `pins.toml`, image templates | Make versions and setup self-describing |
| State | `feature_list.json`, `current-state.md` | Current status, blockers, evidence, and next step |
| Feedback | focused commands, `./init.sh`, runtime smokes when needed | Prevent unsupported completion claims |

## Startup Budget

Always read only:

- `AGENTS.md`
- `feature_list.json`
- `current-state.md`

Then load focused docs only when the task requires them. The harness directory
is reference material, not mandatory startup context.

## On-Demand Map

| Task Surface | First File To Read |
| --- | --- |
| Verification choice | `feedback/verification-matrix.md` |
| Component ownership | `boundaries/component-inventory.md` |
| Security or privacy boundary | `boundaries/security-boundary-map.md` |
| Dependency, pin, or workflow change | `boundaries/dependency-change-policy.md` |
| Release preparation | `release/release-controls.md` |
| Apple `container` pin update | `release/apple-container-update-playbook.md` |
| Historical verification | `evidence/evidence-log.md` |

## Operating Loop

1. Read the three startup files.
2. Pick one objective.
3. Load only the docs for the touched surface.
4. Make the smallest coherent change.
5. Run the smallest check set that can catch likely regressions.
6. Update `feature_list.json` and `current-state.md` only when state changed.
7. Record durable evidence only when it changes what the next session trusts.

## When To Add Harness

Add or expand harness only when it prevents a repeated failure, protects a
security or data-loss boundary, makes verification executable, or preserves
state that the next session cannot cheaply reconstruct.

Do not add harness for style preference, structural score chasing, or one-off
research. Compress or delete guidance that costs startup context without
changing agent behavior.

## Optional Tools

HarnessForge and generated reports are optional owner tools. Treat their output
as advisory until a maintainer promotes a recommendation into repo-owned docs,
tests, policy, or code.
