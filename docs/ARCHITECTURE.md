# Architecture

`runhaven` is a Rust CLI around Apple `container`. It does not try to
replace the agent CLIs. Its job is to make the safe container boundary easy to
choose and hard to accidentally widen.

An alpha Tauri/Svelte desktop scaffold lives under `src-tauri/` and `ui/`.
The Tauri crate depends on the root Rust library and exposes typed commands
for setup, dashboard, profile, folder-pick, run-plan review, and the first
explicitly confirmed launch path. It does not expose a shell, filesystem,
process, HTTP, or Apple `container` bridge to JavaScript. Stop, kill, attach,
repair, logs, image build, state cleanup, and worktree review remain CLI-first
until each gets a typed command, explicit confirmation, focused tests, and a
narrow capability.

Release architecture is two-stage. `v0.5.0` should complete the CLI contract
and keep the CLI as the stable backend and automation surface. `v1.0.0` should
make the desktop app the easiest safe path, backed by the same Rust planner,
validators, records, provider policy, auth metadata, and cleanup rules.

## Runtime Pattern

Default runs use task-scoped `container run`. Apple `container machine` is an
advanced, user-chosen path, not the default RunHaven boundary.

Reason: `container machine` is convenient, but its normal workflow maps the
user's macOS home directory into the guest. That is the wrong beginner default
for AI agents because it can expose dotfiles, cloud credentials, SSH material,
and unrelated repositories.

RunHaven should not block explicit or user-managed machine workflows solely
because they are less secure. If RunHaven adds machine integration, it should
warn about host-home, credential, persistence, and cleanup tradeoffs and require
explicit intent before use.

`runhaven run` generates this shape:

- host workspace mounted at `/workspace`
- per-project/profile/session named volume mounted at `/home/agent`
- read-only root filesystem
- tmpfs at `/tmp`
- non-root `agent` user in bundled images
- no host home mount
- no host secret mount
- explicit environment passthrough only
- `--ssh` fails closed until Apple `container` non-root forwarding is verified
- interactive TTY allocation when requested or attached to a terminal
- explicit lower-security overrides (sensitive workspace, custom or root
  `--user`, custom `--image`, extra `--provider-host`, `--env`), each printed as
  a plain-language security notice at plan and run time

Before a non-root bundled agent starts, `runhaven` prepares the per-project home
volume in a short-lived root container so `/home/agent` is writable by UID/GID
1000. That preflight mounts only the named home volume, uses a read-only root
filesystem, disables DNS, and attaches to a dedicated internal network.

Because Apple container named volumes cannot be attached to two running
containers at the same time, `runhaven run` holds a host-side lock for the
selected state volume until the run exits. Concurrent runs for the same
workspace/profile/session fail early with a clear message instead of surfacing
a low-level VM storage error. The implicit default session preserves existing
per-project/profile volume names. `--session NAME` selects a deterministic
named-session volume for the same project and profile.

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

The default network mode is profile-aware: profiles with bundled provider hosts
default to `provider` so the agent reaches its own API but not arbitrary hosts,
and profiles without bundled hosts default to `internet`, where provider mode
would have an empty allowlist. `internet` uses Apple container's default network
and is unrestricted; pass `--network internet` for any profile that needs it.

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

Provider-mode runtime tests and live Apple `container` smokes prove the proxy
pattern with allowed proxied HTTPS plus denied proxied host, proxied IP
literal, direct DNS, and direct IP paths. Internet mode remains unrestricted
egress.

Managed-network cleanup is explicit. `runhaven network list` reads
`container network list --quiet` and shows only RunHaven-owned network names.
`runhaven network prune --yes` deletes only those managed names: the
volume-preparation network, per-project internal networks, and per-run provider
networks. It does not delete the Apple default network, Apple-managed runtime
networks, arbitrary user networks, images, volumes, or workspace files.

Image diagnostics are read-only. `runhaven image doctor [AGENT]` reads
`container image list --format json`, extracts image names from
`configuration.name` and descriptor annotation values, accepts both RunHaven
tags and `docker.io/`-prefixed tags, and compares the local image with the
current bundled template inputs. New RunHaven image builds carry profile and
source-digest labels; older unlabeled images are checked with image/template
timestamps. `image doctor` returns nonzero when a selected bundled profile
image is absent or stale. It also reads `container volume list --quiet` and the
secret-free active-run markers to report inactive RunHaven state volumes for
the selected profile. It prints repair guidance, but it does not build images,
delete resources, mount workspaces, or reset state.

