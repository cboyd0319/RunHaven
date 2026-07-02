//! Host-side PTY terminal sessions for the desktop app (terminal design
//! sections 1, 3, 5, 6, 9).
//!
//! Security boundary. A terminal session is a raw, interactive attach into one
//! running RunHaven sandbox. Three properties keep it inside the product safety
//! boundary:
//!
//! 1. RunHaven never trusts a WebView-supplied container name. The target is
//!    resolved from the active-run read model in Rust
//!    ([`gui_attach_plan`]), which reuses `runs_attach`'s exact validators and
//!    always runs as the non-root `agent` user (`--allow-root-user` is never
//!    exposed to the GUI).
//! 2. Consent happens once, before anything is spawned. [`open_terminal_session`]
//!    shows the native OK/Cancel dialog naming the run and container and stating
//!    the no-filter/no-store facts; on Cancel nothing is spawned.
//! 3. The live stream is relayed, never stored. The reader thread
//!    (`terminal::pty`) forwards each PTY chunk straight to the Tauri
//!    [`Channel`] and nowhere else: it never appends to the captured-output
//!    registry, writes run records, or writes any log or file. A source
//!    tripwire test (`terminal::source_tripwire_tests`) enforces this,
//!    mirroring the menu dispatch-only tripwire.
//!
//! Concurrent sessions are capped at [`MAX_TERMINAL_SESSIONS`]; opening past the
//! cap fails closed with a fixed error (a later frontend slice reuses/focuses an
//! open terminal instead). Sessions are never persisted or restored across
//! relaunch.
//!
//! Quit-time limitation (gate C, low): on main-window destroy [`close_all`] kills
//! every terminal child, but the Claude sign-in terminal (path A) additionally
//! creates a provider internal network and proxy whose teardown runs on the
//! login's detached runner thread after the child is reaped. Quitting mid-login
//! can win the race and leave an orphaned `runhaven-*` internal network (no
//! secret, no live process). It is cleaned by Maintenance "Clean up stale
//! networks" / `runhaven network prune`. Follow-up: track the login runner
//! thread so `close_all` can drain its network teardown; deferred as a
//! recoverable, low-frequency case, not new quit-latency machinery.

mod pty;
mod registry;

use runhaven_core::runtime::active::{GuiAttachPlan, gui_attach_plan};
use runhaven_core::support::validators::validate_run_id;
use serde::Serialize;
use tauri::ipc::Channel;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::MessageDialogKind;

use crate::commands::dialog::confirm_native;
// The session-registry ownership boundary lives in `registry`; re-export the
// registry state so `terminal::TerminalSessionRegistry` stays the crate-facing
// path (`lib.rs` manages it) and `SpawnedTerminal` is in scope for the flow
// layer and the behavior tests.
use registry::SpawnedTerminal;
pub(crate) use registry::TerminalSessionRegistry;

/// Concurrent terminal sessions allowed at once (terminal design section 9).
/// Each session is a live host PTY plus a `container exec` child, so the cap
/// bounds real resources; a small number is enough to work with a couple of
/// runs side by side.
pub(crate) const MAX_TERMINAL_SESSIONS: usize = 2;

/// Byte bound on a single terminal-input send, matching the 8 KB read window the
/// existing runners and the reader thread use. Keystrokes and pastes are far
/// smaller; this only stops a single `invoke` from shipping an unbounded blob.
const MAX_TERMINAL_INPUT_BYTES: usize = 8 * 1024;

/// Byte bound on a terminal session id used as a registry key, mirroring the
/// session-input registry so the command and the registry agree on the maximum.
const MAX_TERMINAL_SESSION_ID_BYTES: usize = 128;

const TERMINAL_CANCELED: &str = "Terminal canceled before it opened.";
const TERMINAL_ALREADY_OPEN: &str = "A terminal is already open for this session.";
/// Fixed, path-free failure for an unresolved/inactive session so a core error
/// string (which can name containers or template paths) never reaches the
/// WebView.
const TERMINAL_UNRESOLVED: &str =
    "This session is not an active RunHaven run, so RunHaven cannot open a terminal for it.";

/// Ordered host -> frontend terminal stream, sent over the Tauri [`Channel`]
/// (terminal design section 3). Serializes as a tagged object the later frontend
/// slice consumes: `{ "event": "output", "data": [..] }` for raw PTY bytes and
/// `{ "event": "exit", "code": <i32> }` when the session ends.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub(crate) enum TerminalMessage {
    Output { data: Vec<u8> },
    Exit { code: i32 },
}

/// A live terminal session's control surface. The registry keeps one of these
/// per session and never touches the PTY or child directly, so the whole module
/// is unit-testable with a fake handle (no live container needed).
pub(crate) trait TerminalHandle: Send + Sync {
    /// Write bytes to the PTY master for the child. Returns bytes accepted.
    fn write_input(&self, data: &[u8]) -> Result<usize, String>;
    /// Set the PTY window size (`cols`/`rows`), driving the guest `SIGWINCH`.
    fn resize(&self, cols: u16, rows: u16) -> Result<(), String>;
    /// Kill the child and drop the master. Idempotent: safe to call from an
    /// explicit close and again from teardown.
    fn shutdown(&self);
}

