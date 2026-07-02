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

Signing in stores your agent login once so your runs can reuse it. Open
**Agents** in the sidebar to see every agent's sign-in state, or pick an agent
in the first step of a run. Both places show, for most agents, a **Sign in**
button.

- Codex, Copilot, and Antigravity sign in right in the app. Click **Sign in**,
  confirm the box the app pops up, then open the link it shows and enter the
  short code in your browser. The app waits and updates when you are done. It
  shows the link and code, never your token or key.
- Claude signs in from a terminal, because its sign-in needs a code pasted back
  in. The app shows this and the exact command to run:

  ```bash
  runhaven login claude
  ```

  Run it in your terminal, then come back and refresh.

Either way, the login happens inside the sandbox and stays there: RunHaven never
holds any agent token, and the app never shows or stores your token or key. You
can also pass an API key by its environment variable name for a headless run.

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

On the Project step, under **Advanced options**, you can turn on `Run in an
isolated copy (git worktree)`. This is off by default. When it is on, RunHaven runs the agent on a
new branch in a separate copy of your project, so your original files do not
change until you choose to merge. The review and the confirm dialog both say
when a worktree will be created. Your folder must be a clean git repository for
this to work.

The safe defaults are the easiest path. By default a run gets provider-only
network access, no access to your home folder, cloud keys, SSH keys, or browser
profiles, and it does not run as root. See
[docs/SECURITY_MODEL.md](SECURITY_MODEL.md) for the full boundary.

The app says `Run accepted` when it hands the run to the sandbox. The run then
shows up in your sessions as it comes up. Once it does, the acceptance notice
clears and a one-line notice points you at the new session in the sidebar.

## Find your way around

The sidebar is how you move around the app. **Home** is always first and holds
the start-a-run steps. Below it, your sessions appear in two groups: **Active**
for runs that are starting, running, or stopping, and **Finished** for runs
that ended and may need review or cleanup. Empty groups hide. At the bottom,
the **System** group holds three fixed places: **Agents** (sign in and rebuild
agent images), **Diagnostics** (read-only trust checks), and **Maintenance**
(clean up leftovers).

Newer sessions sit higher in each group, and rows never jump around when a
run's status changes. If a session needs you, its row shows a small badge with
a symbol in a dot plus a status line that names the state: Failed, Needs
sign-in, Not responding, or Ready to review. If the session you are viewing is
removed, the app moves you to the nearest session, or Home when none are left.
It never switches views on its own for any other reason.

## Watch your sessions

Each run becomes a managed session in the sidebar. The sidebar shows a short
identity for every session so you can tell them apart: the agent, the project,
the status, the sign-in state, the network, and whether input is attached.

Select a session to see its live status, recent events, and bounded output. The
output is trimmed so it will not flood the window, but raw output can still
include private data, so the app asks you to confirm before it shows any. If
a session is ready for input, you can type a message and send it. The app asks
you to confirm before it sends.

The list updates itself in place. It does not blink, clear your draft, or jump
back to the start screen while a run keeps working.

## If a session stops responding

If a run's sandbox has gone away but its session record is still open, the app
marks the session `Not responding` and highlights that repair is available.
Repair clears the stale session record safely. The app confirms the sandbox is
really gone before it highlights repair, so it never clears a record for a run
that is still alive.

## Control a run

For an active session you can Stop it, Hard stop it, or Repair its session
record. These controls are always available for an active session, and each one
asks you to confirm and shows the exact target first. Stop asks the run to end.
Hard stop ends it right away. `Repair session record` clears a stale session
record; the app also highlights repair when it has confirmed the sandbox is
gone.

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
details, never full paths, secrets, or request bodies. App-wide diagnostics
live in the Diagnostics view in the sidebar; the session view shows only that
session's diagnostics so there is one surface at a time.

## Maintenance

Two safe upkeep tasks live in the sidebar. Each one asks you to confirm in a
native dialog before anything changes.

- Rebuild an agent image, in the **Agents** view. Use this when an agent image
  is missing or out of date. RunHaven rebuilds it from the bundled template in
  the background and shows live progress: building, then succeeded or failed,
  with a short, secret-safe tail of the build output. Sessions that are
  already running keep working until you launch a new one. The confirm dialog
  names the exact agent and image tag.
- Clean up stale networks, in the **Maintenance** view. RunHaven first shows
  how many of its own leftover networks it can remove, then removes only those
  after you confirm. A network still used by a running session is skipped, not
  forced. The app shows only counts, never network names.

Resetting or pruning saved state stays in the command line on purpose. That
task can erase an agent's saved sign-in across sessions, and previewing it needs
saved-state names that the app never receives, so it fails closed rather than
guess.

## What still needs the terminal

For now, do these tasks with the `runhaven` command line
([docs/USAGE.md](USAGE.md)):

- Signing in to Claude (it needs a code pasted back in a terminal). Codex,
  Copilot, and Antigravity now sign in from the app.
- Resetting or pruning saved state.

These will move into the app over time. Until then, the app stays focused on the
core run-and-watch workflow so the safe path stays simple.
