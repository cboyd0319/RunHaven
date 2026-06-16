# Usage

## Before You Start

Install RunHaven and Apple `container` first:
[Installation](INSTALLATION.md).

RunHaven development and runtime verification require macOS 26+ on Apple
silicon. Windows and Linux are not supported.

## Guided Setup

```bash
runhaven setup
runhaven setup --agent codex
```

`setup` is a safe first-run guide. It runs the same prerequisite checks as
`doctor`, prints exact fixes when the Mac is not ready, and then shows the
image build, plan, and run commands for the selected agent. It does not
install Apple `container`, start the container service, build images, run
agents, or mount any workspace. It also explains when to use local-only,
provider-only, package install, or unrestricted internet network modes, and
how to choose a workspace without mounting host credential paths.

## Check the Mac

```bash
runhaven doctor
```

`doctor` checks macOS, Apple silicon, the pinned Apple `container` version, and
the Apple container system status when the CLI is installed. A
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

Use `rebuild` when the local bundled image is stale or missing and you want the
repair intent to be clear:

```bash
runhaven image rebuild claude
runhaven image rebuild claude --dry-run
```

`image rebuild` uses the same exact pinned bundled image template and tag rules
as `image build`; it does not delete workspaces, state volumes, or other local
images. Bundled image builds include RunHaven profile and source-digest labels
so `image doctor` can compare local images with the checked-out templates.

Check local bundled image availability without changing local resources:

```bash
runhaven image doctor
runhaven image doctor claude
```

`image doctor` reads `container image list --format json`, checks for expected
bundled RunHaven image tags, compares RunHaven source-digest labels when
available, and falls back to image/template timestamps for older unlabeled
images. It exits nonzero when a selected image is missing or stale. It also
reviews inactive RunHaven state volumes for the selected profile, prints
rebuild, network, and state recovery commands, and does not build images,
delete resources, mount workspaces, read credentials, or reset state.

## Preview a Run

```bash
cd /path/to/project
runhaven plan claude
```

The plan prints:

- the mounted workspace
- the workspace scope choice
- the selected project session
- the per-project state volume
- the selected network mode
- the egress status for that network mode
- any preflight command
- the exact `container run` command

## Run an Agent

```bash
runhaven run claude
```

`runhaven` allows one active run per project/profile/session state volume. If
another run is already using the same isolated home volume, `runhaven` fails
before starting Apple `container` and tells you to wait or use a different
workspace, profile, or session.
When a run starts, RunHaven prints a run id to stderr. From another terminal,
use that id to request a graceful stop. If the id scrolls away, list active
runs first. To inspect or intervene while the run is active, attach from
another terminal:

```bash
runhaven runs active
runhaven runs status <run-id>
runhaven runs attach <run-id>
runhaven runs logs-follow <run-id>
runhaven runs stop <run-id>
runhaven runs kill <run-id>
runhaven runs repair <run-id>
runhaven runs repair --all
runhaven runs repair <run-id> --json
runhaven runs repair --all --json
```

`runs status` shows the active marker plus sanitized live Apple
`container inspect` state without opening a shell:

```bash
runhaven runs status <run-id>
runhaven runs status <run-id> --json
```

`runs attach` starts a new process inside the active container with Apple
`container exec`; it does not attach to the original agent process stream. By
default it opens `/bin/bash` as the non-root `agent` user in `/workspace`.
Pass a custom command after `--` when needed:

```bash
runhaven runs attach <run-id> -- pwd
```

Follow recent active-run output without opening a shell:

```bash
runhaven runs logs-follow <run-id>
runhaven runs logs-follow <run-id> --lines 50
```

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

## Workspace And Credentials

Run `runhaven` from the smallest project directory the agent needs. That
directory is mounted at `/workspace`. Do not run from your home directory, a
cloud sync root, or a credential folder unless you intentionally want that
broader scope and have reviewed the plan.

When the selected workspace is inside a larger git repository, RunHaven does
not silently broaden the mount. The default `--workspace-scope current` keeps
the selected directory mounted, and the plan prints a note naming the
containing repository root. Use `--workspace-scope git-root` only when the
agent needs the full repository mounted at `/workspace`; non-git directories
fail closed with that scope.

RunHaven does not mount raw SSH keys, browser profiles, cloud credential
folders, or provider login caches by default. Use `--ssh` for SSH agent
forwarding instead of mounting key files. Use `--env NAME` only for a reviewed
variable the agent really needs.

## Worktree Isolation

```bash
runhaven run claude --worktree --dry-run
runhaven run claude --worktree
```

`--worktree` requires a clean source git worktree with a committed `HEAD`.
When the source checkout is dirty, RunHaven refuses before creating a worktree
and prints choices to commit or stash first, run without `--worktree`, or start
from a separate clean clone or git worktree. For a clean source checkout,
RunHaven creates a branch named `runhaven/<agent>/<run-id>` and a git worktree
under its cache directory, then mounts that worktree at `/workspace`. If you
started from a subdirectory with the default `--workspace-scope current`,
RunHaven mounts the matching subdirectory inside the isolated worktree rather
than silently broadening to the whole repo.

