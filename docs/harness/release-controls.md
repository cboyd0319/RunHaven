# Release Controls

Status: draft

Release work is security-sensitive because users run this project on personal
machines. A release cannot rely on chat history or unstated local setup.

## Required Gates

- Run `./init.sh` and the harness audit before release.
- Run `mca doctor` and focused Apple `container` smoke checks for any runtime
  boundary change.
- Verify package, image, GitHub Action, Debian, npm, Python, and Apple
  `container` pins from primary sources before changing release artifacts.
- Produce or review an SBOM before publishing installable artifacts once
  release packaging exists.
- Record provenance for built artifacts: source commit, build command, builder,
  pins, checksums, signing status, and release operator.
- Do not publish from a dirty tree.

## Rollback

- Keep the previous reviewed tag or commit available.
- Revert the release commit, yank the artifact, or publish a pinned corrective
  release when a safety boundary, secret-handling path, or dependency pin is
  wrong.
- Document affected versions, risk, recovery steps, and user impact in
  `docs/harness/evidence-log.md` and `SECURITY.md` when applicable.
