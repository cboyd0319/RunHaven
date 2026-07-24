# Sensor registry

Checks are evidence sources, not guarantees or scores. Use the smallest lane
that can catch the likely failure, then broaden when the affected boundary
requires it.

The literal Sensor value is its stable ID and exact command or evaluator. The
trigger/evidence cell records purpose, owned surface or triggering paths, source
evidence, and expected signal. The remaining cells record false-positive
handling, owner, and retirement condition. Platform coverage is macOS 26+
Apple silicon unless the row explicitly identifies an isolated fixture; no row
implies native proof on another operating system.

| Sensor | Trigger and evidence | False-positive handling | Owner and retirement |
| --- | --- | --- | --- |
| `scripts/python.sh scripts/test_environment.py` | Environment-check changes; proves platform, runtime, command-version, symlink, and path-with-spaces failures | Reproduce the isolated fixture before changing a machine-owned version | Environment readiness; retire with the read-only environment check |
| `bash scripts/test_scan_dev_log.sh` | Diagnostic logging or secret-pattern changes; proves a synthetic leak fails without reproducing the secret, rejects unsafe logs, and preserves the application exit through the tee wrapper | Inspect only the synthetic fixture metadata; never print matched content | Diagnostic log privacy; retire with `scripts/scan-dev-log.sh` and `scripts/dev-with-logs.sh` |
| `scripts/python.sh scripts/check_harness_state.py` | Every harness or state change; validates the live canonical state | Reproduce with `scripts/python.sh scripts/test_harness_state.py`; change policy and fixtures together only for an intentional contract change | Harness state; replace only with an equally strict portable semantic check |
| `scripts/python.sh scripts/test_harness_state.py` | State-validator changes; proves negative cases fail | A fixture must isolate one contract and show the unexpected error | Harness state; retire with the validator |
| `scripts/python.sh scripts/check_file_limits.py` | Every repository structure check; reports hard failures and 90% warnings | Confirm the path ownership class and measured bytes, lines, and maximum line bytes; exceptions need all ownership fields | File policy; replace only with a streaming, symlink-safe equivalent |
| `scripts/python.sh scripts/test_file_limits.py` | File-policy changes; proves overflows, warnings, ignores, symlinks, and physical-line counting | Correct the isolated fixture before changing a calibrated production budget | File policy; retire with the validator |
| `scripts/python.sh scripts/check_repo_layout.py` | Every repository structure check; structurally validates Cargo package identities including aliases, directory ownership, pinned surfaces, no-hosted-CI state, the macOS-only maintained platform projection, and one literal direct-Tauri runtime tripwire | Reproduce with `scripts/python.sh scripts/test_repo_layout.py`; update the owner contract and negative fixture together | Repository topology; replace literal heuristics with compiler-backed checks when broader process construction must be prohibited |
| `scripts/python.sh scripts/test_repo_layout.py` | Workspace, crate, directory, hosted-automation, or architecture-policy changes | A fixture must show one accepted or rejected topology without depending on this checkout | Repository topology; retire with the validator |
| `scripts/check-repo-structure.sh` | Source, docs, state, manifest, or ownership changes; thin entrypoint over file and layout policy | Run each Python core separately to distinguish budget from architecture failures | Repository structure; keep as the stable local command |
| `bash images/NAME/contract.test.sh` | Bundled image manifest, lockfile, Containerfile, entrypoint, or shared-input changes; proves the unit-local contract without building or network access | Run the affected context first; shared behavior belongs in `images/common/test-image-contract.sh` | Bundled image units; retire only with an equivalent unit-local test seam |
| `scripts/check-docs-local-paths.sh` | Docs, instructions, state, examples, or generated docs metadata changes | Inspect the exact match; allow only repo-relative or public references | Documentation privacy; replace with an equally strict checked-in scanner |
| `scripts/cargo.sh run --locked --bin runhaven-check-pins` | Dependency, image, package, runtime, or release-pin changes | Verify the manifest owner and current official source before changing a pin | Supply chain; replace only with the owning packaged command |
| `scripts/cargo.sh fmt --check` | Rust changes | Reformat the changed Rust owner | Rust toolchain; replace only if formatter policy changes |
| `scripts/cargo.sh test --workspace --locked` | Shared Rust behavior or broad Rust changes | Reproduce with the narrow package and test filter | Rust workspace; route narrower checks through the verification matrix |
| `scripts/cargo.sh clippy --workspace --all-targets --locked -- -D warnings` | Shared Rust behavior, dependencies, or release prep | Reproduce in the narrow package; do not suppress a warning without rationale | Rust workspace; replace only by project decision |
| `npm run check` | React or TypeScript changes | Reproduce in the owning component or adapter | Frontend type and static contract |
| `npm test` | Frontend behavior changes | Run the focused Vitest file before the suite | Frontend unit behavior |
| `npm run test:e2e` | Desktop routes, rendering, interaction, or accessibility changes | Reproduce one Playwright flow on a fresh Vite server | Frontend journey smoke |
| `npm run build` | Frontend or Tauri integration changes | Inspect the first Vite build error | Frontend production build |
| `scripts/check-support-issue-form.sh` | Support intake or report changes | Inspect form schema and bounded report contract separately | Support boundary |
| `scripts/cargo.sh run --locked --bin runhaven -- doctor` | Install, runtime, image, provider, or desktop setup changes | Record host version and the exact failed prerequisite without secrets | Read-only runtime health |
| `scripts/apple_container_smoke.sh` | Apple `container` runtime behavior; add `--with-provider` or `--with-ssh` only for those boundaries | Preserve resources and report the first failing phase; never weaken isolation to make the smoke pass | Runtime boundary; requires supported local macOS host |
| `scripts/gui_tauri_live_smoke.sh` | Typed Tauri launch, managed-session, or real-agent ownership changes | Reproduce the smallest selected-session path and record any credentialed mode skipped | Desktop live boundary; local only |
| `./init.sh` | Session baseline and representative smoke | Re-run the first failing focused command, repair baseline, then restart the baseline | Maintainer; no hosted CI exists |
| `scripts/verify-all.sh` | Broad completion or release-sensitive shared changes | Re-run the first failing focused command, repair it, then restart the full lane | Maintainer; full local gate, with the shared-CI authority gap retained separately |

Each custom failure should say what failed, why the boundary matters, where to
look first, and what command proves the repair. Remove a sensor when controlled
exclusion shows no useful detection and no dormant security or data-loss risk.
