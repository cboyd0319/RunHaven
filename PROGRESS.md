# Progress

Last updated: 2026-07-14
Branch: `docs/native-gui-wireframe`
Active feature: `harness-final-canonical-review`
Status: `active`

## Current objective

Re-evaluate the complete repository harness against every atomic requirement in
the installed canonical `harness-engineering` skill. Verify every safely
testable local behavior from the current tree and a clean external snapshot,
retain explicit evidence for every disposition, and do not publish until no
material canonical issue or verification gap remains.

## Verified baseline

- The installed harness audit reports no investigation findings. This is a
  structural inventory result, not proof of compliance or effectiveness.
- Environment readiness passes seven adversarial fixtures. State semantics pass
  with 23 adversarial fixtures. File limits pass across 616 maintained
  source/config files with 22 adversarial fixtures. Ninety files at the review
  threshold have exact closed modularization decisions; no maintained file
  exceeds the hard line or byte ceiling.
- Twenty-four topology fixtures lock root workspace projections, Rust unit and
  dependency ownership, TypeScript references, Tauri build cwd/config behavior,
  fresh-clone frontend-before-Tauri ordering, no-hosted-CI policy, the shared
  runtime boundary, and the TUI's exact vendored dependency boundary and current
  component-owner labels.
- RunHaven-owned adapters nested inside the vendored TUI tree are explicitly
  re-included in source-limit enforcement. Two previously hidden over-limit test
  owners were split into focused child modules; no limit exception was added.
- `./init.sh` and the final `scripts/verify-all.sh` pass on macOS 27 arm64. A
  clean committed snapshot at a path containing spaces also passes
  `./init.sh --bootstrap` from zero dependencies and zero build output, then
  remains Git-clean. Thin Python and Cargo entrypoints enforce the machine-owned
  3.14.6 and 1.97.0 runtimes even when unrelated Homebrew tools lead `PATH`.
- Live Apple `container` checks pass for the internal read-only journey,
  active-run status/logs/stop/record, SSH-agent forwarding, provider allowlist
  and direct-egress denial, exact cleanup, 40 CLI surfaces, a Tauri debug build,
  and two isolated GUI-owned shell sessions.
- Detailed proof is in `docs/harness/evidence/evidence-log.md`.

## Blockers and unknowns

- Distribution is blocked on explicit maintainer authorization and paid Apple
  signing, notarization, clean-Mac, credentialed-agent, and manual accessibility
  evidence. Do not push, tag, release, notarize, or distribute without a direct
  request.
- Canonical blocker: this private pre-alpha project has no hosted CI. The
  owner-directed removal is preserved, but it is not an exception in the
  installed `harness-engineering` standard.
- Canonical blocker: the maintained harness intentionally rejects Windows and
  alternate-shell projections. The selected standard still requires one portable
  Windows/macOS semantic harness implementation and Windows evidence; that
  requirement is unmet rather than waived.
- The validated host is macOS 27 arm64. An exact macOS 26.x host run remains a
  portability validation gap for the lower supported boundary.
- No post-change independent model reviewer has exercised the final harness.
  Presence, local correctness, and an isolated clean-session run are verified;
  independently controlled effectiveness remains a named validation gap.
- The credentialed real-agent GUI broker smoke was not run because no broker
  credential was authorized or supplied. The uncredentialed GUI/Tauri path and
  credential-free provider/SSH runtime boundaries passed.

## Exact next action

Reverse the owner directions that prohibit canonical shared CI and a portable
Windows/macOS harness, implement those two missing contracts, and rerun the
native portability and independent-effectiveness checks. Until then, keep the
review active and do not commit, push, or update a pull request as though the
harness were canonically complete.
