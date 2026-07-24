# RunHaven repository contract

## Project

RunHaven is a Rust CLI with comprehensive terminal and Tauri/React desktop
interfaces for running AI coding agents inside Apple `container` on macOS 26+
on Apple silicon.

This root file is the repository-wide instruction owner. A nested `AGENTS.md`,
if introduced, may only narrow behavior for its directory; the closest scoped
file applies without restating or weakening this contract.

## Fixed session start

Every implementation session follows this order:

1. Confirm the repository root with `pwd` and `git rev-parse --show-toplevel`.
2. Read `PROGRESS.md`.
3. Read `feature_list.json`.
4. Review recent work with `git log -5 --oneline`.
5. Run `./init.sh`. It includes the deterministic frontend smoke and E2E lane.
6. Repair a broken baseline before adding work.
7. Select the highest-priority unfinished feature. Exactly one feature is
   `active`; work only on that feature unless the user explicitly changes scope.

Do not add another startup file or bulk-read `docs/harness/`. Load a routed owner
only when the task touches it.

## Commands

- First dependency synchronization: `./init.sh --bootstrap`
- Session baseline and representative smoke: `./init.sh`
- Desktop development start: `npm run tauri:dev`
- Full local gate: `scripts/verify-all.sh`
- Harness state: `scripts/python.sh scripts/check_harness_state.py`
- Harness fixtures: `scripts/python.sh scripts/test_environment.py`,
  `scripts/python.sh scripts/test_harness_state.py`,
  `scripts/python.sh scripts/test_file_limits.py`, and
  `scripts/python.sh scripts/test_repo_layout.py`
- Repository boundaries and file budgets: `scripts/check-repo-structure.sh`
- Rust: `scripts/cargo.sh fmt --check`,
  `scripts/cargo.sh test --workspace --locked`, and
  `scripts/cargo.sh clippy --workspace --all-targets --locked -- -D warnings`
- Frontend: `npm run check`, `npm test`, `npm run test:e2e`, `npm run build`
- Pins: `scripts/cargo.sh run --locked --bin runhaven-check-pins`
- Docs paths: `scripts/check-docs-local-paths.sh`
- Diff hygiene: `git diff --check`
- Runtime-only changes: `scripts/cargo.sh run --locked --bin runhaven -- doctor` and the
  applicable `scripts/apple_container_smoke.sh` mode

Use [the verification matrix](docs/harness/feedback/verification-matrix.md) to
choose focused checks. `scripts/verify-all.sh` is the manually invoked full gate.
This private pre-alpha project has no hosted CI. Use the local macOS gates.

Harness validators use the exact `.python-version` runtime and only the standard
library. Rust, Node, npm, Apple `container`, and product pins remain owned by
their manifests, lockfiles, `rust-toolchain.toml`, and `pins.toml`.

## Safety boundary

The secure path must remain the easiest path. Do not weaken these defaults
without an explicit user-approved tradeoff and focused verification:

- task-scoped `container run`, not `container machine`;
- no host home, cloud credential folders, raw SSH keys, browser profiles, or
  arbitrary host environment variables mounted by default;
- non-root images, read-only root filesystems, capability drops, and mount
  exclusions;
- default-deny provider egress with explicit provider allowlists;
- exact subprocess argument lists, never executable shell strings;
- explicit warning and intent for supported lower-security choices.

`--ssh` is an explicit warned option that forwards only the host SSH agent
socket. It never mounts raw keys. Read [the security model](docs/SECURITY_MODEL.md)
before changing runtime, auth, provider, image, mount, network, or state behavior.

## Owners and routing

- Product and use: `README.md`, `docs/USAGE.md`, `docs/CAPABILITIES.md`
- Repository topology: `ARCHITECTURE.md`, `repository-structure-policy.json`
- Product runtime architecture: `docs/ARCHITECTURE.md`,
  `docs/harness/boundaries/component-inventory.md`
- Security: `docs/SECURITY_MODEL.md`,
  `docs/harness/boundaries/security-boundary-map.md`
- Desktop behavior: `docs/DESKTOP_APP.md`; GUI changes also require the current
  Apple HIG check and `docs/design/native-gui-uiux-audit.md` update
- Release: `docs/V1_RELEASE_PLAN.md`, `docs/RELEASE_GAP_ANALYSIS.md`,
  `docs/harness/release/`
- Harness: `docs/harness/README.md`
- Current state: `PROGRESS.md`, `feature_list.json`
- Historical evidence: `docs/harness/evidence/evidence-log.md`, Git history

Workspace ownership is fixed: `crates/runhaven` owns binary entrypoints,
`runhaven-core` owns shared runtime truth, `runhaven-cli` owns CLI presentation,
`runhaven-tui` owns terminal presentation, and `src-tauri` plus `ui` own the
desktop shell. The pinned Codex source supplies the TUI's upstream-derived
dependencies; it does not define RunHaven product or harness policy.

## Change discipline

- Define observable success and verification before implementation.
- Prefer no change, deletion, the standard library, native behavior, an existing
  dependency, or one clear line before adding custom code.
- Preserve user changes. Do not revert dirty work or perform irreversible,
  credentialed, costly, release, signing, push, or distribution actions without
  explicit authorization.
- Use `rg` for search and `apply_patch` for manual edits. Keep output bounded.
- Documentation is product behavior. Update its owner in the same slice.
- Never commit secrets, machine-local absolute paths, or local external-source
  references.
- File budgets cover bytes and physical lines and record maximum line bytes. The limits
  and warning threshold live in `scripts/check_file_limits.py`; the enforced
  gate is `scripts/check-repo-structure.sh`. Exceptions require an owner, reason,
  measured baseline, and retirement condition.
- Keep `PROGRESS.md` compact. Put logs and narrative evidence in the evidence log
  or Git history.
- Feature states are only `not_started`, `active`, `blocked`, and `passing`.
  Exactly one is active. `passing` is irreversible; add a corrective feature for
  a later regression.

## Finish

1. Run the smallest focused checks that cover the change.
2. Run the relevant local harness gates and `git diff --check`.
3. Reconcile `feature_list.json` and `PROGRESS.md`, including a no-transition
   result. Record only compact evidence pointers in startup state.
4. Remove stale temporary artifacts and confirm `git status --short --branch`.
5. Report checks run, skipped live checks and their risk, blockers, and the exact
   next action. Do not claim completion from prose or a structural score.
