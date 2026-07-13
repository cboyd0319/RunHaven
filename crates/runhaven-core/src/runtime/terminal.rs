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
//! Plan-under-PTY variants ([`open_gui_login_pty`], [`open_gui_run_pty`]): unlike
//! an attach (which joins an already-running provider container whose egress
//! proxy is already up), a login or a fresh interactive run has no running
//! container, so it must *start* one with the provider egress proxy alive for the
//! whole session. Both therefore run the real provider launch path
//! ([`launch_run_plan_with_runner`]) on a background thread with a PTY-handoff
//! runner (the shared [`run_plan_through_pty`] mechanism): the runner spawns the
//! proxy-injected `container run` child through a host PTY, hands the session to
//! the desktop reader, and blocks until the child exits, which keeps the
//! allowlist proxy and provider-only egress enforced exactly as `runhaven login
//! <agent>` and a background run do. Login uses the agent's native command with
//! no broker; an interactive run uses native container auth unless the user
//! explicitly configured the Claude/Codex API-key broker. Both write the same run
//! *metadata* (run record, run id line) the launch path writes; neither ever
//! reads or persists the terminal byte stream.

use std::io::{self, Read, Write};
use std::process::Child;
use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::Duration;

use anyhow::{Result, anyhow};
use pty_process::Size;
use pty_process::blocking::{Command, Pty};

use crate::provider::agent_sign_in_record::{AGENT_SIGN_IN_SCOPE, record_agent_sign_in};
use crate::runtime::active::GuiAttachPlan;
use crate::runtime::launch::{
    AgentCommandContext, AgentCommandRunner, launch_run_plan_with_runner_and_completion,
};
use crate::runtime::login::build_sandbox_login_terminal_plan;
use crate::runtime::plans::AgentRunPlan;
use crate::runtime::volume_attachment::STATE_VOLUME_IN_USE_MESSAGE;

mod lifecycle;
use lifecycle::{CleanupGuard, CleanupSignal, ExitSignal};

/// Initial PTY size before the frontend's first resize, so the guest shell has
/// a sane geometry immediately. Overwritten by [`GuiPtySession::resize`].
const INITIAL_ROWS: u16 = 24;
const INITIAL_COLS: u16 = 80;
const GENERIC_TERMINAL_OPEN_ERROR: &str = "could not start the terminal";
pub const STATE_VOLUME_IN_USE_TERMINAL_ERROR: &str = STATE_VOLUME_IN_USE_MESSAGE;

type TerminalOpenResult = std::result::Result<GuiPtySession, &'static str>;

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
    cleanup: Option<Arc<CleanupSignal>>,
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
    let child = Command::new(crate::apple_container::program()?)
        .args(&plan.args)
        .spawn(pts)
        .map_err(|error| anyhow!("could not start the terminal: {error}"))?;
    Ok(GuiPtySession {
        pty: Arc::new(pty),
        child: Arc::new(Mutex::new(Some(child))),
        exit: Arc::new(ExitSignal::new()),
        cleanup: None,
    })
}

/// Open a host-PTY-wrapped in-app **login** terminal for a supported product
/// agent, keyed by the caller-supplied `login_id`. Returns the shared session
/// handle once the login child is spawned, or an error (fail closed) if the
/// login cannot start (container CLI missing, image not built, PTY or proxy
/// setup fails), so the desktop app keeps its "run it in your terminal"
/// fallback.
///
/// Why not a bare `container run`: provider-only egress is enforced by a live
/// host allowlist proxy plus injected `HTTP(S)_PROXY` env on an internal
/// network, not by `container run` flags alone. A bare run of the login plan
/// would have no route to the provider login hosts, so the login would fail. This
/// therefore drives the real provider launch path with the proxy alive, and
/// only swaps the child's stdio for a host PTY so interactive login prompts
/// round-trip in-app. Egress, non-root user, read-only rootfs, and cap-drop are
/// unchanged from `runhaven login <agent>`.
pub fn open_gui_login_pty(agent: &str, login_id: &str) -> Result<GuiPtySession> {
    let plan = build_sandbox_login_terminal_plan(agent, login_id)?;
    let agent = agent.to_string();
    run_plan_through_pty(plan, &format!("login-{login_id}"), move || {
        record_agent_sign_in(&agent, AGENT_SIGN_IN_SCOPE)
    })
}

