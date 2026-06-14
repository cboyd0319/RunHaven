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
| 2026-06-14 | Provider egress preparation | Playwright-rendered Apple DocC review, complete user-supplied DocC snapshot review, focused fail-closed tests, `PYTHON=<temporary-venv-python> ./init.sh`, Python 3.13 unit suite, doctor smoke, and HarnessForge audit | passed | Added reserved `--network provider` mode that failed closed at that stage, explicit plan egress status, and docs stating internet mode remained unrestricted until enforcement was proven. The complete snapshot covered 1,022 rendered Markdown pages plus raw JSON with zero fetch failures and no exact hits for egress or allowlist control terms. |
| 2026-06-14 | Provider egress proxy smoke | allowlist proxy unit tests, `PYTHON=<temporary-venv-python> ./init.sh`, Python 3.13 unit suite, default host live smoke, and `api.openai.com` live smoke | passed | Proved the host CONNECT proxy pattern on an internal Apple `container` network: allowed proxied HTTPS succeeded; denied proxied host, proxied IP literal, direct DNS, and direct IP paths failed. At that stage, normal `--network provider` runs remained fail-closed until lifecycle integration landed. HarnessForge audit was skipped for this pass by user instruction while the sibling repo was being worked on. |
| 2026-06-14 | Provider wording cleanup | focused CLI tests, ruff, mypy, and diff check | passed | Aligned CLI help and roadmap wording with the smoke-proven but not yet normal-run-integrated provider proxy state. |
| 2026-06-14 | Provider runtime integration | focused lifecycle tests, `PYTHON=<temporary-venv-python> ./init.sh`, Python 3.13 unit suite, and live `runhaven run shell --network provider --provider-host example.com` smoke | passed | Integrated the proxy lifecycle into normal provider runs. The live run allowed proxied HTTPS and denied proxied host, proxied IP literal, direct DNS, and direct IP paths; cleanup left no provider network or test state volume. |
| 2026-06-14 | Provider host guard cleanup | focused planner and CLI tests, `PYTHON=<temporary-venv-python> ./init.sh`, Python 3.13 unit suite, ruff, and mypy | passed | Rejected single-label provider hosts such as `com` so explicit provider additions must be fully qualified. Docs now state that a listed host permits that host and its subdomains. |

Rules:

- Record command name, scope, result, and risk.
- Do not paste secrets, local absolute paths, or long command output.
- Prefer one current row per meaningful verification event.
