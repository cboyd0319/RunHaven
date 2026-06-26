<p align="center">
  <img src="docs/assets/logo.png" alt="RunHaven logo" width="180">
</p>

# RunHaven

![Rust 1.96.0](https://img.shields.io/badge/rust-1.96.0-orange)
![macOS 26+](https://img.shields.io/badge/macOS-26%2B-black)
![Apple container 1.0.0](https://img.shields.io/badge/apple%20container-1.0.0-555)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

> [!CAUTION]
> **Pre-release: `v0.5.0`, CLI-complete.** This is RunHaven's first release, a
> pre-1.0, CLI-only pre-release. The CLI is the product surface; the desktop app
> and terminal UI are deferred to a later phase. CLI contracts, container and
> image layouts, run-record formats, provider allowlists, auth-broker behavior,
> and `--json` outputs may still change without backward-compatibility
> guarantees before `v1.0.0`.

RunHaven is a Rust CLI (with an alpha Tauri/Svelte desktop shell) for running
Claude Code, Codex, Gemini, Antigravity, Copilot, or custom coding agents inside
Apple `container` on macOS 26+ (Apple silicon only). It does not replace those
tools. It gives them a repeatable local boundary so the secure way to run an
agent is also the easy way.

It is built for people who sign in to their coding agent with a subscription or
OAuth account (Claude Pro/Max, ChatGPT, a Google account, GitHub), not only with
API keys. Signing in is one command and is reused across runs, and your host
login is never read or mounted.

## Why RunHaven

AI coding agents can inspect a project, edit files, run commands, and iterate
fast. That same power is risky when the agent runs directly on your Mac with
ambient access to your home directory, shell environment, SSH keys, cloud
credentials, browser profiles, and unrelated repositories. RunHaven puts each
run inside a bounded sandbox:

| Need | RunHaven answer |
| --- | --- |
| Know what will run | `runhaven plan` prints the workspace, state volume, network mode, egress status, preflight, and Apple `container run` command before execution. |
| Sign in once | `runhaven login <agent>` signs you in inside the sandbox (or, for Claude, with a host setup token) and reuses it across every run. Your `~/.claude.json`, macOS Keychain, and browser profiles are never read or mounted. |
| Avoid broad host access | Runs mount one selected workspace, not your whole home directory or credential folders. |
| Keep agent state separated | Each agent gets an isolated home volume. `--auth-scope` shares one login per agent (the default) or isolates it per project. |
| Choose network scope | Secure default per profile (a provider allowlist where the agent's hosts are bundled, otherwise internet), plus local-only `internal` and an explicit `--network` override. |
| Review risky edits | `--worktree` runs in a RunHaven-owned git worktree you can merge, keep, recover, or discard. |
| Recover local resources | RunHaven-owned images, volumes, networks, runs, egress logs, and auth-broker state have explicit inspection and cleanup commands. |

RunHaven is not a complete data-loss or exfiltration solution. The selected
agent can still read the mounted workspace and its isolated home volume, and
`internet` mode is not domain-restricted. See the
[security model](docs/SECURITY_MODEL.md) for the full boundary.

## Quick start

Start Apple `container`, then install RunHaven from this checkout:

```bash
container system start
cargo install --path . --locked
```

Run the non-mutating setup guide, build an agent image, sign in once, inspect
the plan, then run from the project directory you want the agent to work on:

```bash
runhaven setup
runhaven image build claude
runhaven login claude
runhaven plan claude
runhaven run claude
```

Use the smallest project directory the agent needs. RunHaven mounts that
directory at `/workspace`, not your whole home directory. See
[Installation](docs/INSTALLATION.md) for requirements and
[Usage](docs/USAGE.md) for command-level workflows.

## Signing in

RunHaven never reads your host login state. You sign in once, the credential
lives in an isolated per-agent home volume (or, for Claude, host-side and
injected at run time), and every later run reuses it.

| Agent | Sign in with | How it works |
| --- | --- | --- |
| Claude | `runhaven login claude` | Runs Anthropic's `claude setup-token` on your host (needs Claude Code installed). The token is injected into the sandbox at run time, never written to `~/.claude` and never placed on a command line. |
| Codex | `runhaven login codex` | `codex login --device-auth` inside the sandbox. Enable device-code login in ChatGPT under Settings then Security. |
| Copilot | `runhaven login copilot` | The GitHub device flow inside the sandbox. Answer `y` when it offers to store the token; the file lands in the isolated volume, not on your Mac. |
| Antigravity | `runhaven login antigravity` | Starts `agy`; approve the Google sign-in in your browser, then type `/exit`. |

Each login persists in a per-agent home volume reused across projects
(`--auth-scope agent`, the default), or pass `--auth-scope project` to keep it
to one workspace. Clear a login with `runhaven login <agent> --clear`.

Prefer API keys? A host-side broker keeps the real key on the host and gives the
guest only a placeholder plus a base-URL redirect
(`runhaven run codex --api-key-broker-env OPENAI_API_KEY`). See the
[auth broker](docs/AUTH_BROKER.md) doc.

## Network and egress

In `provider` mode RunHaven limits the agent to its provider's hosts, and you do
not manage host lists. Each agent ships a small maintained allowlist, expressed
as stable domain-family patterns so a provider's channel or region change needs
no update, while data-egress hosts stay closed by default. If something is
blocked, RunHaven says so in plain language and the per-host detail is in
`runhaven egress log`. `internal` is host-only and `internet` is unrestricted;
the default is the secure choice for the agent.

## Status and roadmap

RunHaven is alpha/pre-release. The CLI is the working product surface today. The
Tauri/Svelte desktop shell can read setup, dashboard, profile, folder-pick, and
run-plan state and supports confirmed launch, image readiness, sanitized live
status, and bounded output snapshots; stop, kill, repair, image build, state
cleanup, and worktree review remain CLI-first.

The roadmap leads with the non-UI product and defers all GUI and terminal-UI
work to the very end (full detail in [ROADMAP.md](docs/ROADMAP.md) and the
[release plan](docs/V1_RELEASE_PLAN.md)):

- **`v0.5.0`, CLI complete:** the command set, docs, JSON and local-data
  decisions, runtime smokes, profile support tiers, diagnostics, cleanup, and
  security boundaries finished and verified. Alpha ends after this milestone.
- **Runtime and security hardening,** then the remaining non-UI product scope
  and a CLI-based public release.
- **First-class desktop app and a terminal UI, last:** the Tauri app becomes the
  easiest safe path for the full workflow, signed and notarized, alongside a TUI
  over the same planner and policy. This GUI and TUI phase is deliberately last;
  its release version label is open.

## Product rule

The secure path must be the easy path: secure defaults are the shortest
workflow, supported lower-security choices warn and require explicit intent, and
unsupported, invalid, or hard-boundary violations fail closed. Apple
`container machine` is not the default boundary; explicit or user-managed machine
workflows are warned, not blocked solely for being less secure.

## Documentation

Use the smallest doc that matches the question:

| Need | Start here |
| --- | --- |
| Install and run | [Installation](docs/INSTALLATION.md), [Capabilities](docs/CAPABILITIES.md), [Usage](docs/USAGE.md) |
| Sign-in and credentials | [Auth broker](docs/AUTH_BROKER.md), [Provider endpoints](docs/PROVIDER_ENDPOINTS.md) |
| Security and runtime boundary | [Security model](docs/SECURITY_MODEL.md), [Apple Container gap analysis](docs/APPLE_CONTAINER_GAP_ANALYSIS.md) |
| Architecture and research | [Architecture](docs/ARCHITECTURE.md), [Research](docs/RESEARCH.md) |
| Desktop UI | [Tauri UI guardrails](docs/TAURI_UI_GUARDRAILS.md), [Tauri log viewing design](docs/TAURI_LOG_VIEWING_DESIGN.md), [Tauri UI research plan](docs/TAURI_UI_RESEARCH_PLAN.md) |
| Release planning | [Roadmap](docs/ROADMAP.md), [v0.5.0 release plan](docs/V1_RELEASE_PLAN.md), [Release gap analysis](docs/RELEASE_GAP_ANALYSIS.md), [CLI surface coverage](docs/CLI_SURFACE_COVERAGE.md), [Pinning policy](docs/PINNING.md) |
| Project operations | [Harness](docs/harness/README.md), [Contributing](CONTRIBUTING.md), [Security policy](SECURITY.md) |

## Development

Agent-assisted work starts from three files only: [AGENTS.md](AGENTS.md),
[feature_list.json](feature_list.json), and
[current-state.md](current-state.md). Load the harness docs only when a task
touches that surface.

Use the smallest relevant check for a change:

```bash
cargo fmt --check
cargo test --locked
cargo clippy --all-targets -- -D warnings
cargo run --locked --bin runhaven-check-pins
npm --prefix ui run check
npm --prefix ui test
npm --prefix ui run test:e2e
git diff --check
```

Full local harness verification is `./init.sh`. Opt-in Apple `container` runtime
smokes are `scripts/apple_container_smoke.sh` (add `--with-provider` for the
egress checks). Docs-only changes use the docs checks from
[the verification matrix](docs/harness/feedback/verification-matrix.md); runtime,
security-boundary, image, or install-flow changes need focused tests plus the
relevant Apple `container` smokes.

## License

[MIT](LICENSE)