/// Open a host-PTY-wrapped in-app **agent run** for `plan`, keyed by the
/// caller-supplied `label` (the terminal session id / run label). Returns the
/// shared session handle once the run child is spawned, or an error (fail
/// closed) if the run cannot start, so the desktop app keeps its background-run
/// fallback.
///
/// This is the interactive-run counterpart of [`open_gui_login_pty`]: it drives
/// the exact same provider launch path ([`launch_run_plan_with_runner`]) on a
/// background thread with a PTY-handoff runner, so the egress proxy, non-root
/// `agent` user, read-only rootfs, cap-drop, workspace mount, and any explicit
/// API-key broker all apply exactly as a background run does; only the
/// child's stdio becomes RunHaven's host PTY. The stream is relayed by the
/// desktop reader, never read or persisted here.
///
/// The `plan` MUST already carry `--interactive --tty` in its `container run`
/// argv (see [`AgentRunPlan::make_interactive_tty`]). An interactive agent such
/// as Claude Code needs a real TTY: with none it drops to `--print` (headless)
/// and errors when no piped prompt is supplied. The desktop terminal-run command
/// forces this before calling in; the argument is by value so the run owns the
/// plan for the life of the detached launch thread.
pub fn open_gui_run_pty(plan: AgentRunPlan, label: &str) -> Result<GuiPtySession> {
    run_plan_through_pty(plan, &format!("run-{label}"), || Ok(()))
}

/// Run a full provider launch `plan` on a background thread with the child's
/// stdio swapped for a host PTY, returning the shared session handle the instant
/// the child exists. Shared mechanism behind [`open_gui_login_pty`] and
/// [`open_gui_run_pty`]: the [`PtyHandoffRunner`] spawns the proxy-injected
/// `container run` child through a host PTY, hands the session back over the
/// channel, then blocks until the child exits so the egress proxy stays alive for
/// the whole run. The terminal stream carries the child output, the launch path
/// writes the run record, and the detached thread publishes only a path-free
/// lifecycle outcome after cleanup and recording. If the launch fails before
/// the child exists the sender is dropped and `recv` errors, so this fails
/// closed with a fixed, path-free message.
fn run_plan_through_pty(
    plan: AgentRunPlan,
    label: &str,
    on_success: impl FnOnce() -> Result<()> + Send + 'static,
) -> Result<GuiPtySession> {
    let (session_tx, session_rx) = std::sync::mpsc::sync_channel(1);
    let cleanup = Arc::new(CleanupSignal::new());
    let thread_cleanup = cleanup.clone();
    thread::Builder::new()
        .name(format!("runhaven-{label}"))
        .spawn(move || {
            let cleanup_guard = CleanupGuard::new(thread_cleanup.clone());
            let error_tx = session_tx.clone();
            let mut runner = PtyHandoffRunner {
                session_tx,
                cleanup: thread_cleanup,
                terminal_status: None,
            };
            let result = launch_run_plan_with_runner_and_completion(&plan, &mut runner, on_success);
            let succeeded = result.is_ok();
            if let Err(error) = result {
                let _ = error_tx.send(Err(terminal_open_error(&error)));
            }
            cleanup_guard.finish(succeeded);
        })
        .map_err(|error| anyhow!("could not start the terminal: {error}"))?;
    session_rx
        .recv()
        .map_err(|_| anyhow!(GENERIC_TERMINAL_OPEN_ERROR))?
        .map_err(anyhow::Error::msg)
}

fn terminal_open_error(error: &anyhow::Error) -> &'static str {
    if error
        .chain()
        .any(|cause| cause.to_string() == STATE_VOLUME_IN_USE_MESSAGE)
    {
        STATE_VOLUME_IN_USE_TERMINAL_ERROR
    } else {
        GENERIC_TERMINAL_OPEN_ERROR
    }
}