## Run Records

Actual `runhaven run` executions append one JSON object to `runs.jsonl` under
the RunHaven cache root. While a run is active, RunHaven also writes a
temporary secret-free marker under `active-runs/` with run id, profile,
workspace, network mode, session, state volume, host pid, and the
RunHaven-owned container name. `runhaven runs active` lists current active
markers.
`runhaven runs status RUN_ID` reads one marker, calls Apple `container inspect`
for the named container, and prints only curated state, image, resource, and
network fields.
`runhaven runs attach RUN_ID` reads one marker and calls Apple `container exec`
to start a guarded shell or command in the active container, defaulting to the
non-root `agent` user in `/workspace` with `--user`, `--allow-root-user`,
`--workdir`, and `--tty` overrides. `runhaven runs
logs-follow RUN_ID` reads one marker and calls Apple `container logs --follow`
for recent and live container output. `runhaven runs stop RUN_ID` reads one
marker and calls Apple `container stop` for the named container. `runhaven runs
kill RUN_ID` reads one marker and calls Apple `container kill` for explicit
hard-stop recovery. `runhaven runs repair RUN_ID` removes a stale active marker
only after Apple `container inspect` reports that the recorded RunHaven-owned
container is not found. `runhaven runs repair --all` applies the same
confirmed-missing guard to every valid active marker. `runhaven runs repair`
also supports JSON output for repair results, counts, and exit code without raw
Apple inspect output or marker contents. The marker is removed when the run
finishes.
`runhaven runs list`, `runhaven runs show RUN_ID`, and `runhaven runs log
RUN_ID` read the completed-run ledger. Records include run id,
timestamps, profile, workspace, session, state volume, network mode, return
code, provider policy summary, auth broker summary, cleanup outcome, and git
change metadata when the workspace is inside a git repository. Git metadata
records repo root, before
and after `HEAD`, dirty state, changed file count, and a capped list of
relative paths scoped to the selected workspace. Worktree runs also record the
source repo, RunHaven-owned worktree path, branch, base `HEAD`, mounted
workspace, and recovery commands. `runs log` joins the run record with matching
`egress-policy.jsonl` and `auth-broker.jsonl` entries for the same run id.
Worktree review commands can also suggest detected local test and lint commands
against the recorded worktree, but they do not execute those suggestions.
`runs diff` validates the recorded git metadata against live git state and
then prints a live `git diff`; it does not read or store patches from
`runs.jsonl`. These commands intentionally omit diffs, file contents, prompts,
the `container run` command, agent arguments, environment variable names,
environment values, request bodies, and token values from persisted ledgers and
active-run markers. `runs status` also avoids printing raw `container inspect`
arguments, environment, and mount fields.

## Auth Broker Model

Auth brokering is a separate host-side boundary from provider egress. The
current implementation includes an opt-in Codex API-key broker prototype and
static inspection commands:

```bash
runhaven auth status
runhaven auth explain codex
```

`src/runhaven/provider/auth_profiles.rs` records per-profile auth surfaces,
current safe paths, and broker notes. `src/runhaven/provider/auth_broker.rs`
owns the host-side API-key broker for Codex, Claude, and Gemini. `runhaven auth`
reads static metadata only. It does not inspect Keychain, browser profiles,
provider login caches, cloud credential files, or environment values. During a
real run with `--api-key-broker-env`, the host process reads only the named
environment variable, starts a subnet-restricted broker on the provider network,
and redirects the guest at the broker (Codex custom-provider config, or an
`ANTHROPIC_BASE_URL` / `GOOGLE_GEMINI_BASE_URL` env). Broker decisions are
written to `auth-broker.jsonl` under the RunHaven cache root. The log records
method, sanitized path, allow/deny outcome, reason, upstream status, count, and
run id; it does not record request bodies, token values, or environment variable
names. Optional real non-interactive smokes should use a disposable API key and
the normal `runhaven run AGENT --network provider --api-key-broker-env NAME`
path.

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
