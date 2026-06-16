# Harness Roadmap

Status: live

This file tracks accepted harness and repository operating-model work. Product
feature direction lives in `docs/ROADMAP.md`; this file records harness,
planning, evidence, and maintenance boundaries so decisions do not live only in
chat, issue comments, or one-off research notes.

Update `current-state.md` when roadmap state changes the active objective,
trusted verification, blockers, or next step.

## Source And Evidence Weighting

Use this order when roadmap evidence conflicts:

1. Target repository files, commands, tests, platform contract, and maintainer
   decisions.
2. Reviewed RunHaven product docs and harness docs.
3. Canonical harness patterns from HarnessForge and approved harness-learning
   resources, treated as advisory while HarnessForge is under active
   development.
4. Sibling-project examples, external ideas, generated reports, and research
   notes.

Sibling-project examples, HarnessForge output, and generated reports are useful
evidence, but they are not contracts until a maintainer promotes the pattern
into repo-owned docs, tests, schemas, templates, or code.

## Smallest Correct Work Gate

Use this gate for each accepted roadmap item before implementation starts:

1. Can the outcome be met by no change, deletion, documentation,
   configuration, or existing behavior?
2. Can the Rust standard library cover it?
3. Can native macOS or Apple `container` behavior cover it?
4. Can an existing project dependency cover it without new configuration?
5. Can one clear local change satisfy the contract?
6. Only then add the minimum new code or harness surface area.

Do not add speculative features, unrequested configurability, one-off
abstractions, new workflows, or dependencies unless they are explicit in the
item scope. Do not use this gate to cut input validation at trust boundaries,
data-loss prevention, security, privacy, platform contract, or explicit user
requirements.

## Task Buckets

- `active`: work currently being executed.
- `accepted`: agreed work that is not started yet.
- `candidate`: useful idea that still needs owner, scope, or evidence.
- `debt`: recurring cleanup or drift that should become a sensor, test, or
  stability gate when stable.
- `completed`: shipped work whose durable behavior has moved into docs, tests,
  schemas, templates, or code.
- `archived`: historical context kept for provenance, not restart priority.

## Status Lifecycle

- `candidate`: possible work, not committed.
- `accepted`: agreed direction, needs sequencing.
- `planned`: scoped with an execution gate.
- `in_progress`: currently owned.
- `blocked`: cannot continue without named input or state change.
- `validated`: implementation finished and evidence recorded.
- `shipped`: adopted, tagged, or merged into the durable repo surface.
- `superseded`: replaced by another item or plan.
- `abandoned`: intentionally dropped.

## Surface Impact Checklist

For every accepted roadmap item, classify which surfaces are in scope before
implementation starts.

| Surface | Question |
| --- | --- |
| Local repo harness | Does this affect instructions, state files, harness docs, sensors, or packaging controls? |
| Generated or owned harness files | Should the project add or update a harness file, section, manifest rule, or review-required placeholder? |
| CLI or tool runtime | Does this add or change a command, flag, JSON contract, exit code, report, or default behavior? |
| Existing project files | Could this modify project-owned instructions, specs, workflows, scripts, docs, or image templates? |
| CI or hosted automation | Should CI workflows, summaries, reports, permissions, or artifacts change? |
| Optional workflow scaffolds | Should setup, teardown, push, PR, or other maintenance automation stay opt-in? |
| Tests and fixtures | Which local checks, generated snapshots, fixtures, or representative task runs prove the change? |
| Packaging surface | Does this affect packaging, tags, SBOM, provenance evidence, notes, or rollback? |
| Research and source records | Does this need current primary-source evidence or project-owned source records? |
| Security and privacy | Could this expose secrets, local paths, private code, tool permissions, network access, user data, or cost? |
| Platform contracts | Does this affect macOS 26+, Rust, Apple `container`, future hosted-CI runner labels, or unsupported platform guardrails? |
| Docs and UX | Which user-facing docs, help output, startup instructions, and first-run guidance need to change? |

If a surface is out of scope, say so in the item notes. Do not let target-local
preferences, generated harness defaults, and CI automation drift into each
other silently.

