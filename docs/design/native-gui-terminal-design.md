# Native GUI Embedded Terminal — Design

Status: Historical design record. The PTY design was implemented, then the
sign-in surface was generalized on 2026-07-12 so Claude, Codex, Antigravity,
and Copilot all use their native interactive login inside the agent container.
References below to the former captured-output sign-in runner describe the
pre-implementation state. Use `docs/DESKTOP_APP.md` for current behavior.
Every boundary choice is labeled: [S] verified against source, [C]
Tauri/AppKit convention, [J] product judgment.

## Project shape and platform targets

- Tauri v2 (`tauri` 2.11.3) desktop shell in `src-tauri/`, React + shadcn/ui
  WebView in `ui/`, over typed commands into `runhaven-core`.
- macOS 26+ on Apple silicon only. No Windows/Linux runtime target, so a
  Unix-only PTY path is acceptable.
- Runtime is Apple `container` 1.1.0. Interactive model is
  `container exec -i -t <id> <shell>` (new process in a running container) or
  `container run -i -t ...` (fresh container). There is no docker-style attach
  to a live PID 1 PTY. [S]

## Goal

An in-app terminal (xterm.js in the WebView, bridged to a container PTY) that
(1) lets `claude auth login` paste-back happen in-app, so signing in to Claude
no longer needs an external terminal, and (2) lets a user type live with any
running agent session. Today the GUI has send-one-message input plus bounded,
redacted output (`launch_runner.rs` uses byte pipes, `agent_sign_in_runner.rs`
uses `Stdio::null()` stdin — which is exactly why Claude sign-in cannot finish
in-app). The CLI already has interactive `runs_attach` (real terminal, CLI
only; no attach Tauri command exists).

## 1. Host-side PTY (the load-bearing decision)

Decision: RunHaven MUST wrap every interactive `container` child in a HOST
pseudo-terminal (`openpty`). This is required, not an optimization. [S]

Evidence, read from the Apple `container` source (public repo
`https://github.com/apple/container`, and its pinned dependency
`https://github.com/apple/containerization` at commit
`44bec8b9933bc491d0cbf44abac90a1f6aaebf6b`):

- The client always tries to put its own stdin into raw mode before creating
  the guest process. `Sources/Services/ContainerAPIService/Client/ProcessIO.swift`
  (`create`, ~lines 46-54): with `-i -t` it calls
  `Terminal(descriptor: STDIN_FILENO)` then `setraw()`. `Terminal.init` runs
  `tcgetattr` and `setraw` runs `tcgetattr`+`cfmakeraw`+`tcsetattr`
  (`ContainerizationOS/Terminal.swift`). On a pipe or `/dev/null` these throw
  `ENOTTY`. [S]
- Ordering: in `ContainerExec.swift`, `ProcessIO.create` (~line 82) runs before
  `client.createProcess` (~line 87). Because `create` throws first,
  `createProcess` never runs. The command fails as `failed to exec process ...`
  and NO guest PTY is allocated. There is no graceful no-tty fallback. [S]
- Resize: only when a client console exists does `handleProcess`
  (`ProcessIO.swift` ~176-196) read `Terminal.size` (`ioctl(TIOCGWINSZ)` on
  STDIN), send an initial `process.resize(size)`, then on each `SIGWINCH`
  re-read the client tty and send `.containerResize` over XPC
  (`ClientProcess.resize`). Both the trigger (SIGWINCH) and the numbers
  (`TIOCGWINSZ`) come from the client's controlling terminal. [S]
- There is no `--cols/--rows/--width/--height` flag and no way to inject an
  initial size; `Flags.Process` exposes only `--interactive`/`--tty`. A GUI
  cannot supply size out of band; the CLI insists on querying its own tty. [S]

So a GUI process (piped/null stdio, no controlling terminal) cannot spawn
`container exec -i -t` directly. RunHaven allocates a host PTY, hands the slave
to the `container` child as its stdin/stdout/stderr and controlling terminal
(so `setraw`/`TIOCGWINSZ` succeed on the slave), reads/writes the master, and
on a resize message sets the master winsize (`TIOCSWINSZ`). The kernel delivers
`SIGWINCH` to the child's foreground process group on that winsize change, which
drives the CLI's existing resize handler and the `.containerResize` XPC. [S]

