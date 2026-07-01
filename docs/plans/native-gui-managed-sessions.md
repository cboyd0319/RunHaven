# Native GUI Managed Sessions

Last updated: 2026-07-01 UTC

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
- The frontend can send bounded text input only through the fixed
  `write_session_input` command to an existing backend-owned selected session
  ID after the design gate in `docs/plans/native-gui-session-input-contract.md`
  is satisfied.
- Backend input transport is RunHaven-owned. Repo-vendored utility code can be
  reused only when it fits this contract; Codex app-server, exec-server, MCP,
  login, and filesystem transports are not GUI input shortcuts.
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
- lifecycle state
- one backend-owned next action
- safe event markers for run state, auth, and network posture

It intentionally omits:

- raw workspace paths
- state volume names
- host usernames and home paths
- credentials, tokens, environment values, browser profile paths, and raw
  command arguments

## Tauri Surface

Current session commands:

```text
get_managed_sessions() -> ManagedSessionListData
get_managed_session_status(request) -> ManagedSessionStatusResponse
get_managed_session_diagnostics(request) -> ManagedSessionDiagnosticsResponse
get_managed_session_log_snapshot(request) -> ManagedSessionLogSnapshotResponse
write_session_input(request) -> WriteSessionInputResponse
stop_managed_session(request) -> StopManagedSessionResponse
kill_managed_session(request) -> KillManagedSessionResponse
repair_managed_session(request) -> RepairManagedSessionResponse
```

Capabilities:

```text
main-read: allow-get-managed-sessions
main-read: allow-get-managed-session-status
main-read: allow-get-managed-session-diagnostics
run-control: allow-get-managed-session-log-snapshot
run-control: allow-write-session-input
run-control: allow-stop-managed-session
run-control: allow-kill-managed-session
run-control: allow-repair-managed-session
```

The managed session list and selected-session status commands are read-only and
stay under `main-read`. Raw output, input, and run-control commands stay outside
`main-read` because bounded container stdio can contain secrets and input,
stop, kill, and repair mutate runtime state.

## Launch Access Preflight

Desktop launch planning returns a typed `access` summary with the reviewed
login state before launch:

- `Shared agent login` reuses the per-agent login created by `runhaven login`
  across projects on this Mac.
- `Project session login` keeps login state separate for this project session.
- `API key variable found` means the fixed per-agent host environment variable
  is present for this app process.
- `API key variable missing` blocks launch until that fixed variable is set or
  the user removes the API key variable and signs in inside the session.

The frontend renders this summary in the review step and disables launch when
`access.blocking` is true. The response may include the environment variable
name, but never includes the value. The backend repeats the API-key environment
check before spawning the background launch thread, so the GUI cannot report an
accepted launch when broker credentials are missing. Arbitrary host env names
are rejected before launch.

The GUI says `Run accepted` after the typed launch command returns. That means
RunHaven accepted the reviewed plan and started backend launch ownership; it is
not a claim that the container has already reached a running state.

`get_managed_session_status` accepts only a managed session ID. Rust resolves
that session to the active RunHaven run, checks whether input is attached in the
backend registry, and returns the managed session summary plus sanitized
container status. The response omits raw workspace paths, state volume names,
mounts, environment values, raw command arguments, and backend error text.
When live container status is unavailable, the response also returns typed
recovery guidance with a title, detail, remedy, and whether marker repair is a
proven available next action. Generic status failures use non-mutating guidance
such as refresh or diagnostics first. This keeps recovery judgment in Rust and
gives the WebView plain next-step text without exposing backend errors.

`get_managed_session_diagnostics` accepts only a managed session ID. Rust
resolves it to the backend-owned run, then returns bounded auth broker and
provider network records filtered to that run. The frontend cannot provide a
raw run ID, path, command, mount, environment value, provider host, or network
setting on this path.

The managed run-control commands accept only a backend-owned session ID plus
the explicit confirmation flag for that action. Rust resolves the session ID to
the active RunHaven run, then delegates to the existing validated
`stop_run`/`kill_run`/`repair_run` path. The WebView cannot provide a raw run
ID, command string, host path, mount, environment variable, provider host, or
network setting on this path.

