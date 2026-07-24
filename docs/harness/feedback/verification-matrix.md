# Verification matrix

Choose the smallest commands that can catch likely regressions. `./init.sh` is
the session baseline and representative Playwright smoke;
`scripts/verify-all.sh` is the authoritative full local lane. The comprehensive
TUI and its exact pinned Codex graph are included in the Rust workspace lane.

| Change | Required focused checks |
| --- | --- |
| Harness instructions, state, or policy | `scripts/python.sh scripts/check_harness_state.py`; applicable state, file, or layout fixture script; `scripts/check-repo-structure.sh`; `scripts/check-docs-local-paths.sh`; `git diff --check` |
| Docs only | `scripts/check-repo-structure.sh`; `scripts/check-docs-local-paths.sh`; check changed Markdown links; `git diff --check` |
| Rust | `scripts/cargo.sh fmt --check`; focused `scripts/cargo.sh test -p PACKAGE --locked FILTER`; `scripts/cargo.sh test --workspace --locked`; `scripts/cargo.sh clippy --workspace --all-targets --locked -- -D warnings`; structure gate |
| React or TypeScript | `npm run check`; focused `npm test -- TEST`; `npm test`; `npm run build`; E2E when rendering or interaction changes; structure gate |
| Tauri commands, permissions, or shell | Rust and frontend lanes; `scripts/cargo.sh test -p runhaven-tauri --locked capability_guard`; `npm run tauri:build`; capability review |
| GUI visual, interaction, or macOS design | `scripts/review-gui.test.sh`; update `docs/design/native-gui-uiux-audit.md`; stop existing RunHaven processes; start only `scripts/review-gui.sh`; packaged live review |
| CLI command construction | Focused CLI and core planning tests; Rust lane; `scripts/cli_surface_check.sh` for breadth |
| Repository architecture | Focused moved-behavior tests; stale-reference `rg`; `scripts/check-repo-structure.sh`; applicable Rust, Tauri, or frontend lane |
| Apple `container` boundary | Rust lane; `scripts/cargo.sh run --locked --bin runhaven -- doctor`; focused plan check; `scripts/apple_container_smoke.sh` |
| Provider egress | Focused provider tests; current official endpoint source review; `scripts/apple_container_smoke.sh --with-provider` |
| SSH-agent forwarding | Planner and CLI tests; `scripts/apple_container_smoke.sh --with-ssh`; confirm only the socket is forwarded and unusable private sockets fail safely |
| Authentication, broker, or secrets | Focused auth and provider tests; `scripts/scan-dev-log.sh`; verify records and diagnostics omit secrets; confirm lower-security choices remain explicit and warned |
| Support report or issue intake | Focused core, CLI, Tauri, and frontend tests; `scripts/check-support-issue-form.sh`; isolated-home smoke; no-upload review |
| Bundled images | Unit-local `bash images/NAME/contract.test.sh`; pin check; isolated `npm ci --ignore-scripts` and audit for each npm-backed image; build the current base plus four agent images; image doctor; non-root read-only-root capability-dropped CLI smokes |
| Desktop installer or first setup | Release shell fixture scripts; focused setup and capability tests; frontend setup tests; pin check; signed DMG, copied-app, Gatekeeper, metadata, architecture, and clean-Mac checks only when distribution is explicitly authorized |
| Release preparation | Explicit authorization first; `scripts/verify-all.sh`; doctor; applicable runtime and GUI live smokes; pin and source review; signing and notarization evidence; clean tree |

Common exact commands:

```bash
scripts/python.sh scripts/check_harness_state.py
scripts/python.sh scripts/test_environment.py
scripts/python.sh scripts/test_harness_state.py
scripts/python.sh scripts/test_file_limits.py
scripts/python.sh scripts/test_repo_layout.py
scripts/check-repo-structure.sh
scripts/check-docs-local-paths.sh
scripts/cargo.sh fmt --check
scripts/cargo.sh test --workspace --locked
scripts/cargo.sh clippy --workspace --all-targets --locked -- -D warnings
scripts/cargo.sh run --locked --bin runhaven-check-pins
scripts/cargo.sh build --workspace --locked
npm run check
npm test
npm run test:e2e
npm run build
npm run tauri:build
git diff --check
./init.sh
scripts/verify-all.sh
```

When a command cannot run, record the exact command, reason, risk, best
substitute that ran, and the trigger that will close the gap. A prose review or
aggregate score never substitutes for runnable evidence.

## Reliability ownership

The maintainer owns reliability routing through this matrix and the sensor
registry. Log shape, levels, redaction, and rotation are owned by
`debug-logging.md`. Golden journeys are the `./init.sh` frontend smoke, the
default Apple container smoke, and the selected live Tauri smoke. Each owns its
isolated temporary state and cleanup trap; rerun from a fresh process after a
failure rather than reusing unknown runtime state.

`./init.sh` prints each stage duration and total time. Structure-gate latency is
compared with the same checkout and host rather than given a hardware-dependent
absolute score; investigate a repeatable increase above twice the reviewed
known-good baseline before broadening the gate. The 2026-07-14 local baseline
for 613 maintained source/config files was 0.23 seconds. Runtime benchmarks or
consistency scanners must record their seed state, environment, target, result,
and integrity result in the evidence log before they become required sensors.
