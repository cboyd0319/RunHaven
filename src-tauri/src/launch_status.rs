use std::sync::{Arc, Mutex};

use runhaven_core::provider::observability::utc_timestamp;

use crate::contracts::LaunchStatusResponse;
use crate::registry_support::{BoundedMap, lock_or, strip_control_chars, validate_registry_id};

const MAX_LAUNCH_STATUSES: usize = 50;
const REGISTRY_UNAVAILABLE: &str = "launch status registry is unavailable";

#[derive(Clone, Default)]
pub(crate) struct LaunchStatusRegistry {
    state: Arc<Mutex<BoundedMap<LaunchStatusResponse>>>,
}

impl LaunchStatusRegistry {
    pub(crate) fn mark_starting(
        &self,
        run_id: &str,
        agent: &str,
        session: &str,
    ) -> Result<(), String> {
        self.set_status(
            run_id,
            "starting",
            "Starting",
            format!(
                "RunHaven accepted the {agent} launch for session {session}. Waiting for the managed session."
            ),
        )
    }

    pub(crate) fn mark_running(&self, run_id: &str) -> Result<(), String> {
        self.set_status(
            run_id,
            "running",
            "Running",
            "The managed session is running.".to_string(),
        )
    }

    pub(crate) fn mark_finished(&self, run_id: &str, return_code: i32) -> Result<(), String> {
        let state = if return_code == 0 {
            "finished"
        } else {
            "failed"
        };
        let label = if return_code == 0 {
            "Finished"
        } else {
            "Failed"
        };
        self.set_status(
            run_id,
            state,
            label,
            format!("The session exited with code {return_code}."),
        )
    }

    pub(crate) fn mark_failed(&self, run_id: &str) -> Result<(), String> {
        self.set_status(
            run_id,
            "failed",
            "Failed",
            "RunHaven could not start or monitor this launch. Open diagnostics or try again."
                .to_string(),
        )
    }

    pub(crate) fn get(&self, run_id: &str) -> Result<LaunchStatusResponse, String> {
        validate_registry_id(run_id)?;
        let state = lock_or(self.state.lock(), REGISTRY_UNAVAILABLE)?;
        state
            .get(run_id)
            .cloned()
            .ok_or_else(|| "launch status not found".to_string())
    }

    fn set_status(
        &self,
        run_id: &str,
        state_name: &str,
        label: &str,
        detail: String,
    ) -> Result<(), String> {
        validate_registry_id(run_id)?;
        let mut state = lock_or(self.state.lock(), REGISTRY_UNAVAILABLE)?;
        state.insert_tracked(
            run_id,
            LaunchStatusResponse {
                run_id: run_id.to_string(),
                state: state_name.to_string(),
                label: label.to_string(),
                detail: strip_control_chars(&detail),
                updated_at: utc_timestamp(),
            },
        );
        state.prune(MAX_LAUNCH_STATUSES);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_status_registry_tracks_state_without_control_chars() {
        let registry = LaunchStatusRegistry::default();

        registry
            .mark_starting("launch-status-run", "codex", "project\u{1b}")
            .expect("starting");
        let starting = registry.get("launch-status-run").expect("status");
        assert_eq!(starting.state, "starting");
        assert!(!starting.detail.contains('\u{1b}'));

        registry.mark_running("launch-status-run").expect("running");
        assert_eq!(
            registry.get("launch-status-run").expect("status").state,
            "running"
        );

        registry
            .mark_finished("launch-status-run", 1)
            .expect("finished");
        let failed = registry.get("launch-status-run").expect("status");
        assert_eq!(failed.state, "failed");
        assert!(failed.detail.contains("code 1"));
    }
}
