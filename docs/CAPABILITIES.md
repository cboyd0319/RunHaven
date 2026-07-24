# Capabilities

RunHaven runs AI coding agents inside Apple `container` with local-Mac safety
defaults. This page is the product overview: what RunHaven can do, what it
protects by default, and where the current limits are.

RunHaven runtime and contributor verification require macOS 26+ on Apple
silicon.

RunHaven is alpha/pre-release. `v0.5.0` was the CLI-only release; `v0.7.0` is
the current CLI plus desktop coworker-test candidate. The desktop app is the
primary easy path for nontechnical users. The comprehensive terminal UI remains
a product surface; its pinned Codex dependency graph is outside the default
release workspace and is not in current release artifacts.

For command walkthroughs, use [Usage](USAGE.md). For the full security boundary,
use [Security model](SECURITY_MODEL.md).

## At A Glance

| Area | What RunHaven provides |
| --- | --- |
| Runtime boundary | One selected workspace, one isolated agent home volume, non-root bundled images, read-only root filesystem, dropped Linux capabilities, and explicit command preview. |
| Agent profiles | Bundled Claude, Codex, Antigravity, and Copilot agents, plus a CLI-only `shell` profile for custom images. |
| Network modes | Secure profile-aware default (provider allowlist where the agent's hosts are bundled, otherwise internet), plus local-only internal networking and explicit override. |
| Workspace safety | Current-directory mounts by default, explicit git-root expansion, sensitive-path rejection, and optional RunHaven-owned git worktrees. |
| Credentials | No host home, raw SSH key, browser profile, cloud credential folder, or arbitrary environment passthrough by default. |
| Observability | Secret-free run records, active-run controls, provider policy logs, auth broker status, TUI guide/dashboard/log snapshots/notices, history/diff review, diagnostics, doctor checks, and recovery commands. |
| Cleanup | Focused image, state-volume, network, and worktree recovery for resources RunHaven owns. |

## Runtime Defaults

RunHaven generates Apple `container` commands with these defaults:

| Default | Meaning |
| --- | --- |
| Workspace mount | One selected project directory is mounted at `/workspace`. |
| Agent home | One per-project/profile/session volume is mounted at the agent home path. |
| Host secrets | macOS home, raw SSH keys, browser profiles, cloud credential folders, and arbitrary host environment variables are not mounted or passed. |
| Container user | Bundled images run as the non-root `agent` user. |
| Filesystem | The container root filesystem is read-only, with temporary scratch space. |
| Linux privileges | Linux capabilities are dropped. |
| Preview | `runhaven plan` prints the selected workspace, state volume, network mode, egress status, preflight, and Apple `container run` command before execution. |

The default workspace scope is the current directory. If that directory is
inside a larger git repository, RunHaven does not silently mount the repository
root. Use `--workspace-scope git-root` only when the agent needs the full
repository at `/workspace`.

## Agent Profiles

| Profile | Default image | Bundled tool | Use case |
| --- | --- | --- | --- |
| `claude` | `runhaven/claude:0.7.0` | Claude Code 2.1.207 | Claude Code with isolated project state |
| `codex` | `runhaven/codex:0.7.0` | Codex CLI 0.144.1 | Codex CLI with its own workspace sandbox enabled |
| `antigravity` | `runhaven/antigravity:0.7.0` | Antigravity CLI 1.1.1 | Antigravity CLI in the same container boundary |
| `copilot` | `runhaven/copilot:0.7.0` | GitHub Copilot CLI 1.0.70 | GitHub Copilot CLI with isolated state |
| `shell` | `runhaven/base:0.7.0` | Bash 5.2 | Generic shell profile for custom agent images |

Use `shell` with `--image IMAGE` when you want to run another agent image.

## Profile Support Matrix

Every bundled profile builds an image and starts through `runhaven run PROFILE`.
They differ in provider-network backing and authentication support tiers. Run
`runhaven agents` for the live, code-derived tier summary (sign-in path, default
network, and API-key broker per agent); the table below is the source-backed
detail.

| Profile | Provider hosts | Interactive auth | Headless broker | Key limit |
| --- | --- | --- | --- | --- |
| `claude` | Bundled: `api.anthropic.com`, `claude.ai`, `platform.claude.com` | `runhaven login claude` (`claude auth login` in the sandbox; paste the code back), the login persists in isolated state | API-key broker: `--api-key-broker-env ANTHROPIC_API_KEY`, `x-api-key` via `ANTHROPIC_BASE_URL` | API key brokered host-side; OAuth and subscription logins use isolated state |
| `codex` | Bundled: `api.openai.com`, `chatgpt.com`, `auth.openai.com` | `runhaven login codex` (`codex login --device-auth`), or ChatGPT sign-in in isolated state | API-key broker: `--api-key-broker-env OPENAI_API_KEY`, Responses API (`/v1/responses`) | Key brokered host-side; device-code login needs the ChatGPT account setting on |
| `copilot` | Bundled: the `githubcopilot.com` hosts plus `github.com` and `api.github.com` for the device login | `runhaven login copilot` (GitHub device flow) | Design-only (cannot be brokered cleanly) | Token exchange + dynamically-routed API host; the in-container login stores the token in the isolated volume |
| `antigravity` | Bundled: Google OAuth, userinfo, and Cloud Code hosts plus the `*-cloudcode-pa.googleapis.com` model-endpoint family | First-run Google login in isolated state (`runhaven login antigravity`) | None | No API-key broker; uses isolated login state. Google sign-in consent and redirect happen in the host browser |
| `shell` | None | Decided by the custom image | None | Generic base for custom images; you supply image and credentials |

The API-key broker only accepts the fixed per-agent host env names above:
`OPENAI_API_KEY` and `ANTHROPIC_API_KEY`. Arbitrary host env names are rejected.

"Design-only" means no credential broker is wired yet, so headless use relies on
isolated login state or an explicit `--env NAME`. See
[Provider endpoints](PROVIDER_ENDPOINTS.md) and [Auth broker](AUTH_BROKER.md)
for the source-backed detail.

## Network Modes

| Mode | Use it for | Egress behavior |
| --- | --- | --- |
| `internet` | Hosted-agent runs that need package managers, registries, CDNs, or arbitrary hosts. | Unrestricted outbound access. Provider-domain allowlisting is not enforced. |
| `internal` | Local-only analysis, offline tests, and custom images that do not need internet. | Host-only Apple `container` network. |
| `provider` | Agent runs limited to the agent's own reviewed provider hosts. | Managed internal network plus RunHaven's host-side allowlist CONNECT proxy. |

When you do not pass `--network`, RunHaven picks the secure default that still
works: `provider` for profiles with bundled provider hosts (`claude`, `codex`,
`copilot`, `antigravity`) and `internet` for profiles without them
(`shell`), where provider mode would have an empty allowlist. Pass
`--network` to override. A provider-default run reaches the agent's own API but
not arbitrary hosts; add `--provider-host HOST` or use `--network internet` when
the run needs more.

Provider mode allows bundled provider hosts for the selected profile, their
subdomains, and reviewed fully qualified `--provider-host HOST` additions. The
proxy rejects IP literal targets and destinations that resolve to local-only
addresses such as loopback, private, link-local, or multicast ranges.

Provider mode is intentionally stricter than internet mode. Some login,
telemetry, package-registry, update, or provider feature-path hosts may need
review before being added. Use the
[provider endpoint matrix](PROVIDER_ENDPOINTS.md) for that review.

## Lower-Security Overrides

Secure defaults are the easiest path. Every supported choice that lowers
security below those defaults is an explicit flag, and both `runhaven plan` and
`runhaven run` print plain-language `Security notices` to standard error for
each active one. Hard-boundary violations still fail closed.

| Override | What it lowers | Default |
| --- | --- | --- |
| `--network internet` | Unrestricted outbound egress with no domain allowlist. The default only for profiles without bundled provider hosts (`shell`). | profile-aware |
| `--env NAME` | Exposes one host environment variable to the agent; a secret there can be read by workspace code. `NAME=value` is rejected. | none |
| `--user USER` / `--allow-root-user` | Runs the agent as a different or root container user instead of the non-root `agent` user. Root or UID 0 fails closed without `--allow-root-user`. | `agent` |
| `--provider-host HOST` | Widens the provider allowlist with one reviewed fully qualified host. | bundled hosts only |
| `--allow-sensitive-workspace` | Permits mounting broad or credential-bearing host paths at `/workspace`. Sensitive paths fail closed without it. | rejected |
| `--image IMAGE` | Runs a custom image that may not follow the bundled non-root, read-only-root hardening. | bundled profile image |
| `--ssh` | Explicitly forwards only the SSH-agent socket, never raw keys, and prints a lower-security warning; unusable private sockets fail safely. See the [security model](SECURITY_MODEL.md). | disabled |

The notices clear when you run with secure defaults. `runhaven why network MODE`,
`why workspace PATH`, and `why host HOST` explain each decision in more detail.

## Workspace And Git Safety

Use the smallest project directory the agent needs. RunHaven rejects broad or
credential-bearing workspace paths unless `--allow-sensitive-workspace` is
passed.

For clean git repositories, `runhaven run AGENT --worktree` creates a
RunHaven-owned branch and git worktree under the RunHaven cache directory, then
mounts that worktree at `/workspace`. The source checkout stays untouched until
you explicitly merge worktree changes back.

Worktree support includes:

| Command family | Purpose |
| --- | --- |
| `runs keep` | Validate the recorded worktree and print review guidance. |
| `runs recover` | Show read-only recovery state and suggested project checks. |
| `runs merge` | Validate the recorded source/worktree boundary before bringing changes back. |
| `runs discard` | Remove the recorded RunHaven-owned worktree and branch without touching the source checkout. |

See [Usage](USAGE.md) for the full worktree workflow.

## Credential Handling

RunHaven does not mount raw SSH keys, browser profiles, cloud credential
folders, provider login caches, or arbitrary host environment variables by
default.

| Credential mechanism | What it does |
| --- | --- |
| `--api-key-broker-env NAME` | Enables the host-side API-key broker for provider-network Codex or Claude runs, keeping the raw key on the host (old name `--codex-api-key-broker-env` still works as an alias). |
| `runhaven login <agent>` | Runs the provider's native OAuth inside its isolated container and lets later runs reuse that agent's home volume. The desktop app uses the same terminal path for Claude, Codex, Antigravity, and Copilot. `--clear` deletes the agent volume. See [Authentication](AUTH_BROKER.md). |
| `--auth-scope agent\|project` | `agent` (default) shares one login per agent across all your projects so an OAuth login is done once; `project` isolates the login to this workspace's own volume. |
| `runhaven auth status` / `runhaven auth explain AGENT` | Explains current broker boundaries without reading or printing secrets. |

Pass a single reviewed host variable with `--env NAME`, or forward SSH with
`--ssh`. Both are lower-security choices that print a
security notice; see [Lower-security overrides](#lower-security-overrides).

## Sessions And State

Each project/profile gets an isolated default agent home volume. Use
`--session NAME` to create a named reusable project/profile home volume without
changing the workspace mount or widening host credential access.

Session and state commands manage RunHaven-owned agent home volumes only. They
do not touch workspace files.

## Observability And Recovery

RunHaven records secret-free run metadata under its cache directory. Records
include run id, profile, workspace, workspace scope, network mode, return code,
provider policy summary, auth broker summary, cleanup outcome, and git metadata
when available.

| Need | Commands |
| --- | --- |
| Review completed runs | `runs list`, `runs show`, `runs log`, `runs diff` |
| Inspect or control active runs | `runs active`, `runs status`, `runs attach`, `runs logs-follow`, `runs stop`, `runs kill`, `runs repair` |
| Review safety and policy decisions | `why host`, `why workspace`, `why network`, `why state`, `egress log` |
| Inspect auth broker state | `auth status`, `auth explain`, `auth log` |
| Repair bundled images and inspect builder state | `image doctor`, `image rebuild`, `image build --dry-run` |
| Manage isolated state volumes | `state list`, `state reset`, `state prune` |
| Manage RunHaven networks | `network list`, `network prune --yes` |

Run records do not store diffs, file contents, prompts, command lines, agent
arguments, attach commands, environment variable names, environment values,
request bodies, or token values. Live container logs may still contain whatever
the agent process printed during the run.

### Local Record Files

RunHaven keeps three append-only JSON Lines files under its cache directory
(`RUNHAVEN_CACHE_HOME`, otherwise `~/Library/Caches/runhaven/`), created with
owner-only permissions:

| File | Holds |
| --- | --- |
| `runs.jsonl` | One record per completed run: ids, profile, workspace path, network mode, return code, and provider/auth summaries. |
| `egress-policy.jsonl` | Provider proxy allow/deny decisions with target host and port. |
| `auth-broker.jsonl` | Codex broker allow/deny decisions with method and upstream path. |

These on-disk record shapes are best-effort and pre-stable: they carry no schema
version and may change before `v1.0.0`, so do not parse them as a stable
contract yet. The `--json` output of `runs`, `egress log`, and `auth log` is the
supported read path. The files are append-only with no rotation or size cap, so
they grow over time; delete a file to reset that history. They store metadata
only (paths, hostnames, decisions) and never secrets, tokens, request bodies, or
command arguments.

## Current Limits

RunHaven is not a complete data-loss or exfiltration solution.

- Internet mode does not restrict outbound domains.
- Provider mode uses conservative host allowlists; some provider features may
  need additional reviewed fully qualified hosts.
- The selected agent can still read files inside `/workspace` and its isolated
  agent home volume.
- Credentials inside the agent home volume or passed with `--env NAME` may be
  misused by malicious repository content.
- Host-side API-key brokering covers Codex and Claude. Copilot and Antigravity
  are not brokered and use isolated in-container login state.
- Agent-native approval systems are useful, but they are not a replacement for
  the outer container boundary.

See [Security model](SECURITY_MODEL.md) and [Security policy](../SECURITY.md)
for the full boundary.