The source checkout is left untouched. RunHaven keeps the worktree after the
run and records the worktree path, branch, base commit, and exact git commands
for status, diff, merge, worktree removal, and branch deletion in the run
record. Use `runhaven runs show RUN_ID --json` to retrieve those commands.

After review, choose an explicit lifecycle action:

```bash
runhaven runs diff <run-id>
runhaven runs keep <run-id>
runhaven runs recover <run-id>
runhaven runs recover <run-id> --json
runhaven runs merge <run-id>
runhaven runs discard <run-id>
```

`runs keep` verifies that the recorded RunHaven-owned worktree and branch still
exist, then prints the review, recover, merge, and discard commands without
changing anything. When common project checks are detected, it also suggests
copyable `runhaven run shell --network internal` commands against the recorded
worktree workspace. Suggestions are advisory only; RunHaven does not run them
automatically.

`runs merge` verifies the recorded source repository, RunHaven-owned branch,
worktree path, worktree branch, and base commit. It refuses to run if the
source checkout is dirty or if its `HEAD` moved since the worktree run. When
checks pass, it fast-forwards committed worktree branch changes, applies dirty
and untracked worktree file changes to the source checkout, then removes the
RunHaven worktree and branch. If a merge step fails, RunHaven leaves the
worktree and branch intact and prints the source repo, worktree, branch,
review, retry, keep, and discard commands for recovery.

`runs recover` verifies the same recorded worktree and branch, then prints the
source and worktree `git status --short` output plus a numbered manual recovery
sequence. Use `--json` to print the same recovery state, status lines,
commands, next-step labels, and suggested project checks for automation or a
future UI. It is read-only and does not run checks, merge, delete, commit,
stash, or change files.

`runs discard` verifies the same RunHaven-owned worktree and branch, then
removes the recorded worktree and deletes the recorded branch without touching
the source checkout.

## Local-Only Network

```bash
runhaven run shell --network internal -- cargo test
```

`internal` creates a host-only Apple container network before the run. Hosted AI
agent CLIs usually need internet access for model traffic, so this mode is most
useful for local commands and custom images.

## Package Install Or Unrestricted Internet

```bash
runhaven plan claude
runhaven run claude
```

Default internet mode is intentionally broad. Use it when package managers,
dependency updates, model traffic, or other tools need registry and CDN access.
Review `runhaven plan` first because this mode does not enforce provider-domain
allowlisting.

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

## Recover Local Resources

Diagnose bundled images before rebuilding:

```bash
runhaven image doctor
runhaven image doctor claude
```

Rebuild a stale or missing bundled image:

```bash
runhaven image rebuild claude
```

List RunHaven-managed Apple `container` networks:

```bash
runhaven network list
```

Remove only RunHaven-managed networks after reviewing the list:

```bash
runhaven network prune
runhaven network prune --yes
```

`network prune` filters to RunHaven-owned network names such as the
volume-preparation network, per-project internal networks, and per-run provider
networks. It does not delete Apple-managed networks, the default network,
arbitrary user-created networks, workspaces, images, or state volumes.
`image doctor` is read-only and reports missing or stale bundled images,
inactive RunHaven state volumes, and copyable recovery commands.

## Run History

After an actual agent run, inspect the secret-free run ledger:

```bash
runhaven runs list --limit 20
runhaven runs show <run-id>
runhaven runs log <run-id>
runhaven runs diff <run-id>
runhaven runs keep <run-id>
runhaven runs recover <run-id>
runhaven runs recover <run-id> --json
runhaven runs merge <run-id>
runhaven runs discard <run-id>
runhaven runs active
runhaven runs status <run-id>
runhaven runs attach <run-id>
runhaven runs logs-follow <run-id>
runhaven runs stop <run-id>
runhaven runs kill <run-id>
runhaven runs repair <run-id>
runhaven runs repair --all
runhaven runs repair <run-id> --json
runhaven runs repair --all --json
runhaven runs show <run-id> --json
runhaven runs log <run-id> --json
runhaven runs active --json
runhaven runs status <run-id> --json
```

Run records are stored under RunHaven's cache directory in `runs.jsonl`. They
include run id, profile, workspace, network mode, return code, provider policy
summary, auth broker summary, cleanup outcome, and git change metadata when
the workspace is inside a git repository. Git metadata includes repo root,
before and after `HEAD`, dirty state, changed file count, and a capped list of
relative paths scoped to the selected workspace. Worktree run records also
include source repo, worktree path, branch, base `HEAD`, mounted workspace, and
recovery commands. Run records do not include diffs or file contents. They also
omit command lines, agent arguments, environment variable names, environment
values, request bodies, prompts, and token values. `runs log` joins the run
record with matching provider policy and auth broker entries for the same run
id.

