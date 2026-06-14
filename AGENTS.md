# Repository Instructions

This project builds a Python 3.13+ CLI for running AI coding agents inside
Apple `container` on macOS 26+.

## Defaults

- People will run untrusted code on personal machines through this project.
  Optimize for the most secure beginner-safe path first. If a boundary cannot
  be verified, fail closed or state the limitation plainly; do not trade away
  user safety for convenience.
- Keep runtime dependencies at zero unless a dependency removes real security
  or usability risk.
- Prefer `argparse`, `dataclasses`, `pathlib`, `subprocess`, and `unittest`
  before adding packages.
- Treat command generation as security-sensitive. Tests must cover mounts,
  environment passthrough, user identity, network mode, and dry-run output.
- Never mount host home directories, cloud credential folders, raw SSH keys, or
  arbitrary host environment variables by default.
- Use exact argument lists for subprocesses. Do not use shell strings for
  commands that execute.
- Keep docs direct and beginner-safe. Do not imply a boundary is enforced until
  code or the Apple container runtime actually enforces it.
- Keep `docs/RESEARCH.md` current for runtime, package, security, and agent
  behavior sources. Record volatile source checks with dates.
- All package, image, tool, and CI action dependencies must use the current
  stable release and be hard-pinned. Do not use floating ranges, mutable
  `latest` tags, major-only action refs, unversioned installer scripts, or
  unpinned package installs.

## Verification

Run the focused checks before claiming completion:

```bash
python3 -m compileall src tests
PYTHONPATH=src python3 -m unittest discover -s tests
python3 scripts/check_pins.py
```
