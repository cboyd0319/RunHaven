use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use runhaven_core::provider::observability::utc_timestamp;
use runhaven_core::support::validators::validate_run_id;

use crate::contracts::LaunchStatusResponse;

const MAX_LAUNCH_STATUSES: usize = 50;

#[derive(Clone, Default)]
pub(crate) struct LaunchStatusRegistry {
    state: Arc<Mutex<LaunchStatusState>>,
}

#[derive(Default)]
struct LaunchStatusState {
    statuses: HashMap<String, LaunchStatusResponse>,
    order: VecDeque<String>,
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
        validate_launch_run_id(run_id)?;
        let state = self
            .state
            .lock()
            .map_err(|_| "launch status registry is unavailable".to_string())?;
        state
            .statuses
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
        validate_launch_run_id(run_id)?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| "launch status registry is unavailable".to_string())?;
        if !state.statuses.contains_key(run_id) {
            state.order.push_back(run_id.to_string());
        }
        state.statuses.insert(
            run_id.to_string(),
            LaunchStatusResponse {
                run_id: run_id.to_string(),
                state: state_name.to_string(),
                label: label.to_string(),
                detail: strip_control_chars(&detail),
                updated_at: utc_timestamp(),
            },
        );
        prune_statuses(&mut state);
        Ok(())
    }
}

fn validate_launch_run_id(run_id: &str) -> Result<(), String> {
    validate_run_id(run_id).map_err(|error| error.to_string())
}

fn strip_control_chars(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_control()).collect()
}

fn prune_statuses(state: &mut LaunchStatusState) {
    while state.statuses.len() > MAX_LAUNCH_STATUSES {
        let Some(run_id) = state.order.pop_front() else {
            return;
        };
        state.statuses.remove(&run_id);
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
