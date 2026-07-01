# Native GUI Session Input Contract

Last updated: 2026-06-30 UTC

## Status

No input command is exposed in this slice.

This document is the design gate before RunHaven exposes any GUI session input
or resize command. The current desktop app can observe sessions, load bounded
raw output after acknowledgement, keep selected-session output updated through
bounded polling, and request fixed run-control actions. It still cannot write
bytes to an agent session from the WebView.

The backend now has a Tauri-managed `SessionInputRegistry` for future input.
The registry can accept a Rust-owned writer for an existing session and rejects
writes when no backend transport is registered or when the registered run no
longer matches the active session. No production launch path registers a writer
yet.

RunHaven core now also has an `AgentCommandRunner` seam for standard
non-provider launches. The normal CLI path still uses `Command::status`, while
a future GUI runner can retain a backend-owned child stdin handle and register
it with `SessionInputRegistry`.

Contract checklist: backend-owned session ID, explicit user intent, bounded input,
cancellation, stale target, multi-session, auth, and resize remains out of scope.

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

## Future Request Shape

The first write-capable command should be narrow and typed:

```text
write_session_input(request) -> WriteSessionInputResponse
```

Request fields:

```text
session_id: string
input: string
confirm_send: bool
client_target: string
```

Response fields:

```text
session: ManagedSessionSummaryData
accepted: bool
accepted_bytes: usize
cursor: string
status: string
```

The `client_target` is a stale target guard, for example
`session:<session_id>`. Rust still resolves and authorizes `session_id`; the
target guard only prevents confused frontend state from accidentally sending to
an older selected session.

## Required Guards

Ownership:

- Unknown session IDs fail closed.
- The session ID must resolve to a backend-owned active RunHaven session.
- The resolved run ID must match the active-run marker at the time of write.
- The resolved container must still be a RunHaven-owned container.

Auth:

- Sessions with backend-owned auth attention fail closed.
- A write is allowed only when the session's next action says the user can
  monitor or continue the session.
- Auth checks happen in Rust at write time, not only in the frontend.

Explicit user intent:

- `confirm_send` is required.
- The UI must use a real button or keyboard submit action.
- The app must not auto-forward clipboard contents, drafts, or background text.
- Drafts are component state only and are cleared when the selected target
  changes.

Bounded input:

- Input must be UTF-8 text.
- Empty input is rejected.
- Maximum size for the first slice is 8 KiB per send.
- NUL bytes and terminal escape sequences are rejected.
- Multi-line input is allowed only if the UI presents it as a message composer,
  not as an unbounded terminal paste.

Cancellation:

- A frontend request can be abandoned when the user changes target.
- A backend write that has already been accepted is not promised to be
  reversible.
- The UI must show clear pending, accepted, and failed states.
- In-flight writes are per selected session; changing sessions cannot reuse the
  old pending state.

Stale target:

- The UI sends a `client_target` derived from the selected session.
- Rust rejects mismatched target/session pairs.
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
- The same source shows `container start` does not support attaching to already
  running containers, and `container exec` creates a new process. Neither is a
  safe shortcut for sending input to the original agent process.
- For the GUI, the correct path is to retain the original launch process handle
  and register a backend-owned writer. If a later TTY session needs a PTY,
  evaluate the smallest fitting existing utility before writing custom code.

## Required Tests Before Exposing A Command

Current preflight coverage:

- `WriteSessionInputRequest`, `WriteSessionInputResponse`, and
  `write_session_input_for_test` exist only for Rust tests.
- The test seam resolves the backend-owned session ID before a write can be
  accepted.
- It rejects unknown sessions, oversized session IDs, missing confirmation,
  empty input, input above 8 KiB, NUL bytes, terminal escape sequences, stale
  frontend targets, and auth-attention sessions before any write callback runs.
- It accepts only a ready managed session whose backend-owned next action is
  `monitor`.
- The test seam now routes accepted input through `SessionInputRegistry`, not a
  one-off callback. Missing transports fail closed, stale run/session mappings
  fail closed, and two sessions cannot share a writer.
- It still exposes no Tauri command, no capability grant, and no frontend input
  path.

Rust/Tauri:

- unknown session ID is rejected before any write attempt;
- oversized session ID is rejected before any backend lookup;
- `confirm_send: false` is rejected;
- empty input is rejected;
- input above 8 KiB is rejected;
- NUL bytes and escape sequences are rejected;
- auth-attention sessions are rejected;
- stale `client_target` is rejected;
- resolved run ID and session ID stay in the typed response;
- a registered Rust-owned writer is required before bytes can be accepted;
- a writer registered for an older run ID is rejected before input is sent;
- multi-session tests prove one session's input is not delivered to another
  session's writer;
- forbidden strings such as `attach_session`, `resize_session`, filesystem RPC,
  generic command execution, MCP, login, and app-server transport do not appear
  in the response or capability grants.

Frontend:

- composer is visible only for a selected session that can accept input;
- send is disabled for empty input, blocked auth, and in-flight send;
- changing session clears the draft and pending state;
- stale responses are ignored;
- two preview sessions keep drafts and accepted responses isolated;
- no raw workspace path appears in the selected-session input UI.

Live smoke:

- after a real session input backend exists, a local smoke must prove one
  internal `shell` session can receive a bounded line of text and show the
  echoed output through the existing bounded output path;
- the smoke must also prove stop still works and cleanup still runs.

## First Implementation Sequence

1. Add Rust contract tests around a test-only write seam. Done.
2. Add request and response structs without granting a capability. Done.
3. Add a backend-owned input registry without granting a capability. Done.
4. Add a core launch runner seam before changing GUI launch ownership. In
   progress for standard non-provider launches; provider launches need the same
   treatment before agent input is exposed.
5. Decide the real launch transport from RunHaven's GUI needs first. Reuse a
   repo-vendored utility such as `codex-utils-pty` only if it fits the retained
   process-handle model. Do not import Codex app-server, exec-server, MCP,
   login, or filesystem surfaces for GUI input.
6. Add the fixed Tauri command only after the fail-closed tests exist and the
   launch path registers a Rust-owned writer for the session.
7. Grant only `allow-write-session-input` under `run-control`, not `main-read`.
8. Add a minimal selected-session composer that calls only the typed command.
9. Add preview and Playwright tests for two selected sessions.
10. Add a live smoke only after the Rust path can write to a real backend-owned
   session.

The first implementation must not add `resize_session`. Resize remains out of
scope until there is a separate design with bounds, terminal-size semantics,
and session ownership tests.
