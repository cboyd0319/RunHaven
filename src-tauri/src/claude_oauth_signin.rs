//! In-flight state and background host capture for the Claude Keychain sign-in
//! (design C, Section 4, Slice 4).
//!
//! There is exactly one RunHaven-owned Claude token, so the in-flight state is a
//! singleton: at most one capture runs at a time. The capture (`claude
//! setup-token`) runs on a background thread, exactly like the agent sign-in
//! runner, so the Tauri command never blocks. This module holds ONLY a sanitized
//! phase: an id, a fixed phase, and a path-free failure reason. It never reads,
//! stores, forwards, or logs the token: the token is captured and stored by
//! `runhaven_core::provider::claude_oauth_keychain` on the host and lives only in
//! the Keychain.

use std::sync::{Arc, Mutex};
use std::thread;

use crate::registry_support::lock_or;

const LOCK_MESSAGE: &str = "Claude sign-in state is unavailable";
const ALREADY_SIGNING: &str = "A Claude sign-in is already in progress.";

/// Sanitized in-flight sign-in phase. Combined with Keychain item presence by
/// `get_claude_oauth_status` to produce the status the WebView sees. Carries no
/// token: only an id and, on failure, a fixed path-free reason.
#[derive(Clone, Debug, Default)]
enum Phase {
    #[default]
    Idle,
    Signing {
        sign_in_id: String,
    },
    Failed {
        sign_in_id: String,
        reason: String,
    },
}

/// A read-only snapshot the status command matches against presence.
#[derive(Clone, Debug)]
pub(crate) enum InFlightSnapshot {
    Idle,
    Signing { sign_in_id: String },
    Failed { sign_in_id: String, reason: String },
}

/// Singleton in-flight state for the Claude Keychain sign-in.
#[derive(Clone, Default)]
pub(crate) struct ClaudeOAuthSignInRegistry {
    phase: Arc<Mutex<Phase>>,
}

impl ClaudeOAuthSignInRegistry {
    /// Begin a sign-in. Fails closed if one is already in progress, so a second
    /// concurrent `claude setup-token` can never start.
    pub(crate) fn begin(&self, sign_in_id: &str) -> Result<(), String> {
        let mut phase = lock_or(self.phase.lock(), LOCK_MESSAGE)?;
        if matches!(*phase, Phase::Signing { .. }) {
            return Err(ALREADY_SIGNING.to_string());
        }
        *phase = Phase::Signing {
            sign_in_id: sign_in_id.to_string(),
        };
        Ok(())
    }

    /// Mark the current sign-in finished. Presence then reports `signed_in`.
    /// A stale id (a newer sign-in started) is ignored.
    pub(crate) fn mark_succeeded(&self, sign_in_id: &str) -> Result<(), String> {
        let mut phase = lock_or(self.phase.lock(), LOCK_MESSAGE)?;
        if current_id(&phase) == Some(sign_in_id) {
            *phase = Phase::Idle;
        }
        Ok(())
    }

    /// Mark the current sign-in failed with a fixed, path-free reason. A stale id
    /// is ignored so a late failure cannot clobber a newer attempt.
    pub(crate) fn mark_failed(&self, sign_in_id: &str, reason: String) -> Result<(), String> {
        let mut phase = lock_or(self.phase.lock(), LOCK_MESSAGE)?;
        if current_id(&phase) == Some(sign_in_id) {
            *phase = Phase::Failed {
                sign_in_id: sign_in_id.to_string(),
                reason,
            };
        }
        Ok(())
    }

    pub(crate) fn snapshot(&self) -> Result<InFlightSnapshot, String> {
        let phase = lock_or(self.phase.lock(), LOCK_MESSAGE)?;
        Ok(match &*phase {
            Phase::Idle => InFlightSnapshot::Idle,
            Phase::Signing { sign_in_id } => InFlightSnapshot::Signing {
                sign_in_id: sign_in_id.clone(),
            },
            Phase::Failed { sign_in_id, reason } => InFlightSnapshot::Failed {
                sign_in_id: sign_in_id.clone(),
                reason: reason.clone(),
            },
        })
    }
}

