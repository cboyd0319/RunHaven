//! Host-PTY spawn and child lifecycle for an in-app terminal attach (native GUI
//! terminal design sections 1, 3, 5).
//!
//! Why this lives in `runhaven-core`: the repo structure guard requires every
//! `container` subprocess to be centralized here, not in the Tauri crate. Core
//! owns the `container exec` spawn, the host PTY, and the child lifecycle; the
//! desktop crate keeps only the Tauri `Channel` reader thread and the session
//! registry. This supersedes the design's earlier "core stays PTY-free" note,
//! which predated the structure-boundary rule.
//!
//! No-persist boundary: this module spawns the child and hands back the master
//! fd. It never reads, stores, or logs the terminal *stream*. The desktop reader
//! (`runhaven-tauri`'s `terminal::pty`) forwards each master chunk to the Tauri
//! `Channel` and nowhere else, under that crate's source tripwire.
//!
//! Claude login variant ([`open_gui_login_pty`]): unlike an attach (which joins
//! an already-running provider container whose egress proxy is already up), a
//! login has no running container, so it must *start* one with the provider
//! egress proxy alive for the whole login. It therefore runs the real provider
//! launch path ([`launch_run_plan_with_runner`]) on a background thread with a
//! PTY-handoff runner: the runner spawns the proxy-injected `container run`
//! child through a host PTY, hands the session to the desktop reader, and blocks
//! until the child exits, which keeps the allowlist proxy and provider-only
//! egress enforced exactly as `runhaven login claude` does. That path writes the
//! same login *metadata* (run record, run id line) the CLI login writes; it
//! still never reads or persists the terminal byte stream.

use std::io::{self, Read, Write};
use std::process::Child;
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::thread;

use anyhow::{Result, anyhow};
use pty_process::Size;
use pty_process::blocking::{Command, Pty};

use crate::runtime::active::GuiAttachPlan;
use crate::runtime::launch::{
    AgentCommandContext, AgentCommandRunner, launch_run_plan_with_runner,
};
use crate::runtime::login::build_sandbox_login_terminal_plan;

/// Initial PTY size before the frontend's first resize, so the guest shell has
/// a sane geometry immediately. Overwritten by [`GuiPtySession::resize`].
const INITIAL_ROWS: u16 = 24;
const INITIAL_COLS: u16 = 80;

/// A live host-PTY session wrapping one `container exec` child. The master
/// (`pty`) drives input/output/resize and the child is held so it can be killed
/// on close. Cheap to clone: every clone shares the same master and child slot
/// through `Arc`, so the desktop control handle and its reader thread act on one
/// PTY and reap the child exactly once.
#[derive(Clone)]
pub struct GuiPtySession {
    pty: Arc<Pty>,
    child: Arc<Mutex<Option<Child>>>,
    exit: Arc<ExitSignal>,
}

/// One-shot exit latch: the first reaping path (EOF reap, channel-death kill, or
/// explicit shutdown) records the child's exit code and wakes any waiter. The
/// login path's background runner blocks on [`GuiPtySession::wait_for_exit`] so
/// the egress proxy stays alive for exactly as long as the login child runs; the
/// attach path never waits on it, so this is inert overhead there.
struct ExitSignal {
    code: Mutex<Option<i32>>,
    ready: Condvar,
}

impl ExitSignal {
    fn new() -> Self {
        Self {
            code: Mutex::new(None),
            ready: Condvar::new(),
        }
    }

    /// Record the exit code once (first writer wins) and wake all waiters. A
    /// poisoned lock is recovered so a waiter is never stranded.
    fn signal(&self, code: i32) {
        let mut guard = self.code.lock().unwrap_or_else(PoisonError::into_inner);
        if guard.is_none() {
            *guard = Some(code);
            self.ready.notify_all();
        }
    }

    fn wait(&self) -> i32 {
        let mut guard = self.code.lock().unwrap_or_else(PoisonError::into_inner);
        while guard.is_none() {
            guard = self
                .ready
                .wait(guard)
                .unwrap_or_else(PoisonError::into_inner);
        }
        guard.expect("exit code recorded before wake")
    }
}

