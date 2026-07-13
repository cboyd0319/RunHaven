//! The Tauri-side output reader for a terminal session (terminal design
//! sections 1, 3, 5).
//!
//! The host-PTY allocation, the `container exec` spawn, and the child lifecycle
//! live in `runhaven-core` (`runtime::terminal::GuiPtySession`) so all container
//! subprocess behavior is centralized there, as the repo structure guard
//! requires. This module keeps only the Tauri-layer glue: it wraps the core
//! session in a [`TerminalHandle`] for the registry and runs the reader thread
//! that forwards master output to the Tauri [`Channel`].
//!
//! No-file invariant: [`stream_pty_output`] forwards each chunk to the Tauri
//! [`Channel`] and returns to reading. Interactive run output is also retained
//! in the bounded, process-local output registry for same-app History views.
//! It never writes run records, a log, or any file. The
//! `terminal::source_tripwire_tests` source tripwire enforces this.

use std::sync::Arc;
use std::thread;

use runhaven_core::runtime::active::GuiAttachPlan;
use runhaven_core::runtime::plans::AgentRunPlan;
use runhaven_core::runtime::terminal::{
    GuiPtySession, STATE_VOLUME_IN_USE_TERMINAL_ERROR, open_gui_attach_pty, open_gui_login_pty,
    open_gui_run_pty,
};
use tauri::ipc::Channel;

use crate::session_output::SessionOutputRegistry;

use super::registry::{ReaderStart, SpawnedTerminal, TerminalSessionRegistry};
use super::{TerminalHandle, TerminalMessage};

/// Bounded read window for the master, matching the existing runners' 8 KB
/// chunks so a busy stream cannot grow an unbounded buffer in the reader.
const READ_CHUNK_BYTES: usize = 8 * 1024;

/// The already-registered, plan-owned destination for one interactive run's
/// bounded History output. Run terminals require this value, so the production
/// spawn cannot silently omit retention or substitute a WebView session id.
#[derive(Clone)]
pub(crate) struct RetainedRunOutput {
    run_id: String,
    registry: SessionOutputRegistry,
}

impl RetainedRunOutput {
    pub(super) fn new(run_id: String, registry: SessionOutputRegistry) -> Self {
        Self { run_id, registry }
    }

    fn append(&self, bytes: &[u8]) {
        let _ = self.registry.append(&self.run_id, bytes);
    }
}

/// A live terminal handle. Input, resize, and shutdown delegate to the core PTY
/// session; the registry holds this while the reader thread holds a clone of the
/// same session (both act on one shared master and child slot).
struct PtyTerminal {
    session: GuiPtySession,
}

impl TerminalHandle for PtyTerminal {
    fn write_input(&self, data: &[u8]) -> Result<usize, String> {
        self.session.write_input(data)
    }

    fn resize(&self, cols: u16, rows: u16) -> Result<(), String> {
        self.session.resize(cols, rows)
    }

    fn shutdown(&self) {
        self.session.shutdown();
    }
}

/// Allocate the host PTY and spawn the validated `container exec` in core, then
/// return the control handle plus a one-shot that starts the output reader.
/// `plan` is already validated by
/// [`gui_attach_plan`](runhaven_core::runtime::active::gui_attach_plan); core
/// adds no arguments and passes an exact argument list to `container`.
pub(super) fn spawn_pty_terminal(
    plan: &GuiAttachPlan,
    channel: Channel<TerminalMessage>,
) -> Result<SpawnedTerminal, String> {
    let session = open_gui_attach_pty(plan).map_err(|error| error.to_string())?;
    Ok(spawned_from_session(session, channel, None, None))
}

/// Open a host-PTY-wrapped native **login** terminal in core and wire the same
/// reader/handle glue as an attach. Core selects the exact provider command and
/// runs it with proxy-enforced egress, non-root execution, and a host PTY. On
/// failure, expose the one fixed recovery message core explicitly marks safe;
/// collapse every other core error because it can name host paths.
pub(super) fn spawn_login_terminal(
    agent: &str,
    login_id: &str,
    channel: Channel<TerminalMessage>,
) -> Result<SpawnedTerminal, String> {
    let session = open_gui_login_pty(agent, login_id)
        .map_err(|error| login_open_error_message(&error.to_string()))?;
    Ok(spawned_from_session(
        session,
        channel,
        Some(agent.to_string()),
        None,
    ))
}

