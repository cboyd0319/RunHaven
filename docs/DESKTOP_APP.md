# RunHaven desktop app

The RunHaven desktop app is the easy way to run AI coding assistants inside a safe
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
The launcher asks macOS for an unused local port and prints the chosen address
in the debug log. It does not assume a common development port is free, reuse a
server that is already running, or stop another local app.
Development builds are named **RunHaven Debug**, use the separate bundle ID
`dev.runhaven.desktop.debug`, and show **Debug** plus a build ID after the
version number. Every development launch uses the repository's canonical
`target/` directory and records the same build ID in the debug log. This keeps
Debug distinct from **RunHaven** release builds and **RunHaven Local** signed
test builds, and makes a stale Debug copy visible even when the source version
is the same.

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

The app always opens on **Home**. Home is a launchpad with four choices:
Claude, Codex, Antigravity, and Copilot. Choose the assistant you
want. Home does not show run history, logs, failures, or system checks. Those
details live in **History** and **System details**, where they have room to
explain what happened and what to do next.

After you choose an assistant, **New session** checks whether your Mac and that
assistant are ready. If something needs work, it shows one problem, a plain
explanation, and the safest available fix. You can also open **System details**
for every setup check. This keeps Home calm without hiding a problem that would
stop the session.

RunHaven repeats the supported macOS and Apple `container` checks immediately
before every real run. If the service stopped or a reviewed runtime changed,
the run stays stopped and points you to
`runhaven doctor`.

If Apple container is missing, New session or System details shows **Download
Apple container**. Choose it and confirm the macOS dialog. RunHaven opens
Apple's exact signed 1.1.0 installer download in your browser. Open the
downloaded `.pkg`, finish Apple's installer, return to RunHaven, and choose
**Check again**. The URL, executable, and arguments are fixed in Rust and never
come from the app's web content.

If a different Apple container version is installed, New session or System
details shows **Download required version**. The same fixed action opens
Apple's exact signed 1.1.0 package after native confirmation. It does not
install or replace the existing version by itself; review Apple's installer,
then choose **Check again**.

If Apple container is installed but its local service is off, New session or
System details shows **Start container service**. Choose it and confirm the
macOS dialog. RunHaven starts the service and checks setup again. You do not
need to open Terminal or type a command. Each fixable failed check has one
action.

## Sign in to an assistant

On Home, choose **Start** for Claude, Codex, Antigravity, or Copilot. If that
assistant needs sign-in, choose
**Open sign-in terminal** beside it in New session.
RunHaven shows a native confirmation before it starts anything.

The provider's own CLI runs inside a fresh sandbox. Follow the instructions in
the terminal and finish the browser step:

- Claude prints a link. Approve in your browser, then paste the returned code
  into the terminal.
- Codex prints a link and one-time code, then waits for approval.
- Antigravity asks you to choose Google OAuth, paste a returned code, finish
  first-run setup, and exit the assistant. Because Antigravity can exit with code 1
  even when sign-in worked, RunHaven then runs the non-generative `agy models`
  check inside the same sandbox.
- Copilot prints GitHub's device link and a one-time code, then waits for
  approval.

The terminal is live. RunHaven shows its bytes but does not filter, read, store,
or log them. The provider CLI saves its own credential and settings in that
assistant's isolated storage, not the Mac Keychain or your Mac home folder.
RunHaven never persists sign-in terminal bytes or provider credentials. It appends
secret-free readiness state after the provider command succeeds. Claude must
also pass its own `claude auth status` check. For Antigravity's ambiguous exit
code, RunHaven writes readiness only when the follow-up `agy models` check also
succeeds. It never reads credential contents. As with ordinary runs, it can
retain secret-free lifecycle, cleanup, and egress metadata.

