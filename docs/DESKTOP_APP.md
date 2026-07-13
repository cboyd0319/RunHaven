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
- The signed and notarized RunHaven coworker-test disk image from the release
  owner. Drag RunHaven to Applications before opening it.
- Your Mac administrator password when Apple's `container` installer asks for
  it. RunHaven guides this prerequisite after first launch.

## Open the app

Coworkers open RunHaven from Applications. On first launch, Home checks the Mac
and guides each required setup action. If macOS says the app cannot be verified,
stop and ask the release owner for a new artifact. Do not bypass Gatekeeper.

## Run a development build

From the repo folder, run:

```bash
npm --prefix ui run tauri:dev
```

This opens the app in a development window. The first build takes a few minutes.

To check that the production frontend and desktop executable build together, run
`npm --prefix ui run tauri:build`. This developer check does not create a signed,
shareable app bundle.

To test an installed, development-signed build before release, use the local
Apple Development signing lane. It uses the separate bundle identifier
`dev.runhaven.desktop.local`. A free Xcode Personal Team can create the
**Apple Development** certificate and Xcode-managed profile needed for this
local check, but it cannot sign or notarize the coworker release. In Xcode, use
automatic signing for a macOS app target with that exact identifier, then build
it once to create the profile. Personal Team profiles expire after seven days
and must then be refreshed. Run:

```bash
export RUNHAVEN_DEVELOPMENT_SIGNING_IDENTITY='Apple Development: Example (TEAMID)'
export RUNHAVEN_DEVELOPMENT_PROVISIONING_PROFILE="$HOME/path/to/RunHaven-Local.provisionprofile"
npm --prefix ui run tauri:local-signed
```

