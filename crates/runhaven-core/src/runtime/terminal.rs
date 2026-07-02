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
//! fd. It never reads, stores, or logs the terminal stream. The desktop reader
//! (`runhaven-tauri`'s `terminal::pty`) forwards each master chunk to the Tauri
//! `Channel` and nowhere else, under that crate's source tripwire.

use std::io::{self, Read, Write};
use std::process::Child;
use std::sync::{Arc, Mutex, PoisonError};

use anyhow::{Result, anyhow};
use pty_process::Size;
use pty_process::blocking::{Command, Pty};

use crate::runtime::active::GuiAttachPlan;

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
            let _ = child.wait();
        }
    }

    /// Reap an already-exited child (clean EOF) for its exit code. Returns `1`
    /// if another path (an explicit close) already took and waited on the child.
    /// Never holds the child mutex across `wait()` (see [`take_child`]).
    pub fn reap_after_eof(&self) -> i32 {
        match take_child(&self.child) {
            Some(mut child) => child
                .wait()
                .ok()
                .and_then(|status| status.code())
                .unwrap_or(1),
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
                child
                    .wait()
                    .ok()
                    .and_then(|status| status.code())
                    .unwrap_or(1)
            }
            None => 1,
        }
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
        }
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
