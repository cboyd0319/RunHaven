use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use runhaven_core::support::validators::validate_run_id;

/// Byte bound on a single session-input send. Must stay in sync with the
/// command-layer bound in `commands::managed_session_input`, which references
/// this constant so the early-reject message and the transport enforcement
/// cannot drift apart.
pub(crate) const MAX_SESSION_INPUT_BYTES: usize = 8 * 1024;

/// Byte bound on a session id used as a registry key. Both this bound and the
/// command-layer `validation::MAX_RUN_ID_LEN` are 128 bytes and must stay in
/// sync: the command validates a session id up to that length and the registry
/// keys on the same string, so a divergence would let the command accept an id
/// the registry then rejects.
const MAX_SESSION_ID_BYTES: usize = 128;

type SessionInputWriter = dyn Fn(&[u8]) -> Result<usize, String> + Send + Sync + 'static;

#[derive(Clone)]
struct RegisteredSessionInput {
    id: u64,
    run_id: String,
    writer: Arc<SessionInputWriter>,
}

#[derive(Clone, Default)]
pub struct SessionInputRegistry {
    writers: Arc<Mutex<HashMap<String, RegisteredSessionInput>>>,
    next_writer_id: Arc<AtomicU64>,
    input_in_flight: Arc<AtomicBool>,
}

pub(crate) struct SessionInputFlight {
    input_in_flight: Arc<AtomicBool>,
}

impl Drop for SessionInputFlight {
    fn drop(&mut self) {
        self.input_in_flight.store(false, Ordering::Release);
    }
}

impl SessionInputRegistry {
    pub(crate) fn begin_input_send(&self) -> Result<SessionInputFlight, String> {
        self.input_in_flight
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| {
                "Another message is waiting for confirmation. Finish or cancel it, then try again."
                    .to_string()
            })?;
        Ok(SessionInputFlight {
            input_in_flight: Arc::clone(&self.input_in_flight),
        })
    }

    pub fn register_writer(
        &self,
        session_id: &str,
        run_id: &str,
        writer: impl Fn(&[u8]) -> Result<usize, String> + Send + Sync + 'static,
    ) -> Result<(), String> {
        validate_session_id(session_id)?;
        validate_run_id(run_id).map_err(|error| error.to_string())?;
        let mut writers = self
            .writers
            .lock()
            .map_err(|_| "session input registry is unavailable".to_string())?;
        let id = self.next_writer_id.fetch_add(1, Ordering::Relaxed);
        writers.insert(
            session_id.to_string(),
            RegisteredSessionInput {
                id,
                run_id: run_id.to_string(),
                writer: Arc::new(writer),
            },
        );
        Ok(())
    }

    pub fn unregister(&self, session_id: &str) -> Result<(), String> {
        validate_session_id(session_id)?;
        let mut writers = self
            .writers
            .lock()
            .map_err(|_| "session input registry is unavailable".to_string())?;
        writers.remove(session_id);
        Ok(())
    }

    pub fn write_exact(
        &self,
        session_id: &str,
        expected_run_id: &str,
        expected_writer_id: u64,
        input: &str,
    ) -> Result<usize, String> {
        validate_session_id(session_id)?;
        validate_run_id(expected_run_id).map_err(|error| error.to_string())?;
        if input.len() > MAX_SESSION_INPUT_BYTES {
            return Err(format!(
                "This message is too long. Shorten it to {MAX_SESSION_INPUT_BYTES} bytes or less, then try again."
            ));
        }

        let registered = {
            let writers = self
                .writers
                .lock()
                .map_err(|_| "session input registry is unavailable".to_string())?;
            writers.get(session_id).cloned()
        }
        .ok_or_else(|| "session input transport not found".to_string())?;

        if registered.run_id != expected_run_id || registered.id != expected_writer_id {
            return Err(
                "The session changed before RunHaven could send the message. Nothing was sent. Check the session status, then try again."
                    .to_string(),
            );
        }

        (registered.writer)(input.as_bytes()).map_err(|_| {
            "RunHaven couldn’t send this message. Nothing was sent. Try again.".to_string()
        })
    }

    pub fn writer_identity(&self, session_id: &str, expected_run_id: &str) -> Result<u64, String> {
        validate_session_id(session_id)?;
        validate_run_id(expected_run_id).map_err(|error| error.to_string())?;
        let writers = self
            .writers
            .lock()
            .map_err(|_| "session input registry is unavailable".to_string())?;
        let registered = writers
            .get(session_id)
            .ok_or_else(|| "session input transport not found".to_string())?;
        if registered.run_id != expected_run_id {
            return Err("session input transport no longer matches the active run".to_string());
        }
        Ok(registered.id)
    }

    pub fn has_writer(&self, session_id: &str, expected_run_id: &str) -> Result<bool, String> {
        validate_session_id(session_id)?;
        validate_run_id(expected_run_id).map_err(|error| error.to_string())?;
        let writers = self
            .writers
            .lock()
            .map_err(|_| "session input registry is unavailable".to_string())?;
        Ok(writers
            .get(session_id)
            .is_some_and(|registered| registered.run_id == expected_run_id))
    }
}

fn validate_session_id(session_id: &str) -> Result<(), String> {
    if session_id.len() > MAX_SESSION_ID_BYTES {
        return Err(format!(
            "session id is too long; maximum is {MAX_SESSION_ID_BYTES} bytes"
        ));
    }
    validate_run_id(session_id).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests;
