# AGENTS.md

## Project

RunHaven is a Rust CLI, with a Tauri/React desktop shell prototype, for running
AI coding agents inside Apple `container` on macOS 26+ on Apple silicon.

The product safety boundary matters more than convenience. Do not mount host
home directories, cloud credential folders, raw SSH keys, browser profiles, or
arbitrary host environment variables by default. Do not relax container
isolation, non-root runtime, mount exclusions, read-only root filesystem,
capability drops, or explicit environment passthrough without a user-approved
security tradeoff and focused verification.

Above all else, the secure path must be the easy path. Secure defaults should
be the shortest, clearest workflow. Supported less-secure choices should warn
and require explicit intent, but should not be hidden or blocked only because
they are less secure. Unsupported, invalid, or hard-boundary violations still
fail closed.

Apple `container machine` is not the default RunHaven boundary, but explicit or
user-managed machine workflows are not blocked solely for being less secure. If
RunHaven adds machine integration, it must warn about host-home and credential
exposure, require explicit intent, and preserve focused verification.

## Startup

1. Confirm the working directory with `pwd` and inspect git state with
   `git status --short --branch`.

2. Read these startup files only:

- `AGENTS.md`
- `feature_list.json`
- `current-state.md`

`current-state.md` is this repo's progress and handoff file. Do not recreate
separate root `progress.md` or `session-handoff.md` files.

3. Before stacking new work on a surface that may already be broken, run its
   smallest check from `docs/harness/feedback/verification-matrix.md`. Fix a
   broken baseline before adding new changes.

