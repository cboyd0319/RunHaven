# Security Model

## Boundary

Trusted:

- the user
- this Rust CLI
- Apple `container`
- the selected agent image

Untrusted or partially trusted:

- repository contents
- model output
- package install scripts
- MCP servers
- shell commands selected by an agent
- external network responses

Sensitive:

- macOS home directory
- SSH keys and agent socket
- cloud credentials
- model-provider credentials
- browser profiles
- unrelated repositories
- agent session logs

## Default Protections

`runhaven` protects against accidental broad local access by default:

- mounts only the selected workspace
- isolates agent home state in a project/profile/session named volume
- runs a read-only root filesystem
- drops Linux capabilities
- uses a non-root user in bundled images
- does not mount `~/.ssh`, `~/.aws`, `~/.config`, or the macOS home directory
- does not pass host environment variables unless named with `--env`
- rejects broad or credential-bearing workspace paths unless
  `--allow-sensitive-workspace` is passed
- rejects root agent execution unless `--allow-root-user` is passed
- shows the exact command with `runhaven plan`

For bundled non-root images, `runhaven` runs a short volume-preparation preflight
before the agent starts. That preflight mounts only the project-scoped
`/home/agent` volume, sets ownership for UID/GID 1000, runs without DNS, and
uses a dedicated internal network.

`runhaven` also serializes access to each project/profile/session home volume.
This avoids concurrent attachment of the same named volume and keeps the
failure mode understandable for non-technical users. The in-app Claude sign-in
terminal runs `claude auth login` through the same serialized path, so while it
is open it holds the shared Claude home volume: a Claude run or a second Claude
sign-in fails closed with the "already in use" message until the sign-in
terminal is closed. It self-clears on close, on navigating away, or on quit. Named sessions selected
with `--session NAME` reuse only the isolated `/home/agent` volume; they do not
widen workspace mounts or host credential access. `state reset` and
session-filtered `state prune` delete RunHaven-managed home volumes only.

By default (`--auth-scope agent`) an agent's home volume, including its login,
is shared across all your projects (`runhaven-<agent>-shared-home`), so an OAuth
or subscription login is done once and reused everywhere. This trades isolation
for convenience: a workspace you run could read or alter that shared login,
though in `provider` mode the egress allowlist still blocks exfiltrating the
credential off the host. Use `--auth-scope project` to isolate a project's login
and state to its own per-workspace volume. The shared volume is removed by
`state prune` like any other RunHaven home volume.

`network list` and `network prune --yes` operate only on RunHaven-managed
Apple `container` network names and do not delete arbitrary Apple-managed or
user-created networks.

`image doctor` is read-only. It lists local Apple `container` image metadata,
checks expected bundled RunHaven image tags, compares RunHaven image source
metadata, reviews inactive RunHaven state volume names, and does not build
images, delete resources, mount workspaces, read credentials, or reset state.

## Secure Easy Path And Explicit Overrides

Above all else, the default and easiest path must be the secure path. New CLI
and UI workflows should make the narrow choice the shortest path: smallest
selected workspace, non-root bundled image, no host credential mounts, no
arbitrary environment passthrough, no raw SSH keys, explicit network choice,
and visible plan output.

Supported advanced choices warn but do not hide or block. Lower-security run
configuration (a sensitive workspace, a custom or root `--user`, a custom image,
`--env` passthrough, or an additional provider host) requires an explicit flag
and prints plain-language `Security notices` at plan and run time so the tradeoff
is visible. Destructive resource operations (state deletion, network pruning,
worktree merge or discard, hard-stop recovery) require explicit confirmation such
as `--yes`. Apple `container machine` use is warned and explicit rather than
blocked.

