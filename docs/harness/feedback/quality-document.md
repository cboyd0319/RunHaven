# Quality Document

Last Updated: 2026-06-27

This is a periodic repo health snapshot. It is not startup context.

## Domain Grades

| Domain | Grade | Current Read |
| --- | ---: | --- |
| Harness | A | Startup is compact and on-demand; the 2026-06-27 resource review confirmed the advanced template concepts map to existing RunHaven owners. |
| Product security boundary | A- | Core mount, credential, provider, and runtime boundaries have tests and docs; live Apple container smokes remain required for boundary changes. |
| Provider egress and auth broker | B+ | Proxy, endpoint, auth broker, and diagnostics exist; path-sensitive hosts and non-Codex brokers need design before implementation. |
| Worktree/session/run observability | A- | Recovery commands and secret-free records are discoverable; keep data-loss checks tight as lifecycle commands evolve. |
| Supply chain and release | B | Pin checks exist; SBOM, provenance, signing, and release evidence remain release-prep work. |
| Codebase modularity | A- | Workspace crates now match ownership boundaries. Treat the Codex-vendored TUI as source-first and avoid rebuilding a root compatibility facade. |

## Harness Health

| Subsystem | Current State | Review Trigger |
| --- | --- | --- |
| Instructions | `AGENTS.md` is a short router | Root file grows beyond map role |
| Tools | `init.sh`, focused Cargo/npm commands, and Apple container smokes are discoverable | Tooling or runtime command surface changes |
| Environment | Pins, lockfiles, manifests, and image templates describe setup | Version, dependency, image, or platform changes |
| State | `feature_list.json` and `current-state.md` are compact startup files | Current objective, blocker, or trusted evidence changes |
| Feedback | Verification matrix, pin check, tests, and smokes are mapped by change type | Repeated misses or new release gates |

## Advanced Template Fit

The OpenAI advanced repo template is a shape reference, not a file checklist.
RunHaven already has owners for the advanced pack concepts:

- product behavior lives in public docs and focused plans;
- architecture lives in `docs/ARCHITECTURE.md`, component inventory, and the
  modularization plan;
- security lives in `docs/SECURITY_MODEL.md` and boundary docs;
- reliability and feedback live in the verification matrix, sensor registry,
  release controls, and `init.sh`;
- active state lives in `feature_list.json` and `current-state.md`.

Do not add parallel template files unless the representative task set below
shows that an existing owner cannot hold the concept clearly.

## Representative Task Set

Keep/remove decisions about a harness component should be measured, not guessed.
Before removing or merging a component, run this small fixed task set with the
component present, then again with it removed, and compare whether an agent can
still start, stay in scope, verify, and hand off cleanly:

1. Cold start: from the three startup files only, name the project, the current
   active slice, the active blocker, and the first check to run.
2. CLI plan dry-run: produce a `runhaven plan` for a default run and confirm the
   mount, user, network, and image boundary read as expected.
3. Security-boundary edit: make a small change behind one boundary journey and
   route it to the correct checks from `verification-matrix.md`.
4. Docs/harness edit: change one harness doc and select the smallest correct
   check set without bulk-loading `docs/harness/`.

Removal is safe only when the task set still passes without the component.

## Cleanup Rule

At least monthly, review one harness component. Keep it when it prevents
repeated failures or when the representative task set degrades without it.
Compress, merge, or delete it when the task set still passes without it and it
adds context cost without improving verification, restartability, scope control,
or security review.

Structural scores are not proof of real-agent effectiveness. Use the
representative task set above as evidence before making effectiveness claims.