fn current_id(phase: &Phase) -> Option<&str> {
    match phase {
        Phase::Signing { sign_in_id } | Phase::Failed { sign_in_id, .. } => Some(sign_in_id),
        Phase::Idle => None,
    }
}

/// Apply a finished capture result to the in-flight state. The background thread
/// and the hermetic test share this so the `Signing -> SignedIn/Failed`
/// transition has one implementation.
pub(crate) fn run_capture(
    registry: &ClaudeOAuthSignInRegistry,
    sign_in_id: &str,
    capture: impl FnOnce() -> Result<(), String>,
) {
    match capture() {
        Ok(()) => {
            let _ = registry.mark_succeeded(sign_in_id);
        }
        Err(reason) => {
            let _ = registry.mark_failed(sign_in_id, reason);
        }
    }
}

/// Spawn the host capture on a background thread. Returns once the thread is
/// started; the terminal phase is recorded through `registry`. `capture` is a
/// seam so the command passes the real host `claude setup-token` capture and
/// tests inject a fake that never touches the real Keychain or network.
pub(crate) fn spawn_claude_oauth_capture(
    sign_in_id: String,
    registry: ClaudeOAuthSignInRegistry,
    capture: impl FnOnce() -> Result<(), String> + Send + 'static,
) -> Result<(), String> {
    thread::Builder::new()
        .name(format!("runhaven-claude-oauth-{sign_in_id}"))
        .spawn(move || run_capture(&registry, &sign_in_id, capture))
        .map(|_| ())
        .map_err(|error| format!("could not start Claude sign-in: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_refuses_a_second_concurrent_sign_in() {
        let registry = ClaudeOAuthSignInRegistry::default();
        registry.begin("first").expect("first begins");
        let error = registry
            .begin("second")
            .expect_err("second must fail closed");
        assert!(error.contains("already in progress"));
    }

    #[test]
    fn run_capture_transitions_signing_to_signed_in_on_success() {
        // A stored token is presence-derived, so success returns to Idle and the
        // status command reports signed_in from presence.
        let registry = ClaudeOAuthSignInRegistry::default();
        registry.begin("id-1").expect("begin");
        run_capture(&registry, "id-1", || Ok(()));
        assert!(matches!(
            registry.snapshot().expect("snapshot"),
            InFlightSnapshot::Idle
        ));
    }

    #[test]
    fn run_capture_transitions_signing_to_failed_with_path_free_reason() {
        let registry = ClaudeOAuthSignInRegistry::default();
        registry.begin("id-2").expect("begin");
        run_capture(&registry, "id-2", || {
            Err("Claude sign-in did not finish; nothing was saved.".to_string())
        });
        match registry.snapshot().expect("snapshot") {
            InFlightSnapshot::Failed { sign_in_id, reason } => {
                assert_eq!(sign_in_id, "id-2");
                assert_eq!(reason, "Claude sign-in did not finish; nothing was saved.");
                // The reason is fixed copy: no token, no prefix, no host path.
                assert!(!reason.contains("sk-ant-"));
                assert!(!reason.contains('/'));
            }
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn a_stale_capture_result_cannot_clobber_a_newer_sign_in() {
        let registry = ClaudeOAuthSignInRegistry::default();
        registry.begin("old").expect("begin old");
        // A newer sign-in supersedes the old id.
        registry.begin_replacing_for_test("new");
        // The old capture finishing late must not overwrite the new phase.
        run_capture(&registry, "old", || Ok(()));
        assert!(matches!(
            registry.snapshot().expect("snapshot"),
            InFlightSnapshot::Signing { sign_in_id } if sign_in_id == "new"
        ));
    }

    impl ClaudeOAuthSignInRegistry {
        /// Force-replace the phase with a fresh Signing, bypassing the
        /// already-in-progress guard, so a stale-result test can set up a newer
        /// attempt without racing a real thread.
        fn begin_replacing_for_test(&self, sign_in_id: &str) {
            let mut phase = self.phase.lock().expect("lock");
            *phase = Phase::Signing {
                sign_in_id: sign_in_id.to_string(),
            };
        }
    }
}
