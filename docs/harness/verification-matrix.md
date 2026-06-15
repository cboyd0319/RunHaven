# Verification Matrix

Use the smallest check set that can catch likely regressions for the change.

| Change Type | Required Checks |
| --- | --- |
| Harness docs or templates | `PYTHONPATH=../HarnessForge/src python3 -m harnessforge audit --target .` and inspect generated diff |
| README, AGENTS, or docs-only changes | `python3 scripts/check_pins.py`, `git diff --check`, local Markdown link check |
| Python code | `python3 -m compileall src tests scripts`, `PYTHONPATH=src python3 -m unittest discover -s tests`, `python3 -m ruff check .`, `python3 -m mypy src` |
| Packaging | Python code checks plus `python3 -m build` |
| CLI command construction | Python code checks plus focused tests in `tests/test_plans.py` or `tests/test_cli.py` |
| Apple container runtime boundary | Python code checks plus `runhaven doctor`, `runhaven plan`, and a focused `runhaven run shell` smoke |
| Auth broker runtime boundary | Python code checks plus focused auth broker tests, `runhaven auth log`, and `scripts/codex_broker_smoke.py --require-api-key` when a disposable key is available |
| Run observability | Python code checks plus focused `tests/test_cli.py` run-record tests and manual `runhaven runs list/show` checks |
| Image templates | Pin check, image build for changed profile, and version smoke for affected agent |
| Node or web code | package manager `test`, `lint`, `typecheck`, or `build` scripts when present |
| Go code | `go test ./...` |
| Rust code | `cargo test` |
| Java code | `mvn test`, `./gradlew test`, or project equivalent |
| .NET code | `dotnet test` |
| Terraform or infrastructure | `terraform fmt -check -recursive` plus project-specific validation |
| Security, auth, secrets, data loss, or billing | Focused tests, human review, rollback path, and least-privilege check |
| Dependencies, tool versions, or workflow Actions | Pin check, primary-source version evidence, install smoke, and affected tests |

## Detected Commands

- `python3 -m compileall src tests scripts`
- `PYTHONPATH=src python3 -m unittest discover -s tests`
- `python3 scripts/check_pins.py`
- `python3 -m ruff check .`
- `python3 -m mypy src`
- `python3 -m build`

## When Checks Cannot Run

Record:

- The command that could not run.
- The exact reason.
- The risk.
- The next best check that did run.

## Test Integrity

Generated tests must state test intent through the test name or assertions.
Do not count assertion-free tests, import-only tests, or stubbed tests as
behavioral coverage unless the evidence log records that limited purpose.
