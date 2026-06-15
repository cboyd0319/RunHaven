# Capabilities

RunHaven runs AI coding agents inside Apple `container` with beginner-safe
defaults for local Mac development. This page summarizes what the CLI can do,
what it protects by default, and where the current limits are.

RunHaven only supports macOS 26+ on Apple silicon. Windows and Linux are not
supported runtimes or contributor verification targets.

## Runtime Boundary

RunHaven generates Apple `container` commands with these defaults:

- one selected project mounted at `/workspace`
- one per-project agent home volume mounted at the container agent home path
- no macOS home directory mount
- no raw SSH key mount
- no host cloud credential mount
- no arbitrary host environment passthrough
- read-only container root filesystem
- temporary container scratch directory
- dropped Linux capabilities
- non-root `agent` user in bundled images
- explicit command preview with `runhaven plan`

The default workspace scope is the current directory. If that directory is
inside a larger git repository, RunHaven does not silently broaden the mount.
Use `--workspace-scope git-root` only when the agent needs the full repository
mounted at `/workspace`.

## Agent Profiles

Bundled profiles:

| Profile | Default image | Use case |
| --- | --- | --- |
| `claude` | `runhaven/claude:0.1.0` | Claude Code with isolated project state |
| `codex` | `runhaven/codex:0.1.0` | Codex CLI with its own workspace sandbox enabled |
| `gemini` | `runhaven/gemini:0.1.0` | Gemini CLI with project-scoped home state |
| `antigravity` | `runhaven/antigravity:0.1.0` | Antigravity CLI in the same container boundary |
| `copilot` | `runhaven/copilot:0.1.0` | GitHub Copilot CLI with isolated state |
| `shell` | `runhaven/base:0.1.0` | Generic shell profile for custom agent images |

Use `shell` with `--image IMAGE` for a custom agent image.

## Network Modes

`internet` is the default network mode. It is intentionally broad and does not
enforce provider-domain allowlisting. Use it for package managers, dependency
updates, registries, CDNs, and other tasks that need unrestricted internet
egress.

`internal` creates a host-only Apple container network. It is useful for local
commands and custom images that do not need internet access.

`provider` creates a managed internal Apple `container` network and routes the
agent through RunHaven's host-side allowlist CONNECT proxy. Bundled profiles
include conservative provider hosts. A listed host permits that host and its
subdomains. The proxy resolves allowed hosts before connecting and rejects
non-public resolved addresses such as loopback, private, link-local, multicast,
or otherwise local-only addresses.

Provider mode is stricter than internet mode and may require reviewed
fully-qualified `--provider-host HOST` additions for login, telemetry,
package-registry, update, or provider feature-path hosts. Review
[the provider endpoint matrix](PROVIDER_ENDPOINTS.md) before adding hosts.

## Workspace And Git Safety

Use the smallest project directory the agent needs. RunHaven rejects broad or
credential-bearing workspace paths unless `--allow-sensitive-workspace` is
passed.

For clean git repositories, `runhaven run AGENT --worktree` creates a
RunHaven-owned branch and git worktree under RunHaven's cache directory, then
mounts that worktree at `/workspace`. The source checkout is left untouched.
RunHaven keeps the worktree after the run and records exact review, merge, and
discard commands in the run record.

`runhaven runs keep RUN_ID` validates the recorded RunHaven-owned worktree and
prints review commands without mutating anything. `runhaven runs merge RUN_ID`
validates the source repository, branch, worktree path, and base commit before
bringing committed, dirty, and untracked worktree changes back to the source
checkout and then cleaning up the RunHaven worktree and branch. If a
pre-cleanup merge check fails, RunHaven prints the source repo, worktree,
branch, review, retry, keep, and discard commands without deleting the
recorded worktree. `runhaven runs
discard RUN_ID` validates the same ownership boundary, then removes the
recorded worktree and branch without touching the source checkout.

## Credential Handling

RunHaven does not mount raw SSH keys, browser profiles, cloud credential
folders, or provider login caches by default.

Useful opt-in controls:

- `--ssh` forwards the macOS SSH agent without mounting `~/.ssh`.
- `--env NAME` passes one reviewed host environment variable by name.
- `--codex-api-key-broker-env NAME` enables the Codex API-key broker prototype
  for provider-network Codex runs.
- `runhaven auth status` and `runhaven auth explain AGENT` explain current
  broker boundaries without reading or printing secrets.

`runhaven` rejects `NAME=value` so secrets do not get copied into shell history
or dry-run output.

## Run Observability And Recovery

RunHaven records secret-free run metadata under its cache directory:

- `runhaven runs list`
- `runhaven runs show RUN_ID`
- `runhaven runs log RUN_ID`
- `runhaven runs diff RUN_ID`
- `runhaven runs keep RUN_ID`
- `runhaven runs merge RUN_ID`
- `runhaven runs discard RUN_ID`
- `runhaven runs active`
- `runhaven runs status RUN_ID`
- `runhaven runs attach RUN_ID`
- `runhaven runs logs-follow RUN_ID`
- `runhaven runs stop RUN_ID`
- `runhaven runs kill RUN_ID`
- `runhaven runs repair RUN_ID`
- `runhaven runs repair --all`

Run records include run id, profile, workspace, workspace scope, network mode,
return code, provider policy summary, auth broker summary, cleanup outcome, and
git change metadata when available. Worktree run records also include source
repo, worktree path, branch, base commit, mounted workspace, and recovery
commands.

Run records do not store diffs, file contents, prompts, command lines, agent
arguments, attach commands, environment variable names, environment values,
request bodies, or token values. Live container logs may still contain whatever
the agent process printed during the run.

## State Management

Each project/profile gets an isolated agent home volume. List and prune those
volumes with:

```bash
runhaven state list
runhaven state prune --yes
```

`state prune --yes` deletes RunHaven agent home volumes. It does not touch
workspace files.

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
