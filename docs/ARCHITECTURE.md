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
- interactive TTY allocation when requested or attached to a terminal
- explicit unsafe overrides for sensitive workspaces and root agent execution

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
When reusing an existing network name, `runhaven` checks Apple `container`
network inspection output and requires `configuration.mode` to be `hostOnly`.

`provider` creates a managed internal Apple `container` network, inspects that
network for its IPv4 gateway and subnet, starts a host-side CONNECT proxy, and
injects proxy environment variables into the agent run. The proxy allows only
the bundled provider hosts for the selected profile, their subdomains, and
explicit fully qualified `--provider-host HOST` additions. Bundled hosts are
maintained in the reviewed
[`PROVIDER_ENDPOINTS.md`](PROVIDER_ENDPOINTS.md) matrix and mirrored into the
profile metadata used by the planner.

The enforcement pattern is:

- run the agent on a managed internal Apple `container` network
- inspect the network's host gateway and subnet
- run a host-side CONNECT proxy with a reviewed provider host allowlist
- expose the proxy to the guest through the internal-network gateway
- reject clients outside the internal-network subnet when gateway-specific
  binding is not available
- block IP literal CONNECT targets at the proxy
- resolve allowed hosts before connecting and reject non-public resolved
  addresses
- aggregate allowed and denied proxy policy decisions for the run
- group blocked targets with run id, reason, count, rule, and suggested next
  action after the run
- append provider policy decisions to the RunHaven cache log after the run
- append a secret-free run record with provider policy, auth broker, and cleanup
  summaries
- delete the managed provider network after the run

`scripts/provider_egress_smoke.py` proves the lower-level proxy pattern with
live Apple `container` smokes. It can test a single allowed host or all bundled
hosts for a provider profile with `--agent AGENT`. The `runhaven run --network
provider` runtime path is also smoke-tested with allowed proxied HTTPS plus
denied proxied host, proxied IP literal, direct DNS, and direct IP paths.
Internet mode remains unrestricted egress.

## Run Records

Actual `runhaven run` executions append one JSON object to `runs.jsonl` under
the RunHaven cache root. `runhaven runs list` and `runhaven runs show RUN_ID`
read this ledger. Records include run id, timestamps, profile, workspace,
network mode, return code, provider policy summary, auth broker summary, and
cleanup outcome. They intentionally omit the `container run` command, agent
arguments, environment variable names, environment values, request bodies, and
token values.

## Auth Broker Model

Auth brokering is a separate host-side boundary from provider egress. The
current implementation includes an opt-in Codex API-key broker prototype and
static inspection commands:

```bash
runhaven auth status
runhaven auth explain codex
```

`src/runhaven/auth_broker.py` records per-profile auth surfaces, current safe
paths, broker notes, and the Codex Responses API broker. `runhaven auth` reads
static metadata only. It does not inspect Keychain, browser profiles, provider
login caches, cloud credential files, or environment values. During a real
Codex run with `--codex-api-key-broker-env`, the host process reads only the
named environment variable, starts a subnet-restricted broker on the provider
network, and injects temporary Codex custom-provider overrides into the guest.
Broker decisions are written to `auth-broker.jsonl` under the RunHaven cache
root. The log records method, sanitized path, allow/deny outcome, reason,
upstream status, count, and run id; it does not record request bodies, token
values, or environment variable names. `scripts/codex_broker_smoke.py` can run
an optional real Codex non-interactive smoke with a disposable API key.

The broker shape is:

- keep provider credentials owned by the host
- require explicit user opt-in for each provider account or credential source
- tie provider-specific broker policy to the endpoint matrix
- expose no raw credential to the guest by default
- audit brokered provider actions without logging secret values
- fail closed when the provider, host, path, or credential scope is not
  explicitly supported

A plain HTTPS CONNECT proxy cannot make path-aware decisions for TLS traffic
without intercepting provider TLS, so broad path-sensitive hosts are not solved
by provider egress alone. Host-side credential brokering is the preferred future
direction for those flows.