`runs diff` prints an on-demand live git diff from the recorded metadata. It
refuses when git metadata is unavailable, the recorded repository or workspace
is gone, `HEAD` no longer matches the recorded run, the recorded path list was
truncated, or the current dirty path set differs from the run record. For dirty
working-tree diffs, RunHaven warns that it verified the recorded `HEAD` and
path set, not the exact file contents since the run.

`runs keep`, `runs recover`, `runs merge`, and `runs discard` work only for
completed `--worktree` run records. They validate the recorded source
repository, RunHaven-owned branch, worktree path, and worktree branch before
acting. `runs merge` refuses if the source checkout is dirty or has moved away
from the run's base commit. `runs recover` is the read-only command for source
and worktree status plus manual conflict-resolution steps. The JSON variant
exposes that recovery state without prose parsing. `runs discard` is the
explicit command for deleting a RunHaven worktree that still contains unmerged
work.

`runs stop` works only for currently active runs. It reads a temporary
secret-free active-run marker from the RunHaven cache root, verifies the marker
contains a RunHaven-owned container name, and calls Apple `container stop`.
Finished runs remain inspectable through `runs list/show/log/diff`, but they
cannot be stopped.

`runs kill` uses the same active marker and RunHaven-owned container-name
check before calling Apple `container kill`. It is the hard-stop recovery path
for cases where graceful `runs stop` fails or hangs.

`runs repair` removes a stale active-run marker only after the same
RunHaven-owned container-name check and an Apple `container inspect` result
that says the recorded container is not found. If the container still exists
or RunHaven cannot confirm why inspection failed, the marker is kept.
`runs repair --all` applies that same check to each valid active marker and
returns nonzero if any marker could not be verified.
`--json` emits `results`, `summary`, and `exit_code` fields for automation
without raw Apple inspect output or active-marker contents.

`runs active` lists those currently active markers in text or JSON without
requiring Apple `container` access and skips invalid marker files.

`runs status` uses the same active marker and RunHaven-owned container-name
check before calling Apple `container inspect`. It prints a curated status
summary and omits raw inspect fields that can include process arguments,
environment values, and host mount paths.

`runs attach` uses the same active marker and RunHaven-owned container-name
check before calling Apple `container exec`. Root attach requires an explicit
`--allow-root-user` override.

`runs logs-follow` uses the same active marker and RunHaven-owned
container-name check before calling Apple `container logs --follow`. It shows
the most recent 200 lines by default before following new output; use
`--lines N` to change that history cap.

## Provider Egress Smoke

Build the base image and run a live provider-mode command on macOS 26+:

```bash
runhaven image build shell
runhaven run shell --network provider --provider-host api.openai.com -- \
  curl -fsS https://api.openai.com/
```

Run a provider profile dry run to review bundled hosts before a live agent
smoke:

```bash
runhaven plan codex --network provider
```

Provider mode creates a temporary internal Apple `container` network and starts
a host-side allowlist CONNECT proxy. Runtime tests and live smokes should prove
allowed proxied HTTPS paths succeed while denied proxied hosts, proxied IP
literals, direct DNS, and direct IP paths fail. Normal provider runs use the
same proxy enforcement pattern through `runhaven run --network provider`.

Run an optional Codex broker smoke only with a disposable OpenAI API key:

```bash
export RUNHAVEN_CODEX_SMOKE_API_KEY=...
runhaven run codex --network provider \
  --codex-api-key-broker-env RUNHAVEN_CODEX_SMOKE_API_KEY -- \
  codex --version
```

The key value is inherited by the host process only; it is not placed on the
command line or inside the guest environment.

## Private Git

```bash
runhaven run claude --ssh
```

This forwards the macOS SSH agent socket using Apple `container --ssh`. It does
not mount the host SSH key directory.

## Reusable Sessions And State Volumes

```bash
runhaven run claude --session review
runhaven plan claude --session review
runhaven state list
runhaven state list --session review
runhaven state reset claude --session review --yes
runhaven state prune --session review --yes
runhaven state prune --yes
```

The default session is the existing per-project/profile isolated home volume.
`--session NAME` selects a named reusable home volume for the same
project/profile. Use lowercase letters, numbers, dots, underscores, or dashes;
`default` is reserved for the implicit default session.

Named sessions are useful when you want one warm agent environment for review,
another for dependency work, or a disposable scratch state without changing the
workspace mount. Sessions do not widen filesystem access; they only choose the
RunHaven-managed agent home volume inside the container.

`state list` shows RunHaven agent home volumes. `state list --session NAME` and
`state prune --session NAME --yes` filter named-session volumes. `state reset
AGENT --workspace PATH --session NAME --yes` recomputes the exact
project/profile/session volume and deletes only that volume. Omit `--session`
to reset the default project/profile volume. These commands do not touch
workspace files.
