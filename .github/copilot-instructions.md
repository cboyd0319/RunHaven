# macos-container-agents Copilot Instructions

Use [../AGENTS.md](../AGENTS.md) as the source of truth for repo guidance.

This file exists because GitHub Copilot reads
`.github/copilot-instructions.md`. Keep it short and point to durable docs
instead of duplicating project manuals.

## Required Reading For Non-Trivial Work

- [Architecture](../docs/ARCHITECTURE.md)
- [Security model](../docs/SECURITY_MODEL.md)
- [Pinning policy](../docs/PINNING.md)
- [Research and source ledger](../docs/RESEARCH.md)
- [Usage](../docs/USAGE.md)
- [Contributing](../CONTRIBUTING.md)

## Core Rules

- This repo builds a Python 3.13+ CLI for running AI coding agents inside
  Apple `container` on macOS 26+.
- People will run untrusted code on personal machines through this project.
  User safety is the product. If a boundary cannot be verified, fail closed or
  state the limitation plainly.
- Optimize for zero-technical-knowledge users. Security controls must be easy,
  visible, and hard to accidentally weaken.
- Never mount host home directories, cloud credential folders, raw SSH keys, or
  arbitrary host environment variables by default.
- Keep host secrets out of generated commands unless the user explicitly passes
  a variable name with `--env`.
- Treat command generation as security-sensitive. Tests must cover mounts,
  environment passthrough, user identity, network mode, and dry-run output.
- Use exact subprocess argument lists. Do not use shell strings for commands
  that execute.
- Keep runtime dependencies at zero unless a dependency removes real security
  or usability risk.
- All package, image, tool, and CI action dependencies must use the current
  stable release and be hard-pinned. Do not use floating ranges, mutable
  `latest` tags, major-only action refs, unversioned installer scripts, or
  unpinned package installs.
- Update relevant docs with behavior, setup, architecture, command, security,
  or pinning changes.
- Run the smallest relevant verification set before marking work complete.
