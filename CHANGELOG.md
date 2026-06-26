# Changelog

All notable changes to RunHaven are recorded here. RunHaven is pre-1.0; until
`v1.0.0`, CLI contracts, image layouts, run-record formats, provider allowlists,
and `--json` outputs may change without backward-compatibility guarantees.

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
- Sign in once with `runhaven login`: Claude (host `claude setup-token`,
  injected at run time), Codex (`codex login --device-auth`), Copilot (GitHub
  device flow), and Antigravity (first-run Google OAuth). `--auth-scope agent`
  (default) shares one login per agent across projects; `project` isolates it per
  workspace. Host login state (`~/.claude.json`, Keychain, browser profiles) is
  never read or mounted.
- Host-side API-key broker for Codex, Claude, and Gemini: the real key stays on
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
