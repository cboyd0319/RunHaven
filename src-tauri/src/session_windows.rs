//! Registry for per-session monitor windows (navigation spec section 7).
//!
//! Security boundary: monitor windows are read-only viewers, never a second
//! control surface. This module owns the three rules that keep them that way:
//!
//! 1. Window labels are never user-controlled. A monitor window's label is
//!    `session-<n>` from a process-local counter; no byte of a session id,
//!    title, or path ever reaches a label. The `session-monitor` capability
//!    matches labels with the `session-*` glob, so a label built from
//!    attacker-chosen text could otherwise steer capability matching.
//! 2. The mapping from window label to watched session lives here, in Rust.
//!    A monitor WebView asks which session it shows
//!    (`get_session_window_target`) and Tauri supplies the calling window's
//!    label from the invocation context, so a WebView cannot claim another
//!    window's identity.
//! 3. [`authorize_session_read`] pins every session-scoped read command to
//!    the one session a monitor window was opened for. Only the main window
//!    may read arbitrary session ids; any other label is denied unless the
//!    registry maps it to exactly the requested session (fail closed,
//!    including for unknown future labels).
//!
//! Concurrent monitor windows are capped at [`MAX_SESSION_WINDOWS`]. Entries
//! for closed windows are pruned lazily on the next open attempt using a
//! liveness probe supplied by the command layer, so a stale entry can only
//! ever waste one slot until then, never grant anything: its window is gone
//! and nothing else can invoke with its label.

use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::MAIN_WINDOW_LABEL;
use crate::registry_support::lock_or;

/// Label prefix for monitor windows. Must stay in lockstep with the
/// `session-*` window glob in `capabilities/session-monitor.json` and the
/// exclusions in `lib.rs` (window-state filter, main-close teardown).
pub(crate) const SESSION_WINDOW_LABEL_PREFIX: &str = "session-";

/// Cap on concurrently open monitor windows. Watching runs side by side needs
/// a few windows, not many; each one is a full WebView, so the cap bounds
/// both resource use and the number of live IPC origins.
pub(crate) const MAX_SESSION_WINDOWS: usize = 6;

const REGISTRY_UNAVAILABLE: &str = "Session window records are unavailable.";

struct SessionWindowEntry {
    label: String,
    session_id: String,
}

/// What the command layer should do for an open request: focus the live
/// window that already shows the session, or create a window with the
/// freshly reserved label.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SessionWindowSlot {
    FocusExisting(String),
    Create(String),
}

#[derive(Default)]
pub(crate) struct SessionWindowRegistry {
    entries: Mutex<Vec<SessionWindowEntry>>,
    counter: AtomicU64,
    /// Serializes reserve-then-build in `open_session_window` so a duplicate
    /// request cannot race an unfinished window build. Never taken on the
    /// main thread, so the build's main-thread dispatch cannot deadlock on it.
    open_lock: Mutex<()>,
}

impl SessionWindowRegistry {
    /// Guard that serializes whole open operations (reservation plus window
    /// build). Lookups and reads never take this lock.
    pub(crate) fn serialize_opens(&self) -> Result<std::sync::MutexGuard<'_, ()>, String> {
        lock_or(self.open_lock.lock(), REGISTRY_UNAVAILABLE)
    }

    /// Reserve a window slot for `session_id`. `window_is_open` reports
    /// whether a label still has a live window; entries whose window is gone
    /// are pruned first, so closed monitors free their slots. The entry is
    /// recorded before the caller builds the window; on build failure the
    /// caller must [`Self::release`] the label.
    pub(crate) fn reserve(
        &self,
        session_id: &str,
        window_is_open: impl Fn(&str) -> bool,
    ) -> Result<SessionWindowSlot, String> {
        let mut entries = lock_or(self.entries.lock(), REGISTRY_UNAVAILABLE)?;
        entries.retain(|entry| window_is_open(&entry.label));
        if let Some(entry) = entries.iter().find(|entry| entry.session_id == session_id) {
            return Ok(SessionWindowSlot::FocusExisting(entry.label.clone()));
        }
        if entries.len() >= MAX_SESSION_WINDOWS {
            return Err(format!(
                "You already have {MAX_SESSION_WINDOWS} session windows open. \
                 Close one before opening another."
            ));
        }
        // The counter, not the session id, names the window: labels stay
        // free of user-influenced bytes and are unique for the process
        // lifetime, so a label can never be reused for a different session
        // while any consumer still remembers it.
        let label = format!(
            "{SESSION_WINDOW_LABEL_PREFIX}{}",
            self.counter.fetch_add(1, Ordering::Relaxed) + 1
        );
        entries.push(SessionWindowEntry {
            label: label.clone(),
            session_id: session_id.to_string(),
        });
        Ok(SessionWindowSlot::Create(label))
    }

    /// Drop the reservation for `label` after a failed window build.
    pub(crate) fn release(&self, label: &str) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.retain(|entry| entry.label != label);
        }
    }

    /// The session id registered for `label`, or an error when the label is
    /// not a registered monitor window (fail closed).
    pub(crate) fn session_for_label(&self, label: &str) -> Result<String, String> {
        let entries = lock_or(self.entries.lock(), REGISTRY_UNAVAILABLE)?;
        entries
            .iter()
            .find(|entry| entry.label == label)
            .map(|entry| entry.session_id.clone())
            .ok_or_else(|| "No session is registered for this window.".to_string())
    }
}

