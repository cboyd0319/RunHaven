use runhaven_core::runtime::active::{kill_active_run, repair_active_run, stop_active_run};
use serde_json::Value;

use super::validation::{MAX_RUN_ID_LEN, validate_text_len};
use crate::contracts::{
    KillManagedSessionRequest, KillManagedSessionResponse, KillRunRequest, KillRunResponse,
    RepairManagedSessionRequest, RepairManagedSessionResponse, RepairRunRequest, RepairRunResponse,
    StopManagedSessionRequest, StopManagedSessionResponse, StopRunRequest, StopRunResponse,
};

#[tauri::command]
pub(crate) fn stop_run(request: StopRunRequest) -> Result<StopRunResponse, String> {
    validate_text_len("run id", &request.run_id, MAX_RUN_ID_LEN)?;
    if !request.confirm_stop {
        return Err("Confirm the stop before stopping this run.".to_string());
    }
    let payload = stop_active_run(&request.run_id).map_err(|error| error.to_string())?;
    stop_run_response(&payload)
}

#[tauri::command]
pub(crate) fn stop_managed_session(
    request: StopManagedSessionRequest,
) -> Result<StopManagedSessionResponse, String> {
    stop_managed_session_with(request, |run_id| {
        stop_run(StopRunRequest {
            run_id: run_id.to_string(),
            confirm_stop: true,
        })
    })
}

fn stop_managed_session_with(
    request: StopManagedSessionRequest,
    stop: impl FnOnce(&str) -> Result<StopRunResponse, String>,
) -> Result<StopManagedSessionResponse, String> {
    if !request.confirm_stop {
        return Err("Confirm the stop before stopping this run.".to_string());
    }
    let (session, run_id) = crate::commands::sessions::managed_session_for_id(&request.session_id)?;
    let result = stop(&run_id)?;
    Ok(StopManagedSessionResponse { session, result })
}

#[cfg(test)]
pub(crate) fn stop_managed_session_for_test(
    request: StopManagedSessionRequest,
    stop: impl FnOnce(&str) -> Result<StopRunResponse, String>,
) -> Result<StopManagedSessionResponse, String> {
    stop_managed_session_with(request, stop)
}

fn stop_run_response(payload: &Value) -> Result<StopRunResponse, String> {
    let run_id = required_string(payload, "run_id")?;
    let container_name = required_string(payload, "container_name")?;
    let return_code = payload
        .get("return_code")
        .and_then(Value::as_i64)
        .ok_or_else(|| "stop payload is missing return_code".to_string())?;
    if return_code != 0 {
        return Err(format!(
            "could not stop run {run_id} ({container_name}); container stop exited {return_code}"
        ));
    }
    Ok(StopRunResponse {
        run_id,
        container_name,
        status: "stop-requested".to_string(),
    })
}

#[tauri::command]
pub(crate) fn kill_run(request: KillRunRequest) -> Result<KillRunResponse, String> {
    validate_text_len("run id", &request.run_id, MAX_RUN_ID_LEN)?;
    if !request.confirm_kill {
        return Err("Confirm the hard-stop before killing this run.".to_string());
    }
    let payload = kill_active_run(&request.run_id).map_err(|error| error.to_string())?;
    kill_run_response(&payload)
}

#[tauri::command]
pub(crate) fn kill_managed_session(
    request: KillManagedSessionRequest,
) -> Result<KillManagedSessionResponse, String> {
    kill_managed_session_with(request, |run_id| {
        kill_run(KillRunRequest {
            run_id: run_id.to_string(),
            confirm_kill: true,
        })
    })
}

fn kill_managed_session_with(
    request: KillManagedSessionRequest,
    kill: impl FnOnce(&str) -> Result<KillRunResponse, String>,
) -> Result<KillManagedSessionResponse, String> {
    if !request.confirm_kill {
        return Err("Confirm the hard-stop before killing this run.".to_string());
    }
    let (session, run_id) = crate::commands::sessions::managed_session_for_id(&request.session_id)?;
    let result = kill(&run_id)?;
    Ok(KillManagedSessionResponse { session, result })
}

#[cfg(test)]
pub(crate) fn kill_managed_session_for_test(
    request: KillManagedSessionRequest,
    kill: impl FnOnce(&str) -> Result<KillRunResponse, String>,
) -> Result<KillManagedSessionResponse, String> {
    kill_managed_session_with(request, kill)
}

fn kill_run_response(payload: &Value) -> Result<KillRunResponse, String> {
    let run_id = required_string(payload, "run_id")?;
    let container_name = required_string(payload, "container_name")?;
    let return_code = payload
        .get("return_code")
        .and_then(Value::as_i64)
        .ok_or_else(|| "kill payload is missing return_code".to_string())?;
    if return_code != 0 {
        return Err(format!(
            "could not hard-stop run {run_id} ({container_name}); container kill exited {return_code}"
        ));
    }
    Ok(KillRunResponse {
        run_id,
        container_name,
        status: "kill-requested".to_string(),
    })
}

