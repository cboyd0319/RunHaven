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

## First look: Home

The app opens on **Home**. Home answers two questions: is this Mac ready, and
what needs you right now? It shows a one-line health note when setup is good,
your agents and their sign-in states with a link to the Agents view, a
"Needs you" list of sessions waiting on you, your recent finished runs, and
the **New run** button. Each run shows in one list only: a failed run or a run
waiting for review sits in "Needs you", never again under recent finished
runs.

When the app opens, it checks that your Mac is ready: the right macOS version,
Apple `container` installed and running, and the agent images built. If a check
fails, Home leads with that one problem, a short plain fix, and a
**Check again** button. While setup is broken, **New run** is disabled and says
why, but your running sessions and the sidebar views all stay reachable.

## Sign in to an agent

Signing in stores your agent login once so your runs can reuse it. Open
**Agents** in the sidebar to see every agent in one place: its sign-in state,
whether its image is ready, its read-only auth details, and the image rebuild
task. A **Check again** button refreshes the view. You can also pick an agent
in the first step of a run. Both places show, for most agents, a **Sign in**
button.

- Codex, Copilot, and Antigravity sign in right in the app. Click **Sign in**,
  confirm the box the app pops up, then open the link it shows and enter the
  short code in your browser. The app waits and updates when you are done. It
  shows the link and code, never your token or key.
- Claude signs in on your Mac by default. Click **Sign in on your Mac**, confirm
  the box the app pops up, and RunHaven runs Claude's own sign-in on your Mac.
  RunHaven saves your Claude token to your Mac's Keychain and uses it to sign
  your runs. The token never goes inside the sandbox. RunHaven can use your token
  only after you approve it with Touch ID or your device password, and it never
  shows the token on screen. After you sign in, the Claude card shows how long
  ago the token was saved. Once it is older than the reminder time you set in
  **Settings** (90 days to start), the card suggests signing in again. That is
  only a reminder: the old token keeps working until you replace it.
- **Remove saved token** deletes only the token RunHaven saved in your Mac's
  Keychain. It does not touch your other logins, and it does not cancel the token
  on Anthropic's side. To fully cancel it, revoke it in your Anthropic account.
- Prefer to keep Claude's login inside the sandbox? Click **Sign in inside the
  sandbox instead**. This opens a small terminal that runs `claude auth login`
  inside a sandbox: open the link it prints, approve in your browser, then paste
  the code it gives you back into that terminal, and come back and refresh. Your
  Claude login is then saved in the agent's own space, not on your Mac's
  Keychain. The same standing terminal warning applies (see "Open a terminal"):
  what you type and what it prints, including the code, are shown on screen and
  are not filtered or saved. While that terminal is open it holds Claude's saved
  state, so you cannot start a Claude run until you close it. Closing it removes
  the sign-in sandbox container and its network even if sign-in did not finish;
  a force-quit mid-sign-in may leave one unused network that the Maintenance
  panel's "Clean up stale networks" removes safely. You can also run
  `runhaven login claude` in your own terminal; a **Copy command** button copies
  the exact command.

For Codex, Copilot, Antigravity, and Claude's in-sandbox option, the login stays
in the sandbox and RunHaven holds no token. Claude's on-your-Mac option is the
one place RunHaven keeps a token, saved only in your Mac's Keychain and used only
to sign your runs; see [`AUTH_BROKER.md`](AUTH_BROKER.md) for exactly what it
does and does not do. You can also pass an API key by its environment variable
name for a headless run.

## Start a run

Click **New run** on Home. A window opens over the app with three short steps:

1. Agent: choose which agent runs.
2. Project: pick the folder the agent works in.
3. Review: check what the run can reach before anything starts.

The review step shows the plan in plain language: which agent, which folder, the
network setting, and the sign-in it will use. Nothing starts until you confirm.
If a choice is less safe than the default (for example, opening the internet or
passing an extra environment value), the app says so first.

You can close the window at any time with **Cancel** or the Escape key. That
never starts anything, and the values you entered are kept, so opening
**New run** again picks up where you left off. When you launch and confirm,
the window closes and the app takes you straight to the new session.

