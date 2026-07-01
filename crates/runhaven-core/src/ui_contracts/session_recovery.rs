//! Managed-session recovery reconcile.
//!
//! The list read model is pure over records; it cannot tell that a "running"
//! marker points at a container Apple `container` has already reaped. The command
//! layer confirms that against the authoritative container-existence check and
//! passes the stale run ids here so an orphaned session is shown for repair and
//! fails closed (only a `monitor` action accepts input).

use std::collections::BTreeSet;

use super::{ManagedSessionListData, ManagedSessionNextActionData};

impl ManagedSessionListData {
    /// Downgrade active sessions whose container is proven gone to a recovery
    /// state. `stale_run_ids` must come from the authoritative container check
    /// (the same one marker repair uses), so `recover` is only shown for a marker
    /// repair has proven removable. If the container is recreated between the poll
    /// and the repair click, repair keeps the marker and the flag self-heals on
    /// the next refresh; nothing is removed without a proven-missing container.
    pub fn mark_stale_sessions(&mut self, stale_run_ids: &BTreeSet<String>) {
        for session in &mut self.sessions {
            if matches!(session.lifecycle.as_str(), "running" | "starting")
                && stale_run_ids.contains(&session.run_id)
            {
                session.status_label = "Not responding".to_string();
                session.next_action = recover_next_action();
            }
        }
    }
}

fn recover_next_action() -> ManagedSessionNextActionData {
    ManagedSessionNextActionData {
        kind: "recover".to_string(),
        label: "Repair session".to_string(),
        detail:
            "This session's container is no longer running. Repair the stale marker to clear it."
                .to_string(),
    }
}
