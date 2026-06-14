# AGENTS.md

## Project overview

RunHaven is a Python 3.13+ CLI for running AI coding agents inside
Apple `container` on macOS 26+.

Startup path:

1. Confirm the working directory and inspect `git status --short --branch`.
2. Read this file, `README.md`, and `docs/harness/README.md`.
3. Read `feature_list.json`, `progress.md`, and `session-handoff.md`.
4. Check `docs/harness/component-inventory.md` before changing CLI modules,
   image templates, verification routing, or harness files.
5. Pick one current objective before editing.

This repo is harnessed. Keep root instructions short and place durable detail
in `docs/harness/`.

## Build and test commands

Use the smallest reliable command for the change.

Verification route: use `./init.sh` or `.\init.ps1` for full harness
verification, and use the focused commands below for smaller changes.

Full local verification on macOS or Linux:

```bash
./init.sh
```

Full local verification on Windows import/docs surfaces:

```powershell
.\init.ps1
```

Focused checks:

```bash
python3 -m compileall src tests scripts
PYTHONPATH=src python3 -m unittest discover -s tests
python3 scripts/check_pins.py
python3 -m ruff check .
python3 -m mypy src
python3 -m build
REPO_HARNESS_CREATOR=../repo-harness-creator
PYTHONPATH="${REPO_HARNESS_CREATOR}/src" python3 -m repo_harness_creator audit --target . --min-score 85
```

Use `runhaven doctor` and Apple `container` runtime smokes when changes affect the
actual macOS container boundary, image templates, agent profiles, or install
flow.

## Code style guidelines

- Prefer standard library tools: `argparse`, `dataclasses`, `pathlib`,
  `subprocess`, `unittest`, and structured data APIs.
- Match local code style and file structure. Avoid broad refactors unless they
  are required for the task.
- Keep runtime dependencies at zero unless a dependency removes real security or
  usability risk.
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
- Record skipped checks with reason and risk in `progress.md` or
  `session-handoff.md`.
- Definition Of Done: target behavior or documentation change is complete,
  acceptance criteria are satisfied, relevant checks ran, local Markdown links
  resolve, harness audit passes the threshold, and the next session can restart
  from the harness files.
- End of Session: update `progress.md` and `session-handoff.md` with current
  state, verification evidence, blockers, touched files, and the recommended
  next step. Use `docs/harness/clean-state-checklist.md` before claiming the
  session is complete.

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
  `docs/harness/sources.md`.
- Do not commit secrets, credentials, private data, machine-specific paths, or
  long raw command output.