## Roadmap Items

| Item | Status | User Outcome | Surfaces In Scope | Execution Gate | Owner | Verification Evidence | Done Or Retire When |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Repo harness overhaul | validated | A new agent can understand RunHaven, improve the harness, choose checks, and preserve the macOS 26+ security boundary from repo files alone | Local repo harness, generated/owned harness files, docs, sensors, manifest, state | Documentation/configuration plus generated missing harness artifacts was sufficient | Maintainers | 2026-06-15: first-agent task retired, report/audit rerun, JSON validation, docs checks, and diff hygiene recorded in state and evidence files. 2026-06-16: state was consolidated into `current-state.md`. | Reopen only if report/audit, fresh-session review, or maintainer feedback finds stale routing |
| Tauri/UI research and framework selection | active | The first UI implementation starts from source-backed Tauri architecture, frontend framework, comparable-product UX, and beginner-safe flow decisions | Docs, research, security boundaries, command contracts, UX planning | Research and plan before adding `src-tauri/` or frontend dependencies | Maintainers | Pending | A research doc names the framework recommendation, screen map, command contract, capability map, and first scaffold acceptance criteria |
| Effectiveness evidence for agent runs | candidate | Claims about agent quality are backed by representative tasks, not structural audit | Evaluation, docs, sensors, future reports | Design evidence contract before automating | Maintainers | Pending | Adopt or retire after representative task set is defined |
| Path-aware provider host policy | candidate | Broad hosts such as `github.com` can be constrained by verified path or brokered credential flow | Provider runtime, security, docs, tests, smokes | Do not build until source-backed paths and enforcement mechanism are clear | Maintainers | Pending | Accepted only with proof it avoids credential leakage and broad egress |
| Extension and MCP boundary policy | validated | Future MCP/extension support is deny-by-default and reviewable | Security, docs, CLI/runtime, tests | Documentation before implementation was sufficient for this pass | Maintainers | `docs/EXTENSION_MCP_BOUNDARY.md` and `docs/SECURITY_MODEL.md` | Reopen only when implementation starts |
| Image/state/network repair polish | accepted | Repair commands give exact safe next steps without mutating unrelated resources | CLI, docs, sensors, tests | Extend existing commands before new abstractions | Maintainers | Existing image doctor and network tests; more pending | UX gaps from `ux-research-ideas.md` are resolved or retired |

## Fresh-Session Test

Use this when reviewing whether the harness is actually useful to a new agent.

| Question | Current Answer Source | Gap Or Action |
| --- | --- | --- |
| What is this system or package? | `README.md`, `AGENTS.md`, `docs/CAPABILITIES.md` | Keep product status and macOS 26+ boundary current |
| How is the repo organized? | `docs/ARCHITECTURE.md`, `docs/harness/boundaries/component-inventory.md`, `docs/harness/state/modularization-plan.md` | Update after module extractions or new image/profile surfaces |
| How does it start? | `docs/INSTALLATION.md`, `README.md`, `runhaven setup`, `init.sh` | Keep first-run setup and development setup aligned |
| How is it verified? | `docs/harness/feedback/verification-matrix.md`, `docs/harness/feedback/sensor-registry.md`, `init.sh` | CI is intentionally disabled during alpha/pre-release; local verification remains authoritative |
| What work is current? | `feature_list.json`, `current-state.md`, `docs/ROADMAP.md`, this roadmap | Keep objective and next-session guidance synchronized |

## Instruction Rule Lifecycle

Use this table for durable rules that survive beyond one task. Keep the root
instruction file as a router; move topic detail into focused docs.