Some agents are interactive: they work as a chat in their own terminal, so there
is no separate box to type your task into. When you start one of these (Claude
Code today), RunHaven opens the run right here in a terminal instead of running
it in the background, and you type your task in the terminal the same way you
would in your own. The same confirm step runs first, so nothing starts until you
approve it, and the run uses the same sandbox rules as any other run: the same
non-root agent user, the same network setting, and your saved Claude sign-in
added only to the run's requests, never inside the sandbox. The same standing
terminal warning applies (see [Open a terminal](#open-a-terminal)): what the run
prints is shown but not filtered or saved. Use **Back to review** to change the
run, or **Done** (or closing the window) to end it.

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

On the Project step you can turn on `Run in an
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

The sidebar is how you move around the app. **Home** is always first and shows
what needs you, plus the **New run** button. Below it, your sessions appear in two groups: **Active**
for runs that are starting, running, or stopping, and **Finished** for runs
that ended and may need review or cleanup. Empty groups hide. At the bottom,
the **System** group holds three fixed places: **Agents** (sign in and rebuild
agent images), **Diagnostics** (read-only trust checks), and **Maintenance**
(clean up leftovers).

Newer sessions sit higher in each group, and rows never jump around when a
run's status changes. If a session needs you, its row shows a small badge with
a symbol in a dot plus a status line that names the state: Failed, Needs
sign-in, Not responding, or Ready to review. If the session you are viewing is
removed, the app moves you to the nearest session in the sidebar, or Home when
there is no clear neighbor to pick. It never switches views on its own for any
other reason.

The app icon in your Dock shows the same signal as one number: how many
sessions need you right now, and nothing else. The number goes down as you
take care of each session, and the badge disappears when nothing needs you.
You can turn the number off in Settings (Cmd-,).

## Menu bar and keyboard shortcuts

The app has a normal Mac menu bar: RunHaven, File, Edit, View, Session, Agent,
Window, and Help. Standard items like About, Quit, Undo, Copy, Paste, Close,
Minimize, Zoom, and Full Screen behave the way they do in every Mac app.
Quitting the app does not stop your runs; they keep working in the sandbox and
show up again when you reopen the app.

Menu items never skip a safety step. A menu action does exactly what the same
click in the window would do. **Stop** and **Hard Stop** take you to that
session's stop control; you still check the confirm box and answer the native
dialog before anything stops. Items that do not apply right now are grayed out,
so their shortcut does nothing until it can. For example, the Session menu
items turn on only when a session is selected and can take that action, and
Cmd-1 through Cmd-9 light up only for sessions your sidebar actually has.
Session shortcuts work when the main window is active; while a session watch
window is in front they do nothing, so a shortcut never acts on a session you
are not looking at. Everything else in the menus works from any window.

| Shortcut | What it does |
| --- | --- |
| Cmd-N | New Run: opens the run window |
| Cmd-0 | Go to Home |
| Cmd-1 to Cmd-9 | Select session 1 to 9, top to bottom in the sidebar |
| Shift-Cmd-] / [ | Next / previous session |
| Cmd-J | Jump to the next session that needs you |
| Cmd-. | Go to the selected session's Stop control |
| Opt-Cmd-. | Go to the selected session's Hard stop control |
| Cmd-Return | Send the message you typed in the session's composer |
| Shift-Cmd-O | Go to the output confirm box for the selected session |
| Cmd-R | Check again: refresh the app's status |
| Opt-Cmd-1 / 2 / 3 | Go to Agents / Diagnostics / Maintenance |
| Opt-Cmd-S | Show or hide the sidebar |
| Ctrl-Cmd-F | Enter or leave full screen |
| Cmd-, W M Q | Settings, Close, Minimize, Quit |

A few items are visible but grayed out because their screens are not built
yet: **Filter Sessions** (Cmd-F) and the Help entries. They will turn on when
those parts of the app ship. Other items gray out only until they apply again,
such as **Stop** with no running session selected, or **Next Session That Needs
You** when nothing needs you. The Session menu's worktree actions (Merge, Keep,
Recover, Discard) live in the finished session's review for now, and Open
Recent arrives with a recent-projects list. **Open in New Window** opens a
watching window for the selected session; see "Watch a session in its own
window" below.

**Send Message** in the Session menu takes you to that session's composer, the
same way Stop takes you to its Stop control. The real send shortcut is
Cmd-Return, pressed in the composer: it sends the message you typed through the
same confirm step and native dialog as the Send button. The menu item carries
no shortcut of its own so Cmd-Return has one clear meaning.

## Settings

Open Settings from the **RunHaven** menu (**Settings...**, or Cmd-,). It opens
over the main window and holds only small convenience choices:

- **Show a number on the app icon.** The Dock number counts sessions that
  need you. Turn it off to clear the number.
- **Start new runs with.** The agent picked first when you start a new run.
  Leave it on "Choose for me" to let RunHaven pick, and you can still change
  the agent for each run either way.
- **Ask me to sign in to Claude again after.** How old your saved Claude token
  can get before the Agents view suggests signing in again (30, 60, 90, or 180
  days; 90 to start). It is only a reminder, not a cutoff.

RunHaven uses a dark look for now; more appearance choices may come later.
Safety never lives in Settings: no security default can be changed there and
no confirmation step can be turned off. Your choices are saved on this Mac
only, and nothing sensitive is stored with them.

## The app remembers your place

When you reopen the app, it comes back the way you left it: the window keeps
its size and position, the sidebar stays shown or hidden, and you land on the
screen you were using. If you were viewing a session, the app first checks
that the session still exists; if it is gone, you land on Home instead. Only
the place and your Settings choices are remembered, never anything sensitive:
no drafts, no output, no paths, and no sign-in data are saved this way.

## Watch your sessions

Each run becomes a managed session in the sidebar. Each row stays simple to
scan: the agent and project as the row's name, and one short status line below
it. The sign-in state, network, and input detail live in the session's own view
and its Details section, not in the sidebar row.

Select a session to see it top to bottom in one reading order. If the session
needs you, a banner leads the view and names the one fix: a failed run points
at its output, a session that needs sign-in offers **Open Agents**, a session
that stopped responding points at its repair control, and a finished worktree
run points at its review. The session controls sit near the top, never at the
bottom of the window. Below them come the session overview with its recent
events, the output, and the message box.

You can give a session your own name. Use **Name this session** (or **Rename
session**) near the top of the session, type a short name, and save. The name
then shows on the sidebar row and at the top of the session, in place of the
"agent and project" name. Leave it blank to go back to the default name. This
name is just a label for you: it shows only in this app on this Mac, it does
not rename the session itself, and nothing sensitive is saved with it.

The output is trimmed so it will not flood the window, but raw output can
still include private data, so the app asks you to confirm before it shows
any. If a session is ready for input, you can type a message and send it. The
app asks you to confirm before it sends.

Deeper runtime detail (container state, resources, networks) and the
session's own diagnostics sit behind a **Show details** button, so the view
stays focused on what needs you.

The list updates itself in place. It does not blink, clear your draft, or jump
back to the start screen while a run keeps working.

## Watch a session in its own window

Select a session and choose **Session > Open in New Window**. That session
opens in a small extra window so you can watch two runs side by side, or tile
one next to another app with macOS Split View. If the session already has a
window, the app brings that window to the front instead of opening another.
You can have up to six of these windows at once.

These windows only watch. Each one shows the session's overview, recent
events, and read-only details behind **Show details**, and it says so plainly:
controls live in the main window. To stop a run, repair it, send a message,
or view output, use the main window. This is a safety choice, not a missing
feature: a watching window cannot change anything.

Close a session window with Cmd-W or its close button. If the session's
record is removed while you watch, the window tells you the session is gone
and keeps the last view until you close it. Session windows do not come back
when you restart the app, and closing the main window closes them too.

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

## Open a terminal

A running session has an **Open terminal** button. It gives you a live shell
inside that session's sandbox, so you can type commands and see what they print,
right in the app. The terminal shows only for running sessions, not for ones
that have finished or failed.

When you open it, the app names the sandbox and asks you to confirm first. What
you type goes straight to the sandbox, as the same non-root agent user, with no
new access: the terminal adds typing, not reach.

The terminal card carries a standing warning, and it is the plain truth about
this one stream:

> This is a live terminal inside the sandbox. RunHaven does not filter, read,
> store, or log it. It just shows what the sandbox prints. So anything here,
> including passwords or codes, is visible on this screen.

That warning is about the live terminal only, not the rest of the app. The
terminal text lives in memory while the terminal is open and is thrown away when
you close it or the app. It is never written to disk and never shown again after
you close it. This is different from saved run output, which the app does redact
and keep locally for history.

The interactive run terminal from **New run** (above) works the same way: it is
the same live shell, the same non-root agent user, and the same shown-but-not-
saved rule for what it prints. The only difference is that it starts a fresh run
rather than joining one already running.

Close the terminal with **Close terminal**, or just leave the session or reload
the app; either way the app shuts down the shell in the sandbox.

The Claude sign-in terminal on the Agents view is the same kind of in-app
terminal. It runs `claude auth login` in a fresh sandbox so the sign-in link and
the code you paste back stay in the app. It keeps the same non-root sandbox, the
same provider-only network, and the same standing warning above; the code you
paste is visible on screen and is not saved.

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

If you kept the isolated copy earlier, the finished session offers a
**Clean up session** control that removes that saved copy once you no longer
need it. Like every change, it asks you to confirm first and answers to the
same native dialog.

## Check diagnostics

The **Diagnostics** view in the sidebar is read-only trust and health evidence
for the whole app. It loads by itself when you open it. A **Check again** button
re-reads everything. It never changes anything, and it shows only safe summary
details, never full paths, secrets, or request bodies. From top to bottom it
shows:

- A one-line health summary: green when your Mac passes every setup check, or
  amber with the number of checks that need attention.
- **Versions and pins**: the RunHaven app version and the runtime versions it
  expects (Apple container and the pinned runtime).
- **Agent images**: whether each agent's sandbox image is built and ready.
  Rebuilding an image stays in the Agents view.
- **Stale networks**: how many leftover RunHaven networks could be cleaned up.
  Cleaning them up stays in the Maintenance view.
- Every setup check with its result and fix.
- The trust checks: sign-in broker decisions and provider network decisions.

Everything here only reads; the actions that change things (rebuild, prune)
stay in their own views. A session's own diagnostics live in that session's
view behind **Show details**, so there is one surface at a time.

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

- Resetting or pruning saved state.

These will move into the app over time. Until then, the app stays focused on the
core run-and-watch workflow so the safe path stays simple.