DRY-ladder check: no higher rung covers this. A VT-rendering terminal and a
real PTY genuinely cannot be "a few lines," and the standard library has no
`openpty` + child-controlling-terminal API. The one honest question is whether
to hand-roll `openpty`+`ioctl` (would require `unsafe`, and the workspace sets
`unsafe_code = forbid`) or take a crate. A crate is correct here. [J]

Dependency: `pty-process` 0.5.3 (published 2025-07-12). [S/verified on
crates.io + docs.rs]

- Blocking API (`pty_process::blocking::{Command, Pty}`) matches RunHaven's
  existing thread-per-child runner pattern (`launch_runner.rs`). `Size::new(rows,
  cols)` + `pty.resize(...)` gives the winsize control we need; spawning through
  the pty gives the child a controlling terminal. [S]
- Built on `rustix`; the crate encapsulates its own unsafe, so RunHaven code
  stays `unsafe`-free (compatible with the workspace lint). [S]
- Chosen over `portable-pty` 0.9.0 (wezterm): both work and neither forces
  consumer `unsafe`, but `portable-pty` is a cross-platform crate with a wider
  dependency surface; `pty-process` is leaner for a macOS-only target. [J]

Proposed pins (exact-pinned, transitive locked):

- `pins.toml` `[rust]`: `pty-process = "0.5.3"`.
- `crates/runhaven-core/Cargo.toml` (and the workspace dependency table):
  `pty-process = "0.5.3"`. `runhaven-core` owns the dependency because the repo
  structure guard requires every `container` subprocess to live in core, not in
  the Tauri crate. This supersedes the earlier plan that put the dependency and
  the PTY spawn in `src-tauri` and kept core "PTY-free": the structure-boundary
  rule (`scripts/check-repo-structure.sh` fails on `Command::new("container")`
  under `src-tauri/src`) takes precedence. The host-PTY allocation, the
  `container exec` spawn, and the child lifecycle now live in
  `runhaven-core::runtime::terminal` (`open_gui_attach_pty` -> `GuiPtySession`);
  the Tauri crate keeps only the `Channel` reader thread and the registry.
- `Cargo.lock` records `rustix` and its tree; `runhaven-check-pins` gate applies.

## 2. Frontend terminal

`@xterm/xterm` 6.0.0 + `@xterm/addon-fit` 0.11.0 as the WebView terminal. [J,
versions verified on npm; confirm the 6.0.0/0.11.0 pairing at implementation
time — frontend versions are volatile]

- Justification: xterm.js is the standard, well-tested WebView VT emulator
  (VS Code lineage). Alternatives (hand-rolled VT parser, a plain textarea) fail
  the correctness bar for escape sequences, cursor, and colors, and a plain
  textarea cannot render an agent TUI. [J]
- CSP: no change needed. `@xterm/xterm` ships `css/xterm.css` and injects a
  `<style>` element; the current policy `style-src 'self' 'unsafe-inline'`
  covers both. It uses the system monospace font by default, so no `font-src`
  is required, and all traffic is local IPC, so `connect-src` is unchanged. [C,
  verified: package `style` field is `css/xterm.css`; no default web font]
- Proposed pins: `ui/package.json` dependencies + `pins.toml` `[frontend]`:
  `"@xterm/xterm" = "6.0.0"`, `"@xterm/addon-fit" = "0.11.0"`.
- Scrollback is bounded in WebView memory only (see the see/cannot-see section).

## 3. IPC bridge

Output (host -> web): a Tauri v2 `Channel`, passed as an argument to the
`open_terminal_session` command. Channels are the documented mechanism for
streaming ordered host->frontend data (child-process output is the canonical
example) and preserve order, which a terminal stream requires. [C, verified
against v2.tauri.app "Calling the Frontend"]

Input and resize (web -> host): regular `invoke` commands, not the Channel
(Channels are Rust->frontend only). `invoke` round-trips give natural
backpressure on the write path. [C]