fn validate_terminal_session_id(session_id: &str) -> Result<(), String> {
    if session_id.len() > MAX_TERMINAL_SESSION_ID_BYTES {
        return Err(format!(
            "terminal session id is too long; maximum is {MAX_TERMINAL_SESSION_ID_BYTES} bytes"
        ));
    }
    validate_run_id(session_id).map_err(|error| error.to_string())
}

/// Resolve the target from the active-run read model, then pre-check the cap.
/// Shared by the command and the test seam so both fail closed identically
/// before any dialog or spawn.
fn resolve_and_precheck(
    registry: &TerminalSessionRegistry,
    session_id: &str,
    resolve: impl FnOnce(&str) -> Result<GuiAttachPlan, String>,
) -> Result<GuiAttachPlan, String> {
    validate_terminal_session_id(session_id)?;
    let plan = resolve(session_id)?;
    registry.precheck(session_id)?;
    Ok(plan)
}

/// Register the spawned handle, then start its reader. If the authoritative
/// insert loses the cap race, the just-spawned child is shut down so nothing is
/// left running unregistered. If the reader thread fails to spawn, the entry is
/// removed and the child killed so a registered-but-output-less, cap-consuming
/// session is never left behind, and the open path reports the failure.
fn register_and_stream(
    registry: &TerminalSessionRegistry,
    session_id: &str,
    spawned: SpawnedTerminal,
) -> Result<(), String> {
    if let Err(error) = registry.insert(session_id, spawned.handle.clone()) {
        spawned.handle.shutdown();
        return Err(error);
    }
    if let Some(start_reader) = spawned.start_reader
        && let Err(error) = start_reader(registry.clone(), session_id.to_string())
    {
        if let Some(handle) = registry.remove(session_id) {
            handle.shutdown();
        }
        return Err(error);
    }
    Ok(())
}

/// Open a live terminal into one active run (terminal design sections 1, 4).
///
/// The target is resolved in Rust from the active-run record (never from
/// `on_event` or `session_id` beyond using it as the run key), the cap is
/// enforced, and a native dialog naming the run/container and the
/// no-filter/no-store facts must be accepted before any host PTY or
/// `container exec` child is created. On accept the child is spawned through a
/// host PTY and a reader thread streams output over `on_event`; on cancel
/// nothing is spawned.
#[tauri::command]
pub(crate) async fn open_terminal_session(
    app: AppHandle,
    on_event: Channel<TerminalMessage>,
    registry: State<'_, TerminalSessionRegistry>,
    session_id: String,
) -> Result<(), String> {
    let registry = registry.inner().clone();
    let plan = resolve_and_precheck(&registry, &session_id, |id| {
        gui_attach_plan(id).map_err(|_| TERMINAL_UNRESOLVED.to_string())
    })?;
    if !confirm_terminal_open(app, &plan).await? {
        return Err(TERMINAL_CANCELED.to_string());
    }
    let spawned = pty::spawn_pty_terminal(&plan, on_event)?;
    register_and_stream(&registry, &session_id, spawned)
}

/// Open a live Claude **sign-in** terminal (terminal design section 8).
///
/// This is the terminal path A for Claude sign-in: the captured-output
/// device-code runner (`commands::agent_sign_in`) still refuses Claude because
/// its `claude auth login` needs a code pasted back, which pipes cannot host.
/// This separate, Rust-owned, native-confirmed command opens a host-PTY terminal
/// running that login inside a fresh sandbox, so the pasted code round-trips
/// in-app. The target is fixed to Claude in Rust and core; `session_id` is only
/// the registry key / run label, so a WebView value cannot steer what runs. The
/// shared terminal cap and the same write/resize/close commands apply. On a
/// launch failure nothing is registered and the caller reports it (fail closed),
/// so the UI keeps its "run it in your terminal" fallback.
#[tauri::command]
pub(crate) async fn open_claude_sign_in_terminal(
    app: AppHandle,
    on_event: Channel<TerminalMessage>,
    registry: State<'_, TerminalSessionRegistry>,
    session_id: String,
) -> Result<(), String> {
    let registry = registry.inner().clone();
    // Validate the id and enforce the shared cap before any dialog or child.
    registry.precheck(&session_id)?;
    if !confirm_claude_sign_in_open(app).await? {
        return Err(TERMINAL_CANCELED.to_string());
    }
    let spawned = pty::spawn_login_terminal(&session_id, on_event)?;
    register_and_stream(&registry, &session_id, spawned)
}

