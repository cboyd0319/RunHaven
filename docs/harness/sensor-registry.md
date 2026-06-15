# Sensor Registry

Status: live

This registry records the checks and signals agents and humans use to decide
whether work is ready. A sensor is a signal, not a guarantee. It does not prove
real-agent effectiveness.

## Registered Sensors

| Sensor | Source | Purpose | Owner | Retire When | Review Cadence |
| --- | --- | --- | --- | --- | --- |
| `python3 -m compileall src tests scripts` | Python standard library | Catch syntax/import compilation errors across runtime, tests, and scripts | Maintainers | Replace only if a stronger standard compile step covers the same files | Python source, test, or script changes |
| `PYTHONPATH=src python3 -m unittest discover -s tests` | Project test suite | Catch behavior regressions across CLI, planning, egress, auth, images, state, worktrees, and repo policy | Maintainers | Replace only if the test runner changes across the whole repo | Any code behavior change; full verification before release |
| `python3 scripts/check_pins.py` | `pins.toml`, package locks, image templates, workflow pins | Enforce exact package, image, runner, npm, Debian, and Action pin policy | Maintainers | Replace if pin policy moves into a stronger packaged command | Dependency, image, workflow, release, or docs changes that mention pins |
| `python3 -m ruff check .` | `pyproject.toml` ruff config | Catch style, import, and lint regressions | Maintainers | Replace if the linter changes by accepted project decision | Python source/test/script changes |
| `python3 -m mypy src` | `pyproject.toml` strict mypy config | Catch type regressions in runtime package code | Maintainers | Replace if type-checking policy changes by accepted project decision | Runtime Python source changes |
| `python3 -m build` | Python packaging build backend | Verify package metadata and build artifacts can be produced | Maintainers | Replace when release packaging flow changes | Packaging, manifest, dependency, or release changes |
| `./init.sh` | macOS local verification entrypoint | Run the full macOS harness verification set in one command | Maintainers | Replace if the repo adopts a different full local verification entrypoint | Release prep, broad changes, or shared behavior changes |
| `python3 -m harnessforge report --target .` | HarnessForge unified report | Advisory structural signal for readiness, audit, drift, index, evidence, first-agent task, and platform contract without running target commands | Maintainers | Replace if report evidence moves into a project-owned release command | Harness, release-prep, state, or docs changes when HarnessForge is available |
| `python3 -m harnessforge audit --target . --min-score 85` | HarnessForge structural audit | Advisory structural check for the repo harness floor; repo-owned docs, tests, and maintainer decisions remain authoritative | Maintainers | Replace if a project-owned harness audit supersedes it | Harness changes and release prep when HarnessForge is available |
| Local Markdown link check | One-off local script or reviewer command over tracked Markdown files | Confirm target-relative doc links resolve after docs changes | Maintainers | Replace if a packaged docs checker is added | Docs, README, harness docs, and roadmap changes |
| Platform wording scan | `rg` over docs/state for unsupported platform claims | Preserve macOS 26+ only runtime and contributor verification | Maintainers | Replace if platform contract expands by accepted source-backed decision | Docs, CI, install, runtime, or manifest changes |
| Provider egress smoke | `scripts/provider_egress_smoke.py` | Prove provider proxy allow/deny behavior on Apple `container` internal network | Maintainers | Replace if provider network architecture changes | Provider proxy, endpoint, DNS, or network runtime changes |
| Codex broker smoke | `scripts/codex_broker_smoke.py` | Prove optional Codex API-key broker path with a disposable key | Maintainers | Replace if broker protocol or supported providers change | Auth broker changes and release prep when disposable key is available |
| `runhaven doctor` | Runtime prerequisite checker | Confirm host prerequisites and remediation guidance | Maintainers | Replace if setup/doctor contract changes | Runtime, install, setup, or Apple `container` boundary changes |
| `runhaven plan ...` | CLI planning command | Preview workspace, image, user, network, state, broker, and command boundary before mutation | Maintainers | Replace if planner output contract changes | CLI command construction, security, network, workspace, and image changes |
| `runhaven image doctor [AGENT]` | Image diagnostic command | Detect missing or stale bundled images and inactive state-volume review without mutation | Maintainers | Replace if image lifecycle changes | Image build, image metadata, state review, or setup changes |

## Agent-Oriented Failure Feedback

For custom checks, prefer failure messages with:

- what failed;
- why the RunHaven security, data-loss, or platform boundary matters;
- where an agent should look first to repair it;
- which evidence should be recorded after the fix.

## Promotion Rules

- Do not run commands only because they appear here. Use repo-owned change
  type guidance first. HarnessForge planning or verification output is
  advisory while the tool is under active development.
- Keep owner, source, purpose, and retire conditions current before promoting
  a check into release or automation gates.
- Remove or replace sensors that no longer catch meaningful regressions.
- Structural audit score is a harness-health signal, not real-agent
  effectiveness evidence.