Message shapes:

- Output/exit over the Channel as a tagged enum `TerminalMessage`:
  `{ "event": "output", "data": <bytes> }` and
  `{ "event": "exit", "code": <i32> }`. A terminal stream is human-scale
  (keystrokes plus screen paints, KB not GB), so JSON-encoded bytes are
  acceptable; if profiling ever shows pressure, the byte-optimized
  `Channel<tauri::ipc::Response>` (ArrayBuffer, no base64) is the upgrade path. [J]
- Input web -> host: `write_terminal_input(session_id, bytes)`.
- Resize web -> host: `resize_terminal(session_id, { cols, rows })`, which calls
  `pty.resize(Size::new(rows, cols))`.
- Close web -> host: `close_terminal_session(session_id)`.

Capability placement: the Channel is not itself a grant. It rides the existing
`ipc:`/`connect-src` transport; authorization comes from the
`allow-open-terminal-session` command permission in the new `terminal`
capability (section 7). [C]

Backpressure note: the host reader thread reads a bounded buffer (8 KB, matching
the existing runners) and sends each chunk on the Channel; the WebView bridge
queues in order. This is send-and-forget from Rust, so a wedged WebView cannot
block the PTY reader; xterm's own scrollback cap bounds memory. [J]

## 4. Confirmation model

A raw keystroke stream cannot be confirmed per message, so consent happens once,
at open, and names the exact target before any PTY is created. [J]

- `open_terminal_session` resolves the target run/session in Rust (never trusts
  a WebView-supplied container name), then calls the existing `confirm_native`
  OK/Cancel dialog (`src-tauri/src/commands/dialog.rs`) naming the run and its
  container. No PTY, no `container` child, and no dialog-bypassing boolean: if
  the user cancels, nothing is spawned. [C, matches every existing mutation]
- What the user consents to, in plain words: "You are opening a live shell
  inside sandbox <name>. What you type goes straight to it. What it prints is
  shown here but not filtered and not saved. Passwords or codes you type or that
  print are visible on this screen." The dialog states the no-filter/no-store
  facts, not just "open a terminal." [J]
- After OK the stream is contained: it stays inside this sandbox, as the same
  non-root agent, with no new reach (section 6). The one consent covers the
  whole session.

## 5. Redaction

The live PTY stream is raw. It is deliberately NOT routed through
`redact_run_output` and NOT written to the output registry or any log, matching
the `runs logs -f` precedent (`crates/runhaven-core/src/runtime/active/mod.rs`
warns before the unredacted live stream prints). Full data-flow and the exact
"stored vs shown" line are in the next section, which is a hard requirement.

## What RunHaven can and cannot see in a terminal session

This section is precise on purpose: it must neither overclaim privacy nor
overclaim surveillance. The no-filter/no-store facts below apply ONLY to the
live terminal stream, never to the app as a whole (see the accuracy constraint).

1. Data flow, exactly. Raw PTY bytes travel: guest shell -> `container exec`
   (or `container run`) -> the slave/master of RunHaven's host PTY -> RunHaven's
   Rust host process (a read loop on the master) -> the Tauri Channel -> the
   WebView -> xterm.js. The bytes DO pass through RunHaven's own Rust process
   and through the (untrusted) WebView in transit. RunHaven relays and renders
   the stream; it does not sit outside it.

2. Not redacted (a warning, not a reassurance). Unlike saved output snapshots
   (which pass through `redact_run_output`), the live terminal is raw. Secrets,
   tokens, passwords, and OAuth codes that are typed or printed are visible on
   screen and pass through unfiltered. "Does not filter" means secrets are
   exposed here, not that the terminal is extra-safe. This is a standing on-
   screen warning at the terminal, not buried in a menu.

3. Not persisted. The Rust host reader forwards each PTY chunk straight to the
   Channel and returns to reading; it does NOT append to `SessionOutputRegistry`,
   does NOT write run records, and does NOT write any log or file. The lifecycle
   design (section 9) has no code path from the terminal reader to disk or to
   the output registry, and the capability guard test in section 10 asserts it.
   The only retained copy is xterm.js scrollback: a bounded in-memory ring
   (propose `scrollback: 1000` lines [J]) living only in WebView memory for the
   life of the terminal view. It is discarded when the terminal or window
   closes, never written to disk, and never restored across relaunch (section 9).
   Sharp line: shown in the moment, not stored.

