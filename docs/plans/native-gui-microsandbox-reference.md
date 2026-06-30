# Native GUI Microsandbox Reference Pass

Last updated: 2026-06-30 UTC

## Source

Reviewed `superradcompany/microsandbox` from
https://github.com/superradcompany/microsandbox at commit `744d7f79d815`.

This is idea mining only. Do not import code, runtime choices, networking
claims, or documentation wording without a separate license, security, and
architecture review.

## Why It Matters

microsandbox is not a GUI reference. Its useful value for RunHaven is the
runtime product model around long-lived sandbox sessions, logs, metrics,
security boundaries, and typed SDK errors.

RunHaven should keep Apple `container` as the current runtime boundary and keep
the Tauri WebView behind typed RunHaven commands. The ideas below are product
and contract patterns to consider while building the native-feeling GUI.

## Good Ideas To Borrow

### Session Lifecycle

microsandbox treats lifecycle states as first-class product concepts:
creating, running, draining, stopped, and crashed.

RunHaven should expose similarly clear session states in the GUI instead of
only showing raw process output. Useful RunHaven states include starting,
running, stopping, stopped, failed, blocked, stale marker, and needs repair.

The important idea is `draining` or `stopping`: after a user asks a session to
stop, the UI should show that shutdown is in progress and should reject new
session input until the backend settles.

### Logs With Sources And Cursors

microsandbox separates user output from system diagnostics. It also supports
tailing, filtering by source, and resuming from an opaque cursor.

RunHaven should model GUI session output as typed log entries before embedding
a full terminal stream:

- `output` for PTY-style agent output
- `system` for RunHaven lifecycle markers and backend diagnostics
- `auth` for auth broker events
- `egress` for provider-network decisions

Future GUI log reads should be bounded, source-filtered, and cursor-based. Raw
terminal output should remain a warned detail view because it can contain file
contents or secrets.

### Per-Session Identity

microsandbox log entries carry a per-session ID so output from multiple
sessions can be grouped and colored.

RunHaven should give every managed GUI session a stable backend-owned session
ID and keep that ID on logs, metrics, run-control actions, and diagnostics.
This matters before supporting several Codex sessions plus a Claude session in
one app window.

### Typed Failure Categories

microsandbox distinguishes:

- a program that ran and exited nonzero
- a program that failed to spawn
- a sandbox that failed to boot
- a name conflict
- a timeout

RunHaven should mirror that clarity in the GUI. A failed agent launch should
not just show "failed." It should classify the failure into setup missing,
image missing, auth blocked, container start failed, agent command failed,
network blocked, stopped by user, or stale marker.

Each category should map to one next action.

### Metrics As Secondary Detail

microsandbox supports point-in-time and streaming CPU, memory, disk, and
network metrics, with nullable fields when a runtime cannot provide a value.

RunHaven should not put resource charts on the default launch screen. It should
add a compact secondary Resource view for active sessions later, and every
field should tolerate unavailable data.

### Secrets Stay Host-Side

microsandbox's secret model uses placeholders inside the guest and substitutes
real credentials only at allowed destinations through a host-side network
boundary.

RunHaven should keep the same product principle, even though the implementation
is different: the agent should not receive host credentials by default, and the
GUI should describe credentials as brokered by RunHaven, not mounted into the
workspace.

This is useful language for future auth UI, but it does not authorize TLS
interception, secret substitution, or broader network control in RunHaven.

### Network Claims Need Evidence

microsandbox documents DNS pinning, SNI checks, metadata blocking, host access
groups, and deny-by-default policies.

RunHaven should use this as a checklist for future provider egress hardening:
metadata endpoints, private ranges, host access, DNS rebinding, and domain
fronting. Do not claim an equivalent defense unless Apple `container` plus
RunHaven's provider path actually enforces it under test.

### Recovery And Cleanup

microsandbox documents graceful stop, force kill, remove, replace, idle
timeout, maximum duration, and persisted stopped-session state.

RunHaven should keep stop, hard stop, and repair as explicit GUI actions, and
later consider idle timeout or maximum duration as Advanced safety controls.
Those controls should not be first-screen settings.

### Snapshots As A Later Recovery Pattern

microsandbox snapshots are disk-only, stopped-state artifacts with optional
integrity verification.

RunHaven should not add snapshots now. The useful idea is a future "checkpoint
before risky change" recovery pattern, especially for stateful agent work.

## Ideas To Avoid

- Do not pivot RunHaven from Apple `container` to microsandbox as part of the
  GUI work.
- Do not expose filesystem RPC, arbitrary command execution, or generic shell
  attachment from the WebView.
- Do not make resource metrics, logs, or system diagnostics the default screen
  for nontechnical users.
- Do not claim microVM-grade guarantees for RunHaven's container boundary.
- Do not add secret substitution, TLS interception, or DNS pinning unless the
  implementation and tests exist in RunHaven.
- Do not add multi-language SDK scope to RunHaven. The GUI needs typed local
  Rust/Tauri contracts, not a public SDK surface.

## Next RunHaven Applications

Use these ideas in this order:

1. Done: extend `ManagedSessionSummaryData` only as needed for selected-session
   UI with lifecycle state, one next action, and safe event markers.
2. Done: add a selected-session main pane before terminal input. It shows
   state, project label, agent, auth, network posture, and recent lifecycle
   markers without a write bridge.
3. Add typed `SessionLogEntry` later with source, timestamp, session ID, cursor,
   bounded body, and warning-gated raw output.
4. Add bounded `write_session_input` and `resize_session` only after unknown ID,
   ownership, size, and multi-session isolation tests exist.
5. Add resource metrics only as a secondary detail view.
