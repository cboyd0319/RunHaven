//! The live terminal session registry (terminal design section 9).
//!
//! This is the session-registry ownership boundary split out of the terminal
//! module: the [`TerminalSessionRegistry`] map, the [`SpawnedTerminal`] control
//! surface it stores, and the shared cap check. It owns registration, the cap
//! race, and teardown; it never touches the live PTY stream bytes (the reader
//! thread lives in [`super::pty`] and the command/flow layer in [`super`]).
//!
//! No-persist invariant: this file holds no stream sink. The
//! `terminal::source_tripwire_tests` source tripwire scans this file's
//! production text alongside `mod.rs` and `pty.rs`, so registry code moved here
//! cannot gain a storage or logging sink without failing the build.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use crate::registry_support::lock_or;

use super::{
    MAX_TERMINAL_SESSIONS, TERMINAL_ALREADY_OPEN, TerminalHandle, validate_terminal_session_id,
};

const REGISTRY_UNAVAILABLE: &str = "Terminal sessions are unavailable.";

/// The one-shot that starts a session's output reader thread. Returns `Err` when
/// the reader thread cannot be spawned, so [`super::register_and_stream`] can
/// tear the half-open session down instead of leaving it registered without a
/// reader.
pub(super) type ReaderStart =
    Box<dyn FnOnce(TerminalSessionRegistry, String) -> Result<(), String> + Send>;

/// What a spawn produces: the control handle for the registry, plus a one-shot
/// that starts the output reader thread. The reader is started only *after* the
/// handle is registered, so a child that exits immediately cannot make the
/// reader remove a registry entry that does not exist yet.
pub(crate) struct SpawnedTerminal {
    pub(super) handle: Arc<dyn TerminalHandle>,
    pub(super) start_reader: Option<ReaderStart>,
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
    pub(super) sessions: Arc<Mutex<HashMap<String, Arc<dyn TerminalHandle>>>>,
}

impl TerminalSessionRegistry {
    /// Best-effort pre-check before the confirmation dialog: reject a duplicate
    /// session or a request past the cap so no dialog is shown for a request
    /// that cannot succeed. [`Self::insert`] re-checks authoritatively.
    pub(super) fn precheck(&self, session_id: &str) -> Result<(), String> {
        validate_terminal_session_id(session_id)?;
        let sessions = lock_or(self.sessions.lock(), REGISTRY_UNAVAILABLE)?;
        check_capacity(&sessions, session_id)
    }

    /// Authoritative reserve: re-check duplicate and cap under the lock, then
    /// insert. On error the caller must shut down the freshly spawned handle.
    pub(super) fn insert(
        &self,
        session_id: &str,
        handle: Arc<dyn TerminalHandle>,
    ) -> Result<(), String> {
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

    pub(super) fn write_input(&self, session_id: &str, data: &[u8]) -> Result<usize, String> {
        self.handle(session_id)?.write_input(data)
    }

    pub(super) fn resize(&self, session_id: &str, cols: u16, rows: u16) -> Result<(), String> {
        self.handle(session_id)?.resize(cols, rows)
    }

    /// Remove and return the handle for `session_id`, if present. Used by the
    /// reader thread on EOF; dropping the returned handle drops the master.
    pub(super) fn remove(&self, session_id: &str) -> Option<Arc<dyn TerminalHandle>> {
        let mut sessions = self.sessions.lock().ok()?;
        sessions.remove(session_id)
    }

    /// Explicit close: remove the entry and kill its child.
    pub(super) fn close(&self, session_id: &str) -> Result<(), String> {
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