/// The one native confirmation for the Claude sign-in terminal, stating what it
/// runs and the same no-filter/no-store facts as an attach, plus that the pasted
/// code is visible on screen and the login is kept only inside the sandbox.
async fn confirm_claude_sign_in_open(app: AppHandle) -> Result<bool, String> {
    let message = "You are opening a Claude sign-in terminal in a fresh sandbox.\n\n\
         RunHaven runs `claude auth login` there. It prints a link: open it in your \
         browser, approve, then paste the code back here. What you type goes straight \
         to the sandbox. What it prints is shown here but not filtered and not saved, \
         so the code you paste is visible on this screen. Your login is saved only \
         inside the sandbox; RunHaven never sees or stores the token."
        .to_string();
    confirm_native(
        app,
        "Open Claude sign-in terminal",
        "Open terminal",
        MessageDialogKind::Warning,
        message,
    )
    .await
}

/// Write keystrokes/paste bytes to a live terminal's PTY master.
///
/// This is a synchronous, blocking write on the PTY master. If a guest stops
/// draining its stdin the master's kernel buffer fills and this write blocks the
/// command worker thread until the guest reads. The blast radius is bounded: a
/// single send is capped at [`MAX_TERMINAL_INPUT_BYTES`] (8 KB) and concurrent
/// sessions are capped at [`MAX_TERMINAL_SESSIONS`], so at most a couple of
/// worker threads can be parked. If that bound ever proves too loose, time-box
/// the write (e.g. a bounded `poll`/`write` on the master fd) rather than
/// widening the cap. Not over-engineered this slice: the per-send byte bound and
/// the session cap are the current mitigation.
#[tauri::command]
pub(crate) fn write_terminal_input(
    registry: State<'_, TerminalSessionRegistry>,
    session_id: String,
    data: Vec<u8>,
) -> Result<(), String> {
    if data.len() > MAX_TERMINAL_INPUT_BYTES {
        return Err(format!(
            "terminal input is too long; keep each send at or below {MAX_TERMINAL_INPUT_BYTES} bytes"
        ));
    }
    registry.write_input(&session_id, &data).map(|_| ())
}

/// Resize a live terminal's PTY window (`cols`/`rows`), which the guest sees as
/// a `SIGWINCH` (terminal design section 1).
#[tauri::command]
pub(crate) fn resize_terminal(
    registry: State<'_, TerminalSessionRegistry>,
    session_id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    registry.resize(&session_id, cols, rows)
}

/// Close a live terminal: kill the child, drop the master, remove the entry.
#[tauri::command]
pub(crate) fn close_terminal_session(
    registry: State<'_, TerminalSessionRegistry>,
    session_id: String,
) -> Result<(), String> {
    registry.close(&session_id)
}

/// The one native confirmation, naming the exact run and container and stating
/// the no-filter/no-store facts in plain language (terminal design section 4).
async fn confirm_terminal_open(app: AppHandle, plan: &GuiAttachPlan) -> Result<bool, String> {
    let message = format!(
        "You are opening a live shell inside sandbox {container} (run {run}).\n\n\
         What you type goes straight to it. What it prints is shown here but not \
         filtered and not saved. Passwords or codes you type, or that print, are \
         visible on this screen.",
        container = plan.container_name,
        run = plan.run_id,
    );
    confirm_native(
        app,
        "Open terminal in sandbox",
        "Open terminal",
        MessageDialogKind::Warning,
        message,
    )
    .await
}

/// Test seam mirroring [`open_terminal_session`] with the dialog and the real
/// host-PTY spawn injected, so id validation, unknown/inactive rejection, the
/// cap, confirm-cancel, and register/remove are unit-testable without a live
/// container. The real PTY behavior is the slice-4 `tauri dev` smoke.
#[cfg(test)]
pub(crate) fn open_terminal_session_for_test(
    registry: &TerminalSessionRegistry,
    session_id: &str,
    resolve: impl FnOnce(&str) -> Result<GuiAttachPlan, String>,
    approve: impl FnOnce(&GuiAttachPlan) -> Result<bool, String>,
    spawn: impl FnOnce(&GuiAttachPlan) -> Result<SpawnedTerminal, String>,
) -> Result<(), String> {
    let plan = resolve_and_precheck(registry, session_id, resolve)?;
    if !approve(&plan)? {
        return Err(TERMINAL_CANCELED.to_string());
    }
    let spawned = spawn(&plan)?;
    register_and_stream(registry, session_id, spawned)
}

/// Test seam mirroring [`open_claude_sign_in_terminal`] with the dialog and the
/// real host-PTY login spawn injected, so id validation, the shared cap,
/// confirm-cancel, and register/remove are unit-testable without a live
/// container. The target is fixed to Claude (no resolve step), matching the
/// command. The real login round-trip is the slice `tauri dev` smoke.
#[cfg(test)]
pub(crate) fn open_claude_sign_in_terminal_for_test(
    registry: &TerminalSessionRegistry,
    session_id: &str,
    approve: impl FnOnce() -> Result<bool, String>,
    spawn: impl FnOnce() -> Result<SpawnedTerminal, String>,
) -> Result<(), String> {
    registry.precheck(session_id)?;
    if !approve()? {
        return Err(TERMINAL_CANCELED.to_string());
    }
    let spawned = spawn()?;
    register_and_stream(registry, session_id, spawned)
}

#[cfg(test)]
mod source_tripwire_tests;
#[cfg(test)]
mod tests;
