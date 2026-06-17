# Quality Document

Last Updated: 2026-06-17

This is a periodic repo health snapshot. It is not startup context.

## Domain Grades

| Domain | Grade | Current Read |
| --- | ---: | --- |
| Harness | A- | Startup is compact and on-demand; keep old generated artifacts out of the mandatory path. |
| Product security boundary | A- | Core mount, credential, provider, and runtime boundaries have tests and docs; live Apple container smokes remain required for boundary changes. |
| Provider egress and auth broker | B+ | Proxy, endpoint, auth broker, and diagnostics exist; path-sensitive hosts and non-Codex brokers need design before implementation. |
| Worktree/session/run observability | A- | Recovery commands and secret-free records are discoverable; keep data-loss checks tight as lifecycle commands evolve. |
| Supply chain and release | B | Pin checks exist; SBOM, provenance, signing, and release evidence remain release-prep work. |
| Codebase modularity | B+ | Rust modules are split by ownership; keep watching near-limit files and unrelated responsibility growth. |

## Harness Health

| Subsystem | Current State | Review Trigger |
| --- | --- | --- |
| Instructions | `AGENTS.md` is a short router | Root file grows beyond map role |
| Tools | `init.sh`, focused Cargo/npm commands, and Apple container smokes are discoverable | Tooling or runtime command surface changes |
| Environment | Pins, lockfiles, manifests, and image templates describe setup | Version, dependency, image, or platform changes |
| State | `feature_list.json` and `current-state.md` are compact startup files | Current objective, blocker, or trusted evidence changes |
| Feedback | Verification matrix, pin check, tests, and smokes are mapped by change type | Repeated misses or new release gates |

## Cleanup Rule

At least monthly, review one harness component. Keep it when it prevents
repeated failures. Compress, merge, or delete it when it adds context cost
without improving verification, restartability, scope control, or security
review.

Structural scores are not proof of real-agent effectiveness. Use
representative task evidence before making effectiveness claims.