4. Not inspected. RunHaven does not parse, scan, secret-detect, analyze, log, or
   send telemetry on the terminal contents. Exact verbs: it relays and displays;
   it does not read-for-meaning, log, or transmit the stream anywhere else.

5. What the agent sees vs the terminal. The terminal shows the sandbox's real
   shell — the same isolated view the agent has: the project folder plus its own
   `/home/agent` volume, non-root, read-only root filesystem, provider-only
   egress. RunHaven is not hiding output from the user, and the terminal does
   not widen what the sandbox can reach (section 6).

Accuracy constraint (do not widen this claim). Elsewhere RunHaven deliberately
DOES redact saved output (`redact_run_output`), capture bounded output into the
`SessionOutputRegistry`, and log run records plus auth/egress decisions locally
for history, recovery, and Diagnostics. Those are local (never transmitted off
the Mac) and are protective features. The "does not filter, read, store, or log"
statement is scoped strictly to the live terminal stream, never to the app.

On-screen copy (one crisp line, refine against final UI): "This is a live
terminal inside the sandbox. RunHaven does not filter, read, store, or log it.
It just shows what the sandbox prints. So anything here, including passwords or
codes, is visible on this screen." Shown prominently before/at terminal open;
the attach confirmation dialog states the same no-filter/no-store facts.

## 6. Isolation

- Reuse `runs_attach`'s validation for the attach entry: validate the container
  name (`validate_runhaven_container_name`), workdir
  (`validate_attach_workdir`), and command (`validate_attach_command`), and pass
  `--user agent` (non-root). [S, `runtime/active/mod.rs:77-112`]
- `--allow-root-user` is NOT exposed to the GUI; the terminal is always the
  non-root agent user. Fail closed on anything else. [J]
- The exec inherits the run's hardening automatically: read-only rootfs,
  `--cap-drop`, and provider-only network are `container run` create-time
  properties (`crates/runhaven-core/src/runtime/plans/mod.rs:115-197`), and
  `container exec` runs a new process in the same container's mounts and network
  namespace. Only `--user` must be re-specified, which we do. [S]
- No new mounts, no host env passthrough, no `container machine`. The terminal
  adds interactivity, not reach.

## 7. Capability and windows

A dedicated `terminal` capability, `"windows": ["main"]` only. Read-only monitor
windows (`session-*`) MUST NOT receive it — they are read-only by design. [J]

- New capability file `src-tauri/capabilities/terminal.json`, permissions:
  `allow-open-terminal-session`, `allow-write-terminal-input`,
  `allow-resize-terminal`, `allow-close-terminal-session`. Add its identifier to
  `tauri.conf.json` `app.security.capabilities` (the guard requires exact match).
- `capability_guard` additions (`src-tauri/src/capability_guard/`): a pinned
  `TERMINAL_PERMISSIONS` set with a same-set test (mirror `run-control`); assert
  `terminal.json` is `windows: ["main"]`; assert `main-read` and
  `session-monitor` never grant any terminal permission (extend the read-only
  forbidden lists). [C]

## 8. Claude login integration

- Today `start_agent_sign_in` refuses `agent == "claude"`
  (`src-tauri/src/commands/agent_sign_in.rs:72`) because its captured-output
  runner (`Stdio::null()` stdin) cannot host the paste-back. That refusal stays
  for the captured-output path. [S]
- New terminal-backed path: the Agents "Sign in to Claude" button opens a
  terminal session running the in-sandbox `claude auth login` plan. Core already
  builds it: `build_sandbox_login_plan("claude")` returns the `claude auth login`
  command and is `Ok` for claude (`crates/runhaven-core/src/runtime/login.rs`).
  The URL prints in xterm and the user pastes the code back in xterm. [S]
