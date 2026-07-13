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
After taking that lock, it checks Apple container's captured JSON list for an
existing attachment of the exact volume. This closes the process-crash gap
where the host lock is gone but an orphaned container still holds the volume.
RunHaven never logs that container list and never auto-removes the other
container. It fails before launch with one fixed, path-free recovery message.
The in-app Claude sign-in
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

Immediately before a real run or in-sandbox sign-in, Rust repeats the read-only
host checks used by `runhaven doctor`: macOS 26+, Apple silicon, the pinned
Apple `container` CLI and API server version and commit, the running service,
and the pinned builder, vminit, and Kata kernel settings. Any failed check stops
the launch before the agent container starts. Plan and dry-run paths do not run
this gate.

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
is visible. The desktop's native launch confirmation lists every requested host
environment variable name, but never reads or shows its value. Destructive
resource operations (state deletion, network pruning,
worktree merge or discard, hard-stop recovery) require explicit confirmation such
as `--yes`. Apple `container machine` use is warned and explicit rather than
blocked.

The default network mode is profile-aware so the secure path is also the default
path: profiles with bundled provider hosts default to `provider` (egress limited
to the agent's own API), and profiles without bundled hosts default to `internet`
because provider mode would have an empty allowlist there. Internet mode prints a
security notice and is never blocked; choose it explicitly with `--network
internet` when a run needs package installs or other hosts.

In `provider` mode RunHaven also turns off the agent's own non-essential traffic
where the agent supports it, so it does not even attempt to reach hosts the
allowlist would block. Claude Code, for example, is run with
`CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1`, which disables its telemetry,
error reporting, and auto-update checks. This keeps the egress log to the real
API calls and stops the agent from phoning home; the allowlist is still the
hard boundary if the agent tries anyway.

Unsupported, invalid, or nonfunctional paths still fail closed. Examples include
failed setup checks, invalid input, missing confirmations, and unsupported
platform state. This is not a policy block around a supported choice; it is a
runtime correctness and safety failure.

### Desktop setup recovery

The desktop app can start Apple container's local service when that setup check
fails. This is a narrow host mutation:

- The untrusted WebView supplies only the opaque action ID
  `start-container-system`. Unknown IDs and extra executable or argument fields
  are rejected during deserialization.
- Rust uses Apple's fixed `/usr/local/bin/container` path, verifies that same
  executable is the pinned 1.1.0 release, and maps the ID to the fixed argument
  list `system start --enable-kernel-install`. The flag prevents an unattended
  app process from waiting on a terminal prompt. No shell string or
  frontend-supplied path reaches process creation.
- The command has its own one-permission Tauri capability, scoped only to the
  main window. Read-only and session monitor windows cannot call it.
- Rust rechecks macOS 26+, Apple silicon, the exact CLI version, and that the
  service is still stopped before showing the native confirmation. Canceling
  leaves the host unchanged. A single-flight gate rejects duplicate repair
  requests while one is active.
- After the start command exits successfully, Rust checks the service again and
  reports success only when it is running. The WebView receives only a fixed
  success status or a fixed path-free error. Raw process output is discarded
  and never returned.

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
provider credential broker boundary. The host-side API-key broker covers Codex
and Claude; Copilot and Antigravity are not brokered and use isolated
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

The desktop app can open a live terminal in three cases: attach to a running
sandbox, sign in to an agent, or run an interactive agent. This is the app's
most powerful interactive surface, so it is fenced carefully.

- Rust validates the session id and resolves or builds the exact target. Agent
  sign-in accepts only Claude, Codex, Antigravity, or Copilot.
- A native confirmation appears before any terminal child starts.
- Terminal bytes go directly between the PTY and the WebView. RunHaven does not
  append them to captured output, run records, status registries, or logs.
- The terminal changes only stdio. The container keeps its non-root user,
  read-only root filesystem, dropped capabilities, reviewed mounts, and selected
  network policy.
- Only the main window receives the terminal capability. All terminal types
  share the same two-session resource cap and close-on-window teardown.
- Sign-in uses the same native provider command as `runhaven login <agent>` on
  the agent's isolated home volume. No project folder is mounted.
- After sign-in, RunHaven writes a secret-free completion marker before the UI
  refreshes only after any required provider-owned readiness check succeeds.
  Claude requires `claude auth status`. Antigravity can return code 1 after a
  successful interactive session, so its image wrapper requires both its fixed
  credential file to be nonempty and a successful, non-generative `agy models`
  probe before translating that one code to success. Other exit and
  cancellation codes are preserved. RunHaven never reads credential bytes.
- A force-quit can leave an unused RunHaven internal network while the detached
  cleanup thread is finishing. The Cleanup screen safely removes that network.

The provider CLI may write its own configuration, credential, or support-log
files inside its isolated volume. The no-store statement applies to RunHaven's
terminal relay, not to the provider's own behavior.

The interactive agent run terminal started from **New run** shares that same
boundary and adds a few facts of its own:

- It is Rust-owned and native-confirmed. The New Run flow hands the reviewed
  request to Rust, which reuses the exact normal-launch path: it builds the same
  plan and shows the same native launch confirmation a background run shows.
  Nothing runs, and no worktree branch is created, until you confirm. The
  WebView supplies only a session id (a registry key and run label), never what
  runs; the plan comes from the reviewed request and Rust-side selection.
- Only the execution differs from a background run. The same finalized plan runs
  under the same provider egress proxy, non-root `agent` user, read-only root
  filesystem, dropped capabilities, and workspace mount; nothing is widened. The
  one change is that the run's argv is forced to `--interactive --tty` and its
  stdio is a host PTY, so the agent runs its interactive TUI (an interactive agent
  with no terminal would otherwise drop to headless `--print` and error with no
  prompt). Ordinary OAuth state stays in the selected agent's isolated volume.
  If an advanced API-key broker was explicitly configured, its key remains in
  the host broker and never enters the terminal stream.
- The run byte stream follows the same shown-not-stored rule as any terminal: the
  reader relays each chunk to the WebView `Channel` and nowhere else, and the same
  build-time source test covers the run spawn, so it cannot gain a storage or
  logging sink. The run metadata the launch path writes (a run record) is not the
  stream.
- It shares the terminal capability and the two-terminal cap, main window only,
  and fails closed: if the run terminal cannot open, nothing is registered and the
  app keeps its background-run behavior for non-interactive agents.

## What This Does Not Solve Yet

Internet mode is unrestricted egress within the host's network policy. It is the
default only for profiles without bundled provider hosts, and is available to any
profile with `--network internet`. Use `internal` for local-only runs.

`--network provider` is the constrained egress path and the default for profiles
with bundled provider hosts.
It runs the agent on an internal Apple `container` network, starts a host-side
CONNECT proxy, injects proxy environment variables at runtime, and deletes the
managed run network after the run. Local-only `internal` runs use the same
automatic per-run network cleanup. Apple `container` 1.0.0 exposes an
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

The host listeners also bound untrusted guest resource use. The CONNECT proxy
admits at most 32 concurrent clients, and the credential broker admits at most
8; excess connections close before a worker thread is created. Broker requests
have an 8 MiB body limit, an 8 KiB request-line limit, and a 10-second
local-client timeout. Each service retains at most 256 distinct decision keys
per run and combines later keys into fixed allowed or denied overflow counts.

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

Ordinary OAuth for Claude, Codex, Antigravity, and Copilot runs inside each
agent container. RunHaven does not read Mac Keychain items, provider state from
the Mac home folder, or browser profiles. The provider CLI stores its credential
in the isolated agent volume. That volume is owner-scoped but is not a
hardware-backed secret store.

The optional host-side API-key broker covers explicitly configured Claude and
Codex API keys through `--api-key-broker-env NAME`. It is never selected only
because an environment variable exists. The plan stores the name, not the
value. At launch the broker keeps the key in host process memory and gives the
guest only a placeholder plus a provider base-URL redirect. A compromised guest
cannot read the key directly but can still spend through it within the broker's
pinned host and path. Provider-only network mode is required.

Copilot and Antigravity have no RunHaven API-key broker. `--env NAME` remains
an explicit lower-security fallback that places the named value in the guest.
See [Authentication and API-key brokers](AUTH_BROKER.md).

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
