use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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

/// Cumulative bytes a single native input approval covers before RunHaven asks
/// for a fresh confirmation. The one-click grant is deliberately coarse: it lets
/// a normal back-and-forth continue without nagging, but a long-lived session
/// cannot be driven indefinitely after one approval.
const SESSION_INPUT_GRANT_MAX_BYTES: usize = 256 * 1024;

/// Wall-clock lifetime of a single native input approval. Whichever limit is
/// reached first (this or [`SESSION_INPUT_GRANT_MAX_BYTES`]) forces a fresh
/// confirmation. The time bound catches a slow trickle that never reaches the
/// byte budget.
const SESSION_INPUT_GRANT_TTL: Duration = Duration::from_secs(30 * 60);

type SessionInputWriter = dyn Fn(&[u8]) -> Result<usize, String> + Send + Sync + 'static;

#[derive(Clone)]
struct RegisteredSessionInput {
    run_id: String,
    writer: Arc<SessionInputWriter>,
}

/// A single native confirmation grant, bound to the exact run id it was approved
/// for and metered so it cannot authorize input forever after one click.
struct SessionInputGrant {
    run_id: String,
    granted_at: Instant,
    bytes_written: usize,
}

#[derive(Clone, Default)]
pub struct SessionInputRegistry {
    writers: Arc<Mutex<HashMap<String, RegisteredSessionInput>>>,
    grants: Arc<Mutex<HashMap<String, SessionInputGrant>>>,
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
        let mut grants = self
            .grants
            .lock()
            .map_err(|_| "session input registry is unavailable".to_string())?;
        grants.remove(session_id);
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

        let written = (registered.writer)(input.as_bytes())?;
        self.charge_input_grant(session_id, expected_run_id, input.len())?;
        Ok(written)
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
        let mut grants = self
            .grants
            .lock()
            .map_err(|_| "session input registry is unavailable".to_string())?;
        grants.insert(
            session_id.to_string(),
            SessionInputGrant {
                run_id: expected_run_id.to_string(),
                granted_at: Instant::now(),
                bytes_written: 0,
            },
        );
        Ok(())
    }

    pub fn input_approved(&self, session_id: &str, expected_run_id: &str) -> Result<bool, String> {
        validate_session_id(session_id)?;
        validate_run_id(expected_run_id).map_err(|error| error.to_string())?;
        let grants = self
            .grants
            .lock()
            .map_err(|_| "session input registry is unavailable".to_string())?;
        Ok(grants.get(session_id).is_some_and(|grant| {
            grant.run_id == expected_run_id
                && grant_within_budget(grant.bytes_written, grant.granted_at.elapsed())
        }))
    }

    /// Add `bytes` to the active grant for `session_id` when it still matches the
    /// run. Best-effort accounting: a send with no live grant (grants are cleared
    /// on unregister) simply does not accumulate, and the command layer will ask
    /// for a fresh approval before the next send.
    fn charge_input_grant(
        &self,
        session_id: &str,
        expected_run_id: &str,
        bytes: usize,
    ) -> Result<(), String> {
        let mut grants = self
            .grants
            .lock()
            .map_err(|_| "session input registry is unavailable".to_string())?;
        if let Some(grant) = grants.get_mut(session_id)
            && grant.run_id == expected_run_id
        {
            grant.bytes_written = grant.bytes_written.saturating_add(bytes);
        }
        Ok(())
    }

    /// Force the active grant for `session_id` past its wall-clock TTL so the
    /// time branch of [`input_approved`] can be exercised deterministically.
    #[cfg(test)]
    fn expire_grant_for_test(&self, session_id: &str) {
        let mut grants = self.grants.lock().expect("session input grant lock");
        if let Some(grant) = grants.get_mut(session_id) {
            grant.granted_at = grant
                .granted_at
                .checked_sub(SESSION_INPUT_GRANT_TTL + Duration::from_secs(1))
                .unwrap_or(grant.granted_at);
        }
    }
}

