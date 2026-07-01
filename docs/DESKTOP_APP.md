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

On the Review step you can also turn on `Run in an isolated copy (git
worktree)`. This is off by default. When it is on, RunHaven runs the agent on a
new branch in a separate copy of your project, so your original files do not
change until you choose to merge. The review and the confirm dialog both say
when a worktree will be created. Your folder must be a clean git repository for
this to work.

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

## Review a worktree run

When a run that used its own worktree has finished, the app shows a review for
that session. It shows the branch name and how many files changed, and it offers
four choices, each behind its own confirm step:

- Merge: apply the worktree's changes back into your project, then remove the
  worktree and branch.
- Keep: leave the worktree and branch in place so you can work on them yourself.
- Recover: show safe, plain-language next steps if a merge cannot finish cleanly.
- Discard: delete the worktree and branch for that finished run.

The app resolves the run in the backend, checks that the worktree is one
RunHaven made and that the run has stopped, and refuses if anything is still
active or looks unexpected. It never shows raw file paths, saved state, or
secrets.

## Check diagnostics

The app can show read-only diagnostics for a session: recent sign-in broker
decisions and network decisions for that run. These show only safe summary
details, never full paths, secrets, or request bodies.

## Maintenance

The Maintenance panel handles two safe upkeep tasks. Each one asks you to
confirm in a native dialog before anything changes.

- Rebuild an agent image. Use this when an agent image is missing or out of
  date. RunHaven rebuilds it from the bundled template in the background and
  shows live progress: building, then succeeded or failed, with a short,
  secret-safe tail of the build output. Sessions that are already running keep
  working until you launch a new one. The confirm dialog names the exact agent
  and image tag.
- Clean up stale networks. RunHaven first shows how many of its own leftover
  networks it can remove, then removes only those after you confirm. A network
  still used by a running session is skipped, not forced. The app shows only
  counts, never network names.

Resetting or pruning saved state stays in the command line on purpose. That
task can erase an agent's saved sign-in across sessions, and previewing it needs
saved-state names that the app never receives, so it fails closed rather than
guess.

## What still needs the terminal

For now, do these tasks with the `runhaven` command line
([docs/USAGE.md](USAGE.md)):

- Full sign-in for every agent.
- Resetting or pruning saved state.
- The `why` checks that explain host, workspace, network, and state.

These will move into the app over time. Until then, the app stays focused on the
core run-and-watch workflow so the safe path stays simple.