fn login_open_error_message(error: &str) -> String {
    if error == STATE_VOLUME_IN_USE_TERMINAL_ERROR {
        return STATE_VOLUME_IN_USE_TERMINAL_ERROR.to_string();
    }
    "RunHaven could not open the sign-in terminal.".to_string()
}

/// Open a host-PTY-wrapped interactive **agent run** in core and wire the same
/// reader/handle glue as an attach/login. Core runs the finalized run plan through
/// the real provider launch path (proxy-enforced egress, non-root `agent` user,
/// read-only rootfs, cap-drop, workspace mount, and any explicit API-key broker),
/// with the child's stdio on a host PTY so the agent's interactive TUI can be
/// typed in-app; the plan already carries `--interactive --tty`. On failure the
/// core error is collapsed to a fixed message (it can name host paths) so the
/// WebView keeps its background-run fallback text.
pub(super) fn spawn_run_terminal(
    plan: AgentRunPlan,
    run_label: &str,
    channel: Channel<TerminalMessage>,
    retained_run_output: RetainedRunOutput,
) -> Result<SpawnedTerminal, String> {
    let session = open_gui_run_pty(plan, run_label)
        .map_err(|_| "RunHaven could not open the run terminal.".to_string())?;
    Ok(spawned_run_from_session(
        session,
        channel,
        retained_run_output,
    ))
}

/// Run-specific wrapper keeps the retention sink mandatory while attach and
/// sign-in terminals continue to use the shared reader with no History sink.
fn spawned_run_from_session(
    session: GuiPtySession,
    channel: Channel<TerminalMessage>,
    retained_run_output: RetainedRunOutput,
) -> SpawnedTerminal {
    spawned_from_session(session, channel, None, Some(retained_run_output))
}

/// Build the registry control handle and the output-reader one-shot for a live
/// [`GuiPtySession`], shared by the attach, login, and run open paths so all
/// stream, reap, and tear down identically.
fn spawned_from_session(
    session: GuiPtySession,
    channel: Channel<TerminalMessage>,
    sign_in_agent: Option<String>,
    retained_run_output: Option<RetainedRunOutput>,
) -> SpawnedTerminal {
    let handle: Arc<dyn TerminalHandle> = Arc::new(PtyTerminal {
        session: session.clone(),
    });

    let reader_session = session;
    // Returns `Err` when the reader thread cannot be spawned. A failed spawn
    // means no output/EOF path would ever run, so the caller must not leave the
    // session registered and cap-consuming; `register_and_stream` removes the
    // entry and kills the child. (Returning the error also keeps this module
    // free of any logging sink, per the no-persist/no-log invariant.)
    let start_reader: ReaderStart = Box::new(move |registry, session_id, generation| {
        let thread_name = format!("runhaven-terminal-{session_id}");
        thread::Builder::new()
            .name(thread_name)
            .spawn(move || {
                stream_pty_output(
                    reader_session,
                    channel,
                    registry,
                    session_id,
                    generation,
                    sign_in_agent,
                    retained_run_output,
                );
            })
            .map(|_| ())
            .map_err(|error| format!("could not start the terminal: {error}"))
    });

    SpawnedTerminal {
        handle,
        start_reader: Some(start_reader),
    }
}

