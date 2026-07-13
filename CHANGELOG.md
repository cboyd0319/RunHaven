# Changelog

All notable changes to RunHaven are recorded here. RunHaven is pre-1.0; until
`v1.0.0`, CLI contracts, image layouts, run-record formats, provider allowlists,
and `--json` outputs may change without backward-compatibility guarantees.

## v0.7.0 - 2026-07-11 (pre-release candidate)

CLI plus desktop candidate for limited coworker testing. This is not a public
desktop release until its fail-closed release wrapper produces a signed,
notarized artifact and that artifact passes clean-Mac verification.

### Highlights

- Defined the supported agent set as exactly Claude, Codex, Antigravity, and
  Copilot. Removed the retired Gemini profile, image, pins, broker behavior,
  endpoint policy, and desktop catalog entry. Cleanup still recognizes legacy
  Gemini network names created by older builds.
- Reworked the desktop flow for nontechnical users: guided setup recovery,
  folder-first launches, safe recommended defaults, plain-language warnings,
  filtered run history, agent readiness, System Check, Cleanup, Settings, and
  accessible small-window behavior.
- Kept supported lower-security choices available. Provider networking and
  isolated worktrees are recommended defaults; Internet, local-only, direct
  workspace access, and other supported choices explain their tradeoffs and
  require explicit intent without being hidden or blocked.
- Hardened lifecycle ownership across CLI and desktop runs: single-flight
  background jobs, bounded output/input, past-run handling, signal-aware child
  reaping, exact run-container and network cleanup, recoverable worktrees, and
  locked append-only records with malformed-record recovery.
- Unified ordinary sign-in across all four agents. Each provider's native OAuth
  command now runs in a live, non-root terminal inside its isolated container
  state. RunHaven does not read, copy, or store the provider credential.
- Added a non-publishing Apple Development signing lane for installed-app
  testing. It uses a separate local bundle identifier and accepts only a
  matching development certificate and profile.
- Added a fail-closed desktop release wrapper that requires an exact clean tag,
  full Xcode, matching Developer ID identity and provisioning profile,
  hardened runtime entitlements, notarization, stapling, Gatekeeper approval,
  checksums, SBOMs, and provenance before publishing artifacts.
- Made coworker installation and first setup a no-Terminal path. The mounted
  DMG is verified as one drag-to-Applications action; Finder-launched builds
  resolve Apple's `/usr/local/bin/container`; missing Apple container opens the
  exact pinned signed installer after native confirmation; stopped services
  start in-app; and missing agent images show a visible install action that
  blocks launch until the agent is ready.
- Updated exact dependency pins and split active crate/UI ownership so the CLI,
  shared core, Tauri shell, and React frontend pass the repository structure
  gates. The experimental TUI and vendored Codex source remain archived outside
  the release workspace.
- Updated the source-build toolchain to Rust 1.97.0, the current stable release.
- Refreshed all five bundled images (one base plus four agents) and their
  `0.7.0` tags. The agent images pin
  Claude Code 2.1.207, Codex CLI 0.144.1, Antigravity CLI 1.1.1, and GitHub
  Copilot CLI 1.0.70; the base layers, npm, Debian snapshot,
  package locks, archive checksums, and source-digest checks are exact-pinned.
  Codex now uses its native `$HOME/.codex` default, avoiding a misleading
  missing-directory warning on a fresh isolated home.

### Current limitations

- macOS 26+ on Apple silicon only. Windows and Linux are not supported.
- The local Apple Development build passed its signing and launch checks. A
  shareable Developer ID build still requires paid signing and notarization.
  Gatekeeper, clean-Mac installation, four live native OAuth flows, and manual
  VoiceOver review also remain release gates.

## v0.5.0 - 2026-06-26 (pre-release)

First CLI-complete release. RunHaven runs AI coding agents (Claude Code, Codex,
Gemini, Antigravity, Copilot, or a custom shell image) inside Apple `container`
on macOS 26+ (Apple silicon) with a secure-by-default boundary, so the secure
way to run an agent is also the easy way.

### Highlights

- Sandboxed agent runs: non-root agent, read-only root filesystem, dropped
  capabilities, tmpfs `/tmp`, one selected workspace mount, an isolated per-agent
  home volume, and explicit environment only. No host home, cloud-credential,
  raw SSH key, or browser-profile mounts by default.
- `runhaven plan` prints the workspace, state volume, network mode, egress
  status, preflight, and the exact Apple `container run` command before
  execution, plus plain-language security notices for every lower-security
  choice.
- Sign in once with native OAuth inside each agent container: Claude
  (`claude auth login`), Codex (`codex login --device-auth`), Copilot
  (`copilot login`), and Antigravity (`agy` first-run Google OAuth). The desktop
  app uses the same live terminal flow for all four. `--auth-scope agent`
  (default) shares one login per agent across projects; `project` isolates it per
  workspace. Host login state (`~/.claude.json`, Keychain, browser profiles) is
  never read or mounted.
- Optional host-side API-key broker for Codex and Claude: the real key stays on
  the host and the guest receives only a placeholder plus a base-URL redirect.
- Provider egress allowlist with a profile-aware default network mode (provider
  where the agent's hosts are bundled, otherwise internet) and maintainer-curated
  domain-family patterns, with a calm plain-language blocked-host notice and
  full detail in `runhaven egress log`.
- Run lifecycle: live status, attach, logs-follow, stop, kill, repair; worktree
  runs with diff/keep/recover/merge/discard; image build/rebuild/doctor; managed
  network and state-volume inspection and cleanup; egress and auth-broker logs;
  `runhaven why` safety explanations; and `runhaven agents` with a code-derived
  support-tier matrix.

### Notes and limitations

- Pre-1.0, CLI only. The alpha desktop shell (`runhaven-tauri`) and a terminal UI
  are deferred to a later phase.
- macOS 26+ on Apple silicon only. Windows and Linux are not supported.
- Bundled agent images are built locally with `runhaven image build <agent>`
  from the repo Containerfiles; there is no image registry. The CLI `0.5.0`
  bundles image templates tagged `0.1.0` (unchanged for this release).
- All `--json` outputs and local record files (`runs.jsonl`,
  `egress-policy.jsonl`, `auth-broker.jsonl`, active-run markers) are best-effort
  and unversioned. See `docs/V1_RELEASE_PLAN.md`.
- SSH agent forwarding is intentionally unavailable (fail-closed) until a
  no-secret non-root runtime proof exists.

### Install

```bash
container system start
cargo install --path . --locked
```

See [README.md](README.md) and [docs/USAGE.md](docs/USAGE.md) to get started.