## Future Embedded Session Surface

Input is limited to `write_session_input`. No `resize_session` or session attach
command is exposed by the current desktop app.

Read-only session commands may expand first:

```text
start_session(reviewed_plan_id)
get_session(session_id)
```

Before expanding any input-capable command, use
`docs/plans/native-gui-session-input-contract.md` as the required contract.
Current guard requirements include:

- Unknown session IDs fail closed.
- Session IDs must refer to backend-owned RunHaven sessions.
- Frontend input is bounded and explicit.
- Auth attention and stale frontend targets fail closed.
- Process creation accepts only reviewed RunHaven launch plans.
- No command string, host path, mount, env, provider host, or network setting
  is accepted from the WebView on the write path.
- Multiple sessions cannot overwrite one another's records, logs, or controls.
- Resize stays out of scope until there is a separate design and test gate.

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
- Active and recent managed sessions refresh quietly in the background after a
  launch or selection, updating only when the typed session data changes.
- Selecting a session opens a read-only main-pane summary.
- The summary shows agent, project label, session name, status, auth posture,
  network posture, and start time.
- The summary renders backend-owned lifecycle, next-action, and event markers
  instead of inferring those states in the WebView.
- The selected-session runtime card refreshes through
  `get_managed_session_status`, suppresses no-op updates, and shows sanitized
  container state, image, CPU, memory, and network summary when available.
- Runtime status failures are bounded and generic. They do not expose raw
  backend errors, raw launch arguments, host paths, state volumes, mounts, or
  environment values.
- Runtime status failures include typed recovery guidance. Generic failures use
  refresh or diagnostics first; marker repair is shown only when Rust has proven
  it is an available next action. Historical sessions explain that live runtime
  status is no longer available.
- Selected sessions can load session-scoped diagnostics on demand. The response
  is filtered by the Rust-resolved run ID and shows only bounded auth/network
  records for that session.
- Auth states that need attention use a backend-owned next action such as
  `auth_attention`.
- The summary does not expose raw workspace paths, state volume names, raw
  command arguments, terminal input, resize, attach, filesystem RPC, or process
  control.
- Browser preview records launched preview runs as safe managed-session
  summaries only for local visual QA. Real desktop data still comes from the
  typed `get_managed_sessions` Tauri command.
- Selected sessions can load the existing bounded output snapshot through
  `get_managed_session_log_snapshot` after explicit raw-output confirmation.
  Rust resolves the managed session ID to the active run before delegating to
  the existing log snapshot path. The response adds session-owned log entries
  with source, cursor, timestamp, bounded text, line count, and explicit
  `redaction.status: notApplied`. The card prefers those session entries over
  run-level fallback fields.
- Selected sessions can opt into live output after raw-output confirmation.
  The live path is bounded polling over `get_managed_session_log_snapshot`, not
  Tauri events, `container logs --follow`, terminal attach, input, or resize.
  The React app keeps one in-flight log request, stops polling when the target
  changes or acknowledgement is cleared, and ignores stale responses.
- `LogSnapshotResponse` includes a cursor and typed snapshot entries while
  keeping the previous top-level text fields for compatibility. This is not a
  streaming terminal or session-attach contract.
- Selected-session output uses the backend-owned session ID. It does not add
  streaming transport, terminal input, resize, attach, filesystem RPC, or
  process control. Raw-output permissions stay outside `main-read`.
- Selected sessions can request stop, hard stop, and stale-marker repair
  through fixed typed run-control commands after explicit per-action
  confirmation. The frontend sends only the managed session ID; Rust resolves
  the run ID and reuses the existing validated run-control path.
- Selected-session controls do not add terminal input, resize, attach,
  filesystem RPC, generic command execution, app-server transport, MCP
  execution, login, or host-reaching Codex paths.
- A Tauri-managed `SessionInputRegistry` backs `write_session_input`. It
  rejects missing transports, stale run/session mappings, and cross-session
  writer reuse in Rust tests.
