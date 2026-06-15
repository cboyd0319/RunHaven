# Verification Matrix

Use the smallest check set that can catch likely regressions for the change.
Escalate to `./init.sh` when a change touches shared behavior, release
readiness, runtime boundaries, or multiple components.

## Required Checks

| Change Type | Required Checks |
| --- | --- |
| Harness docs, instructions, state, or templates | `python3 scripts/check_pins.py`, JSON validation, local Markdown link check, platform wording scan, and `git diff --check`; run HarnessForge report/audit as advisory checks when available |
| README, AGENTS, or docs-only changes | `python3 scripts/check_pins.py`, local Markdown link check, platform wording scan, and `git diff --check` |
| Feature state, manifest, or schema changes | JSON validation for changed JSON files, `python3 -m json.tool feature_list.json`, relevant repo-policy tests, and advisory HarnessForge report when available |
| Python code | `python3 -m compileall src tests scripts`, focused `unittest` modules, `PYTHONPATH=src python3 -m unittest discover -s tests`, `python3 -m ruff check .`, and `python3 -m mypy src` |
| Packaging | Python code checks plus `python3 -m build` |
| CLI command construction | Python code checks plus focused tests in `tests/test_plans.py`, `tests/test_cli.py`, or the relevant split CLI test module |
| Setup and prerequisite UX | Python code checks plus focused setup, workspace, credential, network guidance, and doctor tests; add a manual `runhaven setup --agent shell` smoke when user-facing text changes |
| Image and network repair UX | Python code checks plus focused image/network/state tests, `runhaven image doctor shell`, `runhaven image build shell --dry-run`, `runhaven image rebuild --dry-run`, `runhaven network list`, and `runhaven network prune` smokes |
| Apple `container` runtime boundary | Python code checks plus `runhaven doctor`, `runhaven plan`, and a focused `runhaven run shell` smoke that proves the claimed mount, user, network, or filesystem behavior |
| Provider egress or endpoint policy | Focused egress, provider proxy, provider endpoint, and plan tests; source review for endpoint changes; live provider smoke when behavior changes and the host/runtime are available |
| Auth broker boundary | Focused auth broker and provider Codex broker tests, `runhaven auth status`, `runhaven auth explain codex`, `runhaven auth log`, and `scripts/codex_broker_smoke.py --require-api-key` only with a disposable key |
| Run observability | Focused run-history, active-run, repair, and git metadata tests; manual `runhaven runs list/show/log/diff/active/status/attach/logs-follow/stop/kill/repair` smokes when behavior changes |
| Session and state management | Focused planner, standard-run, and state tests for `--session`, active/run-record metadata, `state list --session`, `state reset`, and session-filtered `state prune` |
| Worktree run isolation and lifecycle | Focused standard-run and worktree-lifecycle tests for `--worktree`, dirty-source guidance, clean-source enforcement, isolated mounts, recovery metadata, project check suggestions, `runs keep`, `runs recover`, `runs merge`, and `runs discard` |
| Image templates | Pin check, focused image tests, `runhaven image build PROFILE --dry-run`, and a real image build/version smoke for changed profiles when Apple `container` is available |
| Pin, dependency, runner, or workflow changes | `python3 scripts/check_pins.py`, primary-source version evidence, affected tests, and `.github/workflows/ci.yml` review for macOS 26+ only support |
| Security, auth, secrets, data loss, or billing | Focused tests, human review, rollback path, least-privilege check, and evidence in `docs/harness/evidence-log.md` |
| Release prep | `./init.sh`, harness report/audit, `runhaven doctor`, relevant Apple `container` smokes, dirty-tree check, pin/source review, SBOM/provenance review when packaging exists |

## Detected Commands

- `python3 -m compileall src tests scripts`
- `PYTHONPATH=src python3 -m unittest discover -s tests`
- `python3 scripts/check_pins.py`
- `python3 -m ruff check .`
- `python3 -m mypy src`
- `python3 -m build`

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
