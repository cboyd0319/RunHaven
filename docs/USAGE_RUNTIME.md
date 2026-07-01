# Runtime Usage

Advanced RunHaven runtime checks and less-common CLI paths live here so
`USAGE.md` stays focused on the common workflow.

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
  --api-key-broker-env RUNHAVEN_CODEX_SMOKE_API_KEY -- \
  codex --version
```

The key value is inherited by the host process only; it is not placed on the
command line or inside the guest environment.

## Private Git

```bash
runhaven plan claude --ssh
```

`--ssh` currently fails closed on the pinned Apple `container` 1.0.0 runtime.
RunHaven keeps the flag visible so the CLI can explain the blocker, but it does
not build or launch an SSH-forwarded run while the non-root agent user cannot
use the forwarded socket.

For local verification, use the Apple container smoke harness. Its default path
checks that `runhaven plan --ssh` fails closed. `--with-ssh` also checks that
`runhaven run --ssh` refuses before launching a container:

```bash
scripts/apple_container_smoke.sh --with-ssh
```

The live blocker found on 2026-06-16 was that RunHaven's non-root `agent` user
could see the forwarded socket path, but `ssh-add -l` returned permission
denied. Treat that as an Apple `container` SSH-forwarding blocker, not as a
reason to mount raw SSH keys or run the agent as root.

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
