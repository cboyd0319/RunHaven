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
//!    shows the native OK/Cancel dialog with the no-filter/no-store facts; on
//!    Cancel nothing is spawned.
//! 3. Attach and sign-in streams are relayed directly to the Tauri [`Channel`].
//!    Interactive run output is also kept in the same bounded, process-local
//!    output registry as non-TTY runs so History remains useful until the app
//!    closes. The registry redacts before display and never writes to disk.
//!    No terminal path writes stream bytes to run records, logs, or files; a
//!    source tripwire test (`terminal::source_tripwire_tests`) enforces that.
//!
//! Concurrent sessions are capped at [`MAX_TERMINAL_SESSIONS`]; opening past the
//! cap fails closed with a fixed error (a later frontend slice reuses/focuses an
//! open terminal instead). Sessions are never persisted or restored across
//! relaunch.
//!
//! On app exit [`close_all`] kills every terminal child. Closing the main window
//! only hides and preserves its WebView so active Channels remain connected.
//! Plan-under-PTY sign-in and interactive runs also wait up to five seconds for
//! their detached launch thread to finish run-record, container, proxy, and
//! network cleanup, so normal app quit does not race those owned resources
//! indefinitely.

mod pty;
mod registry;
mod run_identity;
pub(crate) mod user_end;

use runhaven_core::runtime::active::{GuiAttachPlan, gui_attach_plan};
use runhaven_core::runtime::plans::AgentRunPlan;
use runhaven_core::support::validators::validate_run_id;
use serde::Serialize;
use tauri::ipc::Channel;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::MessageDialogKind;

use crate::commands::dialog::confirm_native;
use crate::commands::launch::{prepare_confirmed_terminal_run, record_unstarted_worktree_failure};

const TERMINAL_CONFIRMATION_MESSAGE: &str = "This advanced terminal connects directly to the session. What you type and see is not filtered or saved. Passwords or codes can be visible on screen.";
use crate::contracts::LaunchRunRequest;
use crate::session_output::SessionOutputRegistry;
// The session-registry ownership boundary lives in `registry`; re-export the
// registry state so `terminal::TerminalSessionRegistry` stays the crate-facing
// path (`lib.rs` manages it) and `SpawnedTerminal` is in scope for the flow
// layer and the behavior tests.
pub(crate) use registry::TerminalSessionRegistry;
use registry::{SpawnedTerminal, TerminalReservation};
use run_identity::{RunTerminalOpenResponse, run_terminal_sidebar_label};

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

const TERMINAL_ALREADY_OPEN: &str = "A terminal is already open for this session.";
/// Fixed, path-free failure for an unresolved/inactive session so a core error
/// string (which can name containers or template paths) never reaches the
/// WebView.
const TERMINAL_UNRESOLVED: &str =
    "This session is not an active RunHaven run, so RunHaven cannot open a terminal for it.";

