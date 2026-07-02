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

mod pty;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use runhaven_core::runtime::active::{GuiAttachPlan, gui_attach_plan};
use runhaven_core::support::validators::validate_run_id;
use serde::Serialize;
use tauri::ipc::Channel;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::MessageDialogKind;

use crate::commands::dialog::confirm_native;
use crate::registry_support::lock_or;

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

const REGISTRY_UNAVAILABLE: &str = "Terminal sessions are unavailable.";
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

/// The one-shot that starts a session's output reader thread. Returns `Err` when
/// the reader thread cannot be spawned, so [`register_and_stream`] can tear the
/// half-open session down instead of leaving it registered without a reader.
type ReaderStart = Box<dyn FnOnce(TerminalSessionRegistry, String) -> Result<(), String> + Send>;

/// What a spawn produces: the control handle for the registry, plus a one-shot
/// that starts the output reader thread. The reader is started only *after* the
/// handle is registered, so a child that exits immediately cannot make the
/// reader remove a registry entry that does not exist yet.
pub(crate) struct SpawnedTerminal {
    handle: Arc<dyn TerminalHandle>,
    start_reader: Option<ReaderStart>,
}

impl SpawnedTerminal {
    #[cfg(test)]
    pub(crate) fn without_reader(handle: Arc<dyn TerminalHandle>) -> Self {
        Self {
            handle,
            start_reader: None,
        }
    }
}

/// Registry of live terminal sessions, keyed by session id (which is the run id
/// for an attach). Mirrors `SessionInputRegistry`: a poisoned lock fails closed
/// through [`lock_or`], and every mutation is bounded.
#[derive(Clone, Default)]
pub(crate) struct TerminalSessionRegistry {
    sessions: Arc<Mutex<HashMap<String, Arc<dyn TerminalHandle>>>>,
}

impl TerminalSessionRegistry {
    /// Best-effort pre-check before the confirmation dialog: reject a duplicate
    /// session or a request past the cap so no dialog is shown for a request
    /// that cannot succeed. [`Self::insert`] re-checks authoritatively.
    fn precheck(&self, session_id: &str) -> Result<(), String> {
        validate_terminal_session_id(session_id)?;
        let sessions = lock_or(self.sessions.lock(), REGISTRY_UNAVAILABLE)?;
        check_capacity(&sessions, session_id)
    }

    /// Authoritative reserve: re-check duplicate and cap under the lock, then
    /// insert. On error the caller must shut down the freshly spawned handle.
    fn insert(&self, session_id: &str, handle: Arc<dyn TerminalHandle>) -> Result<(), String> {
        validate_terminal_session_id(session_id)?;
        let mut sessions = lock_or(self.sessions.lock(), REGISTRY_UNAVAILABLE)?;
        check_capacity(&sessions, session_id)?;
        sessions.insert(session_id.to_string(), handle);
        Ok(())
    }

    /// The handle for `session_id`, cloned out so the PTY write/resize syscall
    /// runs without holding the registry lock.
    fn handle(&self, session_id: &str) -> Result<Arc<dyn TerminalHandle>, String> {
        validate_terminal_session_id(session_id)?;
        let sessions = lock_or(self.sessions.lock(), REGISTRY_UNAVAILABLE)?;
        sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| "No open terminal for this session.".to_string())
    }

    fn write_input(&self, session_id: &str, data: &[u8]) -> Result<usize, String> {
        self.handle(session_id)?.write_input(data)
    }

    fn resize(&self, session_id: &str, cols: u16, rows: u16) -> Result<(), String> {
        self.handle(session_id)?.resize(cols, rows)
    }

    /// Remove and return the handle for `session_id`, if present. Used by the
    /// reader thread on EOF; dropping the returned handle drops the master.
    fn remove(&self, session_id: &str) -> Option<Arc<dyn TerminalHandle>> {
        let mut sessions = self.sessions.lock().ok()?;
        sessions.remove(session_id)
    }

    /// Explicit close: remove the entry and kill its child.
    fn close(&self, session_id: &str) -> Result<(), String> {
        validate_terminal_session_id(session_id)?;
        if let Some(handle) = self.remove(session_id) {
            handle.shutdown();
        }
        Ok(())
    }

    /// Tear down every live session. Called when the main window is destroyed so
    /// no `container exec` child outlives the app (terminal design section 9).
    pub(crate) fn close_all(&self) {
        // Recover a poisoned lock instead of returning: on app quit every child
        // must be killed even if a writer panicked, so a poisoned registry can
        // never leave an orphaned `container exec` child alive.
        let handles: Vec<Arc<dyn TerminalHandle>> = self
            .sessions
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .drain()
            .map(|(_, handle)| handle)
            .collect();
        for handle in handles {
            handle.shutdown();
        }
    }
}

fn check_capacity(
    sessions: &HashMap<String, Arc<dyn TerminalHandle>>,
    session_id: &str,
) -> Result<(), String> {
    if sessions.contains_key(session_id) {
        return Err(TERMINAL_ALREADY_OPEN.to_string());
    }
    if sessions.len() >= MAX_TERMINAL_SESSIONS {
        return Err(format!(
            "You already have {MAX_TERMINAL_SESSIONS} terminals open. \
             Close one before opening another."
        ));
    }
    Ok(())
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

#[cfg(test)]
mod source_tripwire_tests;
#[cfg(test)]
mod tests;
