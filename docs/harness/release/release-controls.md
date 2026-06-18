# Release Controls

Status: live

Release work is security-sensitive because users run this project on personal
machines. A release cannot rely on chat history or unstated local setup.

## Required Gates

- Confirm the secure path is the easiest shipped workflow. Supported
  lower-security choices must warn and require explicit intent; unsupported or
  hard-boundary violations still fail closed.
- Run `./init.sh` and the relevant repo-owned docs, pin, and policy checks
  before release. Optional structural reports may inform review, but they are
  not the release gate.
- Run `runhaven doctor` and focused Apple `container` smoke checks for any runtime
  boundary change.
- Verify package, image, GitHub Action, Debian, npm, Rust/Cargo, and Apple
  `container` pins from primary sources before changing release artifacts.
- Confirm touched files, modules, crates, Tauri commands, frontend components,
  and harness docs remain cohesive, non-duplicative, and reasonably reviewable.
- Use `apple-container-update-playbook.md` for Apple `container` runtime,
  helper image, installer, and Kata kernel pin updates.
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
  `docs/harness/evidence/evidence-log.md` and `SECURITY.md` when applicable.
