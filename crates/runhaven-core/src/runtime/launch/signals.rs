use std::io;
use std::sync::Arc;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use signal_hook::SigId;
use signal_hook::consts::signal::{SIGHUP, SIGINT, SIGTERM};
use signal_hook::flag;

static SIGNAL_STATE: OnceLock<Result<CleanupSignalState, (io::ErrorKind, String)>> =
    OnceLock::new();

pub(super) struct CleanupSignalState {
    received: Arc<AtomicUsize>,
    default_enabled: Arc<AtomicBool>,
    _registrations: Vec<SigId>,
}

/// Keep the CLI parent alive for termination signals so it can stop its child
/// and finish the normal run-record and resource cleanup path.
pub(super) fn install_cleanup_signal_handlers() -> io::Result<&'static CleanupSignalState> {
    let state = SIGNAL_STATE
        .get_or_init(|| build_signal_state().map_err(|error| (error.kind(), error.to_string())));
    match state {
        Ok(state) => {
            state.arm_for_child();
            Ok(state)
        }
        Err((kind, message)) => Err(io::Error::new(*kind, message.clone())),
    }
}

fn build_signal_state() -> io::Result<CleanupSignalState> {
    let received = Arc::new(AtomicUsize::new(0));
    let default_enabled = Arc::new(AtomicBool::new(false));
    let mut registrations = Vec::new();
    for signal in [SIGINT, SIGTERM, SIGHUP] {
        // Registration order is load-bearing: the default action checks the
        // old phase before the first signal stores its value. Once the child is
        // reaped, `finish_child` arms the default action for any later signal.
        registrations.push(flag::register_conditional_default(
            signal,
            Arc::clone(&default_enabled),
        )?);
        registrations.push(flag::register_usize(
            signal,
            Arc::clone(&received),
            signal as usize,
        )?);
    }
    Ok(CleanupSignalState {
        received,
        default_enabled,
        _registrations: registrations,
    })
}

impl CleanupSignalState {
    fn arm_for_child(&self) {
        self.received.store(0, Ordering::SeqCst);
        self.default_enabled.store(false, Ordering::SeqCst);
    }

    pub(super) fn finish_child(&self) {
        self.default_enabled.store(true, Ordering::SeqCst);
    }
}

pub(super) fn wait_for_child(
    child: &mut std::process::Child,
    state: &CleanupSignalState,
) -> io::Result<i32> {
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(super::command_exit_code(status));
        }
        let signal = state.received.load(Ordering::SeqCst);
        if signal != 0 {
            let _ = child.kill();
            child.wait()?;
            return Ok(128 + signal as i32);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}
