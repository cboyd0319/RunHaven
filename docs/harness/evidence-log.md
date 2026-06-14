# Evidence Log

Use this for compact current evidence. Keep raw logs out of this file.

| Date | Scope | Command Or Review | Result | Notes |
| --- | --- | --- | --- | --- |
| 2026-06-14 | Harness initialization | `repo_harness_creator init --target .` | passed | Existing `AGENTS.md` was preserved; missing harness files were created. |
| 2026-06-14 | Harness audit | `PYTHONPATH=../repo-harness-creator/src python3 -m harnessforge audit --target . --min-score 85` | passed | Reported 100/100 after the macOS-only correction. |
| 2026-06-14 | POSIX entrypoint | `PYTHON=.venv314/bin/python ./init.sh` | passed | Ran compileall, 17 unit tests, pin policy, ruff, mypy, and build. |
| 2026-06-14 | macOS-only support | source and docs review | passed | Removed the non-macOS verification entrypoint and unsupported platform claims. |
| 2026-06-14 | Project logo | `magick identify docs/assets/logo.png` plus visual inspection | passed | Tracked logo asset is a stripped 512x512 PNG used by `README.md`. |
| 2026-06-14 | Harness audit | `PYTHONPATH=../repo-harness-creator/src python3 -m harnessforge audit --target . --min-score 85` | passed | Current score is 100/100 after removing non-macOS verification surfaces. |
| 2026-06-14 | RunHaven rename | source checks, temporary-venv static checks, build, wheel smoke, no-ignore old-name scan, harness audit | passed | Package, module, command, image tags, docs, tests, and harness state use RunHaven/`runhaven`; ignored local virtualenvs were removed because they encoded stale checkout paths. |
| 2026-06-14 | Runtime hardening and macOS-only boundary | unit tests, static checks, build, `./init.sh`, harness audit, `runhaven doctor`, `runhaven state list`, internal-network smoke, and `runhaven plan` smoke | passed | Command validation, unsafe overrides, TTY controls, state commands, host-only internal network creation, and macOS 26+ only verification are covered. |
| 2026-06-14 | Follow-up hardening pass | focused unit tests, full unit suite, and pin check | passed | Added root group rejection, parser help cwd safety, dynamic image template pin discovery, and run/doctor edge-case coverage. |

Rules:

- Record command name, scope, result, and risk.
- Do not paste secrets, local absolute paths, or long command output.
- Prefer one current row per meaningful verification event.