4. Load more context only when the task needs it:

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
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo run --locked --bin runhaven-check-pins
cargo build --workspace --locked
npm --prefix ui run check
npm --prefix ui test
npm --prefix ui run test:e2e
npm --prefix ui run build
scripts/check-docs-local-paths.sh
scripts/check-repo-structure.sh
git diff --check
```

Full local verification on macOS 26+: `./init.sh`.

Use `runhaven doctor` and Apple `container` smokes only when changes affect the
actual runtime boundary, image templates, provider behavior, install flow, or
Tauri launch/run-control behavior.

## Working Rules

- All current and future development is DRY. Walk the build-necessity ladder
  and stop at the first rung that satisfies the request: (1) does it need to
  exist at all (YAGNI); (2) does the standard library do it; (3) does a native
  platform feature cover it (`<input type="date">` over a picker library, CSS
  over JS, a schema or DB constraint over app code); (4) does an
  already-installed dependency solve it, and never add a new one for what a few
  lines can do; (5) can it be one clear line. Then write the minimum custom
  code that works. `docs/harness/boundaries/change-contract.md` holds the full
  gate, including the security and correctness carve-outs.
- Documentation is product: if a behavior is not documented it does not exist.
  Ship the doc change in the same slice as the behavior.
- Never commit machine-local absolute paths or local external-source references
  in docs, startup state, repo-local skills, examples, or generated docs
  artifacts. Use repo-relative paths or public URLs; cite upstream source as a
  public repo plus pinned commit. If a local external document is needed, import
  or summarize it into `docs/` first, then cite the repo copy.
- Write UI and public docs for non-technical users at roughly an 8th grade
  reading level. Follow the [Apple Style Guide](https://support.apple.com/guide/applestyleguide/welcome/web)
  first, then [Google style](https://developers.google.com/style) and
  [inclusive guidance](https://developers.google.com/style/inclusive-documentation).
  Use active voice, exact control names, sentence-style capitalization, plain
  verbs, and short sentences. Alerts state the problem, consequence, and next
  action. Avoid idioms, ableist language, violent metaphors, and technical
  filler. Keep exact commands and security facts when needed. Apply Google's
  [release guidance](https://opensource.google/documentation/reference/releasing/preparing#inclusive) before release.
- Assume desktop users do not know container, Git, terminal, or network terms.
  Give each screen one task, disclose advanced choices in context, use a folder
  browser instead of path entry, show changed filenames, name actions by result,
  keep secure defaults easiest, show feedback, and provide recovery without dead ends.
- Before every GUI change, enumerate each touched window, destination, control,
  state, and interaction, then check its current official Apple HIG topic and
  update `docs/design/native-gui-uiux-audit.md`. Document security or Tauri
  deviations. Require macOS design source and packaged live review.
- TUI backlog rule: during the current CLI plus native GUI push, do not spend
  implementation time on `crates/runhaven-tui` or Codex TUI surfaces except to
  revert accidental edits, preserve workspace compilation after CLI/GUI
  changes, or update docs/state saying TUI is deferred. Do not expand, polish,
  refactor, test, or plan TUI features. TUI may be exercised only indirectly by
  repo-level checks when unavoidable.
- If explicit future TUI work reopens, restore the source-first rule: vendor or
  adapt from the pinned upstream Codex TUI source
  (`openai/codex@5267e805fb830891c0b23376bcd9cbd382c3473c:codex-rs/tui`) or
  repo-vendored Codex files before writing custom TUI code.
- Boring over clever: choose the obvious construct, because clever is what
  someone has to decode at 3am. Between two standard-library options of similar
  size, take the one correct on edge cases; lazy means writing less code, not
  picking the flimsier algorithm.
- Eliminate meaningful duplication everywhere. Prefer deletion or one small
  shared helper over copy/paste, but do not add speculative abstractions.
- Work hardest-first. Start each slice with the largest remaining risk or
  blocker, especially where security, real runtime behavior, or data loss can
  fail. Do smaller cleanup only after the hard path is proven.
- Think before coding: define success criteria, surface uncertainty, and name
  meaningful tradeoffs before implementation.
- Design workflows so the secure path is the default and easiest path; make
  supported less-secure paths explicit and warned.
- Match local style and helper APIs.
- Keep files, modules, crates, Tauri commands, and frontend components cohesive.
  `scripts/check-repo-structure.sh` is a hard gate: all file types have static
  limits, and splittable repo-owned files at 90% of a ceiling must be split by
  ownership now, not trimmed or deferred.
- Keep the Rust workspace organized by ownership boundary. `crates/runhaven`
  owns binary entrypoints only. `crates/runhaven-core` owns runtime, provider,
  records, image, doctor, diagnostics, support, harness, and shared UI
  contracts. `crates/runhaven-cli` owns Clap dispatch and human CLI
  presentation. `crates/runhaven-tui` owns the Codex-vendored terminal UI and
  RunHaven TUI adapters. `src-tauri` is a workspace member that depends on
  `runhaven-core` through typed commands. Do not recreate a root compatibility
  facade or put shared runtime truth in the CLI/TUI crates.
- Use exact subprocess argument lists, not executable shell strings, for
  runtime command generation.
- Keep direct dependencies, package manifests, runtime pins, and image package
  pins exact-pinned, minimal, and current stable. Lock transitive dependencies.
  Verify volatile version claims against current official sources before
  changing pins.
- Preserve user changes. Never revert dirty work unless explicitly requested.
- Use `rg` for repository searches and keep noisy output bounded.
- Use `apply_patch` for manual edits.
- Keep project-specific facts in repo docs, not chat history.
- Keep the repo harness updated when active state, release scope, verification
  routing, or operating rules change.
- If code, files, docs, config, dependencies, or harness surface do not need to
  exist, delete them.
- Do not add Windows or Linux runtime or contributor-verification targets.

## Specialist Routing

- For desktop design, UI, UX, or GUI work, use the installed macOS design skill
  and `macos-design-expert`; require its live and implementation review.
- For Rust work in this repo, use the installed `rust` skill.
- For non-trivial Rust implementation, review, debugging, or test-gate work,
  use the `rust-expert` and `rust-test-debug-architect` agents with bounded
  ownership, then verify their findings against live files.
- For security-sensitive changes, use `security-engineering`; use
  `adversarial-review` for major architecture or boundary claims before
  committing.
- Antigravity (`agy`) is research-only in this repo. Do not use it for
  end-of-slice code review, adversarial review, verification, or proof of
  correctness.
- For direct Codex CLI behavior, use `codex-cli-guide`, repo-vendored Codex
  files, and pinned upstream Codex source as evidence.
- For RunHaven TUI work, do nothing unless the user explicitly reopens TUI
  scope or a CLI/GUI change requires compile-only maintenance. If explicit TUI
  work reopens, use the repo-local `.agents/skills/codex-tui` skill first.
- For Codex-vendored TUI and `codex-*` dependencies, preserving the original
  Codex package name, crate name, and module path is the default. Use a local
  bridge only when compiling or activating the real Codex surface would cross a
  RunHaven security boundary that has not been designed and tested.

## Definition Of Done

- Target behavior or documentation change is complete.
- Any changed behavior ships its documentation in the same slice; an
  undocumented behavior is treated as not done.
- The DRY build-necessity ladder was applied: no higher rung (no change,
  deletion, documentation, standard library, native platform, installed
  dependency, one line) already covered the work.
- Relevant checks ran, or skipped checks are named with reason and risk.
- Security, data-loss, accessibility, and platform-parity requirements were not
  weakened.
- File size, modularity, duplication, dependency use, and crate/component
  organization were considered for touched surfaces.
- `feature_list.json` and `current-state.md` reflect any changed active state.
- The current Apple HIG was checked for every changed GUI surface, the audit was
  updated, and any packaged macOS verification gap is named.
- The next session can restart from the three startup files above.