/// Ordered host -> frontend terminal stream, sent over the Tauri [`Channel`]
/// (terminal design section 3). Serializes as a tagged object the later frontend
/// slice consumes: `{ "event": "output", "data": [..] }` for raw PTY bytes and
/// `{ "event": "exit", "code": <i32>, "completionRecorded": <bool>? }`
/// when the session ends. `completionRecorded` is present only for a successful
/// sign-in command and tells the UI whether secret-free readiness was saved.
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub(crate) enum TerminalMessage {
    Output {
        data: Vec<u8>,
    },
    Exit {
        code: i32,
        #[serde(rename = "completionRecorded", skip_serializing_if = "Option::is_none")]
        completion_recorded: Option<bool>,
    },
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

/// Resolve the target from the active-run read model, then reserve the session
/// before any dialog or spawn. Shared by the command and test seam.
fn resolve_and_reserve(
    registry: &TerminalSessionRegistry,
    session_id: &str,
    resolve: impl FnOnce(&str) -> Result<GuiAttachPlan, String>,
) -> Result<(GuiAttachPlan, TerminalReservation), String> {
    validate_terminal_session_id(session_id)?;
    let plan = resolve(session_id)?;
    let reservation = registry.reserve(session_id)?;
    Ok((plan, reservation))
}

/// Register the spawned handle, then start its reader. If the authoritative
/// insert loses the cap race, the just-spawned child is shut down so nothing is
/// left running unregistered. If the reader thread fails to spawn, the entry is
/// removed and the child killed so a registered-but-output-less, cap-consuming
/// session is never left behind, and the open path reports the failure.
fn register_and_stream(
    registry: &TerminalSessionRegistry,
    reservation: TerminalReservation,
    spawned: SpawnedTerminal,
) -> Result<(), String> {
    let session_id = reservation.session_id().to_string();
    let generation = match reservation.activate(spawned.handle.clone()) {
        Ok(generation) => generation,
        Err(error) => {
            spawned.handle.shutdown();
            return Err(error);
        }
    };
    if let Some(start_reader) = spawned.start_reader
        && let Err(error) = start_reader(registry.clone(), session_id.clone(), generation)
    {
        if let Some(handle) = registry.remove_if_generation(&session_id, generation) {
            handle.shutdown();
        }
        return Err(error);
    }
    Ok(())
}

/// Register the plan's Rust-owned run id for bounded History output, make that
/// exact sink mandatory for the PTY spawn, then register the live terminal.
/// The command and no-container behavior tests share this glue so a missing or
/// WebView-substituted retention id cannot hide behind a test-only append path.
fn spawn_confirmed_run_terminal(
    registry: &TerminalSessionRegistry,
    reservation: TerminalReservation,
    output_registry: &SessionOutputRegistry,
    session_id: &str,
    mut plan: AgentRunPlan,
    spawn: impl FnOnce(AgentRunPlan, &str, pty::RetainedRunOutput) -> Result<SpawnedTerminal, String>,
) -> Result<String, String> {
    plan.make_interactive_tty();
    let run_id = plan
        .run_id
        .clone()
        .ok_or_else(|| "RunHaven could not open the run terminal.".to_string())?;
    let recovery_plan = plan.worktree.as_ref().map(|_| plan.clone());
    if let Err(error) = output_registry.register(&run_id) {
        if let Some(plan) = recovery_plan.as_ref() {
            record_unstarted_worktree_failure(plan)?;
        }
        return Err(error);
    }
    let retained_run_output = pty::RetainedRunOutput::new(run_id.clone(), output_registry.clone());
    let spawned = match spawn(plan, session_id, retained_run_output) {
        Ok(spawned) => spawned,
        Err(error) => {
            if let Some(plan) = recovery_plan.as_ref() {
                record_unstarted_worktree_failure(plan)?;
            }
            return Err(error);
        }
    };
    register_and_stream(registry, reservation, spawned)?;
    Ok(run_id)
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
) -> Result<bool, String> {
    let registry = registry.inner().clone();
    let (plan, reservation) = resolve_and_reserve(&registry, &session_id, |id| {
        gui_attach_plan(id).map_err(|_| TERMINAL_UNRESOLVED.to_string())
    })?;
    if !confirm_terminal_open(app).await? {
        return Ok(false);
    }
    let spawned = pty::spawn_pty_terminal(&plan, on_event)?;
    register_and_stream(&registry, reservation, spawned)?;
    Ok(true)
}

/// Open a live native **sign-in** terminal for one supported product agent.
///
/// This Rust-owned, native-confirmed command opens a host-PTY terminal running
/// the agent's exact login command inside a fresh sandbox, so interactive links,
/// device codes, and paste-back prompts round-trip in-app. Rust validates the
/// agent against the exact supported set; `session_id` is only the registry key
/// and run label, so a WebView value cannot steer arbitrary commands. The
/// shared terminal cap and the same write/resize/close commands apply. On a
/// launch failure nothing is registered and the caller reports it (fail closed),
/// so the UI keeps its "run it in your terminal" fallback.
#[tauri::command]
pub(crate) async fn open_agent_sign_in_terminal(
    app: AppHandle,
    on_event: Channel<TerminalMessage>,
    registry: State<'_, TerminalSessionRegistry>,
    agent: String,
    session_id: String,
) -> Result<bool, String> {
    let registry = registry.inner().clone();
    validate_terminal_session_id(&session_id)?;
    if !runhaven_core::runtime::login::supports_sandbox_login(&agent) {
        return Err("Choose a supported agent before opening sign-in.".to_string());
    }
    let reservation = registry.reserve(&session_id)?;
    if !confirm_agent_sign_in_open(app, &agent).await? {
        return Ok(false);
    }
    // The WebView session id is only a terminal-registry key. The login launch
    // writes normal lifecycle metadata, so its persistent run id must be minted
    // in trusted Rust and can never collide with a WebView-selected active run.
    let execution_id = runhaven_core::runtime::launch::new_run_id();
    let spawned = pty::spawn_login_terminal(&agent, &execution_id, on_event)?;
    register_and_stream(&registry, reservation, spawned)?;
    Ok(true)
}

/// Open a live **interactive agent run** terminal (task #23, Option A).
///
/// An interactive agent needs a controlling terminal for its native UI. This
/// command routes such a launch into a host PTY instead of the background
/// captured runner, so the user types their prompt in the agent's own UI.
///
/// It reuses the normal launch machinery end to end: [`prepare_confirmed_launch`]
/// builds the exact same plan, the same Rust-owned native launch confirmation
/// is shown (on cancel
/// nothing runs and no worktree branch is created), and `open_gui_run_pty` runs
/// the finalized plan under the same provider egress proxy, non-root `agent`
/// user, read-only rootfs, cap-drop, and workspace mount as a background run. Only
/// the EXECUTION differs: the child's stdio is a host PTY streamed over `on_event`.
/// A bounded copy is also kept in process memory for same-app History views; it
/// is never logged or written to disk. `session_id` is only the registry key /
/// run label, so a WebView value cannot steer what runs; the plan content comes
/// from `request` and Rust-side selection. An explicitly configured API-key
/// broker stays in the normal launch path and never touches the terminal stream. The
/// shared terminal cap and the write/resize/close commands apply. Cancellation
/// resolves as `false`, so it is a quiet no-op; a launch failure still returns an
/// error with nothing registered, so the UI keeps its background-run fallback.
///
/// [`prepare_confirmed_launch`]: crate::commands::launch
#[tauri::command]
pub(crate) async fn open_agent_run_terminal(
    app: AppHandle,
    on_event: Channel<TerminalMessage>,
    registry: State<'_, TerminalSessionRegistry>,
    output_registry: State<'_, SessionOutputRegistry>,
    session_id: String,
    request: LaunchRunRequest,
) -> Result<Option<RunTerminalOpenResponse>, String> {
    let registry = registry.inner().clone();
    // Validate the id and enforce the shared cap before any dialog, worktree
    // branch, or child.
    let reservation = registry.reserve(&session_id)?;
    let Some(plan) = prepare_confirmed_terminal_run(app, request).await? else {
        return Ok(None);
    };
    let label = run_terminal_sidebar_label(&plan);
    let run_id = spawn_confirmed_run_terminal(
        &registry,
        reservation,
        output_registry.inner(),
        &session_id,
        plan,
        |plan, run_label, retained_run_output| {
            pty::spawn_run_terminal(plan, run_label, on_event, retained_run_output)
        },
    )?;
    Ok(Some(RunTerminalOpenResponse { run_id, label }))
}

/// The one native confirmation for an agent sign-in terminal, stating what it
/// runs and the same no-filter/no-store facts as an attach, plus that codes are
/// visible on screen and provider credentials stay inside isolated agent state.
async fn confirm_agent_sign_in_open(app: AppHandle, agent: &str) -> Result<bool, String> {
    let label = match agent {
        "claude" => "Claude",
        "codex" => "Codex",
        "antigravity" => "Antigravity",
        "copilot" => "Copilot",
        _ => return Err("Choose a supported agent before opening sign-in.".to_string()),
    };
    confirm_native(
        app,
        &format!("Sign in to {label}?"),
        "Open",
        MessageDialogKind::Warning,
        sign_in_confirmation_message(label),
    )
    .await
}

fn sign_in_confirmation_message(agent: &str) -> String {
    format!(
        "RunHaven will open a private sign-in terminal for {agent}. Follow its steps in your browser. What you type and see is not filtered or saved. Passwords or codes can be visible on screen. Your sign-in stays in {agent}'s separate agent storage."
    )
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

/// The one native confirmation for the advanced direct terminal, stating the
/// no-filter/no-store facts in plain language (terminal design section 4).
async fn confirm_terminal_open(app: AppHandle) -> Result<bool, String> {
    confirm_native(
        app,
        "Open advanced terminal?",
        "Open",
        MessageDialogKind::Warning,
        TERMINAL_CONFIRMATION_MESSAGE.to_string(),
    )
    .await
}

#[cfg(test)]
pub(crate) fn sign_in_confirmation_message_for_test(agent: &str) -> String {
    sign_in_confirmation_message(agent)
}

#[cfg(test)]
pub(crate) fn terminal_confirmation_message_for_test() -> &'static str {
    TERMINAL_CONFIRMATION_MESSAGE
}

#[cfg(test)]
mod source_tripwire_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
pub(crate) use test_support::{
    end_terminal_session_with_approval_for_test, open_agent_run_terminal_for_test,
    open_agent_run_terminal_with_chunks_for_test, open_agent_sign_in_terminal_for_test,
    open_terminal_session_for_test,
};
#[cfg(test)]
mod tests;
