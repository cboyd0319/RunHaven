//! The host-PTY spawn and output reader for a terminal session (terminal design
//! sections 1, 3, 5).
//!
//! Why a host PTY: Apple `container exec -i -t` puts the client's own stdin into
//! raw mode and reads `TIOCGWINSZ` from its controlling terminal before creating
//! the guest process. A GUI child with piped/null stdio has no controlling
//! terminal, so that step throws `ENOTTY` and no guest process is created. We
//! therefore allocate a host PTY, spawn `container exec` through it (giving the
//! child a controlling terminal), read/write the master, and set the master
//! winsize on resize (which the kernel turns into `SIGWINCH` for the guest).
//!
//! No-persist invariant: [`stream_pty_output`] forwards each chunk to the Tauri
//! [`Channel`] and returns to reading. It never writes the output registry, run
//! records, a log, or any file. The `terminal::source_tripwire_tests` source
//! tripwire enforces this.

use std::io::{Read, Write};
use std::process::Child;
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;

use pty_process::Size;
use pty_process::blocking::{Command, Pty};
use runhaven_core::runtime::active::GuiAttachPlan;
use tauri::ipc::Channel;

use super::{
    ReaderStart, SpawnedTerminal, TerminalHandle, TerminalMessage, TerminalSessionRegistry,
};

/// Initial PTY size before the frontend's first resize, so the guest shell has a
/// sane geometry immediately. Overwritten by `resize_terminal`.
const INITIAL_ROWS: u16 = 24;
const INITIAL_COLS: u16 = 80;

/// Bounded read window for the master, matching the existing runners' 8 KB
/// chunks so a busy stream cannot grow an unbounded buffer in the reader.
const READ_CHUNK_BYTES: usize = 8 * 1024;

/// A live host-PTY terminal: the master (`pty`) drives input/resize and the
/// `container exec` child is held so it can be killed on close. Both are shared
/// with the reader thread through `Arc`.
struct PtyTerminal {
    pty: Arc<Pty>,
    child: Arc<Mutex<Option<Child>>>,
}

impl TerminalHandle for PtyTerminal {
    fn write_input(&self, data: &[u8]) -> Result<usize, String> {
        // `&Pty` implements `Write`, so a shared reference writes the master
        // without a mutable handle; the reader thread reads it concurrently,
        // which is sound because each is a separate syscall on the same fd.
        let mut writer: &Pty = &self.pty;
        writer
            .write_all(data)
            .map_err(|error| format!("could not send input to the terminal: {error}"))?;
        writer
            .flush()
            .map_err(|error| format!("could not send input to the terminal: {error}"))?;
        Ok(data.len())
    }

    fn resize(&self, cols: u16, rows: u16) -> Result<(), String> {
        // `Size::new(row, col)`: rows first. `resize` takes `&self`.
        self.pty
            .resize(Size::new(rows, cols))
            .map_err(|error| format!("could not resize the terminal: {error}"))
    }

    fn shutdown(&self) {
        kill_child(&self.child);
    }
}

/// Take the `Child` out of the shared slot under the lock, then drop the guard
/// before returning. No caller ever holds the child mutex across a blocking
/// `wait()`/`kill()`, so an explicit close ([`kill_child`]) can never deadlock
/// against the reader's teardown ([`reap_child`]/[`kill_and_reap_child`]).
/// A poisoned lock is recovered rather than skipped so a child is never
/// orphaned. `take()` also makes ownership single-shot: whichever path wins the
/// race gets the `Child`; the loser gets `None`.
fn take_child(child: &Arc<Mutex<Option<Child>>>) -> Option<Child> {
    child.lock().unwrap_or_else(PoisonError::into_inner).take()
}

/// Explicit-close teardown: kill the child and reap it so no zombie is left.
/// Used by [`PtyTerminal::shutdown`] (close command, `close_all`, and the
/// reader's channel-death backstop).
pub(super) fn kill_child(child: &Arc<Mutex<Option<Child>>>) {
    if let Some(mut child) = take_child(child) {
        let _ = child.kill();
        let _ = child.wait();
    }
}

