# Architecture

`runhaven` is a thin Python wrapper around Apple `container`. It does not try to
replace the agent CLIs. Its job is to make the safe container boundary easy to
choose and hard to accidentally widen.

## Runtime Pattern

Default runs use task-scoped `container run`, not `container machine`.

Reason: `container machine` is convenient, but its normal workflow maps the
user's macOS home directory into the guest. That is the wrong beginner default
for AI agents because it can expose dotfiles, cloud credentials, SSH material,
and unrelated repositories.

`runhaven run` generates this shape:

- host workspace mounted at `/workspace`
- per-project named volume mounted at `/home/agent`
- read-only root filesystem
- tmpfs at `/tmp`
- non-root `agent` user in bundled images
- no host home mount
- no host secret mount
- explicit environment passthrough only
- optional SSH agent forwarding with Apple `container --ssh`

Before a non-root bundled agent starts, `runhaven` prepares the per-project home
volume in a short-lived root container so `/home/agent` is writable by UID/GID
1000. That preflight mounts only the named home volume, uses a read-only root
filesystem, disables DNS, and attaches to a dedicated internal network.

Because Apple container named volumes cannot be attached to two running
containers at the same time, `runhaven run` holds a host-side lock for the selected
state volume until the run exits. Concurrent runs for the same workspace/profile
fail early with a clear message instead of surfacing a low-level VM storage
error.

## Profiles

Profiles define the image, default command, and agent-specific home variables.
They do not define trust in the agent.

Bundled profiles:

- `claude`
- `codex`
- `gemini`
- `antigravity`
- `copilot`
- `shell`

The `shell` profile is the escape hatch for any other agent image:

```bash
runhaven plan shell --image my/agent:2026.06.14 -- my-agent --help
```

## Network Model

`internet` is the default because most hosted AI CLIs need model-provider
network access to run. It uses Apple container's default network.

`internal` creates a per-project `container network create --internal` network
and runs the agent there. Use it for local-only analysis, offline tests, or
workflows that do not need a model-provider connection from inside the guest.

Domain egress allowlisting is a required future boundary. Until it lands,
internet-enabled runs can reach whatever the host and Apple container runtime
allow.
