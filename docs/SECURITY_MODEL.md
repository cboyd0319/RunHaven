# Security Model

## Boundary

Trusted:

- the user
- this Rust CLI
- Apple `container`
- the selected agent image

Untrusted or partially trusted:

- repository contents
- model output
- package install scripts
- MCP servers
- shell commands selected by an agent
- external network responses

Sensitive:

- macOS home directory
- SSH keys and agent socket
- cloud credentials
- model-provider credentials
- browser profiles
- unrelated repositories
- agent session logs

## Default Protections

`runhaven` protects against accidental broad local access by default:

- mounts only the selected workspace
- isolates agent home state in a project/profile/session named volume
- runs a read-only root filesystem
- drops Linux capabilities
- uses a non-root user in bundled images
- does not mount `~/.ssh`, `~/.aws`, `~/.config`, or the macOS home directory
- does not pass host environment variables unless named with `--env`
- rejects broad or credential-bearing workspace paths unless
  `--allow-sensitive-workspace` is passed
- rejects root agent execution unless `--allow-root-user` is passed
- shows the exact command with `runhaven plan`

For bundled non-root images, `runhaven` runs a short volume-preparation preflight
before the agent starts. That preflight mounts only the project-scoped
`/home/agent` volume, sets ownership for UID/GID 1000, runs without DNS, and
uses a dedicated internal network.

`runhaven` also serializes access to each project/profile/session home volume.
This avoids concurrent attachment of the same named volume and keeps the
failure mode understandable for non-technical users. Named sessions selected
with `--session NAME` reuse only the isolated `/home/agent` volume; they do not
widen workspace mounts or host credential access. `state reset` and
session-filtered `state prune` delete RunHaven-managed home volumes only.

By default (`--auth-scope agent`) an agent's home volume, including its login,
is shared across all your projects (`runhaven-<agent>-shared-home`), so an OAuth
or subscription login is done once and reused everywhere. This trades isolation
for convenience: a workspace you run could read or alter that shared login,
though in `provider` mode the egress allowlist still blocks exfiltrating the
credential off the host. Use `--auth-scope project` to isolate a project's login
and state to its own per-workspace volume. The shared volume is removed by
`state prune` like any other RunHaven home volume.

`network list` and `network prune --yes` operate only on RunHaven-managed
Apple `container` network names and do not delete arbitrary Apple-managed or
user-created networks.

`image doctor` is read-only. It lists local Apple `container` image metadata,
checks expected bundled RunHaven image tags, compares RunHaven image source
metadata, reviews inactive RunHaven state volume names, and does not build
images, delete resources, mount workspaces, read credentials, or reset state.

## Secure Easy Path And Explicit Overrides

Above all else, the default and easiest path must be the secure path. New CLI
and UI workflows should make the narrow choice the shortest path: smallest
selected workspace, non-root bundled image, no host credential mounts, no
arbitrary environment passthrough, no raw SSH keys, explicit network choice,
and visible plan output.

Supported advanced choices warn but do not hide or block. Lower-security run
configuration (a sensitive workspace, a custom or root `--user`, a custom image,
`--env` passthrough, or an additional provider host) requires an explicit flag
and prints plain-language `Security notices` at plan and run time so the tradeoff
is visible. Destructive resource operations (state deletion, network pruning,
worktree merge or discard, hard-stop recovery) require explicit confirmation such
as `--yes`. Apple `container machine` use is warned and explicit rather than
blocked.

