# Capabilities

RunHaven runs AI coding agents inside Apple `container` with local-Mac safety
defaults. This page is the product overview: what RunHaven can do, what it
protects by default, and where the current limits are.

RunHaven only supports macOS 26+ on Apple silicon. Windows and Linux are not
supported runtimes or contributor verification targets.

RunHaven remains alpha/pre-release until after `v0.5.0`. The CLI is the
current working product surface. `v0.5.0` should finish and verify the CLI
contract; `v1.0.0` should make the Tauri desktop app the first-class safe path
for less-technical users.

For command walkthroughs, use [Usage](USAGE.md). For the full security boundary,
use [Security model](SECURITY_MODEL.md).

## At A Glance

| Area | What RunHaven provides |
| --- | --- |
| Runtime boundary | One selected workspace, one isolated agent home volume, non-root bundled images, read-only root filesystem, dropped Linux capabilities, and explicit command preview. |
| Agent profiles | Bundled Claude, Codex, Gemini, Antigravity, Copilot, and shell profiles, plus custom images through the `shell` profile. |
| Network modes | Broad default internet, local-only internal networking, or provider allowlist proxy mode. |
| Workspace safety | Current-directory mounts by default, explicit git-root expansion, sensitive-path rejection, and optional RunHaven-owned git worktrees. |
| Credentials | No host home, raw SSH key, browser profile, cloud credential folder, or arbitrary environment passthrough by default. |
| Observability | Secret-free run records, active-run controls, provider policy logs, auth broker status, and recovery commands. |
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

| Profile | Default image | Use case |
| --- | --- | --- |
| `claude` | `runhaven/claude:0.1.0` | Claude Code with isolated project state |
| `codex` | `runhaven/codex:0.1.0` | Codex CLI with its own workspace sandbox enabled |
| `gemini` | `runhaven/gemini:0.1.0` | Gemini CLI with project-scoped home state |
| `antigravity` | `runhaven/antigravity:0.1.0` | Antigravity CLI in the same container boundary |
| `copilot` | `runhaven/copilot:0.1.0` | GitHub Copilot CLI with isolated state |
| `shell` | `runhaven/base:0.1.0` | Generic shell profile for custom agent images |

Use `shell` with `--image IMAGE` when you want to run another agent image.

## Network Modes

| Mode | Use it for | Egress behavior |
| --- | --- | --- |
| `internet` | Normal hosted-agent runs, package managers, registries, CDNs, and dependency updates. | Broad default internet access. Provider-domain allowlisting is not enforced. |
| `internal` | Local-only analysis, offline tests, and custom images that do not need internet. | Host-only Apple `container` network. |
| `provider` | Agent runs that should be limited to reviewed provider hosts. | Managed internal network plus RunHaven's host-side allowlist CONNECT proxy. |

Provider mode allows bundled provider hosts for the selected profile, their
subdomains, and reviewed fully qualified `--provider-host HOST` additions. The
proxy rejects IP literal targets and destinations that resolve to local-only
addresses such as loopback, private, link-local, or multicast ranges.

Provider mode is intentionally stricter than internet mode. Some login,
telemetry, package-registry, update, or provider feature-path hosts may need
review before being added. Use the
[provider endpoint matrix](PROVIDER_ENDPOINTS.md) for that review.

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

| Opt-in | What it does |
| --- | --- |
| `--ssh` | Currently fails closed until Apple `container` non-root SSH forwarding is verified. |
| `--env NAME` | Passes one reviewed host environment variable by name. `NAME=value` is rejected. |
| `--codex-api-key-broker-env NAME` | Enables the Codex API-key broker prototype for provider-network Codex runs. |
| `runhaven auth status` / `runhaven auth explain AGENT` | Explains current broker boundaries without reading or printing secrets. |

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

## Current Limits

RunHaven is not a complete data-loss or exfiltration solution.

- Default internet mode does not restrict outbound domains.
- Provider mode uses conservative host allowlists; some provider features may
  need additional reviewed fully qualified hosts.
- The selected agent can still read files inside `/workspace` and its isolated
  agent home volume.
- Credentials inside the agent home volume or passed with `--env NAME` may be
  misused by malicious repository content.
- Host-side auth brokering has an opt-in Codex API-key prototype. Other agent
  auth brokers remain design-only.
- Agent-native approval systems are useful, but they are not a replacement for
  the outer container boundary.

See [Security model](SECURITY_MODEL.md) and [Security policy](../SECURITY.md)
for the full boundary.
