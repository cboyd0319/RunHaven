# Verification Matrix

Use the smallest check set that can catch likely regressions for the change.
Escalate to `./init.sh` when a change touches shared behavior, release
readiness, runtime boundaries, or multiple components.

## Required Checks

| Change Type | Required Checks |
| --- | --- |
| Harness docs, instructions, state, or templates | `cargo run --locked --bin runhaven-check-pins`, JSON validation, local Markdown link check, platform wording scan, and `git diff --check`; run HarnessForge report/audit as advisory checks when available |
| README, AGENTS, or docs-only changes | `cargo run --locked --bin runhaven-check-pins`, local Markdown link check, platform wording scan, and `git diff --check` |
| Feature state, manifest, or schema changes | JSON validation for changed JSON files, relevant Rust tests, and advisory HarnessForge report when available |
| Rust code | `cargo fmt --check`, focused `cargo test` targets, `cargo test --locked`, and `cargo clippy --all-targets -- -D warnings` |
| Frontend UI | `npm --prefix ui run check`, `npm --prefix ui test`, `npm --prefix ui run build`, relevant Tauri command tests, and `git diff --check` |
| Tauri desktop shell | Frontend UI checks plus `cargo fmt --manifest-path src-tauri/Cargo.toml --check`, `cargo test --manifest-path src-tauri/Cargo.toml --locked`, `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --locked -- -D warnings`, `npm --prefix ui run tauri:build`, pin policy, and capability review |
| Packaging | Rust code checks plus `cargo build --locked` |
| CLI command construction | Rust code checks plus focused CLI and planning tests |
| Setup and prerequisite UX | Rust code checks plus focused setup, workspace, credential, network guidance, and doctor tests; add a manual `runhaven setup --agent shell` smoke when user-facing text changes |
| Image and network repair UX | Rust code checks plus focused image/network/state tests, `runhaven image doctor shell`, `runhaven image build shell --dry-run`, `runhaven image rebuild --dry-run`, `runhaven network list`, and `runhaven network prune` smokes |
| Apple `container` runtime boundary | Rust code checks plus `runhaven doctor`, `runhaven plan`, and a focused `runhaven run shell` smoke that proves the claimed mount, user, network, or filesystem behavior; use `scripts/apple_container_smoke.sh` for pre-Tauri and release local runtime evidence |
| Provider egress or endpoint policy | Focused egress, provider proxy, provider endpoint, and plan tests; source review for endpoint changes; run `scripts/apple_container_smoke.sh --with-provider` when behavior changes and the host/runtime are available |
| SSH forwarding boundary | Planner and CLI tests that prove `--ssh` fails closed; run `scripts/apple_container_smoke.sh --with-ssh` when SSH-forwarding guard behavior changes and the host/runtime are available |
| Auth broker boundary | Focused auth broker and provider Codex broker tests, `runhaven auth status`, `runhaven auth explain codex`, and `runhaven auth log`; use a disposable-key live smoke only when one is explicitly available |
| Run observability | Focused run-history, active-run, repair, and git metadata tests; manual `runhaven runs list/show/log/diff/active/status/attach/logs-follow/stop/kill/repair` smokes when behavior changes |
| Session and state management | Focused planner, standard-run, and state tests for `--session`, active/run-record metadata, `state list --session`, `state reset`, and session-filtered `state prune` |
| Worktree run isolation and lifecycle | Focused standard-run and worktree-lifecycle tests for `--worktree`, dirty-source guidance, clean-source enforcement, isolated mounts, recovery metadata, project check suggestions, `runs keep`, `runs recover`, `runs merge`, and `runs discard` |
| Image templates | Pin check, focused image tests, `runhaven image build PROFILE --dry-run`, and a real image build/version smoke for changed profiles when Apple `container` is available |
| Pin, dependency, runner, or workflow changes | `cargo run --locked --bin runhaven-check-pins`, primary-source version evidence, and affected tests. If workflows are reintroduced, review every workflow file for macOS 26+ only support and immutable Action refs |
| Security, auth, secrets, data loss, or billing | Focused tests, human review, rollback path, least-privilege check, and evidence in `docs/harness/evidence/evidence-log.md` |
| Release prep | `./init.sh`, harness report/audit, `runhaven doctor`, relevant Apple `container` smokes, dirty-tree check, pin/source review, SBOM/provenance review when packaging exists |

## Detected Commands

- `cargo fmt --check`
- `cargo test --locked`
- `cargo clippy --all-targets -- -D warnings`
- `cargo run --locked --bin runhaven-check-pins`
- `npm --prefix ui run check`
- `npm --prefix ui test`
- `npm --prefix ui run build`
- `npm --prefix ui run tauri:build`
- `cargo build --locked`

## Advisory HarnessForge Review Commands

These commands are read-only unless a target-relative report path is supplied.
Use them as structural signals, not as the sole source of truth for RunHaven
while HarnessForge is under active development:

```bash
harnessforge index --target . --json
harnessforge session --target .
harnessforge report --target .
harnessforge plan --target . --since HEAD
harnessforge enhance --target .
harnessforge audit --target . --min-score 85
```

Use `harnessforge verify --target . --json --run --json-report
docs/harness/evidence/verify-YYYY-MM-DD.json` only when the project is ready to
record command-execution evidence. Do not confuse structural harness audit with
real-agent effectiveness.

## When Checks Cannot Run

Record:

- The command that could not run.
- The exact reason.
- The risk.
- The next best check that did run.
- The follow-up needed to close the evidence gap.

## Test Integrity

Generated tests must state test intent through the test name or assertions.
Do not count assertion-free tests, import-only tests, or stubbed tests as
behavioral coverage unless the evidence log records that limited purpose.

## Agent-Oriented Failure Messages

Prefer failure output that states:

- what failed;
- why the RunHaven boundary matters;
- where the agent should look first to repair it;
- which command or evidence should be recorded after repair.
