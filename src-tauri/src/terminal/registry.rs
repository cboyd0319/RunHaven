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
    Box<dyn FnOnce(TerminalSessionRegistry, String, u64) -> Result<(), String> + Send>;

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

enum TerminalEntry {
    Pending {
        generation: u64,
    },
    Live {
        generation: u64,
        handle: Arc<dyn TerminalHandle>,
    },
}

#[derive(Default)]
struct RegistryState {
    sessions: HashMap<String, TerminalEntry>,
    next_generation: u64,
}

/// An in-progress open reserved before native confirmation or process spawn.
/// Dropping it removes only its own generation. Explicit close can remove the
/// pending entry first, in which case a later activation fails and the newly
/// spawned child is shut down.
pub(super) struct TerminalReservation {
    registry: TerminalSessionRegistry,
    session_id: String,
    generation: u64,
    armed: bool,
}

impl TerminalReservation {
    pub(super) fn session_id(&self) -> &str {
        &self.session_id
    }

    pub(super) fn activate(mut self, handle: Arc<dyn TerminalHandle>) -> Result<u64, String> {
        self.registry
            .activate(&self.session_id, self.generation, handle)?;
        self.armed = false;
        Ok(self.generation)
    }
}

impl Drop for TerminalReservation {
    fn drop(&mut self) {
        if self.armed {
            self.registry
                .remove_pending(&self.session_id, self.generation);
        }
    }
}

/// Registry of pending and live terminal sessions, keyed by session id (which
/// is the run id for an attach). Mirrors `SessionInputRegistry`: a poisoned lock
/// fails closed through [`lock_or`], and every mutation is bounded.
#[derive(Clone, Default)]
pub(crate) struct TerminalSessionRegistry {
    state: Arc<Mutex<RegistryState>>,
}

impl TerminalSessionRegistry {
    #[cfg(test)]
    pub(super) fn poison_for_test(&self) {
        let _guard = self.state.lock().expect("hold registry lock");
        panic!("poison the registry mutex");
    }

    #[cfg(test)]
    pub(super) fn is_poisoned_for_test(&self) -> bool {
        self.state.lock().is_err()
    }

    /// Reserve a generation before native confirmation and process spawn. The
    /// reservation counts toward the cap and prevents duplicate opens. This is
    /// authoritative, so close/app teardown can cancel a slow pending open.
    pub(super) fn reserve(&self, session_id: &str) -> Result<TerminalReservation, String> {
        validate_terminal_session_id(session_id)?;
        let mut state = lock_or(self.state.lock(), REGISTRY_UNAVAILABLE)?;
        check_capacity(&state.sessions, session_id)?;
        state.next_generation = state
            .next_generation
            .checked_add(1)
            .ok_or_else(|| REGISTRY_UNAVAILABLE.to_string())?;
        let generation = state.next_generation;
        state.sessions.insert(
            session_id.to_string(),
            TerminalEntry::Pending { generation },
        );
        Ok(TerminalReservation {
            registry: self.clone(),
            session_id: session_id.to_string(),
            generation,
            armed: true,
        })
    }

    fn activate(
        &self,
        session_id: &str,
        generation: u64,
        handle: Arc<dyn TerminalHandle>,
    ) -> Result<(), String> {
        let mut state = lock_or(self.state.lock(), REGISTRY_UNAVAILABLE)?;
        match state.sessions.get(session_id) {
            Some(TerminalEntry::Pending {
                generation: pending,
            }) if *pending == generation => {
                state.sessions.insert(
                    session_id.to_string(),
                    TerminalEntry::Live { generation, handle },
                );
                Ok(())
            }
            _ => Err("This terminal was closed before it finished opening.".to_string()),
        }
    }

    /// The handle for `session_id`, cloned out so the PTY write/resize syscall
    /// runs without holding the registry lock.
    fn handle(&self, session_id: &str) -> Result<Arc<dyn TerminalHandle>, String> {
        validate_terminal_session_id(session_id)?;
        let state = lock_or(self.state.lock(), REGISTRY_UNAVAILABLE)?;
        state
            .sessions
            .get(session_id)
            .and_then(|entry| match entry {
                TerminalEntry::Live { handle, .. } => Some(handle.clone()),
                TerminalEntry::Pending { .. } => None,
            })
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
        let mut state = self.state.lock().ok()?;
        match state.sessions.remove(session_id) {
            Some(TerminalEntry::Live { handle, .. }) => Some(handle),
            _ => None,
        }
    }

    /// Remove a live entry only when it is still the reader's generation. A
    /// delayed reader from an older session can never remove a reopened session
    /// that happens to reuse the same id.
    pub(super) fn remove_if_generation(
        &self,
        session_id: &str,
        generation: u64,
    ) -> Option<Arc<dyn TerminalHandle>> {
        let mut state = self.state.lock().ok()?;
        let matches = matches!(
            state.sessions.get(session_id),
            Some(TerminalEntry::Live {
                generation: current,
                ..
            }) if *current == generation
        );
        if !matches {
            return None;
        }
        match state.sessions.remove(session_id) {
            Some(TerminalEntry::Live { handle, .. }) => Some(handle),
            _ => None,
        }
    }

    fn remove_pending(&self, session_id: &str, generation: u64) {
        let Ok(mut state) = self.state.lock() else {
            return;
        };
        if matches!(
            state.sessions.get(session_id),
            Some(TerminalEntry::Pending {
                generation: current
            }) if *current == generation
        ) {
            state.sessions.remove(session_id);
        }
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
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .sessions
            .drain()
            .filter_map(|(_, entry)| match entry {
                TerminalEntry::Live { handle, .. } => Some(handle),
                TerminalEntry::Pending { .. } => None,
            })
            .collect();
        for handle in handles {
            handle.shutdown();
        }
    }
}

fn check_capacity(
    sessions: &HashMap<String, TerminalEntry>,
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
