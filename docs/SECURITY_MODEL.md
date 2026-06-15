# Security Model

## Boundary

Trusted:

- the user
- this Python wrapper
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
- isolates agent home state in a project-specific named volume
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

`runhaven` also serializes access to each project/profile home volume. This avoids
concurrent attachment of the same named volume and keeps the failure mode
understandable for non-technical users.

## Why Not Container Machine

RunHaven uses task-scoped `container run` commands instead of Apple's persistent
`container machine` workflow. The machine workflow is useful for Linux
development environments, but it is the wrong beginner-safe default for AI
coding agents because it can map the host user and home directory into the
guest. Secondary hands-on reporting also notes that the safer machine option is
to disable that home mount.

RunHaven's default boundary is narrower: mount one selected workspace, attach
one project-scoped agent home volume, and never mount the macOS home directory
or raw credential folders by default.

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

## What This Does Not Solve Yet

The default `internet` network mode should still be treated as unrestricted
egress within the host's network policy. Use `internal` for local-only runs.

`--network provider` is now the constrained egress path for normal agent runs.
It runs the agent on an internal Apple `container` network, starts a host-side
CONNECT proxy, injects proxy environment variables at runtime, and deletes the
managed provider network after the run. The proxy permits the bundled provider
hosts for the selected profile, their subdomains, and explicit fully qualified
`--provider-host HOST` additions. It rejects IP literal proxy targets and
single-label provider hosts. Before opening an upstream connection, the proxy
resolves the destination and rejects non-public resolved addresses such as
loopback, private, link-local, multicast, or otherwise local-only addresses.
It relies on the internal network to block direct guest egress. Blocked proxy
targets are grouped after provider runs with run id, count, denial reason,
matched rule, and suggested next action so users can review missing endpoints
without weakening the default policy.

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
status, and container name. The same marker lets `runs attach` call Apple
`container exec` for a guarded shell or command, lets `runs logs-follow` call
Apple `container logs --follow`, and lets `runs stop` call Apple
`container stop`; the marker is removed after the run finishes.
When the workspace is inside a git repository, the run ledger also records repo
root, before and after `HEAD`, dirty state, changed file count, and a capped
list of relative paths scoped to the selected workspace.
The run ledger and active-run markers do not record diffs, file contents,
prompts, command lines, agent arguments, attach commands, environment variable
names, environment values, request bodies, or token values. Live container logs
may still contain whatever the agent process printed during the run. `runs diff`
prints a live git diff on demand only after verifying the recorded repo root,
`HEAD`, and path set still match; dirty working-tree diffs include a warning
because metadata cannot prove file contents stayed unchanged after the run.

Provider host allowlists are intentionally conservative and source-backed.
Bundled auth and provider routing hosts are tracked in
[`PROVIDER_ENDPOINTS.md`](PROVIDER_ENDPOINTS.md). Telemetry, reporting,
release-note, update, plugin marketplace, and broad path-sensitive hosts may
fail until an additional fully qualified host is reviewed and passed with
`--provider-host`.

The host-side auth broker currently has an opt-in Codex API-key prototype. It
keeps the raw API key in the RunHaven host process and gives the guest only a
placeholder token plus temporary Codex custom-provider config. Broker decisions
are logged without request bodies, token values, or environment variable names.
Other providers remain design-only, and credentials can still reach the guest
through isolated in-agent login state or explicit `--env NAME` passthrough. Use
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
6. Use `--ssh` only when private Git access is required.
7. Use `--allow-sensitive-workspace` or `--allow-root-user` only when the
   security tradeoff is intentional.
