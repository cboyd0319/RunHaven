# AGENTS.md

## Project

RunHaven is a Rust CLI, with an alpha Tauri/Svelte desktop shell, for running
AI coding agents inside Apple `container` on macOS 26+ on Apple silicon.

The product safety boundary matters more than convenience. Do not mount host
home directories, cloud credential folders, raw SSH keys, browser profiles, or
arbitrary host environment variables by default. Do not relax container
isolation, non-root runtime, mount exclusions, read-only root filesystem,
capability drops, or explicit environment passthrough without a user-approved
security tradeoff and focused verification.

## Startup

1. Confirm the working directory and inspect git state:

```bash
pwd
git status --short --branch
```

2. Read these startup files only:

- `AGENTS.md`
- `feature_list.json`
- `current-state.md`

`current-state.md` is this repo's progress and handoff file. Do not recreate
separate root `progress.md` or `session-handoff.md` files.

3. Load more context only when the task needs it:

- Product, install, usage, or public docs: `README.md` and relevant `docs/`.
- Security boundary changes: `docs/SECURITY_MODEL.md` and focused tests.
- CLI, image, provider, Tauri, or frontend changes: inspect that component's
  manifests, tests, and local modules first.
- Harness maintenance: `.agents/skills/harness/SKILL.md` and
  `docs/harness/README.md`.

## Harness Contract

Keep the harness small and useful:

- Instructions: this file is a map, not a manual.
- Tools: shell, file edits, git, and `init.sh` are enough for normal work.
- Environment: versions and pins live in manifests, lockfiles, and `pins.toml`.
- State: `feature_list.json` plus `current-state.md` record status and next
  steps.
- Feedback: use explicit checks before claiming completion.

If a harness file is not needed for the current task, do not read it at startup.
If a harness rule keeps causing context cost without preventing failures,
delete or compress it.

## Verification

Use the smallest reliable check set for the change.

Focused checks:

```bash
cargo fmt --check
cargo test --locked
cargo clippy --all-targets -- -D warnings
cargo run --locked --bin runhaven-check-pins
cargo build --locked
npm --prefix ui run check
npm --prefix ui test
npm --prefix ui run test:e2e
npm --prefix ui run build
git diff --check
```

Full local verification on macOS 26+:

```bash
./init.sh
```

Use `runhaven doctor` and Apple `container` smokes only when changes affect the
actual runtime boundary, image templates, provider behavior, install flow, or
Tauri launch/run-control behavior.

## Working Rules

- Prefer the smallest correct change: no change, deletion, documentation,
  configuration, standard library, native platform behavior, existing
  dependency, then minimum custom code.
- Match local style and helper APIs.
- Use exact subprocess argument lists, not executable shell strings, for
  runtime command generation.
- Keep direct dependencies exact-pinned and minimal.
- Preserve user changes. Never revert dirty work unless explicitly requested.
- Use `rg` for repository searches and keep noisy output bounded.
- Use `apply_patch` for manual edits.
- Keep project-specific facts in repo docs, not chat history.
- Do not add Windows or Linux runtime or contributor-verification targets.

## Definition Of Done

- Target behavior or documentation change is complete.
- Relevant checks ran, or skipped checks are named with reason and risk.
- Security, data-loss, accessibility, and platform-parity requirements were not
  weakened.
- `feature_list.json` and `current-state.md` reflect any changed active state.
- The next session can restart from the three startup files above.
