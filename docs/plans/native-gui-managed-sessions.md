# Native GUI Managed Sessions

Last updated: 2026-06-30 UTC

## Goal

Give the desktop GUI a typed, observable session model before embedding live
terminal I/O. A user should be able to see which agent sessions are running,
which agent owns each one, whether auth is ready or blocked, and which session
is selected without reading raw terminal output.

This plan does not authorize arbitrary shell execution, a generic PTY bridge,
host filesystem access, MCP execution, app-server transport, or frontend-owned
process spawning.

## Boundary

Trusted:

- `runhaven-core` plans, active-run records, auth status, diagnostics, and run
  control.
- Tauri Rust commands that validate typed requests and return bounded typed
  responses.

Untrusted:

- WebView input.
- Session IDs, resize dimensions, and input bytes sent from the frontend.
- Raw terminal output. It can contain secrets or workspace file contents.

Rules:

- The frontend can observe sessions through typed read-only commands.
- The frontend can send input only to an existing backend-owned session ID.
- The frontend cannot request arbitrary commands, shells, mounts, environment
  variables, network modes, provider hosts, or workspace paths.
- Rust/Tauri owns process creation, launch review, auth setup, run records, and
  run control.
- Raw output remains warning-gated and bounded.

## Read Model

The first contract is `ManagedSessionListData` in `runhaven-core`.

It merges:

- active RunHaven run records
- secret-free auth status by agent

It returns display-safe summaries:

- session ID and run ID
- agent
- agent state session name
- project label, not the raw workspace path
- status and status label
- auth status
- network mode
- start time
- container name
- detail command

It intentionally omits:

- raw workspace paths
- state volume names
- host usernames and home paths
- credentials, tokens, environment values, browser profile paths, and raw
  command arguments

## Tauri Surface

Current read-only command:

```text
get_managed_sessions() -> ManagedSessionListData
```

Capability:

```text
main-read: allow-get-managed-sessions
```

This is read-only. It does not start, stop, attach to, or write to a process.

## Future Session Control Surface

Add these only after the read model and security tests are stable:

```text
start_session(reviewed_plan_id)
get_session(session_id)
write_session_input(session_id, bytes)
resize_session(session_id, cols, rows)
stop_session(session_id)
```

Guard requirements before any write-capable command:

- Unknown session IDs fail closed.
- Session IDs must refer to backend-owned RunHaven sessions.
- Frontend input is bounded.
- Resize dimensions are bounded.
- Process creation accepts only reviewed RunHaven launch plans.
- No command string, host path, mount, env, provider host, or network setting
  is accepted from the WebView on the write path.
- Multiple sessions cannot overwrite one another's records, logs, or controls.

## GUI Shape

Default layout remains two-pane:

- left sidebar for sections and active sessions
- main pane for the selected workflow or selected session

The sidebar may show an `Active sessions` group only when sessions exist. It
should list compact rows such as `Codex - docs` or `Claude - backend` with a
short state badge.

The selected session owns the main pane. Details such as mounts, network
policy, auth decisions, approvals, diagnostics, and raw output open as the
active detail view, sheet, or temporary inspector. They are not a permanent
right pane.

## Selected Session Surface

Current GUI behavior:

- Active sessions appear in the sidebar only when the read model returns at
  least one session.
- Selecting a session opens a read-only main-pane summary.
- The summary shows agent, project label, session name, status, auth posture,
  network posture, and start time.
- Auth states that are not brokered, ready, available, or not applicable are
  shown as needing attention.
- The summary does not expose raw workspace paths, state volume names, raw
  command arguments, terminal input, resize, attach, filesystem RPC, or process
  control.

## Verification

Current slice:

- core contract tests prove active runs merge with auth status and omit raw
  paths/secrets
- Tauri command test proves `get_managed_sessions` is read-only and secret-free
- capability guard proves the command is under an `allow-*` scope only
- frontend helper test proves the browser fallback keeps the same shape
- frontend selected-session card test proves the rendered summary stays
  read-only and does not include host paths or write-session command names

Future embedded-session slices need focused Rust tests for session ownership,
unknown ID rejection, bounded input, bounded resize, raw output gating, and
multi-session isolation before adding UI controls.
