# Repository Instructions

This project builds a Python 3.13+ CLI for running AI coding agents inside
Apple `container` on macOS 26+.

## Project Shape

- CLI code lives in `src/macos_container_agents/`.
- Container image templates live in `src/macos_container_agents/images/`.
- Runtime and dependency pins live in `pins.toml`, `requirements-dev.txt`, npm
  lockfiles, and `src/macos_container_agents/images/common/`.
- Human-facing docs live in `README.md` and `docs/`.
- Research sources and volatile version checks live in `docs/RESEARCH.md`.

## Security Boundaries

- People run this on personal machines. Optimize for the most secure
  beginner-safe path first.
- Never mount host home directories, cloud credential folders, raw SSH keys,
  browser profiles, or arbitrary host environment variables by default.
- Do not relax default container isolation, mount exclusions, non-root runtime,
  read-only root filesystem, capability drops, or explicit env passthrough
  unless the user explicitly asks for that security tradeoff.
- Keep command generation security-sensitive. Use exact subprocess argument
  lists, not executable shell strings.
- Do not imply a boundary is enforced until code, tests, or live Apple
  `container` behavior prove it.
- Fail closed or state limitations plainly when a boundary cannot be verified.

## Dependency And Pinning Rules

- All package, image, tool, and CI action dependencies must use the current
  stable release and be hard-pinned.
- Do not use floating version ranges, mutable `latest` tags, major-only GitHub
  Action refs, unversioned installer scripts, or unpinned package installs.
- If a runtime, package, policy, CVE, release, or vendor claim affects a change,
  verify it from current primary sources and update `docs/RESEARCH.md`.
- Keep runtime dependencies at zero unless a dependency removes real security or
  usability risk.

## Implementation Rules

- Prefer standard library tools: `argparse`, `dataclasses`, `pathlib`,
  `subprocess`, `unittest`, and structured data APIs.
- Match local code style and file structure. Avoid broad refactors unless they
  are required for the task.
- Add comments only where they explain a non-obvious security or runtime
  decision.
- Preserve existing user changes. Never revert dirty work unless explicitly
  requested.
- Use `rg` for repository searches. Keep command output bounded when possible.
- Use `apply_patch` for manual edits.

## Documentation Rules

- Keep `README.md` as the human front door: what this does, why it is useful,
  how to get started, what is safe by default, where the limits are, and where
  to get help.
- Keep `AGENTS.md` operational and concise. Do not duplicate long human docs;
  point to the durable docs instead.
- Keep `CLAUDE.md`, `GEMINI.md`, and `.github/copilot-instructions.md` as thin
  compatibility shims over `AGENTS.md`.
- Use direct language. No AI attribution, emojis, or em dashes in docs,
  comments, commits, or PR text.

## Verification

For code or command-generation changes, run:

```bash
python3 -m compileall src tests scripts
PYTHONPATH=src python3 -m unittest discover -s tests
python3 scripts/check_pins.py
```

For docs-only changes, run:

```bash
python3 scripts/check_pins.py
git diff --check
```

Broaden verification when the change affects image builds, runtime boundaries,
packaging, CI, or user-facing security behavior.