| Rule Or Topic | Source | Applies When | Mechanical Check | Retire Or Replace When |
| --- | --- | --- | --- | --- |
| macOS 26+ only runtime and contributor verification | Product docs, `pins.toml`, local Rust tests, future CI if re-enabled | Any runtime, future CI, install, or docs change | `cargo test --locked`, platform wording scan | Maintainer accepts a source-backed platform expansion |
| No host home, raw SSH key, browser profile, cloud credential, or arbitrary env passthrough by default | Security model and command validators | Any workspace, env, SSH, auth, or container invocation change | Focused plan/run validator tests and runtime smokes | Replaced by a stronger least-privilege boundary |
| Provider hosts are explicit and source-backed | `docs/PROVIDER_ENDPOINTS.md`, `src/runhaven/provider/endpoints.rs` | Provider endpoint, proxy, auth, or network change | Endpoint tests, `why host`, provider smoke when needed | Path-aware or brokered policy supersedes broad host handling |
| Run records stay secret-free | Run observability docs and tests | Any runs, auth, egress, active, or git metadata change | Run-history and active-run tests | Replaced by stronger redaction and evidence contract |
| Root instructions stay map-like | `AGENTS.md`, this harness | Any instruction or harness doc change | Harness audit and manual review | Platform-specific routers become canonical by accepted decision |

## Completion Evidence Ladder

| Evidence Layer | Use When | Example Evidence |
| --- | --- | --- |
| Static | Any code, docs-link, config, schema, or template change | lint, type check, compile, schema validation, docs link check, platform wording scan |
| Runtime/startup | The CLI, smoke script, Apple `container` boundary, image, or broker must start or execute | `runhaven doctor`, `runhaven plan`, image dry-run, provider smoke, broker smoke |
| System/user flow | A change crosses components or affects user-visible behavior | run lifecycle scenario, worktree recovery flow, provider run |

Skipping a required layer means the item is not complete. Record the reason,
risk, and next best evidence when a layer cannot run.

## Technical Debt And Drift

| Item | Evidence | Risk | Next Step | Status |
| --- | --- | --- | --- | --- |
| Script purpose header sensor | Maintained scripts and image templates carry top-of-file descriptions, and `runhaven-check-pins` enforces the convention | New scripts fail the local pin check when the description is missing | Keep the sensor scoped to maintained scripts and image templates | validated |
| Real-agent effectiveness evidence is absent | Repo docs intentionally block effectiveness claims without representative evidence | Structural score could be overclaimed | Define representative tasks before public claims | candidate |
| Historical evidence includes old local HarnessForge invocation examples | Older evidence rows preserve exact commands | Active docs could copy stale sibling-checkout patterns | Keep active guidance self-contained; do not edit old evidence unless cleaning history is in scope | debt |

## Failure-Mode Map

| Failure Mode | First Artifact To Check |
| --- | --- |
| Cold-start confusion | `current-state.md` |
| Scope sprawl | `feature_list.json`, `docs/ROADMAP.md`, and this roadmap |
| Premature completion | `clean-state-checklist.md`, `verification-matrix.md`, and `sensor-registry.md` |
| Fragile startup | `init.sh`, `docs/INSTALLATION.md`, and `docs/harness/README.md` |
| Weak handoff | `current-state.md` |
| Subjective review | `evaluator-rubric.md` and recorded evidence |
| Overbuilt solution | smallest-correct work gate and change contract |
| Knowledge visibility gap | fresh-session test and component inventory |
| Instruction bloat | instruction rule lifecycle and topic-doc routing |
| Missing runtime signal | verification matrix, sensor registry, and evidence log |
| Entropy growth | clean-state checklist, quality document, and periodic cleanup sensors |

## Rules

- Keep roadmap entries target-relative and portable.
- Record accepted work here when it changes harness behavior, generated files,
  CI behavior, packaging evidence, or recurring maintenance.
- Keep candidate ideas separate from accepted commitments.
- Do not mark work passing, validated, shipped, or complete from agent
  assertion alone. Record verification evidence.
- Do not promote speculative features, abstractions, workflows, dependencies,
  or broad cleanup unless the accepted item names them.
- Record ceilings and upgrade paths for intentional simplifications.
- Move completed behavior into durable docs, tests, schemas, templates, or code
  instead of leaving the roadmap as the only source of truth.
- Preserve existing project-owned planning systems. Link to `docs/ROADMAP.md`
  instead of replacing it.
- Keep sensor and validation error messages actionable for agents: what failed,
  why it matters, and where to repair.
- Do not turn structural audit score into proof of real-agent effectiveness.