/// Allocate a host PTY, spawn the validated `container exec` through it so the
/// child gets a controlling terminal, and return the control handle plus a
/// one-shot that starts the output reader. `plan.args` is already validated by
/// [`gui_attach_plan`](runhaven_core::runtime::active::gui_attach_plan); this
/// function adds no arguments and passes an exact argument list (never a shell
/// string) to `container`.
pub(super) fn spawn_pty_terminal(
    plan: &GuiAttachPlan,
    channel: Channel<TerminalMessage>,
) -> Result<SpawnedTerminal, String> {
    let (pty, pts) = pty_process::blocking::open()
        .map_err(|error| format!("could not open a terminal: {error}"))?;
    pty.resize(Size::new(INITIAL_ROWS, INITIAL_COLS))
        .map_err(|error| format!("could not size the terminal: {error}"))?;
    let child = Command::new("container")
        .args(&plan.args)
        .spawn(pts)
        .map_err(|error| format!("could not start the terminal: {error}"))?;

    let pty = Arc::new(pty);
    let child = Arc::new(Mutex::new(Some(child)));
    let handle: Arc<dyn TerminalHandle> = Arc::new(PtyTerminal {
        pty: Arc::clone(&pty),
        child: Arc::clone(&child),
    });

    let reader_pty = Arc::clone(&pty);
    let reader_child = Arc::clone(&child);
    // Returns `Err` when the reader thread cannot be spawned. A failed spawn
    // means no output/EOF path would ever run, so the caller must not leave the
    // session registered and cap-consuming; `register_and_stream` removes the
    // entry and kills the child. (Returning the error also keeps this module
    // free of any logging sink, per the no-persist/no-log invariant.)
    let start_reader: ReaderStart = Box::new(move |registry, session_id| {
        let thread_name = format!("runhaven-terminal-{session_id}");
        thread::Builder::new()
            .name(thread_name)
            .spawn(move || {
                stream_pty_output(reader_pty, reader_child, channel, registry, session_id);
            })
            .map(|_| ())
            .map_err(|error| format!("could not start the terminal: {error}"))
    });

    Ok(SpawnedTerminal {
        handle,
        start_reader: Some(start_reader),
    })
}

/// Read the PTY master in bounded chunks and relay each chunk to the frontend
/// over `channel`. On EOF (the child exited, or on macOS an `EIO` once the slave
/// closes) it reaps the child for an exit code, sends a final `Exit`, and clears
/// the registry entry. It writes nothing to disk or any registry other than
/// removing its own entry (no-persist invariant).
///
/// Channel death is a teardown trigger, not just a loop exit. If a `send` fails
/// the WebView is gone (the view unmounted/navigated without calling
/// `close_terminal_session`, or the window is being destroyed). The child is
/// still live in that case, so a bare `wait()` would block this reader thread
/// forever; instead the child is killed. This is the backstop for the frontend
/// contract (terminal design section 9): the view is expected to call
/// `close_terminal_session` on unmount/navigation, and this kill guarantees no
/// orphaned `container exec` child even when it does not.
fn stream_pty_output(
    pty: Arc<Pty>,
    child: Arc<Mutex<Option<Child>>>,
    channel: Channel<TerminalMessage>,
    registry: TerminalSessionRegistry,
    session_id: String,
) {
    let mut reader: &Pty = &pty;
    let mut buffer = [0_u8; READ_CHUNK_BYTES];
    let mut channel_alive = true;
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => {
                let message = TerminalMessage::Output {
                    data: buffer[..read].to_vec(),
                };
                if channel.send(message).is_err() {
                    // The WebView/window is gone and the child is still live.
                    channel_alive = false;
                    break;
                }
            }
            // A closed slave surfaces as an error (EIO) on macOS; treat any read
            // error as end-of-session rather than looping.
            Err(_) => break,
        }
    }

    // On a clean EOF/EIO the child has already exited, so just reap it. On
    // channel death the child is still running, so kill it first.
    let code = if channel_alive {
        reap_child(&child)
    } else {
        kill_and_reap_child(&child)
    };
    let _ = channel.send(TerminalMessage::Exit { code });
    let _ = registry.remove(&session_id);
}

/// Reap an already-exited child for its exit code. Returns `1` if another path
/// (an explicit close) already took and waited on the child. Never holds the
/// child mutex across `wait()` (see [`take_child`]).
fn reap_child(child: &Arc<Mutex<Option<Child>>>) -> i32 {
    match take_child(child) {
        Some(mut child) => child
            .wait()
            .ok()
            .and_then(|status| status.code())
            .unwrap_or(1),
        None => 1,
    }
}

/// Kill a still-live child (channel-death backstop) and reap it. Returns `1` if
/// another path already took the child. Never holds the child mutex across
/// `kill()`/`wait()` (see [`take_child`]).
pub(super) fn kill_and_reap_child(child: &Arc<Mutex<Option<Child>>>) -> i32 {
    match take_child(child) {
        Some(mut child) => {
            let _ = child.kill();
            child
                .wait()
                .ok()
                .and_then(|status| status.code())
                .unwrap_or(1)
        }
        None => 1,
    }
}
