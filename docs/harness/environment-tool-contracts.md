# Environment tool contracts

This file records the operational boundary for tools and services used during
normal repository work. Version facts remain in machine-readable owners. Product
behavior remains in the routed product documents.

## Development toolchain

| Tool | Purpose and owner | Start or capability | Writes and external effects | Unavailable, timeout, cancellation, and recovery | Platform |
| --- | --- | --- | --- | --- | --- |
| Python harness | Runs the standard-library harness validators. `.python-version` owns the runtime and `scripts/` owns behavior. | `./init.sh`; no credential. | Validators are read-only. Fixture writes stay in temporary directories. | `scripts/check_environment.py` fails with the missing name or after a 30-second version-check timeout. Cancel the foreground process and rerun. | macOS 26+ on Apple silicon. |
| Rust and Cargo | Build and test the Rust workspace. `rust-toolchain.toml`, root `Cargo.toml`, and root `Cargo.lock` own the contract. | `scripts/cargo.sh`; no credential. | Cargo writes ignored output under `target/` and may fetch locked crates during bootstrap or a build. | Readiness fails before a build when the toolchain is missing. Cancel the foreground process, preserve source, then rerun the same locked command. | macOS 26+ on Apple silicon. |
| Node, npm, Tauri CLI | Build, test, and start the React and Tauri surfaces. `.node-version`, root `package.json`, `package-lock.json`, and `ui/package.json` own the contract. | `./init.sh --bootstrap`, then `npm run tauri:dev`; no credential for development. | Bootstrap replaces ignored `node_modules/` from the lockfile. Builds write ignored output under `ui/dist/` and `target/`. No service or user configuration is persisted. | Readiness names a missing or mismatched command. Cancel and rerun bootstrap after removing only repository `node_modules/` when cleanup is required. | macOS 26+ on Apple silicon. |
| Git | Provides repository identity, status, history, and worktree support. `.git/` and the root repository contract own its use. | Existing checkout; remote operations use the developer's separately configured Git credential. | Harness checks are read-only. Commit, push, branch, and worktree mutations require task authority. | Readiness names a missing command. A failed local query leaves source unchanged; repair Git access and rerun. | macOS 26+ on Apple silicon. |
| Xcode tools | Compile and bundle the native desktop surface. Xcode owns its installation; Tauri and Cargo manifests own invocation. | `xcodebuild`; signing identities and notarization credentials are needed only for an explicitly authorized release. | Local development writes repository build output. Authorized release work may contact Apple and use the selected signing identity. | The full readiness check names a missing command. Cancel local builds and rerun. Release recovery follows `release/release-controls.md`. | macOS 26+ on Apple silicon. |

## Browser and local services

| Tool or service | Purpose and owner | Start or capability | Writes and external effects | Unavailable, timeout, cancellation, and recovery | Platform |
| --- | --- | --- | --- | --- | --- |
| Playwright Chromium and loopback Vite | Runs deterministic browser journeys. `ui/playwright.config.ts`, `ui/e2e/fixtures.ts`, and the root npm lock own it. | `npm run test:e2e`; no credential. | Uses an OS-assigned `127.0.0.1` port and writes ignored evidence under `target/playwright/`. It does not need public network access after locked dependencies are present. | Playwright owns per-test cancellation and timeout. The worker fixture always closes Vite. Stop the foreground command and rerun from a fresh process after failure. | macOS 26+ contributor lane; browser assertions remain web-platform tests, not native desktop proof. |
| Apple `container` service | Provides the product isolation runtime. `pins.toml` owns exact inputs; `docs/INSTALLATION.md` and `docs/SECURITY_MODEL.md` own product setup and safety. | `container system start`; no RunHaven credential. macOS may require administrator approval for Apple's installer. | The service owns Apple container images, networks, volumes, and runtime state outside the repository. Product commands mutate only the explicitly selected RunHaven resources. | `runhaven doctor` is the health check. Runtime and provider smokes bound network probes with explicit timeouts and clean up through traps. Cancel a foreground run, inspect active state, then use the documented stop, repair, or cleanup action. | macOS 26+ on Apple silicon only. |

## Agent provider tools

Claude, Codex, Antigravity, and Copilot CLIs run inside their declared image and
isolated agent-home volume. Image `Containerfile`, package manifest, lockfile,
and login wrapper own each projection. `docs/AUTH_BROKER.md` owns authentication;
`docs/PROVIDER_ENDPOINTS.md` owns network destinations.

- Startup is `runhaven login AGENT` for sign-in and `runhaven run AGENT` for a
  session. Account login state is stored only in that agent's isolated volume.
  The optional explicit brokers accept `ANTHROPIC_API_KEY` for Claude and
  `OPENAI_API_KEY` for Codex; values stay on the host.
- Tool writes are limited to the selected workspace, isolated agent home, and
  RunHaven-owned runtime records. Provider access uses the selected network mode;
  internal mode is the network-disabled behavior.
- `runhaven doctor`, `runhaven image doctor AGENT`, `runhaven auth status`, and
  `runhaven plan AGENT` are the health and preflight paths. A missing image,
  service, login, or provider route fails with a recovery command.
- Cancel login or a run through the owning terminal or desktop action. RunHaven
  reaps owned children, preserves recoverable records, and exposes stop, kill,
  repair, or cleanup only for confirmed RunHaven resources. Provider proxy and
  broker connections have bounded I/O timeouts.
- These tools are supported only inside Apple `container` on macOS 26+ Apple
  silicon. No repository-owned MCP server, database, or remote agent service is
  required by the harness.
