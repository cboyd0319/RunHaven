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

## What This Does Not Solve Yet

The default `internet` network mode should still be treated as unrestricted
egress within the host's network policy. Use `internal` for local-only runs.

`--network provider` is now the constrained egress path for normal agent runs.
It runs the agent on an internal Apple `container` network, starts a host-side
CONNECT proxy, injects proxy environment variables at runtime, and deletes the
managed provider network after the run. The proxy permits the bundled provider
hosts for the selected profile, their subdomains, and explicit fully qualified
`--provider-host HOST` additions. It rejects IP literal proxy targets and
single-label provider hosts, and relies on the internal network to block direct
guest egress.

Provider host allowlists are intentionally conservative. Authentication,
telemetry, or optional provider feature paths may fail until an additional
fully qualified host is reviewed and passed with `--provider-host`.

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