The default network mode is profile-aware so the secure path is also the default
path: profiles with bundled provider hosts default to `provider` (egress limited
to the agent's own API), and profiles without bundled hosts default to `internet`
because provider mode would have an empty allowlist there. Internet mode prints a
security notice and is never blocked; choose it explicitly with `--network
internet` when a run needs package installs or other hosts.

Unsupported, invalid, or nonfunctional paths still fail closed. Examples include
failed setup checks, invalid input, missing confirmations, unsupported platform
state, and `--ssh` while the verified Apple `container` non-root forwarding path
does not work. This is not a policy block around a supported choice; it is a
runtime correctness and safety failure.

## Container Machine Is Not Default

RunHaven uses task-scoped `container run` commands instead of Apple's persistent
`container machine` workflow by default. The machine workflow is useful for
Linux development environments, but it is the wrong beginner-safe default for
AI coding agents because it can map the host user and home directory into the
guest. Secondary hands-on reporting also notes that the safer machine option is
to disable that home mount.

RunHaven's default boundary is narrower: mount one selected workspace, attach
one project-scoped agent home volume, and never mount the macOS home directory
or raw credential folders by default.

Explicit or user-managed `container machine` workflows are supported
lower-security choices, not hard policy violations. RunHaven should warn about
host-home, credential, persistence, and cleanup tradeoffs before integrating
with or managing a machine workflow, and require explicit user intent. It should
fail only for concrete unsupported or unsafe states, such as invalid targets,
missing confirmation, unimplemented management flows, or destructive operations
without a reviewed approval gate.

## Auth Broker Boundary

`runhaven auth status` and `runhaven auth explain AGENT` describe the host-side
provider credential broker boundary. The current broker status is a Codex
API-key prototype with all other agent brokers design-only. Those commands read
static profile metadata. They do not inspect Keychain, browser profiles, cloud
credential files, provider login caches, or environment variable values, and
they do not print secrets.

The intended pattern is host-owned credentials with provider-specific policy
tied to the endpoint matrix. The guest should receive only a narrow
provider action or short-lived run credential when that flow is explicitly
implemented and verified. Broad host credential import, implicit environment
passthrough, and host home or credential-folder mounts remain out of scope.

## Extension And MCP Boundary

RunHaven does not currently enable MCP servers, editor extensions, or plugin
marketplaces inside managed agent runs. Future support is deny-by-default and
must follow [`EXTENSION_MCP_BOUNDARY.md`](EXTENSION_MCP_BOUNDARY.md): no host
socket, credential helper, extension, or MCP server is exposed unless the user
explicitly enables it and the plan can show the exact access before launch.

## What This Does Not Solve Yet

Internet mode is unrestricted egress within the host's network policy. It is the
default only for profiles without bundled provider hosts, and is available to any
profile with `--network internet`. Use `internal` for local-only runs.

`--network provider` is the constrained egress path and the default for profiles
with bundled provider hosts.
It runs the agent on an internal Apple `container` network, starts a host-side
CONNECT proxy, injects proxy environment variables at runtime, and deletes the
managed provider network after the run. Apple `container` 1.0.0 exposes an
internal gateway address to guests that is not always bindable on the macOS
host, so the host listener can bind wildcard while rejecting clients outside the
inspected Apple `container` subnet. The proxy permits the bundled provider hosts
for the selected profile, their subdomains, and explicit fully qualified
`--provider-host HOST` additions. It rejects IP literal proxy targets and
single-label provider hosts. Before opening an upstream connection, the proxy
resolves the destination and rejects non-public resolved addresses such as
loopback, private, link-local, multicast, or otherwise local-only addresses.
It relies on the internal network to block direct guest egress. Blocked proxy
targets are grouped after provider runs with run id, count, denial reason,
matched rule, and suggested next action so users can review missing endpoints
without weakening the default policy.

Host-only networks block guest egress to the internet, but they do not firewall
the host's own listening ports. A guest on an `internal` or `provider` network
can open a raw TCP connection to any service bound to the host on the Apple
`container` gateway address or `0.0.0.0`, for example a local dev server, a
database, or another tool's proxy. Apple `container` 1.0.0 has no per-port
guest-to-host firewalling, so this is a runtime limitation, not a
misconfiguration. It was empirically confirmed on 2026-06-26 (macOS 27.0): a
guest retrieved a sentinel from a host listener on the gateway while its direct
internet egress was refused. Treat the host as reachable from active runs and do
not bind sensitive services to `0.0.0.0` or the gateway interface while a run is
active. An in-guest egress filter could close this as defense in depth; it is
tracked as a design-first item in [`NON_UI_BACKLOG.md`](NON_UI_BACKLOG.md).

Provider runs also append allowed and denied CONNECT policy decisions to a
RunHaven cache log. `runhaven egress log` shows recent decisions, including the
matched rule, denial reason, count, profile, workspace, and run id. `runhaven
why host HOST` explains bundled provider-host matches, IP-literal rejection,
and the next review step before the user adds a new host.

Actual agent runs also append one secret-free record to `runs.jsonl`.
`runhaven runs list`, `runhaven runs show RUN_ID`,
`runhaven runs log RUN_ID`, and `runhaven runs diff RUN_ID` expose
completed-run metadata such as run id, profile, workspace, network mode,
return code, provider policy summary, auth broker summary, cleanup outcome, and
matching provider/auth log entries for the run. While a run is active, a
temporary active-run marker records the RunHaven-owned container name so
`runs active` can list current run id, profile, workspace, network mode,
status, and container name. The same marker lets `runs status` call Apple
`container inspect` for curated live state, lets `runs attach` call Apple
`container exec` for a guarded shell or command, lets `runs logs-follow` call
Apple `container logs --follow`, lets `runs stop` call Apple `container stop`,
lets `runs kill` call Apple `container kill`, and lets `runs repair` remove a
stale marker only after Apple `container inspect` reports that the recorded
RunHaven-owned container is not found. `runs repair --all` applies the same
confirmed-missing guard to each valid active marker. Repair JSON output
contains only result status, counts, exit code, run id, container name, and
inspect return code; the marker is removed after the run finishes.
When the workspace is inside a git repository, the run ledger also records repo
root, before and after `HEAD`, dirty state, changed file count, and a capped
list of relative paths scoped to the selected workspace. Run records also store
the workspace scope choice, so later review can distinguish the default
current-directory mount from an explicit `--workspace-scope git-root` run.
For `--worktree` runs, RunHaven requires a clean source repository, creates a
RunHaven-owned branch and git worktree, mounts the worktree instead of the
source checkout, and records the source repo, worktree path, branch, base
commit, and recovery commands. Dirty source checkouts fail before worktree
creation and print safe next choices. `runs recover` and `runs recover --json`
are read-only. Suggested project checks are advisory commands and are not run
automatically. RunHaven does not automatically merge or delete that worktree
after the run.
The run ledger and active-run markers do not record diffs, file contents,
prompts, command lines, agent arguments, attach commands, environment variable
names, environment values, request bodies, or token values. Live container logs
may still contain whatever the agent process printed during the run. `runs diff`
prints a live git diff on demand only after verifying the recorded repo root,
`HEAD`, and path set still match; dirty working-tree diffs include a warning
because metadata cannot prove file contents stayed unchanged after the run.
`runs status` does not print raw Apple `container inspect` data because that
data can include process arguments, environment, and mount details.

Provider host allowlists are intentionally conservative and source-backed.
Bundled auth and provider routing hosts are tracked in
[`PROVIDER_ENDPOINTS.md`](PROVIDER_ENDPOINTS.md). Telemetry, reporting,
release-note, update, plugin marketplace, and broad path-sensitive hosts may
fail until an additional fully qualified host is reviewed and passed with
`--provider-host`.

The host-side auth broker covers the API-key path for Codex, Claude, and Gemini
through `--api-key-broker-env NAME`. It keeps the raw API key in the RunHaven
host process and gives the guest only a placeholder key plus a base-URL redirect
at the broker (Codex custom-provider config, or `ANTHROPIC_BASE_URL` /
`GOOGLE_GEMINI_BASE_URL`). Broker decisions are logged without request bodies,
token values, or environment variable names. Even with the broker the credential
is usable, not readable: a compromised guest cannot read the key but can still
spend through it within the pinned host and path the broker forwards to. The
broker is for API keys only; OAuth and subscription logins (and Copilot, which
cannot be brokered without TLS interception) use isolated in-container state, and
RunHaven never reads your host `~/.claude.json` or Keychain. `--env NAME` remains
an explicit fallback that places a token in the guest. See
[`AUTH_BROKER.md`](AUTH_BROKER.md). Use
[`AUTH_BROKER.md`](AUTH_BROKER.md) for the current boundary and non-goals.

The selected agent still controls what it reads inside `/workspace` and
`/home/agent`. If the agent has model credentials inside its project volume and
internet access, malicious repository content may still try to exfiltrate those
credentials. Agent-native permission systems remain useful, but they are not a
replacement for the outer container boundary.

## Safe Defaults for Beginners

Use this order:

1. Run `runhaven plan`.
2. Build or select a known image.
3. Run without `--env` first and authenticate inside the isolated agent home
   volume when possible.
4. Use `--env NAME` only when a headless run needs a token.
5. Use `--read-only-workspace` for review and audit tasks.
6. Treat `--ssh` as disabled until Apple `container` non-root forwarding is
   verified; do not mount raw SSH keys as a workaround.
7. Use `--allow-sensitive-workspace` or `--allow-root-user` only when the
   security tradeoff is intentional.
