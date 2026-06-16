# AGENTS.md

## Project overview

RunHaven is a Rust CLI for running AI coding agents inside
Apple `container` on macOS 26+.

Core harness contract: instructions, tools, environment, state, and feedback
are all required. Keep instructions map-like, tool access sufficient but least
privileged, environment facts self-describing, state current across sessions,
and verification commands explicit. Changing instruction files, shell access,
filesystem scope, git state, startup scripts, verification commands, container
defaults, provider allowlists, hooks, lint checks, or evaluator loops changes
the effective agent. Treat those as product changes.

Startup path:

1. Confirm the working directory and inspect `git status --short --branch`.
2. Read this file, `feature_list.json`, and `current-state.md`.
3. Read `README.md` for product, install, usage, or public-doc changes.
4. Read `docs/harness/README.md` and
   `docs/harness/authoritative-facts.md` for harness-doc, generated-output,
   scoring, report, or maintenance-policy changes.
5. Check `docs/harness/state/roadmap.md` before selecting, deferring, or
   reshaping backlog, release-prep, or product-scope work.
6. Check `docs/harness/boundaries/component-inventory.md` before changing CLI
   modules, image templates, verification routing, or harness files.
7. For harness-maintenance work, use `.agents/skills/harness/SKILL.md`.
8. If `docs/harness/state/first-agent-task.md` still exists and is not marked
   retired, complete or retire it before unrelated feature work.
9. Pick one current objective before editing.

This repo is harnessed. Keep root instructions short and place durable detail
in `docs/harness/`.

## Build and test commands

Use the smallest reliable command for the change.

Verification route: use `./init.sh` for full macOS harness verification, and
use the focused commands below for smaller changes.

Full local verification on macOS 26+:

```bash
./init.sh
```

Focused checks:

```bash
cargo fmt --check
cargo test --locked
cargo clippy --all-targets -- -D warnings
cargo run --locked --bin runhaven-check-pins
cargo build --locked
harnessforge report --target .
harnessforge audit --target . --min-score 85
```

Use `runhaven doctor` and Apple `container` runtime smokes when changes affect the
actual macOS container boundary, image templates, agent profiles, or install
flow.

Before running `harnessforge` commands, install HarnessForge in the current
development environment or run from an environment where the package is already
available. Treat HarnessForge as advisory while it is under active development:
repo-owned files, tests, policy docs, and manual review are authoritative. Do
not commit machine-specific checkout paths for local tooling.

## Code style guidelines

- Prefer standard library tools and existing crates already pinned in
  `Cargo.toml`; add crates only when they remove real security or usability
  risk.
- Match local code style and file structure. Avoid broad refactors unless they
  are required for the task.
- Keep direct dependencies exact-pinned and minimal.
- Before writing code, stop at the first rung that solves the request: no
  change, deletion, documentation, configuration, standard library, native
  platform behavior, existing project dependency, then minimum custom code.
- Use exact subprocess argument lists, not executable shell strings, for
  runtime command generation.
- Use `rg` for repository searches. Keep command output bounded when possible.
- Use `apply_patch` for manual edits.
- Preserve existing user changes. Never revert dirty work unless explicitly
  requested.
- Keep project-specific facts in repo docs, not in chat history.

## Testing instructions

- Do not claim done without fresh verification evidence.
- Add or update focused tests for changed command construction, security
  boundaries, pins, or docs routing when practical.
- Record skipped checks with reason and risk in `current-state.md` when the
  skip changes restart state, blockers, trusted verification, or the next step.
- Definition Of Done: target behavior or documentation change is complete,
  acceptance criteria are satisfied, relevant repo-owned checks ran, local
  Markdown links resolve, harness report/audit were considered when available
  for harness changes, and the next session can restart from the harness files.
- End of Session: update `current-state.md` with current state, verification
  evidence, blockers, touched files, and the recommended next step. Use
  `docs/harness/state/clean-state-checklist.md` before claiming the session is
  complete.

## Security considerations

- People run this on personal machines. Optimize for the most secure
  beginner-safe path first.
- Never mount host home directories, cloud credential folders, raw SSH keys,
  browser profiles, or arbitrary host environment variables by default.
- Do not relax default container isolation, mount exclusions, non-root runtime,
  read-only root filesystem, capability drops, or explicit env passthrough
  unless the user explicitly asks for that security tradeoff.
- Do not imply a boundary is enforced until code, tests, or live Apple
  `container` behavior prove it.
- Fail closed or state limitations plainly when a boundary cannot be verified.
- All package, image, tool, and CI action dependencies must use the current
  stable release and exact pins. GitHub Actions use full-length commit SHAs
  with version comments.
- If a runtime, package, policy, CVE, release, or vendor claim affects a change,
  verify it from current primary sources and update `docs/RESEARCH.md` or
  `docs/harness/research/sources.md`.
- Do not commit secrets, credentials, private data, machine-specific paths, or
  long raw command output.