/// A one-session command runner that swaps the container child's stdio for a
/// host PTY and blocks until it exits. Used by [`open_gui_login_pty`] and
/// [`open_gui_run_pty`] through [`run_plan_through_pty`]; it plugs into the normal
/// provider launch path so the egress proxy and all run hardening apply, while
/// the session's controlling terminal is RunHaven's host PTY (so `claude auth
/// login` can read the pasted code, or an interactive agent TUI can be typed in).
struct PtyHandoffRunner {
    session_tx: SyncSender<TerminalOpenResult>,
    cleanup: Arc<CleanupSignal>,
    terminal_status: Option<&'static str>,
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
        let mut session = spawn_through_pty(command, env)?;
        session.cleanup = Some(self.cleanup.clone());
        // Hand a clone to the desktop reader/control path. If the receiver is
        // already gone (the open was aborted), still fall through to wait so the
        // child is reaped rather than orphaned.
        let _ = self.session_tx.send(Ok(session.clone()));
        // Block until the login child exits. The desktop reader reaps it on EOF
        // (or an explicit close kills it); either way the exit latch fires and
        // this returns. Staying blocked here keeps the egress proxy alive for
        // the whole login.
        let code = session.wait_for_exit();
        if session.exit_was_forced() {
            self.terminal_status = Some("ended");
        }
        Ok(code)
    }

    fn terminal_status(&self) -> Option<&'static str> {
        self.terminal_status
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
    let mut builder = Command::new(crate::apple_container::program_for_program(&command[0])?);
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
        cleanup: None,
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
            self.exit.signal(code, true);
        }
        let _ = self.wait_for_launch_completion_bounded();
    }

    /// Reap an already-exited child (clean EOF) for its exit code. If another
    /// path already took the child, wait for and return that path's recorded
    /// exit code. Never holds the child mutex across `wait()` (see
    /// [`take_child`]).
    pub fn reap_after_eof(&self) -> i32 {
        match take_child(&self.child) {
            Some(mut child) => {
                let code = child
                    .wait()
                    .ok()
                    .and_then(|status| status.code())
                    .unwrap_or(1);
                self.exit.signal(code, false);
                code
            }
            None => self.exit.wait(),
        }
    }

    /// Kill a still-live child (the reader's channel-death backstop) and reap
    /// it. If another path already took the child, wait for and return that
    /// path's recorded exit code. Never holds the child mutex across
    /// `kill()`/`wait()` (see [`take_child`]).
    pub fn kill_and_reap(&self) -> i32 {
        match take_child(&self.child) {
            Some(mut child) => {
                let _ = child.kill();
                let code = child
                    .wait()
                    .ok()
                    .and_then(|status| status.code())
                    .unwrap_or(1);
                self.exit.signal(code, true);
                code
            }
            None => self.exit.wait(),
        }
    }

    /// Block until the child has been reaped by any path, returning its exit
    /// code. Used only by the login path's background runner to hold the egress
    /// proxy open for the life of the login child; the attach path never calls
    /// this.
    pub fn wait_for_exit(&self) -> i32 {
        self.exit.wait()
    }

    fn exit_was_forced(&self) -> bool {
        self.exit.forced()
    }

    /// Wait for the detached provider launch to finish cleanup, record its run,
    /// and publish any login completion marker. `None` means this is a direct
    /// attach with no detached lifecycle. Natural EOF must not report a false
    /// failure while a slow but successful completion is still in progress;
    /// explicit shutdown uses its separate bounded wait above.
    pub fn wait_for_launch_completion(&self) -> Option<bool> {
        self.cleanup.as_ref().map(|cleanup| cleanup.wait())
    }

    /// Wait a bounded time for detached cleanup during explicit teardown. This
    /// prevents a closing app or dead WebView from hanging indefinitely; a
    /// timeout is reported as failure without exposing an internal error.
    pub fn wait_for_launch_completion_bounded(&self) -> Option<bool> {
        self.cleanup.as_ref().map(|cleanup| {
            cleanup
                .wait_timeout(Duration::from_secs(5))
                .unwrap_or(false)
        })
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
mod tests;