- Mechanism: because login may have no running container, the terminal opens a
  host-PTY-wrapped `container run -i -t ...` of the login plan (same host-PTY
  plumbing as attach, section 1), keeping `--user agent`, read-only root,
  cap-drop, and provider-only egress. `platform.claude.com` and `claude.ai` are
  already bundled provider hosts, so no egress change. [S]
- The `agent == "claude"` refusal is lifted ONLY for the terminal path, and that
  path is still Rust-owned and native-confirmed. [J]
- Fail closed: if the container CLI is missing or the PTY fails to open, the
  terminal does not start and the UI keeps the existing "run `runhaven login
  claude` in your terminal" fallback text. No silent partial state. [J]

## 9. Lifecycle

- A `TerminalSessionRegistry` (mirroring `SessionInputRegistry`) keyed by
  `run_id` for attach, or the login sign-in id, holds the pty master handle and
  the child. `open_terminal_session` inserts; `close_terminal_session` removes.
- Killed on: explicit close command, session stop/kill
  (`stop_active_run`/`kill_active_run` tear down the container, the reader hits
  EOF), terminal view/tab close, and main-window destroy. On close, RunHaven
  kills the `container` child and drops the master, closing the guest process.
- Concurrent bound: cap at a small number (propose 2) [J]; opening past the cap
  focuses/reuses rather than spawning, matching the monitor-window cap pattern.
- Underlying session dies mid-stream: the master read returns EOF, the reader
  sends `{ "event": "exit", code }`, xterm shows a plain "This session ended"
  line, and the registry entry is cleared. No dangling child.
- Not persisted or restored across relaunch (matches session monitor windows).

## 10. Slice plan

Dependency-ordered. Each slice ships its docs in the same pass.

1. Rust host-PTY seam + registry + commands + capability. [L] Add `pty-process`
   pin; the core `open_gui_attach_pty` -> `GuiPtySession` spawn in
   `runhaven-core::runtime::terminal` (host PTY, `container exec` spawn, child
   kill/reap); `TerminalSessionRegistry`; the four commands; `terminal.json`;
   `capability_guard` pins (`TERMINAL_PERMISSIONS`, main-only, monitor/main-read
   exclusion). Changes: `tauri.conf.json` capability list, capability_guard
   tests, `pins.toml`, `crates/runhaven-core/Cargo.toml` (owns `pty-process`).
   Adversarial review gate A: new interactive trust boundary before any frontend
   wiring.
2. Frontend xterm + Channel bridge + fit/resize + the on-screen disclosure line
   and the attach confirmation copy. [M] Prominent disclosure rendered
   before/at terminal open (this is its own review checklist item).
3. Claude login terminal entry. [M] New terminal-backed command that lifts the
   claude refusal for the terminal path only, native-confirmed, fail-closed
   fallback preserved; docs in `docs/DESKTOP_APP.md` + `docs/SECURITY_MODEL.md`.
   Adversarial review gate B: credential-adjacent path.
4. Tests + verification. [M] capability_guard pins; a no-persist test asserting
   the terminal reader never writes the output registry, records, or logs; a
   no-leak test on any structured terminal response; lifecycle teardown
   (close/stop/window-destroy) tests. `tauri dev` live smoke on a macOS 26 host
   is required (openpty child vs. pipe-stdin child, plus SIGWINCH resize end to
   end); Playwright cannot drive Tauri windows.

Final adversarial review gate C: whole-terminal boundary review before commit.

## Open questions and verification gaps

- The container source was read at checkout HEAD `9bd3c473...`, which differs
  from `pins.toml`'s recorded `apple_container.commit = ee848e3e...`. Reconcile
  before implementing: confirm the ENOTTY-before-createProcess ordering still
  holds at the pinned commit, or update the pin. [S/gap]
- Live smoke not yet run: confirm on a real macOS 26 / container 1.0.0 host that
  an openpty-wrapped child succeeds where pipe-stdin fails, and that a master
  `TIOCSWINSZ` reaches the guest (`.containerResize`). [gap]
- Confirm the exact `@xterm/xterm` 6.0.0 / `@xterm/addon-fit` 0.11.0 pairing and
  that no `font-src`/`connect-src` change is needed at implementation time.