/// Allocate a host PTY and spawn the validated `container exec` through it so
/// the child gets a controlling terminal, returning the shared session handle.
/// `plan.args` is already validated by
/// [`gui_attach_plan`](crate::runtime::active::gui_attach_plan); this adds no
/// arguments and passes an exact argument list (never a shell string) to
/// `container`.
pub fn open_gui_attach_pty(plan: &GuiAttachPlan) -> Result<GuiPtySession> {
    let (pty, pts) = pty_process::blocking::open()
        .map_err(|error| anyhow!("could not open a terminal: {error}"))?;
    pty.resize(Size::new(INITIAL_ROWS, INITIAL_COLS))
        .map_err(|error| anyhow!("could not size the terminal: {error}"))?;
    let child = Command::new("container")
        .args(&plan.args)
        .spawn(pts)
        .map_err(|error| anyhow!("could not start the terminal: {error}"))?;
    Ok(GuiPtySession {
        pty: Arc::new(pty),
        child: Arc::new(Mutex::new(Some(child))),
        exit: Arc::new(ExitSignal::new()),
    })
}

/// Open a host-PTY-wrapped in-app **login** terminal for `agent` (only `claude`
/// today), keyed by the caller-supplied `login_id`. Returns the shared session
/// handle once the login child is spawned, or an error (fail closed) if the
/// login cannot start (container CLI missing, image not built, PTY or proxy
/// setup fails), so the desktop app keeps its "run it in your terminal"
/// fallback.
///
/// Why not a bare `container run`: provider-only egress is enforced by a live
/// host allowlist proxy plus injected `HTTP(S)_PROXY` env on an internal
/// network, not by `container run` flags alone. A bare run of the login plan
/// would have no route to `platform.claude.com`, so the login would fail. This
/// therefore drives the real provider launch path with the proxy alive, and
/// only swaps the child's stdio for a host PTY so `claude auth login`'s pasted
/// code round-trips in-app. Egress, non-root user, read-only rootfs, and
/// cap-drop are unchanged from `runhaven login claude`.
pub fn open_gui_login_pty(agent: &str, login_id: &str) -> Result<GuiPtySession> {
    if agent != "claude" {
        return Err(anyhow!("the in-app login terminal only supports claude"));
    }
    let plan = build_sandbox_login_terminal_plan(agent, login_id)?;
    // The runner hands the spawned session back over this channel the instant
    // the child exists; if the launch fails before that, the sender is dropped
    // and `recv` errors, so we fail closed with a fixed, path-free message.
    let (session_tx, session_rx) = std::sync::mpsc::sync_channel(1);
    thread::Builder::new()
        .name(format!("runhaven-login-{login_id}"))
        .spawn(move || {
            let mut runner = PtyHandoffRunner { session_tx };
            // The result is intentionally dropped: the terminal stream carries
            // the login outcome to the user, and the run record is written by
            // the launch path itself. Nothing to return from the detached
            // thread. It self-terminates when the login child exits.
            let _ = launch_run_plan_with_runner(&plan, &mut runner);
        })
        .map_err(|error| anyhow!("could not start the login terminal: {error}"))?;
    session_rx
        .recv()
        .map_err(|_| anyhow!("could not start the Claude login terminal"))
}

/// A one-login command runner that swaps the container child's stdio for a host
/// PTY and blocks until it exits. Used only by [`open_gui_login_pty`]; it plugs
/// into the normal provider launch path so the egress proxy and all run
/// hardening apply, while the login's controlling terminal is RunHaven's host
/// PTY (so `claude auth login` can read the pasted code).
struct PtyHandoffRunner {
    session_tx: SyncSender<GuiPtySession>,
}

