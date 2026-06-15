# Usage

## Install for Development

RunHaven development and runtime verification require macOS 26+ on Apple
silicon. Windows and Linux are not supported.

```bash
python3.14 -m venv .venv
source .venv/bin/activate
python -m pip install pip==26.1.2
python -m pip install -r requirements-dev.txt
python -m pip install --no-deps -e .
```

Development tools are exact-pinned in `pyproject.toml` and
`requirements-dev.txt`. When updating them, use the current stable release and
commit the exact new version.

## Check the Mac

```bash
runhaven doctor
```

`doctor` checks Python, macOS, Apple silicon, the pinned Apple `container`
version, and the Apple container system status when the CLI is installed. A
newer Apple `container` release should fail until this repo updates its reviewed
pin.

## Build an Agent Image

```bash
runhaven image build claude
runhaven image build codex
runhaven image build gemini
runhaven image build antigravity
runhaven image build copilot
```

Dry-run first if you want to inspect the exact build command:

```bash
runhaven image build claude --dry-run
```

## Preview a Run

```bash
cd /path/to/project
runhaven plan claude
```

The plan prints:

- the mounted workspace
- the per-project state volume
- the selected network mode
- the egress status for that network mode
- any preflight command
- the exact `container run` command

## Run an Agent

```bash
runhaven run claude
```

`runhaven` allows one active run per project/profile state volume. If another run is
already using the same isolated home volume, `runhaven` fails before starting Apple
`container` and tells you to wait or use a different workspace/profile.

RunHaven allocates an interactive TTY when attached to a terminal. Use
`--tty never` for non-interactive automation.

Broker a Codex API key without placing the raw value in the guest:

```bash
runhaven run codex --network provider --codex-api-key-broker-env OPENAI_API_KEY
```

Pass a host environment variable by name only when the run deliberately needs
that value inside the guest:

```bash
runhaven run claude --env ANTHROPIC_API_KEY
runhaven run codex --env OPENAI_API_KEY
runhaven run gemini --env GEMINI_API_KEY
runhaven run copilot --env COPILOT_GITHUB_TOKEN
```

`runhaven` intentionally rejects `NAME=value` so secrets do not get copied into shell
history or dry-run output.

## Auth Broker Status

```bash
runhaven auth status
runhaven auth explain codex
runhaven auth explain codex --json
runhaven auth log --limit 20
runhaven auth log --json
```

The Codex API-key broker is an opt-in prototype. Other agent auth brokers remain
design-only. These commands describe the host-side broker boundary and the
current safe paths for each profile. They do not inspect Keychain, browser
profiles, provider login caches, cloud credential files, or environment variable
values, and they do not print secrets.

After a brokered Codex run, `runhaven auth log` shows secret-free broker
decisions: method, sanitized path, allow/deny outcome, reason, upstream status,
count, and run id. It never records request bodies, token values, or environment
variable names.

For non-Codex providers, authenticate inside the isolated agent state volume
when the provider supports interactive login. Use `--env NAME` only when a
headless run deliberately needs one token value inside the guest.

## Read-Only Review

```bash
runhaven run codex --read-only-workspace
```

This lets an agent inspect the project without writing to the mounted
workspace.

## Local-Only Network

```bash
runhaven run shell --network internal -- python -m unittest discover -s tests
```

`internal` creates a host-only Apple container network before the run. Hosted AI
agent CLIs usually need internet access for model traffic, so this mode is most
useful for local commands and custom images.

## Provider Network

```bash
runhaven plan claude --network provider
runhaven run claude --network provider
```

`provider` creates a managed internal Apple `container` network and routes the
agent through RunHaven's host-side allowlist CONNECT proxy. Bundled profiles
include conservative provider hosts. A listed host permits that host and its
subdomains. The proxy resolves allowed hosts before connecting and rejects
non-public resolved addresses such as loopback, private, link-local, or other
local-only addresses. Review the
[provider endpoint matrix](PROVIDER_ENDPOINTS.md) before adding fully qualified
extra hosts explicitly:

```bash
runhaven run shell --network provider --provider-host api.example.com
```

Run `runhaven plan` first. Provider plans show the managed provider network and
allowed hosts; the exact proxy URL is injected by `runhaven run` after the
internal-network gateway is inspected.

If a provider run tries to reach a host outside the allowlist, RunHaven prints a
grouped blocked-host review after the agent exits. The review includes the run
id, host, port, count, denial reason, matched rule, and suggested next action.
Review each blocked hostname before adding it with `--provider-host`; IP
literal targets cannot be allowed.

Explain a host before adding it:

```bash
runhaven why host api.openai.com --agent codex
runhaven why host api.example.com
runhaven why host 1.1.1.1
```

Inspect recent provider proxy policy decisions:

```bash
runhaven egress log --limit 20
runhaven egress log --json
```

The log is stored under RunHaven's cache directory. It records the profile,
workspace, host, port, decision, reason, matched rule, count, and run id.

## Run History

After an actual agent run, inspect the secret-free run ledger:

```bash
runhaven runs list --limit 20
runhaven runs show <run-id>
runhaven runs log <run-id>
runhaven runs show <run-id> --json
runhaven runs log <run-id> --json
```

Run records are stored under RunHaven's cache directory in `runs.jsonl`. They
include run id, profile, workspace, network mode, return code, provider policy
summary, auth broker summary, and cleanup outcome. They do not include command
lines, agent arguments, environment variable names, environment values, request
bodies, or token values. `runs log` joins the run record with matching
provider policy and auth broker entries for the same run id.

## Provider Egress Smoke

Build the base image and run the live smoke on macOS 26+:

```bash
runhaven image build shell
PYTHONPATH=src python3.14 scripts/provider_egress_smoke.py \
  --allowed-host api.openai.com \
  --allowed-url https://api.openai.com/ \
  --denied-host example.com
```

Run the same smoke against the bundled hosts for a provider profile:

```bash
PYTHONPATH=src python3.14 scripts/provider_egress_smoke.py --agent codex
```

The smoke creates a temporary internal Apple `container` network and starts a
host-side allowlist CONNECT proxy. It passes only when the allowed proxied HTTPS
path or paths succeed and denied proxied host, proxied IP literal, direct DNS,
and direct IP paths fail. Profile smoke mode proves that bundled provider hosts
are reachable through the proxy without requiring provider credentials. Normal
provider runs use the same proxy enforcement pattern through
`runhaven run --network provider`.

Run the optional Codex broker smoke only with a disposable OpenAI API key:

```bash
export RUNHAVEN_CODEX_BROKER_SMOKE_API_KEY=...
PYTHONPATH=src python3.14 scripts/codex_broker_smoke.py --require-api-key
```

Without the environment variable, the smoke prints `SKIP` and exits
successfully. The key value is inherited by the host process only; it is not
placed on the command line.

## Private Git

```bash
runhaven run claude --ssh
```

This forwards the macOS SSH agent socket using Apple `container --ssh`. It does
not mount `~/.ssh`.

## State Volumes

```bash
runhaven state list
runhaven state prune --yes
```

`state list` shows RunHaven agent home volumes. `state prune --yes` deletes
those isolated agent home volumes and does not touch workspace files.