/// A grant is spent once it has carried its byte budget or aged out, whichever
/// happens first. Kept as a free function so both boundaries are unit-testable
/// without a real clock.
fn grant_within_budget(bytes_written: usize, age: Duration) -> bool {
    bytes_written < SESSION_INPUT_GRANT_MAX_BYTES && age < SESSION_INPUT_GRANT_TTL
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
mod tests {
    use super::*;

    #[test]
    fn grant_budget_boundary_covers_bytes_and_ttl() {
        assert!(grant_within_budget(0, Duration::ZERO));
        assert!(grant_within_budget(
            SESSION_INPUT_GRANT_MAX_BYTES - 1,
            Duration::from_secs(1)
        ));
        // Reaching the byte budget spends the grant.
        assert!(!grant_within_budget(
            SESSION_INPUT_GRANT_MAX_BYTES,
            Duration::from_secs(1)
        ));
        // Reaching the TTL spends the grant even with bytes to spare.
        assert!(!grant_within_budget(0, SESSION_INPUT_GRANT_TTL));
    }

    #[test]
    fn input_grant_requires_reconfirm_past_byte_budget() {
        let registry = SessionInputRegistry::default();
        registry
            .register_writer("budget-session", "budget-session", |bytes| Ok(bytes.len()))
            .expect("register writer");
        registry
            .approve_input("budget-session", "budget-session")
            .expect("approve input");
        assert!(
            registry
                .input_approved("budget-session", "budget-session")
                .expect("approved fresh grant")
        );

        let chunk = "a".repeat(MAX_SESSION_INPUT_BYTES);
        let mut sent = 0;
        while sent < SESSION_INPUT_GRANT_MAX_BYTES {
            registry
                .write("budget-session", "budget-session", &chunk)
                .expect("send within grant");
            sent += chunk.len();
            if sent < SESSION_INPUT_GRANT_MAX_BYTES {
                assert!(
                    registry
                        .input_approved("budget-session", "budget-session")
                        .expect("still approved under budget"),
                    "grant should remain valid until the byte budget is reached"
                );
            }
        }

        // Cumulative bytes now meet the budget, so a fresh confirmation is required.
        assert!(
            !registry
                .input_approved("budget-session", "budget-session")
                .expect("grant checked"),
            "grant must require reconfirm once the byte budget is spent"
        );

        // A fresh approval restores the grant and resets the meter.
        registry
            .approve_input("budget-session", "budget-session")
            .expect("re-approve input");
        assert!(
            registry
                .input_approved("budget-session", "budget-session")
                .expect("approved after reconfirm")
        );
    }

    #[test]
    fn input_grant_requires_reconfirm_past_ttl() {
        let registry = SessionInputRegistry::default();
        registry
            .approve_input("ttl-session", "ttl-session")
            .expect("approve input");
        assert!(
            registry
                .input_approved("ttl-session", "ttl-session")
                .expect("approved fresh grant")
        );

        registry.expire_grant_for_test("ttl-session");
        assert!(
            !registry
                .input_approved("ttl-session", "ttl-session")
                .expect("grant checked"),
            "grant must require reconfirm once the wall-clock TTL passes"
        );
    }

    #[test]
    fn grant_is_bound_to_run_and_cleared_on_unregister() {
        let registry = SessionInputRegistry::default();
        registry
            .approve_input("bound-session", "bound-run")
            .expect("approve input");
        assert!(
            registry
                .input_approved("bound-session", "bound-run")
                .expect("approved for exact run")
        );
        // A different run id never inherits the grant.
        assert!(
            !registry
                .input_approved("bound-session", "other-run")
                .expect("grant checked")
        );

        registry.unregister("bound-session").expect("unregister");
        assert!(
            !registry
                .input_approved("bound-session", "bound-run")
                .expect("grant checked after unregister"),
            "unregister must clear the grant"
        );
    }
}
