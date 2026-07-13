# Native GUI Session Input Contract

Last updated: 2026-07-13 UTC

## Status

A narrow `write_session_input` Tauri command is exposed under the `run-control`
capability. The frontend composer calls only that command for the selected
session.

This document is the design gate before RunHaven exposes any GUI session input
or resize command. The current desktop app can observe sessions, load bounded
raw output after acknowledgement, keep selected-session output updated through
bounded polling, request fixed run-control actions, and write bounded text to a
backend-owned session only through the fixed input command.

The backend now has a Tauri-managed `SessionInputRegistry` for future input.
The registry can accept a Rust-owned writer for an existing session and rejects
writes when no backend transport is registered or when the registered run no
longer matches the active session. Tauri-launched runs now register a
run-id-scoped writer while the child process is alive and unregister it when the
child exits.

Tauri-launched runs also capture child stdout/stderr into a bounded
process-local `SessionOutputRegistry`. The selected-session output command still
tries the existing `container logs` path first; it falls back to the desktop
capture only when a Tauri-owned live session has registered output. Raw output
remains confirmation-gated.

RunHaven core now also has an `AgentCommandRunner` seam for standard and
provider launches. The normal CLI path still uses `Command::status`, while the
Tauri launch path requests `container run --interactive` without `--tty`,
retains a backend-owned child stdin handle, and registers it with
`SessionInputRegistry`.

Contract checklist: active backend-owned session ID, native approval for every
exact message, bounded input and confirmation copy, cancellation, stale target,
multi-session, sign-in state, and app-wide single flight. Resize remains out of scope.

## Goal

Let a future desktop user send text to an existing RunHaven-owned agent session
without turning the WebView into a terminal, shell, filesystem bridge, or generic
command runner.

The secure path must stay the easy path:

- the user chooses an existing session;
- the UI shows the selected agent and project label;
- the user types a bounded message and explicitly sends it;
- Rust resolves the backend-owned session ID at the point of use;
- unsupported, stale, blocked, or unsafe targets fail closed.

## Trust Boundary

Trusted:

- `runhaven-core` launch plans, active-run records, auth status, and runtime
  ownership checks.
- Tauri Rust commands that validate typed requests and perform fixed
  operations.

Untrusted:

- WebView state.
- Session IDs sent from the frontend.
- User input text.
- Focus, selection, and target state in the frontend.

Sensitive:

- Raw agent output can contain workspace content or secrets.
- User input can contain prompts, secrets, file contents, commands, or data that
  causes the agent to act.

## Non-Goals

- No generic terminal attach.
- No arbitrary command execution.
- No filesystem RPC.
- No MCP execution.
- No app-server transport.
- No Codex exec-server transport.
- No login or host-reaching Codex path.
- No host path, mount, environment variable, provider host, or network setting
  accepted from the WebView.
- `resize_session` is not part of the first input slice. resize remains out of
  scope until the input contract is proven and a separate resize design exists.

## Command Shape

The write-capable command is narrow and typed:

```text
write_session_input(request) -> WriteSessionInputResponse
```

Request fields:

```text
session_id: string
input: string
```

Response fields:

```text
session: ManagedSessionSummaryData
accepted: bool
accepted_bytes: usize
cursor: string
status: string
```

Unknown request fields are rejected. The WebView cannot send an approval flag
or a trusted target label. Rust derives the target from current active-session
state.

## Required Guards

Ownership:

- Unknown or history-only session IDs fail closed.
- The session ID must resolve from an active RunHaven marker, never from a past
  run record.
- The resolved run ID and Rust-owned writer generation must match before and
  after native approval.

Auth:

- Sessions with backend-owned auth attention fail closed.
- A write is allowed only when the session's next action says the user can
  monitor or continue the session.
- Auth checks happen in Rust at write time, not only in the frontend.

Explicit user intent:

- Rust opens a native macOS confirmation sheet for every exact message. An
  earlier approval cannot be reused.
- The sheet shows a control-safe backend target of at most 120 Unicode
  characters, the exact byte count, and a head-and-tail preview of at most 240
  Unicode characters and six lines.
- Only one message approval can be in flight across the app. A second attempt
  fails before another sheet or write can start.
- The UI must use a real button or keyboard submit action.
- The app must not auto-forward clipboard contents, drafts, or background text.
- Drafts are component state only and are cleared when the selected target
  changes.

Bounded input:

- Input must be UTF-8 text.
- Empty input is rejected.
- Maximum size for the first slice is 8 KiB per send.
- All controls except ordinary line feeds are rejected. Bidirectional controls,
  invisible separators, terminal escapes, and other unsafe formatting are also
  rejected instead of being altered.
- Multi-line input is allowed only if the UI presents it as a message composer,
  not as an unbounded terminal paste.

Cancellation:

- Canceling the native sheet writes zero bytes.
- A backend write that has already been accepted is not promised to be
  reversible.
- The UI must show clear pending, accepted, and failed states.
- The Rust-owned app-wide single-flight guard is released on every exit path.

Stale target:

- Rust derives the target from the active marker and never trusts a renderer
  target label.
- After native approval, Rust re-resolves the active marker, run ID, sign-in
  state, normal running state, and exact writer generation before writing.
- A removed marker, stopped run, replaced writer, or changed sign-in state
  writes zero bytes.
