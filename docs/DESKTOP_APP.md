# RunHaven desktop app

The RunHaven desktop app is the easy way to run AI coding agents inside a safe
sandbox on your Mac. It does the same work as the `runhaven` command line, but
with a window, buttons, and clear prompts. You do not need to know the command
line to use it.

The app is an early preview. It covers the main workflow: start a run, watch it,
send input, stop it, and recover it. A few advanced tasks still need the command
line, and this guide says which ones.

## Before you start

- A Mac with Apple silicon running macOS 26 or newer.
- Apple `container` installed and started. See
  [docs/INSTALLATION.md](INSTALLATION.md).
- The RunHaven app built from this repo (there is no signed download yet).

## Open the app

From the repo folder, run:

```bash
npm --prefix ui run tauri:dev
```

This opens the app in a development window. The first build takes a few minutes.

To make a local app build instead, run `npm --prefix ui run tauri:build`.

## First look: setup check

When the app opens, it checks that your Mac is ready: the right macOS version,
Apple `container` installed and running, and the agent images built. If a check
fails, the app shows a short, plain fix. Follow it, then reopen or refresh.

## Sign in to an agent

Signing in stores your agent login once so your runs can reuse it. Right now you
start sign-in from a terminal, and the app shows the result:

```bash
runhaven login claude
```

`runhaven login` supports `claude`, `codex`, `copilot`, and `antigravity`. Other
agents sign in inside the sandbox on first run, or you can pass an API key by its
environment variable name.

The app shows each agent's sign-in state so you know what is ready before you
start. It never shows your token or key. In-app sign-in is planned; until then,
use the command above.

## Start a run

Starting a run takes three short steps at the top of the app:

1. Agent: choose which agent runs.
2. Project: pick the folder the agent works in.
3. Review: check what the run can reach before anything starts.

The review step shows the plan in plain language: which agent, which folder, the
network setting, and the sign-in it will use. Nothing starts until you confirm.
If a choice is less safe than the default (for example, opening the internet or
passing an extra environment value), the app says so first.

On the Agent step you can open `Auth details` to see, for the chosen agent, how
it signs in, what stays on this Mac, what the agent gets, and which provider
hosts it can reach. These are read-only summaries with no secrets, tokens, or
file paths.

On the Project step, the `Why is this safe?` panel explains four boundaries in
plain language: the network setting, the workspace folder, a provider host you
type in, and where the agent's saved state is kept. The workspace answer shows
only a verdict, such as whether the folder can be mounted, never the full path.
These are read-only checks; they do not change your run and show no secrets or
file paths.

The safe defaults are the easiest path. By default a run gets provider-only
network access, no access to your home folder, cloud keys, SSH keys, or browser
profiles, and it does not run as root. See
[docs/SECURITY_MODEL.md](SECURITY_MODEL.md) for the full boundary.

The app says `Run accepted` when it hands the run to the sandbox. The run then
shows up in your sessions as it comes up.

## Watch your sessions

Each run becomes a managed session in the sidebar. The sidebar shows a short
identity for every session so you can tell them apart: the agent, the project,
the sign-in state, the network, and whether input is attached.

Select a session to see its live status, recent events, and bounded output. The
output is trimmed and cleaned so it will not flood the window or leak secrets. If
a session is ready for input, you can type a message and send it. The app asks
you to confirm before it sends.

The list updates itself in place. It does not blink, clear your draft, or jump
back to the start screen while a run keeps working.

## If a session stops responding

If a run's sandbox has gone away but its record is still open, the app marks the
session `Not responding` and offers `Repair session`. Repair clears the stale
record safely. The app only offers repair when it has confirmed the sandbox is
really gone, so repair never removes a record for a run that is still alive.

## Control a run

For an active session you can Stop it, Hard stop it, or Repair its record. Every
one of these asks you to confirm and shows the exact target first. Stop asks the
run to end. Hard stop ends it right away. Repair clears a stale record.

## Clean up

When a run that used its own worktree has finished, the app can offer to clean up
that worktree. It shows what it will remove and asks you to confirm. It only
removes the worktree it created for that run, and it refuses if anything looks
unexpected.

## Check diagnostics

The app can show read-only diagnostics for a session: recent sign-in broker
decisions and network decisions for that run. These show only safe summary
details, never full paths, secrets, or request bodies.

## What still needs the terminal

For now, do these tasks with the `runhaven` command line
([docs/USAGE.md](USAGE.md)):

- Full sign-in for every agent.
- Creating a run in a new git worktree, and the review, keep, merge, recover, or
  discard steps for it.
- Maintenance such as resetting or pruning saved state, pruning networks, and
  building or rebuilding agent images.
- The `why` checks that explain host, workspace, network, and state.

These will move into the app over time. Until then, the app stays focused on the
core run-and-watch workflow so the safe path stays simple.