/// Read the PTY master in bounded chunks and relay each chunk to the frontend
/// over `channel`. On EOF (the child exited, or on macOS an `EIO` once the slave
/// closes) it reaps the child for an exit code, waits for any detached launch
/// cleanup/recording, sends a final `Exit`, and clears the terminal registry
/// entry. Interactive run chunks also enter the bounded process-local output
/// registry for same-app History views. No stream bytes are logged or written
/// to disk (no-persist invariant).
///
/// Channel death is a teardown trigger, not just a loop exit. If a `send` fails
/// the WebView is gone (the view unmounted/navigated without calling
/// `close_terminal_session`, or the window is being destroyed). The child is
/// still live in that case, so a bare `wait()` would block this reader thread
/// forever; instead the child is killed through the core session. This is the
/// backstop for the frontend contract (terminal design section 9): the view is
/// expected to call `close_terminal_session` on unmount/navigation, and this
/// kill guarantees no orphaned `container exec` child even when it does not.
fn stream_pty_output(
    session: GuiPtySession,
    channel: Channel<TerminalMessage>,
    registry: TerminalSessionRegistry,
    session_id: String,
    generation: u64,
    sign_in_agent: Option<String>,
    retained_run_output: Option<RetainedRunOutput>,
) {
    let channel_alive = forward_pty_chunks(
        |buffer| session.read(buffer),
        |message| channel.send(message).is_ok(),
        retained_run_output.as_ref(),
    );

    // On a clean EOF/EIO the child has already exited, so just reap it. On
    // channel death the child is still running, so kill it first.
    let code = if channel_alive {
        session.reap_after_eof()
    } else {
        session.kill_and_reap()
    };
    // A zero child code is not enough: provider cleanup, run recording, and
    // login-status publication happen after the child exits. Core reports only
    // a path-free success/failure bit after that entire lifecycle completes.
    let lifecycle_succeeded = if channel_alive {
        session.wait_for_launch_completion()
    } else {
        session.wait_for_launch_completion_bounded()
    };
    let completion_recorded =
        sign_in_completion_recorded(code, sign_in_agent.as_deref(), lifecycle_succeeded);
    let _ = channel.send(TerminalMessage::Exit {
        code,
        completion_recorded,
    });
    let _ = registry.remove_if_generation(&session_id, generation);
}

/// Forward PTY chunks through the live Channel and, for interactive runs, the
/// bounded in-memory History buffer. Keeping this loop independent of the
/// concrete PTY makes the reader-to-History path behavior-testable without
/// launching Apple Container in unit tests.
fn forward_pty_chunks(
    mut read: impl FnMut(&mut [u8]) -> std::io::Result<usize>,
    mut send: impl FnMut(TerminalMessage) -> bool,
    retained_run_output: Option<&RetainedRunOutput>,
) -> bool {
    let mut buffer = [0_u8; READ_CHUNK_BYTES];
    loop {
        match read(&mut buffer) {
            Ok(0) => return true,
            Ok(read) => {
                if let Some(retained_run_output) = retained_run_output {
                    retained_run_output.append(&buffer[..read]);
                }
                if !send(TerminalMessage::Output {
                    data: buffer[..read].to_vec(),
                }) {
                    // The WebView/window is gone and the child is still live.
                    return false;
                }
            }
            // A closed slave surfaces as an error (EIO) on macOS; treat any read
            // error as end-of-session rather than looping.
            Err(_) => return true,
        }
    }
}

#[cfg(test)]
pub(crate) fn retain_terminal_chunks_for_test(
    retained_run_output: &RetainedRunOutput,
    chunks: &[&[u8]],
) {
    let mut next = 0;
    let channel_alive = forward_pty_chunks(
        |buffer| {
            let Some(chunk) = chunks.get(next) else {
                return Ok(0);
            };
            next += 1;
            buffer[..chunk.len()].copy_from_slice(chunk);
            Ok(chunk.len())
        },
        |_| true,
        Some(retained_run_output),
    );
    assert!(
        channel_alive,
        "the test Channel remains available through EOF"
    );
}

fn sign_in_completion_recorded(
    code: i32,
    agent: Option<&str>,
    lifecycle_succeeded: Option<bool>,
) -> Option<bool> {
    if code != 0 {
        return None;
    }
    agent.map(|_| lifecycle_succeeded.unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use runhaven_core::runtime::terminal::STATE_VOLUME_IN_USE_TERMINAL_ERROR;

    use super::{login_open_error_message, sign_in_completion_recorded};

    #[test]
    fn login_open_exposes_only_the_fixed_state_conflict_message() {
        assert_eq!(
            login_open_error_message(STATE_VOLUME_IN_USE_TERMINAL_ERROR),
            STATE_VOLUME_IN_USE_TERMINAL_ERROR
        );
        assert_eq!(
            login_open_error_message("could not mount /Users/private/workspace"),
            "RunHaven could not open the sign-in terminal."
        );
    }

    #[test]
    fn completion_record_reports_success_failure_and_non_login_exit() {
        assert_eq!(
            sign_in_completion_recorded(0, Some("codex"), Some(true)),
            Some(true),
        );
        assert_eq!(
            sign_in_completion_recorded(0, Some("codex"), Some(false)),
            Some(false),
        );
        assert_eq!(
            sign_in_completion_recorded(1, Some("codex"), Some(true)),
            None
        );
        assert_eq!(sign_in_completion_recorded(0, None, Some(true)), None);
    }
}