/// Per-window scope check for session-keyed read commands. The main window
/// may read any session (it owns selection and controls). A monitor window
/// may read only the session it was opened for. Every other label, present or
/// future, is denied.
pub(crate) fn authorize_session_read(
    registry: &SessionWindowRegistry,
    window_label: &str,
    requested_session_id: &str,
) -> Result<(), String> {
    if window_label == MAIN_WINDOW_LABEL {
        return Ok(());
    }
    let registered = registry.session_for_label(window_label)?;
    if registered == requested_session_id {
        Ok(())
    } else {
        Err("This window can only read its own session.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_SESSION_WINDOWS, SESSION_WINDOW_LABEL_PREFIX, SessionWindowRegistry, SessionWindowSlot,
        authorize_session_read,
    };

    fn create_label(registry: &SessionWindowRegistry, session_id: &str) -> String {
        match registry.reserve(session_id, |_| true) {
            Ok(SessionWindowSlot::Create(label)) => label,
            other => panic!("expected a fresh label, got {other:?}"),
        }
    }

    #[test]
    fn open_serialization_guard_locks_and_releases() {
        let registry = SessionWindowRegistry::default();
        {
            let _guard = registry.serialize_opens().expect("open lock");
        }
        // Reservation locks are independent of the open lock and still work.
        assert!(registry.reserve("run-a", |_| true).is_ok());
    }

    #[test]
    fn labels_are_counter_named_and_carry_no_session_bytes() {
        let registry = SessionWindowRegistry::default();
        let hostile_id = "../--; DROP session-*\u{1b}[2J";
        let label = create_label(&registry, hostile_id);
        assert_eq!(label, "session-1");
        let suffix = label
            .strip_prefix(SESSION_WINDOW_LABEL_PREFIX)
            .expect("monitor label prefix");
        assert!(suffix.chars().all(|c| c.is_ascii_digit()));
        assert_eq!(create_label(&registry, "another"), "session-2");
    }

    #[test]
    fn second_open_for_the_same_session_focuses_instead_of_creating() {
        let registry = SessionWindowRegistry::default();
        let label = create_label(&registry, "run-a");
        assert_eq!(
            registry.reserve("run-a", |_| true),
            Ok(SessionWindowSlot::FocusExisting(label))
        );
    }

    #[test]
    fn open_windows_are_capped_and_the_error_names_the_limit() {
        let registry = SessionWindowRegistry::default();
        for index in 0..MAX_SESSION_WINDOWS {
            create_label(&registry, &format!("run-{index}"));
        }
        let error = registry
            .reserve("run-overflow", |_| true)
            .expect_err("cap must hold");
        assert!(error.contains(&MAX_SESSION_WINDOWS.to_string()));
    }

    #[test]
    fn closed_windows_free_their_slots_but_labels_are_never_reused() {
        let registry = SessionWindowRegistry::default();
        for index in 0..MAX_SESSION_WINDOWS {
            create_label(&registry, &format!("run-{index}"));
        }
        // Every window has been closed: the next open prunes and succeeds,
        // and the counter keeps advancing so no label is ever reissued.
        let label = match registry.reserve("run-fresh", |_| false) {
            Ok(SessionWindowSlot::Create(label)) => label,
            other => panic!("expected a fresh label, got {other:?}"),
        };
        assert_eq!(
            label,
            format!("{SESSION_WINDOW_LABEL_PREFIX}{}", MAX_SESSION_WINDOWS + 1)
        );
    }

    #[test]
    fn released_reservation_frees_the_slot_after_a_failed_build() {
        let registry = SessionWindowRegistry::default();
        let label = create_label(&registry, "run-a");
        registry.release(&label);
        assert!(registry.session_for_label(&label).is_err());
        // The same session reserves again instead of focusing a dead label.
        assert!(matches!(
            registry.reserve("run-a", |_| true),
            Ok(SessionWindowSlot::Create(_))
        ));
    }

    #[test]
    fn target_lookup_fails_closed_for_unknown_labels() {
        let registry = SessionWindowRegistry::default();
        let error = registry
            .session_for_label("session-99")
            .expect_err("unknown label");
        assert!(error.contains("No session is registered"));
        assert!(registry.session_for_label("main").is_err());
    }

    #[test]
    fn target_lookup_returns_the_registered_session() {
        let registry = SessionWindowRegistry::default();
        let label = create_label(&registry, "run-a");
        assert_eq!(registry.session_for_label(&label), Ok("run-a".to_string()));
    }

    #[test]
    fn main_window_reads_any_session() {
        let registry = SessionWindowRegistry::default();
        assert_eq!(
            authorize_session_read(&registry, "main", "anything"),
            Ok(())
        );
    }

    #[test]
    fn monitor_window_reads_only_its_own_session() {
        let registry = SessionWindowRegistry::default();
        let label = create_label(&registry, "run-a");
        assert_eq!(authorize_session_read(&registry, &label, "run-a"), Ok(()));
        let denied = authorize_session_read(&registry, &label, "run-b").expect_err("cross-session");
        assert!(denied.contains("its own session"));
    }

    #[test]
    fn unknown_and_future_labels_are_denied_by_default() {
        let registry = SessionWindowRegistry::default();
        assert!(authorize_session_read(&registry, "session-7", "run-a").is_err());
        assert!(authorize_session_read(&registry, "settings", "run-a").is_err());
        assert!(authorize_session_read(&registry, "", "run-a").is_err());
    }
}
