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

`--ssh` forwards the host SSH agent socket only after an explicit warning. It
never mounts private key files, and the container still runs as the non-root
`agent` user. The sandboxed agent can act as your SSH identity while the run is
active, so leave this off unless the project needs private Git access.

For local verification, use the Apple container smoke harness. `--with-ssh`
uses an empty disposable agent and verifies that the non-root container can
reach the forwarded socket without exposing a private key:

```bash
scripts/apple_container_smoke.sh --with-ssh
```

The original 2026-06-16 permission failure used a private `0600` agent socket.
That case still fails safe. The default macOS launchd agent socket was later
verified as accessible to the non-root container user; `ssh-add -l` returned
the expected no-identities status.

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