- The UI ignores late responses if the selected target changed.
- Tests must prove stale responses cannot update the wrong selected session.

Multi-session:

- Each write resolves through its own session ID.
- Two sessions cannot share draft state, pending state, accepted cursors, or
  output buffers.
- Tests must cover at least two active preview sessions and prove input for one
  session does not appear in the other session's UI state.

Transport evidence:

- Installed Apple `container` 1.0.0 help says `container run --interactive`
  keeps standard input open and `--tty` opens a TTY.
- `apple/container@9bd3c473451a40a1ee28f02104826d224636948d` documents that
  `container run` closes stdin unless `--interactive` is set.
- Live smoke found `--tty` with piped stdin fails under Tauri-owned launch with
  `NSPOSIXErrorDomain Code=25`, so the current GUI message composer uses
  `--interactive` only. A future embedded terminal needs a real PTY design
  before `--tty` can be reintroduced for that path.
- The same smoke found `container logs` is not reliable enough for the
  Tauri-owned interactive child process, so desktop-owned stdout/stderr capture
  backs selected-session output for that launch path.
- The same source shows `container start` does not support attaching to already
  running containers, and `container exec` creates a new process. Neither is a
  safe shortcut for sending input to the original agent process.
- For the GUI, the correct path is to retain the original launch process handle
  and register a backend-owned writer. If a later TTY session needs a PTY,
  evaluate the smallest fitting existing utility before writing custom code.

## Required Tests Before Frontend Composer

Current coverage:

- `WriteSessionInputRequest`, `WriteSessionInputResponse`, and
  `write_session_input` are narrow typed Tauri contracts.
- The test seam resolves the backend-owned session ID before a write can be
  accepted.
- It rejects unknown or history-only sessions, oversized session IDs, empty
  input, input above 8 KiB, unsafe controls or invisible formatting, and
  auth-attention sessions before any write callback runs.
- It accepts only a ready managed session whose backend-owned next action is
  `monitor`.
- The test seam now routes accepted input through `SessionInputRegistry`, not a
  one-off callback. Missing transports fail closed, stale run/session mappings
  fail closed, and two sessions cannot share a writer.
- It exposes only the fixed `write_session_input` command under `run-control`.
- The React composer is visible only for selected sessions whose backend-owned
  next action is `monitor`.
- Drafts clear when the selected target changes, and late responses are ignored
  if the selected session changed while the request was in flight.

Rust/Tauri:

- unknown or history-only session ID is rejected before any write attempt;
- oversized session ID is rejected before any backend lookup;
- renderer approval and target fields are rejected as unknown fields;
- empty input is rejected;
- input above 8 KiB is rejected;
- NUL bytes, escape sequences, bidirectional controls, and invisible separators
  are rejected without echoing the input;
- auth-attention sessions are rejected;
- every exact message requires a new native approval;
- cancellation writes zero bytes;
- a removed active marker, changed running state, or replaced writer after
  approval writes zero bytes;
- a second concurrent approval is rejected by one app-wide RAII guard;
- writer errors are replaced with fixed text that contains no message or
  backend detail;
- resolved run ID and session ID stay in the typed response;
- a registered Rust-owned writer is required before bytes can be accepted;
- a writer registered for an older run ID is rejected before input is sent;
- multi-session tests prove one session's input is not delivered to another
  session's writer;
- forbidden strings such as `attach_session`, `resize_session`, filesystem RPC,
  generic command execution, MCP, login, and app-server transport do not appear
  in the response or capability grants.

Frontend:

- composer is visible only for a selected session that can accept input. Done.
- send is disabled for empty input, blocked auth, and in-flight send. Done.
- changing session clears the draft and pending state. Done.
- stale responses are ignored. Done.
- two preview sessions keep drafts and accepted responses isolated. Done.
- no raw workspace path appears in the selected-session input UI. Done.

Live smoke:

- the local smoke proves one internal `shell` session can receive a bounded line
  of text and show the echoed output through the selected-session output path.
  Done.
- the smoke proves stop still works and cleanup still runs. Done.

## First Implementation Sequence

1. Add Rust contract tests around a test-only write seam. Done.
2. Add request and response structs without granting a capability. Done.
3. Add a backend-owned input registry without granting a capability. Done.
4. Add a core launch runner seam before changing GUI launch ownership. Done for
   standard and provider launches.
5. Decide the real launch transport from RunHaven's GUI needs first. Done:
   Tauri launch uses a backend-owned child stdin handle and does not import
   Codex app-server, exec-server, MCP, login, or filesystem surfaces for GUI
   input. Reuse a repo-vendored utility such as `codex-utils-pty` only if a
   later terminal-quality issue proves the standard process handle is not
   enough.
6. Add the fixed Tauri command only after the fail-closed tests exist. Done.
7. Grant only `allow-write-session-input` under `run-control`, not `main-read`.
   Done.
8. Add a minimal selected-session composer that calls only the typed command.
   Done.
9. Add preview and Playwright tests for two selected sessions. Done.
10. Add a live smoke after the Rust path can write to a real backend-owned
   session. Done.

The first implementation must not add `resize_session`. Resize remains out of
scope until there is a separate design with bounds, terminal-size semantics,
and session ownership tests.