impl AgentCommandRunner for PtyHandoffRunner {
    fn run_agent_command(
        &mut self,
        command: &[String],
        env: Option<(&str, &str)>,
        _context: AgentCommandContext<'_>,
    ) -> Result<i32> {
        // `command` is the fully proxy-injected `container run ...` argv from the
        // provider launch path (`command[0] == "container"`); this adds no
        // arguments and passes an exact argument list, never a shell string.
        let session = spawn_through_pty(command, env)?;
        // Hand a clone to the desktop reader/control path. If the receiver is
        // already gone (the open was aborted), still fall through to wait so the
        // child is reaped rather than orphaned.
        let _ = self.session_tx.send(session.clone());
        // Block until the login child exits. The desktop reader reaps it on EOF
        // (or an explicit close kills it); either way the exit latch fires and
        // this returns. Staying blocked here keeps the egress proxy alive for
        // the whole login.
        Ok(session.wait_for_exit())
    }
}

/// Allocate a host PTY and spawn `command` through it, giving the child a
/// controlling terminal. Shared shape with [`open_gui_attach_pty`]; the only
/// difference is the argv comes from the provider launch path.
fn spawn_through_pty(command: &[String], env: Option<(&str, &str)>) -> Result<GuiPtySession> {
    let (pty, pts) = pty_process::blocking::open()
        .map_err(|error| anyhow!("could not open a terminal: {error}"))?;
    pty.resize(Size::new(INITIAL_ROWS, INITIAL_COLS))
        .map_err(|error| anyhow!("could not size the terminal: {error}"))?;
    let mut builder = Command::new(&command[0]);
    builder = builder.args(&command[1..]);
    if let Some((name, value)) = env {
        builder = builder.env(name, value);
    }
    let child = builder
        .spawn(pts)
        .map_err(|error| anyhow!("could not start the terminal: {error}"))?;
    Ok(GuiPtySession {
        pty: Arc::new(pty),
        child: Arc::new(Mutex::new(Some(child))),
        exit: Arc::new(ExitSignal::new()),
    })
}

impl GuiPtySession {
    /// Read a chunk from the PTY master. `&Pty` implements `Read`, so a shared
    /// reference reads without a mutable handle; a concurrent [`write_input`] is
    /// sound because each is a separate syscall on the same fd.
    ///
    /// [`write_input`]: Self::write_input
    pub fn read(&self, buffer: &mut [u8]) -> io::Result<usize> {
        let mut reader: &Pty = &self.pty;
        reader.read(buffer)
    }

    /// Write bytes to the PTY master for the child. Returns bytes accepted.
    pub fn write_input(&self, data: &[u8]) -> Result<usize, String> {
        let mut writer: &Pty = &self.pty;
        writer
            .write_all(data)
            .map_err(|error| format!("could not send input to the terminal: {error}"))?;
        writer
            .flush()
            .map_err(|error| format!("could not send input to the terminal: {error}"))?;
        Ok(data.len())
    }

    /// Set the PTY window size (`cols`/`rows`), which the guest sees as a
    /// `SIGWINCH`. `Size::new(row, col)` takes rows first.
    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), String> {
        self.pty
            .resize(Size::new(rows, cols))
            .map_err(|error| format!("could not resize the terminal: {error}"))
    }

    /// Explicit-close teardown: kill the child and reap it so no zombie is left.
    /// Idempotent and lock-safe (see [`take_child`]). Used by the desktop close
    /// command, `close_all`, and the reader's channel-death backstop.
    pub fn shutdown(&self) {
        if let Some(mut child) = take_child(&self.child) {
            let _ = child.kill();
            let code = child
                .wait()
                .ok()
                .and_then(|status| status.code())
                .unwrap_or(1);
            self.exit.signal(code);
        }
    }

    /// Reap an already-exited child (clean EOF) for its exit code. Returns `1`
    /// if another path (an explicit close) already took and waited on the child.
    /// Never holds the child mutex across `wait()` (see [`take_child`]).
    pub fn reap_after_eof(&self) -> i32 {
        match take_child(&self.child) {
            Some(mut child) => {
                let code = child
                    .wait()
                    .ok()
                    .and_then(|status| status.code())
                    .unwrap_or(1);
                self.exit.signal(code);
                code
            }
            None => 1,
        }
    }

    /// Kill a still-live child (the reader's channel-death backstop) and reap
    /// it. Returns `1` if another path already took the child. Never holds the
    /// child mutex across `kill()`/`wait()` (see [`take_child`]).
    pub fn kill_and_reap(&self) -> i32 {
        match take_child(&self.child) {
            Some(mut child) => {
                let _ = child.kill();
                let code = child
                    .wait()
                    .ok()
                    .and_then(|status| status.code())
                    .unwrap_or(1);
                self.exit.signal(code);
                code
            }
            None => 1,
        }
    }

    /// Block until the child has been reaped by any path, returning its exit
    /// code. Used only by the login path's background runner to hold the egress
    /// proxy open for the life of the login child; the attach path never calls
    /// this.
    pub fn wait_for_exit(&self) -> i32 {
        self.exit.wait()
    }
}