The command validates the installed certificate, profile, application ID, and
expiration. Xcode's macOS profile may omit
`com.apple.security.get-task-allow` because macOS does not require a profile to
authorize it. RunHaven still requires that entitlement on the final signed
development app and rejects it from a release signature. The command builds and
opens **RunHaven Local**. It never creates a DMG, notarizes, or publishes an
artifact. Use it to verify installation, launch, native confirmations, and the
four in-container sign-in terminals. An ad-hoc signature (`-`) is not enough
for this signing test because it has no Apple development identity or profile.
Apple documents [development profiles](https://developer.apple.com/help/account/provisioning-profiles/create-a-development-provisioning-profile)
and [macOS provisioning entitlements](https://developer.apple.com/documentation/technotes/tn3125-inside-code-signing-provisioning-profiles).

`npm --prefix ui run tauri:bundle` is the shareable-release command. It refuses
to build from a dirty or untagged commit and requires full Xcode, a matching
Developer ID Application identity and provisioning profile, and saved Apple
notary credentials:

```bash
export RUNHAVEN_SIGNING_IDENTITY='Developer ID Application: Example (TEAMID)'
export RUNHAVEN_PROVISIONING_PROFILE="$HOME/path/to/RunHaven.provisionprofile"
export RUNHAVEN_NOTARY_PROFILE='runhaven-notary'
npm --prefix ui run tauri:bundle
```

The command validates the profile and certificate, builds an arm64 app with the
hardened runtime and the exact reviewed release entitlements, runs the full local harness
and non-credentialed CLI/desktop container smokes, signs and notarizes the app
and DMG, staples both tickets, asks Gatekeeper to assess both artifacts, and
produces checksums, CycloneDX SBOMs, notarization receipts, and provenance under
`target/release-artifacts/v<version>/`. It publishes nothing if any check fails.
The finished DMG still needs a clean supported-Mac install test before sharing.

## First look: Home

The app opens on **Home**. Home answers two questions: is this Mac ready, and
what needs you right now? It shows one health note when setup is good, an agent
summary, a **Needs you** list, recent activity, and the **New run** button. A
run that needs action appears in **Needs you** instead of being repeated in the
recent list.

When the app opens, it checks that your Mac is ready: the right macOS version
and Apple `container` installed and running. Agent image readiness is checked
when you open Agents or choose an agent in New Run. If a host check fails, Home
leads with that one problem, a short plain fix, and a
**Check again** button. While setup is broken, **New run** is disabled and says
why, but your running sessions and the sidebar views all stay reachable.

RunHaven repeats the supported macOS and Apple `container` checks immediately
before every real run. If the service stopped or a reviewed runtime changed
after Home last checked, the run stays stopped and points you to
`runhaven doctor`.

If Apple container is missing, Home shows **Download Apple container**. Choose
it and confirm the macOS dialog. RunHaven opens Apple's exact signed 1.1.0
installer download in your browser. Open the downloaded `.pkg`, finish Apple's
installer, return to RunHaven, and choose **Check again**. The URL, executable,
and arguments are fixed in Rust and never come from the app's web content.

If a different Apple container version is installed, Home shows **Download
required version**. The same fixed action opens Apple's exact signed 1.1.0
package after native confirmation. It does not install or replace the existing
version by itself; review Apple's installer, then choose **Check again**.

If Apple container is installed but its local service is off, Home shows
**Start container service**. Choose it and confirm the macOS dialog. RunHaven
starts the service and checks setup again. You do not need to open Terminal or
type a command. Other setup problems still link to **System Check** until they
have their own safe in-app fix.

## Sign in to an agent

Open **Agents** in the sidebar. Choose Claude, Codex, Antigravity, or Copilot,
then choose **Open sign-in terminal**. RunHaven shows a native confirmation
before it starts anything.

The provider's own CLI runs inside a fresh sandbox. Follow the instructions in
the terminal and finish the browser step:

- Claude prints a link. Approve in your browser, then paste the returned code
  into the terminal.
- Codex prints a link and one-time code, then waits for approval.
- Antigravity asks you to choose Google OAuth, paste a returned code, finish
  first-run setup, and exit the agent. Because Antigravity can exit with code 1
  even when sign-in worked, RunHaven then runs the non-generative `agy models`
  check inside the same sandbox.
- Copilot prints GitHub's device link and a one-time code, then waits for
  approval.

The terminal is live. RunHaven shows its bytes but does not filter, read, store,
or log them. The provider CLI saves its own credential and settings in that
agent's isolated container volume, not the Mac Keychain or your Mac home folder.
RunHaven never persists terminal bytes or provider credentials. It appends
secret-free readiness state after the provider command succeeds. Claude must
also pass its own `claude auth status` check. For Antigravity's ambiguous exit
code, RunHaven writes readiness only when the follow-up `agy models` check also
succeeds. It never reads credential contents. As with ordinary runs, it can
retain secret-free lifecycle, cleanup, and egress metadata.

Closing the terminal stops the sign-in container. A force-quit may leave a
RunHaven container or network. If the container still holds the agent's saved
state, the next sign-in fails before launch with a safe message. Stop the other
run or use **Cleanup** to remove leftover resources, then try again. **Check
again** refreshes agent status. “Signed in” means RunHaven witnessed a
successful provider command and any required provider-owned readiness check. It
never reads the provider credential.

The optional API-key broker remains under **Advanced options** for explicitly
configured Claude or Codex API keys. It is not selected automatically and is not
the ordinary sign-in path. See [Authentication and API-key brokers](AUTH_BROKER.md).

## Start a run

Click **New run** on Home. A window opens over the app with three short steps:

1. Agent: choose which agent runs.
2. Project: pick the folder the agent works in.
3. Review: check what the run can reach before anything starts.

The Agent step shows the agent choice, readiness, and sign-in state without
showing broker or image internals. The Project step uses the normal Mac folder
picker. Sign-in sharing, network access, resource limits, and raw paths stay
under **Advanced options**.
Recommended isolated-copy protection stays visible on the Project step. The
Review step leads with the agent, project, and access result. Network, image,
and run-name details stay under **Technical details**.

Choose **Start run** when the summary is right. RunHaven then asks for final
confirmation in a native Mac dialog. There is no second confirmation checkbox
in the app. If a choice is less safe than the default, such as full internet
access, its specific warning still requires explicit acknowledgement.

You can close the window at any time with **Cancel** or the Escape key. That
never starts anything, and the values you entered are kept, so opening
**New run** again picks up where you left off. When you start and confirm, the
window closes and the app takes you straight to the new run.

Some agents are interactive: they work as a chat in their own terminal, so there
is no separate box to type your task into. When you start one of these (Claude
Code today), the **New run** window closes and RunHaven opens the run
in a full terminal that fills the main area of the window, the same place a
session opens. You type your task in the terminal the same way you would in your
own. The native confirm step runs as the terminal opens, so nothing starts
until you approve it,
and the run uses the same sandbox rules as any other run: the same non-root agent
user, the same network setting, and your saved Claude sign-in added only to the
run's requests, never inside the sandbox. The same standing terminal warning
applies (see [Open a terminal](#open-a-terminal)): what the run prints is shown
but not filtered or saved. **End run** stays visible while the terminal starts
and runs. It ends the run and takes you back Home, so the terminal cannot leave
you stuck. Closing the window also ends the run cleanly. Afterward the run shows as
**Ended** under Past, not Failed: you close an interactive run
yourself, so ending it is normal and never marked as an error.

Under **Advanced options** on the Project step, the `Why is this safe?` panel explains four boundaries in
plain language: the network setting, the workspace folder, a provider host you
type in, and where the agent's saved state is kept. The workspace answer shows
only a verdict, such as whether the folder can be mounted, never the full path.
These are read-only checks; they do not change your run and show no secrets or
file paths.

**Provider only** stays the default, but every supported network mode remains
available. Choosing another mode turns off the optional API key broker. Native
OAuth state remains inside the agent's isolated volume. **Internet** still
shows its warning and requires explicit confirmation. Rust independently
guarantees that a host-broker secret never crosses into an incompatible run.

On the Project step, **Protect my project with an isolated copy** is the
recommended default. RunHaven uses a new branch in a separate copy of your
project, so your original files do not change until you choose to merge. You
can turn this protection off to run directly in the selected folder. The review
then warns that a writable direct run can change your original files and asks
you to confirm that choice. Read-only direct runs do not show that warning. The
review and confirm dialog both say when a copy will be created. Your folder
must be a clean git repository for the copy to work. RunHaven checks that before confirm,
then checks setup, the image, and sign-in choices before it creates the copy
when possible. If a later launch step fails, the failed run appears under Past
with the isolated copy kept for review or removal.

The safe defaults are the easiest path. By default a run gets provider-only
network access, no access to your home folder, cloud keys, SSH keys, or browser
profiles, and it does not run as root. See
[docs/SECURITY_MODEL.md](SECURITY_MODEL.md) for the full boundary.

The app says `Run accepted` when it hands the run to the sandbox. The run then
shows up in your sessions as it comes up. Once it does, the acceptance notice
clears and a one-line notice points you at the new run in Recent activity.

## Find your way around

The sidebar is how you move around the app. Its fixed destinations are **Home**,
**Agents**, **System Check**, and **Cleanup**. Below them, three run counts open
the same Run history with a real filter: **Needs you**, **Running**, and
**Past**. Individual runs stay out of the sidebar so it remains short and
stable. **Settings** is always available at the bottom. Open a run from Home or
Run history. If the run you are viewing is
removed, the app moves to the nearest available run or Home. It never switches
views on its own for any other reason.

The app icon in your Dock shows the same signal as one number: how many
sessions need you right now, and nothing else. The number goes down as you
take care of each session, and the badge disappears when nothing needs you.
You can turn the number off from **Settings** in the sidebar or with Cmd-,.

## Menu bar and keyboard shortcuts

The app has a normal Mac menu bar: RunHaven, File, Edit, View, Session, Agent,
Window, and Help. Standard items like About, Quit, Undo, Copy, Paste, Close,
Minimize, Zoom, and Full Screen behave the way they do in every Mac app.
Quitting the app does not stop background runs; they keep working in the
sandbox and show up again when you reopen the app. A live **Run terminal** or
sign-in terminal is different: closing its window or quitting ends that live
terminal. Use **End run** when you are finished.

Menu items never skip a safety step. A menu action does exactly what the same
click in the window would do. **Stop** and **Hard Stop** take you to that
session's stop control; you still check the confirm box and answer the native
dialog before anything stops. Items that do not apply right now are grayed out,
so their shortcut does nothing until it can. For example, the Run menu
items turn on only when a session is selected and can take that action, and
Cmd-1 through Cmd-9 light up only for sessions your sidebar actually has.
Session shortcuts work when the main window is active; while a session watch
window is in front they do nothing, so a shortcut never acts on a session you
are not looking at. Everything else in the menus works from any window.

| Shortcut | What it does |
| --- | --- |
| Cmd-N | New Run: opens the run window |
| Cmd-0 | Go to Home |
| Cmd-1 to Cmd-9 | Select run 1 to 9 in run-list order |
| Shift-Cmd-] / [ | Next / previous run |
| Cmd-J | Jump to the next run that needs you |
| Cmd-. | Go to the selected session's Stop control |
| Opt-Cmd-. | Go to the selected session's Hard stop control |
| Cmd-Return | Send the message you typed in the session's composer |
| Shift-Cmd-O | Go to the output confirm box for the selected session |
| Cmd-R | Check again: refresh the app's status |
| Opt-Cmd-1 / 2 / 3 | Go to Agents / System Check / Cleanup |
| Opt-Cmd-S | Show or hide the sidebar |
| Ctrl-Cmd-F | Enter or leave full screen |
| Cmd-, W M Q | Settings, Close, Minimize, Quit |

A few items are visible but grayed out because their screens are not built
yet: **Filter Runs** (Cmd-F) and the Help entries. They will turn on when
those parts of the app ship. Other items gray out only until they apply again,
such as **Stop** with no active run selected, or **Next Run That Needs You**
when nothing needs you. The Run menu's worktree actions (Merge, Keep,
Recover, Discard) live in the finished session's review for now, and Open
Recent arrives with a recent-projects list. **Open in New Window** opens a
watching window for the selected session; see "Watch a session in its own
window" below.

**Send Message** in the Run menu takes you to that run's composer, the
same way Stop takes you to its Stop control. The real send shortcut is
Cmd-Return, pressed in the composer: it sends the message you typed through the
same confirm step and native dialog as the Send button. The menu item carries
no shortcut of its own so Cmd-Return has one clear meaning.

## Settings

Open **Settings** at the bottom of the sidebar, or use the **RunHaven** menu
(**Settings...**, or Cmd-,). It opens over the main window and holds only small
convenience choices:

- **Show a number on the app icon.** The Dock number counts sessions that
  need you. Turn it off to clear the number.
- **Start new runs with.** The agent picked first when you start a new run.
  Leave it on "Choose for me" to let RunHaven pick, and you can still change
  the agent for each run either way.

RunHaven follows your Mac's light or dark appearance. There is no separate
theme setting to keep in sync.
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

Each run appears on Home and in Run history with its agent, project, and one
short status. Open it to see one reading order. If the run
needs you, a banner leads the view and names the one fix: a failed run points
at its output, a run that needs sign-in offers **Open Agents**, a run that
stopped responding points at **More actions**, and a finished worktree run
points at its review. **Run actions** sits near the top. Normal stopping is
immediate to find; hard stop and record repair stay under **More actions**.
Rust asks for native confirmation before any of these changes. Below that are a
compact overview, output, and the message box. Events and technical status stay
behind disclosures.

You can give a run your own name. Use **Name this run** or **Rename run** near
the top, type a short name, and save. The name
then shows in Run history and at the top of the run, in place of the
"agent and project" name. Leave it blank to go back to the default name. This
name is just a label for you: it shows only in this app on this Mac, it does
not rename the saved run itself, and nothing sensitive is saved with it.

The output is trimmed so it will not flood the window, but raw output can
still include private data, so the app asks you to confirm before it shows
any. **Keep output updated** is available only while a run is active. A past
run can still show output retained by this app, but it does not keep polling.
If older output is no longer available, the app says so without showing an
internal run ID or command error. If a session is ready for input, you can type
a message and send it. The app asks you to confirm before it sends.

Deeper runtime detail (container state, resources, networks) and the
run's own diagnostics sit behind a **Show details** button, so the view
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

The sign-in terminal on the Agents view is the same kind of in-app terminal.
It runs the selected provider's native login command in a fresh sandbox. It
keeps the same non-root sandbox, provider-only network, and standing warning
above; any code you paste is visible on screen and is not saved by RunHaven.

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
**Clean up run** control that removes that saved copy once you no longer
need it. Like every change, it asks you to confirm first and answers to the
same native dialog.

## Check this Mac

The **System Check** view in the sidebar is read-only trust and health evidence
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
  Cleaning them up stays in Cleanup.
- Every setup check with its result and fix.
- The trust checks: sign-in broker decisions and provider network decisions.

Everything here only reads; the actions that change things (rebuild, prune)
stay in their own views. A session's own diagnostics live in that session's
view behind **Show details**, so there is one surface at a time.

The desktop check omits the Rust toolchain because coworkers do not need Rust.
The contributor-only CLI `doctor` command still checks the pinned toolchain.

When setup is not ready, System Check does not ask the stopped container
service for image or network details. Those cards say to finish setup first,
while the setup checks still show the problem and its next step.

## Debug logs

The desktop app writes a local log file while it runs. If something looks
wrong, share this file instead of a screenshot:

`~/Library/Logs/dev.runhaven.desktop/runhaven.log`

The log records app events, errors, and the name and timing of each action.
It does not record what you see or type in the live terminal, request bodies,
or tokens: the log level is capped below the point where the network layer
would print raw request bytes, so a token or a request body cannot reach the
file. The file rotates at 5 MiB and the app keeps the five most recent older
files, so it stays small.

Before sharing a log, you can double-check it with `scripts/scan-dev-log.sh`,
which fails if any token-shaped text ever appears.

## Cleanup and repair

Two safe upkeep tasks live in the sidebar. Each one asks you to confirm in a
native dialog before anything changes.

- Rebuild an agent image, in the **Agents** view. Use this when an agent image
  is missing or out of date. RunHaven rebuilds it from the bundled template in
  the background and shows live progress: building, then succeeded or failed,
  with a short, secret-safe tail of the build output. Sessions that are
  already running keep working until you launch a new one. The confirm dialog
  names the exact agent and image tag. Leaving and returning does not start a
  second build for the same tag; choosing rebuild again shows the existing job.
  Immediately before building, RunHaven repeats the full supported-runtime
  check shown in System Check. Quitting RunHaven cancels and reaps the active
  builder process.
- Clean up stale networks, in **Cleanup**. RunHaven first shows
  how many of its own leftover networks it can remove, then removes only those
  after you confirm. Cleanup stays unavailable until setup is ready. If listing
  networks fails, RunHaven stops before deleting anything. A network that
  cannot be removed is left alone and reported as **Could not remove**, not as
  a successful cleanup. The app shows only counts, never network names.

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