The default network mode is profile-aware so the secure path is also the default
path: profiles with bundled provider hosts default to `provider` (egress limited
to the agent's own API), and profiles without bundled hosts default to `internet`
because provider mode would have an empty allowlist there. Internet mode prints a
security notice and is never blocked; choose it explicitly with `--network
internet` when a run needs package installs or other hosts.

Unsupported, invalid, or nonfunctional paths still fail closed. Examples include
failed setup checks, invalid input, missing confirmations, and unsupported
platform state. This is not a policy block around a supported choice; it is a
runtime correctness and safety failure.

## Container Machine Is Not Default

RunHaven uses task-scoped `container run` commands instead of Apple's persistent
`container machine` workflow by default. The machine workflow is useful for
Linux development environments, but it is the wrong beginner-safe default for
AI coding agents because it can map the host user and home directory into the
guest. Secondary hands-on reporting also notes that the safer machine option is
to disable that home mount.

RunHaven's default boundary is narrower: mount one selected workspace, attach
one project-scoped agent home volume, and never mount the macOS home directory
or raw credential folders by default.

Explicit or user-managed `container machine` workflows are supported
lower-security choices, not hard policy violations. RunHaven should warn about
host-home, credential, persistence, and cleanup tradeoffs before integrating
with or managing a machine workflow, and require explicit user intent. It should
fail only for concrete unsupported or unsafe states, such as invalid targets,
missing confirmation, unimplemented management flows, or destructive operations
without a reviewed approval gate.

## Auth Broker Boundary

`runhaven auth status` and `runhaven auth explain AGENT` describe the host-side
provider credential broker boundary. The host-side API-key broker covers Codex,
Claude, and Gemini; Copilot and Antigravity are not brokered and use isolated
in-container login state. Those commands read static profile metadata. They do
not inspect Keychain, browser profiles, cloud credential files, provider login
caches, or environment variable values, and they do not print secrets.

The intended pattern is host-owned credentials with provider-specific policy
tied to the endpoint matrix. The guest should receive only a narrow
provider action or short-lived run credential when that flow is explicitly
implemented and verified. Broad host credential import, implicit environment
passthrough, and host home or credential-folder mounts remain out of scope.

## TUI Easter Egg Boundary

The hidden TUI Zork I easter egg is local to the terminal UI. It does not call
Apple `container`, spawn subprocesses, open network sockets, mount workspaces,
read provider credentials, inspect the environment for secrets, or access the
selected project. The game story is bundled at compile time from
`third_party/zork1/COMPILED/zork1.z3`; RunHaven checks the exact byte length and
SHA-256 before starting the in-process Z-machine.

Zork `save` and `restore` use a single RunHaven-owned cache slot:
`$RUNHAVEN_CACHE_HOME/tui/zork1-save.ifzs`, or
`~/Library/Caches/runhaven/tui/zork1-save.ifzs` by default. The file is written
with owner-only permissions through the same private-cache helpers as other
RunHaven state. Restore never accepts a user-supplied path: RunHaven reads only
that fixed slot, rejects symlinked or non-file save slots, oversized files,
non-Quetzal data, unknown IFF chunks, malformed stack frames, duplicate required
chunks, and incomplete save headers before passing bytes to the vendored parser.
Parser and VM restore panics are contained as restore failures, and the active
game VM is replaced only after a fresh VM successfully restores the save. The VM
then validates release number, serial, and checksum against the bundled story
before restoring.

## Extension And MCP Boundary

RunHaven does not currently enable MCP servers, editor extensions, or plugin
marketplaces inside managed agent runs. Future support is deny-by-default and
must follow [`EXTENSION_MCP_BOUNDARY.md`](EXTENSION_MCP_BOUNDARY.md): no host
socket, credential helper, extension, or MCP server is exposed unless the user
explicitly enables it and the plan can show the exact access before launch.

## Session Input Grant

The desktop app can send typed messages to a running managed session. Sending
input is a mutating action, so it is gated the same careful way as stop, kill,
and cleanup: RunHaven shows a native macOS confirmation dialog that names the
session and its run before any message reaches the agent. The WebView cannot
approve on its own.

Each message is bounded before it is sent. RunHaven rejects an empty message, a
message over 8 KB, a message with a NUL byte, and a message that carries a
terminal escape sequence. It also checks that the selected session still matches
the one the message was typed for. Any of these checks failing stops the send
(fail closed).

One confirmation does not last forever. The approval is a grant tied to the
exact run id, and it only applies while the session is in its normal
"monitor" state. The grant is metered: RunHaven asks for a fresh native
confirmation once the messages sent under it add up to 256 KB, or once 30
minutes have passed, whichever comes first. This keeps a normal back-and-forth
smooth while making sure a long-lived session cannot be driven indefinitely
after a single click. The grant is cleared as soon as the session's input
transport is unregistered (for example when the run ends), so a later run that
happens to reuse a session name never inherits an old approval.

## Session Monitor Windows

The desktop app can open a small extra window that watches one managed
session (`Open in New Window`). These monitor windows are read-only by
construction, not by convention:

- Every mutating desktop command (stop, kill, repair, cleanup, worktree
  actions, launch, image rebuild, sign-in, network prune, input send, Dock
  badge) is granted only to the main window. A build-time test gate
  (`capability_guard` in `src-tauri`) fails if any capability ever widens
  past the main window or if a monitor window gains anything beyond its
  fixed read set.
- A monitor window's whole grant is three read commands: which session it
  shows, that session's status, and that session's secret-free diagnostics.
  Rust also checks, on every call, that a monitor window reads only the
  session it was opened for; requests from unknown window labels are denied.
- Raw output is not viewable from a monitor window. The output confirmation
  dialog is app-owned today rather than parented to the asking window, and
  even redacted output can carry sensitive text, so output viewing stays
  with the main window (fail closed).
- Only the main window can open monitor windows, through one validated Rust
  command: the session must exist in RunHaven's own records, at most six
  monitor windows can be open at once, and the window's label is minted by
  Rust from a counter, never from user input. Monitor windows cannot open
  further windows, listen to menu events, or write the app's remembered
  place and settings.
- Monitor windows are never restored across relaunch, and closing the main
  window closes them too, so no watching window outlives its controls.

## Terminal Sessions

The desktop app can open a live terminal inside a sandbox, in two shapes: an
attach into one running sandbox, and a Claude sign-in terminal. This is the most
powerful interactive surface it has, so it is fenced carefully.

- RunHaven never trusts a name from the WebView. When you open a terminal,
  RunHaven looks up the run in its own active-run records and builds the
  `container exec` command in Rust, reusing the exact same checks as the CLI
  `runs attach`: the container must be RunHaven-owned, and the shell always runs
  as the non-root `agent` user. There is no way for the GUI to run the terminal
  as root; `--allow-root-user` is never offered here.
- Consent happens once, before anything starts. A raw keystroke stream cannot be
  confirmed message by message, so RunHaven shows a native confirmation dialog
  that names the run and its container and states, in plain words, that what you
  type goes straight to the sandbox and what it prints is shown but not filtered
  and not saved. If you cancel, no terminal and no `container exec` process is
  ever started.
- The live stream is shown, not stored. This is the one place RunHaven does not
  redact, and it says so up front. The reader relays each chunk from the sandbox
  straight to the terminal view and nowhere else: it does not append to the
  captured-output history, write run records, or write any log or file. A
  build-time source test (`terminal` in `src-tauri`) fails if that code ever
  gains a path to disk or to the output history. This "does not filter, read,
  store, or log" statement is scoped strictly to the live terminal stream, not
  to the whole app: saved output snapshots, run records, and auth/egress history
  are still redacted and kept locally as before.
- The terminal adds interactivity, not reach. It attaches into the run's
  existing sandbox, so it inherits that run's read-only root filesystem, dropped
  capabilities, and provider-only network. No new mounts, no host environment,
  no `container machine`.
- Only the main window can open a terminal. The `terminal` capability is granted
  to the main window alone; the read-only monitor windows never receive it, and
  the `capability_guard` tests fail closed if that ever changes. At most two
  terminals can be open at once; opening past that fails with a clear message.
- Terminals are never restored across relaunch, and closing the main window
  kills every live terminal child, so no sandbox shell outlives the app.

The Claude sign-in terminal shares that whole boundary and adds a few facts of
its own:

- It is Rust-owned and native-confirmed. The Agents view button opens a terminal
  whose target is fixed to Claude in Rust and core; the WebView supplies only a
  session id (a registry key and run label), never what runs. A native dialog
  names that it opens a Claude sign-in terminal and states the same
  no-filter/no-store facts, plus that the code you paste is visible on screen.
  Nothing starts until you confirm.
- It runs the same in-sandbox login as `runhaven login claude`, not a raw host
  command. Because a sign-in has no already-running container, it starts a fresh
  one through the normal provider launch path, so provider-only egress stays
  enforced by the live allowlist proxy the whole time (a bare `container run`
  would have no route to the sign-in hosts). It keeps the non-root `agent` user,
  read-only root filesystem, dropped capabilities, and the existing
  `platform.claude.com` / `claude.ai` provider hosts. No new egress, no new
  mounts, no host environment. Only the child's stdio is a host PTY so the pasted
  code can round-trip.
- The captured-output device-code sign-in still refuses Claude. That runner
  feeds the login `/dev/null` stdin and cannot host a paste-back, so it stays
  refused; the terminal path is a separate, narrower command and does not weaken
  it. The login byte stream follows the same shown-not-stored rule as any
  terminal; the run metadata it writes (a run record, like the CLI login) is not
  the stream.
- It shares the terminal capability and the two-terminal cap, main window only,
  and fails closed: if the login terminal cannot open (the container tool is
  missing, the image is not built, the PTY or proxy setup fails), nothing is
  registered and the app keeps its "run `runhaven login claude` in your terminal"
  fallback.

## What This Does Not Solve Yet

Internet mode is unrestricted egress within the host's network policy. It is the
default only for profiles without bundled provider hosts, and is available to any
profile with `--network internet`. Use `internal` for local-only runs.

`--network provider` is the constrained egress path and the default for profiles
with bundled provider hosts.
It runs the agent on an internal Apple `container` network, starts a host-side
CONNECT proxy, injects proxy environment variables at runtime, and deletes the
managed provider network after the run. Apple `container` 1.0.0 exposes an
internal gateway address to guests that is not always bindable on the macOS
host, so the host listener can bind wildcard while rejecting clients outside the
inspected Apple `container` subnet. The proxy permits the bundled provider hosts
for the selected profile, their subdomains, maintainer-curated domain-family
patterns (for example `*-cloudcode-pa.googleapis.com`, anchored inside one
registrable domain), and explicit fully qualified `--provider-host HOST`
additions. It rejects IP literal proxy targets and single-label provider hosts.
Before opening an upstream connection, the proxy resolves the destination and
rejects non-public resolved addresses such as loopback, private, link-local,
multicast, or otherwise local-only addresses. It relies on the internal network
to block direct guest egress. After a provider run that denied any target,
RunHaven prints a calm two-line notice that names the agent and the count of
blocked destinations and points to `runhaven egress log`; the per-host detail
(host, port, count, denial reason, matched rule) stays in that log so users can
review missing endpoints without weakening the default policy.

Host-only networks block guest egress to the internet, but they do not firewall
the host's own listening ports. A guest on an `internal` or `provider` network
can open a raw TCP connection to any service bound to the host on the Apple
`container` gateway address or `0.0.0.0`, for example a local dev server, a
database, or another tool's proxy. Apple `container` 1.0.0 has no per-port
guest-to-host firewalling, so this is a runtime limitation, not a
misconfiguration. It was empirically confirmed on 2026-06-26 (macOS 27.0): a
guest retrieved a sentinel from a host listener on the gateway while its direct
internet egress was refused. Treat the host as reachable from active runs and do
not bind sensitive services to `0.0.0.0` or the gateway interface while a run is
active. An in-guest egress filter could close this as defense in depth; it is
tracked as a design-first item in [`NON_UI_BACKLOG.md`](NON_UI_BACKLOG.md).

Provider runs also append allowed and denied CONNECT policy decisions to a
RunHaven cache log. `runhaven egress log` shows recent decisions, including the
matched rule, denial reason, count, profile, workspace, and run id. `runhaven
why host HOST` explains bundled provider-host matches, IP-literal rejection,
and the next review step before the user adds a new host.

Actual agent runs also append one secret-free record to `runs.jsonl`.
`runhaven runs list`, `runhaven runs show RUN_ID`,
`runhaven runs log RUN_ID`, and `runhaven runs diff RUN_ID` expose
completed-run metadata such as run id, profile, workspace, network mode,
return code, provider policy summary, auth broker summary, cleanup outcome, and
matching provider/auth log entries for the run. While a run is active, a
temporary active-run marker records the RunHaven-owned container name so
`runs active` can list current run id, profile, workspace, network mode,
status, and container name. The same marker lets `runs status` call Apple
`container inspect` for curated live state, lets `runs attach` call Apple
`container exec` for a guarded shell or command, lets `runs logs-follow` call
Apple `container logs --follow`, lets `runs stop` call Apple `container stop`,
lets `runs kill` call Apple `container kill`, and lets `runs repair` remove a
stale marker only after Apple `container inspect` reports that the recorded
RunHaven-owned container is not found. `runs repair --all` applies the same
confirmed-missing guard to each valid active marker. Repair JSON output
contains only result status, counts, exit code, run id, container name, and
inspect return code; the marker is removed after the run finishes.
When the workspace is inside a git repository, the run ledger also records repo
root, before and after `HEAD`, dirty state, changed file count, and a capped
list of relative paths scoped to the selected workspace. Run records also store
the workspace scope choice, so later review can distinguish the default
current-directory mount from an explicit `--workspace-scope git-root` run.
For `--worktree` runs, RunHaven requires a clean source repository, creates a
RunHaven-owned branch and git worktree, mounts the worktree instead of the
source checkout, and records the source repo, worktree path, branch, base
commit, and recovery commands. Dirty source checkouts fail before worktree
creation and print safe next choices. `runs recover` and `runs recover --json`
are read-only. Suggested project checks are advisory commands and are not run
automatically. RunHaven does not automatically merge or delete that worktree
after the run.
The run ledger and active-run markers do not record diffs, file contents,
prompts, command lines, agent arguments, attach commands, environment variable
names, environment values, request bodies, or token values. Live container logs
may still contain whatever the agent process printed during the run. `runs diff`
prints a live git diff on demand only after verifying the recorded repo root,
`HEAD`, and path set still match; dirty working-tree diffs include a warning
because metadata cannot prove file contents stayed unchanged after the run.
`runs status` does not print raw Apple `container inspect` data because that
data can include process arguments, environment, and mount details.

Provider host allowlists are intentionally conservative and source-backed.
Bundled auth and provider routing hosts are tracked in
[`PROVIDER_ENDPOINTS.md`](PROVIDER_ENDPOINTS.md). Telemetry, reporting,
release-note, update, plugin marketplace, and broad path-sensitive hosts may
fail until an additional fully qualified host is reviewed and passed with
`--provider-host`.

The host-side auth broker covers the API-key path for Codex, Claude, and Gemini
through `--api-key-broker-env NAME`. It keeps the raw API key in the RunHaven
host process and gives the guest only a placeholder key plus a base-URL redirect
at the broker (Codex custom-provider config, or `ANTHROPIC_BASE_URL` /
`GOOGLE_GEMINI_BASE_URL`). Broker decisions are logged without request bodies,
token values, or environment variable names. Even with the broker the credential
is usable, not readable: a compromised guest cannot read the key but can still
spend through it within the pinned host and path the broker forwards to. The
broker is for API keys only; OAuth and subscription logins (and Copilot, which
cannot be brokered without TLS interception) use isolated in-container state, and
RunHaven never reads your host `~/.claude.json` or Keychain. `--env NAME` remains
an explicit fallback that places a token in the guest. See
[`AUTH_BROKER.md`](AUTH_BROKER.md). Use
[`AUTH_BROKER.md`](AUTH_BROKER.md) for the current boundary and non-goals.

RunHaven never holds any agent token. All four sign-in agents (Claude, Codex,
Copilot, Antigravity) log in inside the sandbox with `runhaven login <agent>`,
and the resulting credential lives only in that agent's isolated home volume, so
RunHaven never sees, stores, logs, or injects it. Claude signs in the same way
now: `runhaven login claude` runs `claude auth login` in the sandbox instead of
capturing a host token. `runhaven login claude --clear` also best-effort removes
any token an older RunHaven stored on your Mac before this change, so nothing
lingers.

The selected agent still controls what it reads inside `/workspace` and
`/home/agent`. If the agent has model credentials inside its project volume and
internet access, malicious repository content may still try to exfiltrate those
credentials. Agent-native permission systems remain useful, but they are not a
replacement for the outer container boundary.

## Safe Defaults for Beginners

Use this order:

1. Run `runhaven plan`.
2. Build or select a known image.
3. Run without `--env` first and authenticate inside the isolated agent home
   volume when possible.
4. Use `--env NAME` only when a headless run needs a token.
5. Use `--read-only-workspace` for review and audit tasks.
6. Use `--ssh` only when a run needs Git-over-SSH or remote access: it forwards
   your SSH agent socket (never raw keys) so the sandboxed agent can use your
   host SSH identities, which is a warned lower-security choice.
7. Use `--allow-sensitive-workspace` or `--allow-root-user` only when the
   security tradeoff is intentional.