/// Take the `Child` out of the shared slot under the lock, then drop the guard
/// before returning. No caller ever holds the child mutex across a blocking
/// `wait()`/`kill()`, so an explicit close can never deadlock against the
/// reader's teardown. A poisoned lock is recovered rather than skipped so a
/// child is never orphaned. `take()` makes ownership single-shot: whichever path
/// wins the race gets the `Child`; the loser gets `None`.
fn take_child(child: &Arc<Mutex<Option<Child>>>) -> Option<Child> {
    child.lock().unwrap_or_else(PoisonError::into_inner).take()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a session over a real host PTY with a long-lived `/bin/sleep`
    /// stand-in for the guest shell. Needs no container: it exercises only the
    /// child-slot locking, which is what the deadlock property is about.
    fn sleep_session() -> GuiPtySession {
        let (pty, pts) = pty_process::blocking::open().expect("open a host pty");
        let child = Command::new("/bin/sleep")
            .arg("30")
            .spawn(pts)
            .expect("spawn a stand-in child");
        GuiPtySession {
            pty: Arc::new(pty),
            child: Arc::new(Mutex::new(Some(child))),
            exit: Arc::new(ExitSignal::new()),
        }
    }

    #[test]
    fn wait_for_exit_unblocks_when_a_reader_reaps_the_child() {
        // The login runner blocks on wait_for_exit so the egress proxy stays up
        // for the whole login. Prove that a reap on another thread (standing in
        // for the desktop reader hitting EOF/close) releases that wait with the
        // reaped exit code, so the proxy can never be stranded open.
        let session = sleep_session();
        let waiter = session.clone();
        let (code_tx, code_rx) = std::sync::mpsc::channel();
        let waiter_thread = std::thread::spawn(move || {
            let _ = code_tx.send(waiter.wait_for_exit());
        });
        // No exit yet: the child is a live /bin/sleep, so the waiter must block.
        assert!(
            code_rx
                .recv_timeout(std::time::Duration::from_millis(150))
                .is_err(),
            "wait_for_exit must block while the child is still alive"
        );
        // The reader-side reap kills and latches the exit, releasing the waiter.
        let reaped = session.kill_and_reap();
        let observed = code_rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("wait_for_exit must release once the child is reaped");
        assert_eq!(observed, reaped, "the waiter must see the reaped exit code");
        waiter_thread.join().expect("waiter joins");
    }

    #[test]
    fn channel_death_kill_and_a_concurrent_close_do_not_deadlock() {
        // F1: the reader's channel-death kill must not deadlock against an
        // explicit close racing it. A real long-lived child stands in for the
        // guest shell: `wait()` on it blocks forever unless something kills it,
        // so completion proves the kill happened and both reap paths are
        // lock-safe. The child must end reaped exactly once.
        let session = sleep_session();
        let reaper = session.clone();
        let closer = session.clone();
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        let coordinator = std::thread::spawn(move || {
            let reaper = std::thread::spawn(move || reaper.kill_and_reap());
            let closer = std::thread::spawn(move || closer.shutdown());
            reaper.join().expect("reaper joins");
            closer.join().expect("closer joins");
            let _ = done_tx.send(());
        });

        done_rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("channel-death teardown and a concurrent close must not deadlock");
        coordinator.join().expect("coordinator joins");

        assert!(
            session.child.lock().expect("child slot").is_none(),
            "the child must be reaped exactly once and left removed"
        );
    }
}
