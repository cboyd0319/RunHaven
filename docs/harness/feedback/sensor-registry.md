# Sensor Registry

Status: live

This registry records the checks and signals agents and humans use to decide
whether work is ready. A sensor is a signal, not a guarantee. It does not prove
real-agent effectiveness.

## Registered Sensors

| Sensor | Source | Purpose | Owner | Retire When | Review Cadence |
| --- | --- | --- | --- | --- | --- |
| `cargo fmt --check` | rustfmt | Catch Rust formatting drift | Maintainers | Replace only if formatter policy changes | Rust source or test changes |
| `cargo test --workspace --locked` | Cargo test suite | Catch behavior regressions across CLI, TUI, Tauri, planning, egress, auth, images, state, worktrees, and repo policy | Maintainers | Replace only if the test runner changes across the whole repo | Any code behavior change; full verification before release |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Clippy | Catch Rust lint and correctness regressions across workspace members | Maintainers | Replace if lint policy changes by accepted project decision | Rust source or test changes |
| `cargo run --locked --bin runhaven-check-pins` | `pins.toml`, Cargo manifests, image templates, and future workflow pins | Enforce exact package, image, npm, Debian, and future Action pin policy | Maintainers | Replace if pin policy moves into a stronger packaged command | Dependency, image, workflow, release, or docs changes that mention pins |
| `cargo build --workspace --locked` | Cargo build | Verify workspace binaries and libraries can be produced from the locked dependency graph | Maintainers | Replace when release packaging flow changes | Packaging, manifest, dependency, or release changes |
| `npm --prefix ui run check` | Svelte and TypeScript | Catch frontend type and Svelte contract regressions | Maintainers | Replace if frontend tooling changes by accepted project decision | Frontend UI changes |
| `npm --prefix ui test` | Vitest | Catch frontend helper, warning, and preview-command regressions | Maintainers | Replace if frontend test runner changes | Frontend UI command-helper changes |
| `npm --prefix ui run test:e2e` | Playwright | Catch browser runtime errors and blank-page regressions in the desktop UI frontend | Maintainers | Replace if browser test tooling changes | Frontend UI rendering, routing, or entrypoint changes |
| `npm --prefix ui run build` | Vite | Verify static frontend assets build for Tauri | Maintainers | Replace if frontend build tooling changes | Frontend UI changes |
| `npm --prefix ui run tauri:build` | Tauri CLI | Verify the desktop shell config, capabilities, frontend build, and Rust backend compile together without bundling | Maintainers | Replace when desktop packaging flow changes | Tauri config, capabilities, desktop backend, or frontend integration changes |
| `capability_guard` test (`cargo test --workspace --locked capability_guard`) | `src-tauri/capabilities/*.json` | Fail closed if a Tauri capability grants any scope beyond RunHaven `allow-*` command permissions plus a vetted plugin allowlist, so the untrusted WebView cannot gain a host bridge | Maintainers | Replace if the capability model or permission scheme changes | Tauri capability, command surface, or plugin permission changes |
| `./init.sh` | macOS local verification entrypoint | Run the full macOS harness verification set in one command | Maintainers | Replace if the repo adopts a different full local verification entrypoint | Release prep, broad changes, or shared behavior changes |
| `harnessforge report --target .` | HarnessForge unified report | Optional structural signal for readiness, audit, drift, evidence, and platform contract without running target commands | Maintainers | Replace if report evidence moves into a project-owned release command | Harness, release-prep, state, or docs changes when HarnessForge is available |
| `harnessforge audit --target . --min-score 85` | HarnessForge structural audit | Advisory structural check for the repo harness floor; repo-owned docs, tests, and maintainer decisions remain authoritative | Maintainers | Replace if a project-owned harness audit supersedes it | Harness changes and release prep when HarnessForge is available |
| Local Markdown link check | One-off local script or reviewer command over tracked Markdown files | Confirm target-relative doc links resolve after docs changes | Maintainers | Replace if a packaged docs checker is added | Docs, README, harness docs, and roadmap changes |
| Platform wording scan | `rg` over docs/state for unsupported platform claims | Preserve macOS 26+ only runtime and contributor verification | Maintainers | Replace if platform contract expands by accepted source-backed decision | Docs, future CI, install, runtime, or manifest changes |
| Provider egress smoke | Manual `runhaven run ... --network provider` smoke | Prove provider proxy allow/deny behavior on Apple `container` internal network | Maintainers | Replace if provider network architecture changes | Provider proxy, endpoint, DNS, or network runtime changes |
| `scripts/apple_container_smoke.sh` | Repo-owned opt-in local smoke script | Prove Apple `container` runtime wiring, read-only workspace behavior, active-run status/logs-follow/stop cleanup, SSH forwarding fail-closed guidance, optional SSH run refusal with `--with-ssh`, provider plan guidance, optional provider allowlist behavior with `--with-provider`, and exact temporary resource cleanup | Maintainers | Replace if the CLI gains an equivalent packaged smoke command or runtime architecture changes | Pre-Tauri checks, release prep, Apple `container` runtime changes, SSH forwarding changes, provider network changes, or run-observability changes |
| Codex broker smoke | Manual Codex broker run with a disposable key | Prove optional Codex API-key broker path | Maintainers | Replace if broker protocol or supported providers change | Auth broker changes and release prep when disposable key is available |
| `runhaven doctor` | Runtime prerequisite checker | Confirm host prerequisites and remediation guidance | Maintainers | Replace if setup/doctor contract changes | Runtime, install, setup, or Apple `container` boundary changes |
| `runhaven plan ...` | CLI planning command | Preview workspace, image, user, network, state, broker, and command boundary before mutation | Maintainers | Replace if planner output contract changes | CLI command construction, security, network, workspace, and image changes |
| `runhaven image doctor [AGENT]` | Image diagnostic command | Detect missing or stale bundled images and inactive state-volume review without mutation | Maintainers | Replace if image lifecycle changes | Image build, image metadata, state review, or setup changes |

## Agent-Oriented Failure Feedback

For custom checks, prefer failure messages with:

- what failed;
- why the RunHaven security, data-loss, or platform boundary matters;
- where an agent should look first to repair it;
- which evidence should be recorded after the fix.

## Promotion Rules

- Do not run commands only because they appear here. Use repo-owned change
  type guidance first. Optional structural-review output is advisory unless a
  maintainer promotes a finding into repo-owned artifacts.
- Keep owner, source, purpose, and retire conditions current before promoting
  a check into release or automation gates.
- Remove or replace sensors that no longer catch meaningful regressions.
- Structural audit score is a harness-health signal, not real-agent
  effectiveness evidence.
