# Evidence Log

Use this for compact current evidence. Keep raw logs out of this file.

| Date | Scope | Command Or Review | Result | Notes |
| --- | --- | --- | --- | --- |
| 2026-06-14 | Harness initialization | `harnessforge init --target .` | passed | Existing `AGENTS.md` was preserved; missing harness files were created. |
| 2026-06-14 | Harness audit | `PYTHONPATH=../HarnessForge/src python3 -m harnessforge audit --target . --min-score 85` | passed | Reported 100/100 after the macOS-only correction. |
| 2026-06-14 | POSIX entrypoint | `PYTHON=<temporary-venv-python> ./init.sh` | passed | Ran compileall, unit tests, pin policy, ruff, mypy, and build. |
| 2026-06-14 | macOS-only support | source and docs review | passed | Removed the non-macOS verification entrypoint and unsupported platform claims. |
| 2026-06-14 | Project logo | `magick identify docs/assets/logo.png` plus visual inspection | passed | Tracked logo asset is a stripped 512x512 PNG used by `README.md`. |
| 2026-06-14 | Harness audit | `PYTHONPATH=../HarnessForge/src python3 -m harnessforge audit --target . --min-score 85` | passed | Current score is 100/100 after removing non-macOS verification surfaces. |
| 2026-06-14 | RunHaven rename | source checks, temporary-venv static checks, build, wheel smoke, no-ignore old-name scan, harness audit | passed | Package, module, command, image tags, docs, tests, and harness state use RunHaven/`runhaven`; ignored local virtualenvs were removed because they encoded stale checkout paths. |
| 2026-06-14 | Runtime hardening and macOS-only boundary | unit tests, static checks, build, `./init.sh`, harness audit, `runhaven doctor`, `runhaven state list`, internal-network smoke, and `runhaven plan` smoke | passed | Command validation, unsafe overrides, TTY controls, state commands, host-only internal network creation, and macOS 26+ only verification are covered. |
| 2026-06-14 | Follow-up hardening pass | focused unit tests, full unit suite, and pin check | passed | Added root group rejection, parser help cwd safety, dynamic image template pin discovery, and run/doctor edge-case coverage. |
| 2026-06-14 | Cleanup pass | stale-reference scan, pin check, JSON validation, diff check, and HarnessForge audit | passed | Removed stale local paths, stale local-venv evidence, and old HarnessForge predecessor references from tracked docs. |
| 2026-06-14 | Second follow-up hardening pass | sandboxed Antigravity audit, `PYTHON=<temporary-venv-python> ./init.sh`, Python 3.13 unit suite, help/plan/doctor smokes, HarnessForge audit, cleanup scans | passed | Added fail-closed network mode validation, leading-zero root identity rejection, sensitive macOS system path blocking, doctor remedies, agent-argument help, and macOS-only pin-ledger enforcement. |
| 2026-06-14 | Provider egress preparation | Playwright-rendered Apple DocC review, generated DocC JSON review, focused fail-closed tests, `PYTHON=<temporary-venv-python> ./init.sh`, Python 3.13 unit suite, doctor smoke, and HarnessForge audit | passed | Added reserved `--network provider` mode that fails closed, explicit plan egress status, and docs stating internet mode remains unrestricted until enforcement is proven. |

Rules:

- Record command name, scope, result, and risk.
- Do not paste secrets, local absolute paths, or long command output.
- Prefer one current row per meaningful verification event.