- The selected-session Message composer is visible only when the backend-owned
  next action is `monitor`. It sends only the selected managed session ID,
  explicit confirmation, stale-target ID, and bounded input bytes through
  `write_session_input`.

## Verification

Current slice:

- core contract tests prove active runs merge with auth status and omit raw
  paths/secrets
- core contract tests prove lifecycle, next action, and event markers are
  derived from secret-free fields
- Tauri command test proves `get_managed_sessions` is read-only and secret-free
- Tauri command test proves the endpoint does not expose write-session or
  attach affordances
- capability guard proves the command is under an `allow-*` scope only
- Tauri command tests prove `get_managed_session_status` resolves the session
  ID in Rust, returns only sanitized container status, marks input attachment
  from the backend registry, and maps status lookup failure to a bounded generic
  message
- frontend selected-session status tests prove the runtime card renders typed
  status and unavailable states without raw paths or bridge command names
- frontend helper test proves the browser fallback keeps the same shape
- frontend selected-session card test proves the rendered summary stays
  read-only, renders the typed next action and event markers, and does not
  include host paths or write-session command names
- Playwright preview smoke proves a launched preview run can open the
  `Active sessions` sidebar item and selected-session pane without raw preview
  paths
- Playwright preview smoke proves selected-session output remains
  confirmation-gated before loading bounded raw output
- frontend output-card test proves source, captured cursor, line count,
  truncation state, warnings, and bounded text render from the typed snapshot
  entry
- Tauri command test proves the log snapshot response maps the bounded payload
  into a cursor plus one typed entry without changing the gated backend lookup
- Tauri command test proves selected-session output resolves the session ID in
  Rust, rejects unknown sessions, returns session-owned log entries, marks
  redaction as not applied, and stays free of write-session or attach
  affordances
- capability guard proves raw-output permissions stay outside `main-read`
- frontend helper test proves browser preview output loads through a managed
  session ID and rejects unknown sessions
- frontend helper test proves browser preview live-output refreshes produce
  per-session cursors and do not mix selected-session output
- frontend output-card test proves live output is disabled until raw-output
  acknowledgement and reports when automatic updates are on
- Playwright preview smoke proves selected-session live output is
  acknowledgement-gated and receives a refreshed bounded output snapshot
- Tauri command test proves selected-session stop, hard stop, and repair
  resolve the managed session ID in Rust, reject unknown sessions, require
  confirmation before mutation, and keep forbidden paths and command names out
  of the serialized response
- frontend helper test proves browser preview run control works through a
  managed session ID and rejects unknown sessions
- Playwright preview smoke proves selected-session stop, hard stop, and repair
  are reachable only after explicit per-action confirmation
- capability guard proves managed session run-control commands stay in the
  narrow `run-control` capability
- `scripts/gui_tauri_live_smoke.sh` proves the no-bundle Tauri shell builds and
  the selected-session bounded-output plus confirmed-stop command path works
  against one live Apple `container` run. When exercised with a real-agent
  profile, it uses the explicit broker env for that profile. This is an opt-in
  local smoke, not a WebView click automation harness or a TTY/OAuth parity
  check.
- capability guard proves no resize, attach, app-server transport, filesystem
  RPC, MCP, generic command execution, login, or host-reaching bridge is
  exposed while `write_session_input` stays in the narrow `run-control`
  capability
- Tauri contract tests prove the test-only session input seam rejects unknown
  sessions, oversized session IDs, missing confirmation, empty input, input
  above 8 KiB, NUL bytes, terminal escape sequences, stale frontend targets,
  and auth-attention sessions before a write callback runs. A ready monitored
  session can produce a typed accepted response without leaking workspace paths
  or state volume names.
- Tauri contract tests prove the input seam now requires a registered
  `SessionInputRegistry` transport. Missing transports fail closed, transports
  registered for older run IDs fail closed, and two active sessions do not share
  writers.

Future embedded-session slices need focused Rust tests for unknown ID
rejection, bounded input, auth attention, stale targets, raw output gating, and
multi-session isolation before adding terminal input or live session UI.