#[tauri::command]
pub(crate) fn repair_run(request: RepairRunRequest) -> Result<RepairRunResponse, String> {
    validate_text_len("run id", &request.run_id, MAX_RUN_ID_LEN)?;
    if !request.confirm_repair {
        return Err("Confirm the repair before clearing a stale marker.".to_string());
    }
    let payload = repair_active_run(&request.run_id).map_err(|error| error.to_string())?;
    repair_run_response(&payload)
}

#[tauri::command]
pub(crate) fn repair_managed_session(
    request: RepairManagedSessionRequest,
) -> Result<RepairManagedSessionResponse, String> {
    repair_managed_session_with(request, |run_id| {
        repair_run(RepairRunRequest {
            run_id: run_id.to_string(),
            confirm_repair: true,
        })
    })
}

fn repair_managed_session_with(
    request: RepairManagedSessionRequest,
    repair: impl FnOnce(&str) -> Result<RepairRunResponse, String>,
) -> Result<RepairManagedSessionResponse, String> {
    if !request.confirm_repair {
        return Err("Confirm the repair before clearing a stale marker.".to_string());
    }
    let (session, run_id) = crate::commands::sessions::managed_session_for_id(&request.session_id)?;
    let result = repair(&run_id)?;
    Ok(RepairManagedSessionResponse { session, result })
}

#[cfg(test)]
pub(crate) fn repair_managed_session_for_test(
    request: RepairManagedSessionRequest,
    repair: impl FnOnce(&str) -> Result<RepairRunResponse, String>,
) -> Result<RepairManagedSessionResponse, String> {
    repair_managed_session_with(request, repair)
}

fn repair_run_response(payload: &Value) -> Result<RepairRunResponse, String> {
    Ok(RepairRunResponse {
        run_id: required_string(payload, "run_id")?,
        container_name: required_string(payload, "container_name")?,
        status: required_string(payload, "status")?,
        marker_removed: payload
            .get("marker_removed")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

fn required_string(payload: &Value, name: &str) -> Result<String, String> {
    payload
        .get(name)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("run-control payload is missing {name}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_run_requires_explicit_confirmation() {
        let error = stop_run(StopRunRequest {
            run_id: "a".repeat(32),
            confirm_stop: false,
        })
        .expect_err("stop without confirmation should fail");
        assert!(error.contains("Confirm the stop"));
    }

    #[test]
    fn stop_run_rejects_oversized_run_id() {
        let error = stop_run(StopRunRequest {
            run_id: "a".repeat(MAX_RUN_ID_LEN + 1),
            confirm_stop: true,
        })
        .expect_err("oversized run id should fail");
        assert!(error.contains("run id is too long"));
    }

    #[test]
    fn stop_run_response_maps_success_payload() {
        let payload = serde_json::json!({
            "run_id": "abc",
            "container_name": "runhaven-shell-run",
            "return_code": 0,
        });
        let response = stop_run_response(&payload).expect("success payload");
        assert_eq!(response.run_id, "abc");
        assert_eq!(response.container_name, "runhaven-shell-run");
        assert_eq!(response.status, "stop-requested");
    }

    #[test]
    fn stop_run_response_reports_nonzero_stop() {
        let payload = serde_json::json!({
            "run_id": "abc",
            "container_name": "runhaven-shell-run",
            "return_code": 1,
        });
        let error = stop_run_response(&payload).expect_err("nonzero stop should fail");
        assert!(error.contains("container stop exited 1"));
    }

    #[test]
    fn kill_run_requires_explicit_confirmation() {
        let error = kill_run(KillRunRequest {
            run_id: "a".repeat(32),
            confirm_kill: false,
        })
        .expect_err("kill without confirmation should fail");
        assert!(error.contains("Confirm the hard-stop"));
    }

    #[test]
    fn kill_run_response_maps_success_payload() {
        let payload = serde_json::json!({
            "run_id": "abc",
            "container_name": "runhaven-shell-run",
            "return_code": 0,
        });
        let response = kill_run_response(&payload).expect("success payload");
        assert_eq!(response.status, "kill-requested");
    }

    #[test]
    fn repair_run_requires_explicit_confirmation() {
        let error = repair_run(RepairRunRequest {
            run_id: "a".repeat(32),
            confirm_repair: false,
        })
        .expect_err("repair without confirmation should fail");
        assert!(error.contains("Confirm the repair"));
    }

    #[test]
    fn repair_run_response_maps_removed_marker() {
        let payload = serde_json::json!({
            "run_id": "abc",
            "container_name": "runhaven-shell-run",
            "status": "removed",
            "marker_removed": true,
        });
        let response = repair_run_response(&payload).expect("removed payload");
        assert_eq!(response.status, "removed");
        assert!(response.marker_removed);
    }
}