Closing the terminal stops the sign-in container. A force-quit may leave a
RunHaven container or network. If it still holds the assistant's saved
state, the next sign-in fails before launch with a safe message. Stop the other
run or open **System details** and remove leftover networks, then try again.
Refreshing RunHaven rechecks assistant status. “Signed in” means RunHaven witnessed a
successful provider command and any required provider-owned readiness check. It
never reads the provider credential.

The optional API-key broker remains under **Advanced options** for explicitly
configured Claude or Codex API keys. It is not selected automatically and is not
the ordinary sign-in path. See [Authentication and API-key brokers](AUTH_BROKER.md).

## Start a run

Choose an assistant on Home. The main area opens a two-step flow with that assistant
already selected:

1. Project: pick the folder the selected assistant works in.
2. Review: check what the run can reach before anything starts.

Opening **New session** directly adds an Assistant step before these two steps.
Starting from Home keeps the chosen assistant and does not ask again. The Project
step uses the normal Mac folder picker; there is no path to type by hand.
Sign-in sharing, network access, resource limits, and direct-folder access stay
under **Advanced options**. The safer protected copy is automatic. Advanced
users can explicitly choose to work in the original folder after reading its
warning. The Review step leads with the assistant, project, and access result.
Network, image, and run-name details stay under **Technical details**.

Choose **Start run** when the summary is right. RunHaven then asks for final
confirmation in a native Mac dialog. There is no second confirmation checkbox
in the app. If a choice is less safe than the default, such as full internet
access, its specific warning still requires explicit acknowledgement. The
normal confirmation stays short. If you chose advanced host access, it lists
the exact extra sites, Mac environment names, API-key source name, or custom
image so the trusted Mac dialog cannot hide what the run requested. It never
shows an environment value.

You can leave at any time with **Cancel**. That never starts anything, and the
values you entered are kept, so returning to **New session** picks up where you
left off. When you start and confirm, the app takes you straight to the new run.

