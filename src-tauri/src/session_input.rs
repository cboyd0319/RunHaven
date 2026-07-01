use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use runhaven_core::support::validators::validate_run_id;

const MAX_SESSION_ID_BYTES: usize = 128;
const MAX_SESSION_INPUT_BYTES: usize = 8 * 1024;

type SessionInputWriter = dyn Fn(&[u8]) -> Result<usize, String> + Send + Sync + 'static;

#[derive(Clone)]
struct RegisteredSessionInput {
    run_id: String,
    writer: Arc<SessionInputWriter>,
}

#[derive(Clone, Default)]
pub struct SessionInputRegistry {
    writers: Arc<Mutex<HashMap<String, RegisteredSessionInput>>>,
    approved_inputs: Arc<Mutex<HashMap<String, String>>>,
}

impl SessionInputRegistry {
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
        writers.insert(
            session_id.to_string(),
            RegisteredSessionInput {
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
        drop(writers);
        let mut approved_inputs = self
            .approved_inputs
            .lock()
            .map_err(|_| "session input registry is unavailable".to_string())?;
        approved_inputs.remove(session_id);
        Ok(())
    }

    pub fn write(
        &self,
        session_id: &str,
        expected_run_id: &str,
        input: &str,
    ) -> Result<usize, String> {
        validate_session_id(session_id)?;
        validate_run_id(expected_run_id).map_err(|error| error.to_string())?;
        if input.len() > MAX_SESSION_INPUT_BYTES {
            return Err(format!(
                "session input is too long; keep each send at or below {MAX_SESSION_INPUT_BYTES} bytes"
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

        if registered.run_id != expected_run_id {
            return Err("session input transport no longer matches the active run".to_string());
        }

        (registered.writer)(input.as_bytes())
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

    pub fn approve_input(&self, session_id: &str, expected_run_id: &str) -> Result<(), String> {
        validate_session_id(session_id)?;
        validate_run_id(expected_run_id).map_err(|error| error.to_string())?;
        let mut approved_inputs = self
            .approved_inputs
            .lock()
            .map_err(|_| "session input registry is unavailable".to_string())?;
        approved_inputs.insert(session_id.to_string(), expected_run_id.to_string());
        Ok(())
    }

    pub fn input_approved(&self, session_id: &str, expected_run_id: &str) -> Result<bool, String> {
        validate_session_id(session_id)?;
        validate_run_id(expected_run_id).map_err(|error| error.to_string())?;
        let approved_inputs = self
            .approved_inputs
            .lock()
            .map_err(|_| "session input registry is unavailable".to_string())?;
        Ok(approved_inputs
            .get(session_id)
            .is_some_and(|run_id| run_id == expected_run_id))
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