Some assistants are interactive: they work as a chat in their own terminal, so there
is no separate box to type your task into. When you start one of these (Claude
Code today), RunHaven opens the run
in a full terminal that fills the main area of the window, the same place a
session opens. You type your task in the terminal the same way you would in your
own. The native confirm step runs as the terminal opens, so nothing starts
until you approve it,
and the run uses the same sandbox rules as any other run: the same restricted
user, the same network setting, and your saved Claude sign-in added only to the
run's requests, never inside the sandbox. The same standing terminal warning
applies (see [Open a terminal](#open-a-terminal)): what the run prints is shown
live. A bounded copy is filtered before History displays it and stays only in
this app process. It is never written to disk or the debug log, and it is gone
after RunHaven closes. **End run** stays visible while the terminal starts
and runs. It ends the run and takes you back Home, so the terminal cannot leave
you stuck. Closing the window also ends the run cleanly. Afterward the run shows as
**Ended** under Past, not Failed: you close an interactive run
yourself, so ending it is normal and never marked as an error.

Under **Advanced options** on the Project step, people can change project access,
sign-in sharing, network access, CPU, memory, and supported API-key behavior.
These controls stay collapsed because the secure defaults already cover the
ordinary path. Raw host paths, commands, keys, and implementation details do not
appear there.

**Assistant services only** stays the default, but every supported network mode remains
available. Choosing another mode turns off the optional API key broker. Native
OAuth state remains inside the assistant's isolated storage. **Full internet** still
shows its warning and requires explicit confirmation. Rust independently
guarantees that a host-broker secret never crosses into an incompatible run.

RunHaven protects the selected project with a separate working copy by default,
so the original folder does not change while the assistant works. The implementation
details stay out of the ordinary flow. Under **Advanced options**, **Project
access** is one mutually exclusive choice: **Protected copy (recommended)**,
**Read-only original folder**, or **Original folder with changes (less safe)**.
The last choice warns that the assistant can change the original files. RunHaven
validates that it can make the protected copy before confirmation and keeps a
failed copy available for review or cleanup when a later step fails.

If the native Start sheet is canceled, RunHaven removes the temporary terminal
destination and returns to the retained project choices. A canceled launch never
leaves an empty session page. Finished sessions do not poll or show live runtime
status; they keep only their ended summary, available output, and on-demand
diagnostics. Output actions disappear after RunHaven confirms that past output is
no longer available.

The safe defaults are the easiest path. By default a session gets assistant-services-only
network access, no access to your home folder, cloud keys, SSH keys, or browser
profiles, and it does not run as root. See
[docs/SECURITY_MODEL.md](SECURITY_MODEL.md) for the full boundary.

The app says `Run accepted` when it hands the run to the sandbox. The run then
shows up in your sessions as it comes up. Once it does, the acceptance notice
clears and RunHaven opens the run. You can find it again in History.

## Find your way around

The sidebar is how you move around the app. It has **Home**, **History**,
**System**, and, while an interactive terminal is open, **Run terminal**.
Individual runs, duplicate status filters, and a separate Settings screen stay
out of it. Open a run from History. If the run you are
viewing is removed, the app moves to the nearest available run or Home. It
never switches views on its own for any other reason.

The app icon in your Dock shows the same signal as one number: how many
sessions need you right now, and nothing else. The number goes down as you
take care of each session, and the badge disappears when nothing needs you.
You can turn the number off with **View > Show Count on App Icon**.

## Menu bar and keyboard shortcuts

The app has a normal Mac menu bar: RunHaven, File, Edit, View, Session, and
Window. Standard items like About, Quit, Undo, Copy, Paste, Close,
Minimize, Zoom, and Full Screen behave the way they do in every Mac app.
Quitting the app does not stop background runs; they keep working in the
sandbox and show up again when you reopen the app. A live **Run terminal** or
sign-in terminal is different: closing its window or quitting ends that live
terminal. Use **End run** when you are finished.

Menu items never skip a safety step. A menu action does exactly what the same
click in the window would do. **Stop** and **Force Stop** take you to that
session's control, and the native dialog remains the confirmation boundary.
Items that do not apply right now are grayed out, so their shortcut does
nothing until it can. Session menu items turn on only when a selected session can
take that action.

| Shortcut | What it does |
| --- | --- |
| Cmd-N | Open New session |
| Cmd-. | Go to the selected session's Stop control |
| Opt-Cmd-. | Go to the selected session's Force Stop control |
| Ctrl-Cmd-S | Show or hide the sidebar |
| Ctrl-Cmd-F | Enter or leave full screen |
| Cmd-W, Cmd-M, Cmd-Q | Close, Minimize, Quit |

Worktree actions stay in the finished session's review.

**Send Message** in the Session menu takes you to that run's composer, the
same way Stop takes you to its Stop control. The real send shortcut is
Cmd-Return, pressed in the composer: it sends the message you typed through the
same confirm step and native dialog as the Send button. The menu item carries
no shortcut of its own so Cmd-Return has one clear meaning.

RunHaven follows your Mac's light or dark appearance. It has no separate theme
setting to keep in sync. Security defaults and confirmation steps cannot be
turned off.

## What the app remembers

When you reopen the app, it always lands on Home, ready to start. The window
keeps its size and position, the sidebar stays shown or hidden, and the app-icon
count choice remains. RunHaven does not save the last screen you viewed.
It also does not save drafts, output, project paths, or sign-in data this way.

## Watch your sessions

Every run appears in **History** with its assistant, project, and one short status.
Runs that need action lead in **Needs attention**, with the exact reason and
one next step. Other runs stay in chronological order below. Open a run to see
one reading order. A failed run explains the failure, a run that needs sign-in
offers contextual **Sign in**, a run that stopped responding offers **Repair
run record**, and a finished worktree run points at its review. **Run actions**
sits near the top. Normal stopping is
immediate to find; **Force Stop** stays under **More actions**. Repair appears only
after RunHaven confirms that a stale record needs it.
Rust asks for native confirmation before any of these changes. Below that are a
compact overview, output, and the message box. Events and technical status stay
behind disclosures. Canceling a native confirmation is a quiet no-op, not an
error.

You can give a run your own name. Use **Name this run** or **Rename run** near
the top, type a short name, and save. The name
then shows in Run history and at the top of the run, in place of the
"assistant and project" name. Leave it blank to go back to the default name. This
name is just a label for you: it shows only in this app on this Mac, it does
not rename the saved run itself, and nothing sensitive is saved with it.

The output is trimmed so it will not flood the window, but raw output can still
include private data. **Show latest output** asks once in a native Mac dialog,
then loads one bounded snapshot. The page does not poll or stream it. It is
never revealed automatically. If older output is unavailable, the app says so
without showing an internal run ID or command error. If a session is ready for
input, you can type a message and send it. A native Mac sheet shows the target,
byte count, and a short preview. You must confirm each exact message.

Deeper runtime detail (container state, resources, networks) and the
run's own diagnostics sit behind a **Show details** button, so the view
stays focused on what needs you.

The list updates itself in place. It does not blink, clear your draft, or jump
back to the start screen while a run keeps working.

## If a session stops responding

If a run's sandbox has gone away but its session record is still open, the app
marks the session `Not responding` and highlights that repair is available.
Repair clears the stale session record safely. The app confirms the sandbox is
really gone before it highlights repair, so it never clears a record for a run
that is still alive.

## Control a run

For an active session you can Stop it or Hard stop it. Both ask you to confirm
and show the exact target first. Stop asks the run to end. Hard stop ends it
right away. **Repair run record** appears only after RunHaven confirms that the
sandbox is gone; it clears that stale record after native confirmation.

## Open a terminal

A running session has an **Open terminal** button. It gives you a live shell
inside that session's sandbox, so you can type commands and see what they print,
right in the app. The terminal shows only for running sessions, not for ones
that have finished or failed.

When you open it, the app names the sandbox and asks you to confirm first. What
you type goes straight to the sandbox, as the same restricted user, with no
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

The interactive run terminal from **New session** uses the same sandbox and
restricted user, but keeps a bounded, filtered copy of its output in process
memory so History can help diagnose the run. That copy is never written to disk
or logs and disappears when the app closes. An attached shell and a sign-in
terminal remain shown-only and are discarded when the terminal closes.

Close the terminal with **Close terminal**, or just leave the session or reload
the app; either way the app shuts down the shell in the sandbox.

The sign-in terminal in New session is the same kind of in-app terminal.
It runs the selected provider's native login command in a fresh sandbox. It
keeps the same restricted sandbox, assistant-services-only network, and standing warning
above; any code you paste is visible on screen and is not saved by RunHaven.

## Review changes

When an assistant finishes in a protected copy, the session shows the files it
added, changed, or removed. These are project-relative names, never full paths.
RunHaven shows at most 100 names and says when more are present. If your project
also has local changes, those files appear in a separate list and **Apply
changes** stays unavailable until you finish or undo them.

There are two routine choices:

- **Apply changes** adds the assistant's changes to your project and deletes the
  protected copy.
- **Discard changes** permanently deletes the assistant's changes and the
  protected copy. Your project does not change.

Leaving the page keeps the protected copy for later. Recovery steps appear only
when Apply cannot finish safely. Each routine choice uses a native Mac
confirmation.

Before and after confirmation, RunHaven checks the exact reviewed state again.
If a file changed after the list loaded, nothing is applied or deleted. Refresh
the review and choose again. RunHaven never shows full paths, saved state, file
contents, or secrets in the review.

## Check this Mac

Open **System** from the sidebar. It loads by itself and shows one
health result. Passing checks stay out of the way. A failed check explains the
problem and, when RunHaven has a safe automatic fix, shows one fix button.
**Check again** re-reads the status.

The same page checks for leftover RunHaven networks and removes them with one
button plus the native Mac confirmation. If internet access stops working even
though a run allows it, **Restart container service** safely stops and starts
Apple Container after a native warning. This affects every Apple Container
workload on the Mac and ends current RunHaven sessions, so RunHaven never
performs it silently. If stopping, starting, or the final readiness check fails,
the page refreshes and tells you which recovery action to take next.

Version information and the sanitized Apple Container readiness checks stay
under **Advanced system information**. RunHaven-scoped network cleanup remains
on the same page; assistant images and saved sign-in repair stay next to the chosen
assistant in New session. The app
does not mirror the entire `container` command line. Arbitrary `exec`, `copy`,
`export`, registry login, machine management, and deletion of non-RunHaven
resources stay command-line only because they are not needed to start or repair
a RunHaven session. The page does not show broker implementation labels, raw
trust-event feeds, network names, full paths, secrets, or request bodies.
A session's own allowed or denied sign-in and network decisions live only in
that session under **Show details**, next to the reason for each decision.

The desktop check omits the Rust toolchain because coworkers do not need Rust.
The contributor-only CLI `doctor` command still checks the pinned toolchain.

When setup is not ready, System details does not ask the stopped container
service for image or network details. The setup checks still show the problem
and its next step.

## Debug logs

The desktop app writes a local log file while it runs. If something looks
wrong, share this file instead of a screenshot:

Development log: `~/Library/Logs/dev.runhaven.desktop.debug/runhaven.log`

Release log: `~/Library/Logs/dev.runhaven.desktop/runhaven.log`

The log records app events, errors, and the name and timing of each action.
It does not record what you see or type in the live terminal, request bodies,
or tokens: the log level is capped below the point where the network layer
would print raw request bytes, so a token or a request body cannot reach the
file. The file rotates at 5 MiB and the app keeps the five most recent older
files, so it stays small.

Before sharing a log, you can double-check it with `scripts/scan-dev-log.sh`,
which fails if any token-shaped text ever appears.

## Cleanup and repair

Repair appears where it is needed. Each action asks you to confirm in a native
dialog before anything changes.

- Install or rebuild an assistant image from **New session**. Use this when an assistant image
  is missing or out of date. RunHaven rebuilds it from the bundled template in
  the background and shows live progress: building, then succeeded or failed,
  with a short, secret-safe tail of the build output. Sessions that are
  already running keep working until you launch a new one. The confirm dialog
  names the exact assistant and image tag. Leaving and returning does not start a
  second build for the same tag; choosing rebuild again shows the existing job.
  Immediately before building, RunHaven repeats the full supported-runtime
  check shown in System details. Quitting RunHaven cancels and reaps the active
  builder process.
- Remove stale networks from **System details**. RunHaven first shows
  how many of its own leftover networks it can remove, then removes only those
  after you confirm. Cleanup stays unavailable until setup is ready. If listing
  networks fails, RunHaven stops before deleting anything. A network that
  cannot be removed is left alone and reported as **Could not remove**, not as
  a successful cleanup. The app shows only counts, never network names.
- Restart Apple Container from **System** when its local network or DNS has
  stopped working. RunHaven verifies the supported runtime, warns that all Apple
  Container workloads will stop, runs the exact stop/start sequence, and checks
  the service afterward. It does not change images, volumes, registry state, or
  machine configuration.

Resetting or pruning saved state stays in the command line on purpose. That
task can erase an assistant's saved sign-in across sessions, and previewing it needs
saved-state names that the app never receives, so it fails closed rather than
guess.

## What still needs the terminal

For now, do these tasks with the `runhaven` command line
([docs/USAGE.md](USAGE.md)):

- Resetting or pruning saved state.

These will move into the app over time. Until then, the app stays focused on the
core run-and-watch workflow so the safe path stays simple.
