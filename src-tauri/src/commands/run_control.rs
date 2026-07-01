use runhaven_core::runtime::active::{kill_active_run, repair_active_run, stop_active_run};
use runhaven_core::ui_contracts::ManagedSessionSummaryData;
use serde_json::Value;
use tauri::AppHandle;

use super::validation::{MAX_RUN_ID_LEN, validate_text_len};
use crate::contracts::{
    KillManagedSessionRequest, KillManagedSessionResponse, KillRunRequest, KillRunResponse,
    RepairManagedSessionRequest, RepairManagedSessionResponse, RepairRunRequest, RepairRunResponse,
    StopManagedSessionRequest, StopManagedSessionResponse, StopRunRequest, StopRunResponse,
};

mod approval;
use approval::{RunControlAction, RunControlTarget, trusted_run_control_confirmed};

#[tauri::command]
pub(crate) async fn stop_run(
    app: AppHandle,
    request: StopRunRequest,
) -> Result<StopRunResponse, String> {
    validate_direct_run_control_request(
        RunControlAction::Stop,
        &request.run_id,
        request.confirm_stop,
    )?;
    if !trusted_run_control_confirmed(
        app,
        RunControlAction::Stop,
        RunControlTarget::Run {
            run_id: &request.run_id,
        },
    )
    .await?
    {
        return Err(RunControlAction::Stop.canceled_error().to_string());
    }
    stop_run_checked(&request.run_id)
}

#[tauri::command]
pub(crate) async fn stop_managed_session(
    app: AppHandle,
    request: StopManagedSessionRequest,
) -> Result<StopManagedSessionResponse, String> {
    let (session, run_id) = validate_managed_session_run_control_request(
        RunControlAction::Stop,
        &request.session_id,
        request.confirm_stop,
    )?;
    if !trusted_run_control_confirmed(
        app,
        RunControlAction::Stop,
        RunControlTarget::ManagedSession {
            session: &session,
            run_id: &run_id,
        },
    )
    .await?
    {
        return Err(RunControlAction::Stop.canceled_error().to_string());
    }
    let result = stop_run_checked(&run_id)?;
    Ok(StopManagedSessionResponse { session, result })
}

#[cfg(test)]
fn stop_managed_session_with_approval(
    request: StopManagedSessionRequest,
    approve: impl FnOnce(&ManagedSessionSummaryData, &str) -> Result<bool, String>,
    stop: impl FnOnce(&str) -> Result<StopRunResponse, String>,
) -> Result<StopManagedSessionResponse, String> {
    let (session, run_id) = validate_managed_session_run_control_request(
        RunControlAction::Stop,
        &request.session_id,
        request.confirm_stop,
    )?;
    if !approve(&session, &run_id)? {
        return Err(RunControlAction::Stop.canceled_error().to_string());
    }
    let result = stop(&run_id)?;
    Ok(StopManagedSessionResponse { session, result })
}

#[cfg(test)]
pub(crate) fn stop_managed_session_for_test(
    request: StopManagedSessionRequest,
    approve: impl FnOnce(&ManagedSessionSummaryData, &str) -> Result<bool, String>,
    stop: impl FnOnce(&str) -> Result<StopRunResponse, String>,
) -> Result<StopManagedSessionResponse, String> {
    stop_managed_session_with_approval(request, approve, stop)
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
pub(crate) async fn kill_run(
    app: AppHandle,
    request: KillRunRequest,
) -> Result<KillRunResponse, String> {
    validate_direct_run_control_request(
        RunControlAction::Kill,
        &request.run_id,
        request.confirm_kill,
    )?;
    if !trusted_run_control_confirmed(
        app,
        RunControlAction::Kill,
        RunControlTarget::Run {
            run_id: &request.run_id,
        },
    )
    .await?
    {
        return Err(RunControlAction::Kill.canceled_error().to_string());
    }
    kill_run_checked(&request.run_id)
}

#[tauri::command]
pub(crate) async fn kill_managed_session(
    app: AppHandle,
    request: KillManagedSessionRequest,
) -> Result<KillManagedSessionResponse, String> {
    let (session, run_id) = validate_managed_session_run_control_request(
        RunControlAction::Kill,
        &request.session_id,
        request.confirm_kill,
    )?;
    if !trusted_run_control_confirmed(
        app,
        RunControlAction::Kill,
        RunControlTarget::ManagedSession {
            session: &session,
            run_id: &run_id,
        },
    )
    .await?
    {
        return Err(RunControlAction::Kill.canceled_error().to_string());
    }
    let result = kill_run_checked(&run_id)?;
    Ok(KillManagedSessionResponse { session, result })
}

#[cfg(test)]
fn kill_managed_session_with_approval(
    request: KillManagedSessionRequest,
    approve: impl FnOnce(&ManagedSessionSummaryData, &str) -> Result<bool, String>,
    kill: impl FnOnce(&str) -> Result<KillRunResponse, String>,
) -> Result<KillManagedSessionResponse, String> {
    let (session, run_id) = validate_managed_session_run_control_request(
        RunControlAction::Kill,
        &request.session_id,
        request.confirm_kill,
    )?;
    if !approve(&session, &run_id)? {
        return Err(RunControlAction::Kill.canceled_error().to_string());
    }
    let result = kill(&run_id)?;
    Ok(KillManagedSessionResponse { session, result })
}

#[cfg(test)]
pub(crate) fn kill_managed_session_for_test(
    request: KillManagedSessionRequest,
    approve: impl FnOnce(&ManagedSessionSummaryData, &str) -> Result<bool, String>,
    kill: impl FnOnce(&str) -> Result<KillRunResponse, String>,
) -> Result<KillManagedSessionResponse, String> {
    kill_managed_session_with_approval(request, approve, kill)
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
pub(crate) async fn repair_run(
    app: AppHandle,
    request: RepairRunRequest,
) -> Result<RepairRunResponse, String> {
    validate_direct_run_control_request(
        RunControlAction::Repair,
        &request.run_id,
        request.confirm_repair,
    )?;
    if !trusted_run_control_confirmed(
        app,
        RunControlAction::Repair,
        RunControlTarget::Run {
            run_id: &request.run_id,
        },
    )
    .await?
    {
        return Err(RunControlAction::Repair.canceled_error().to_string());
    }
    repair_run_checked(&request.run_id)
}

#[tauri::command]
pub(crate) async fn repair_managed_session(
    app: AppHandle,
    request: RepairManagedSessionRequest,
) -> Result<RepairManagedSessionResponse, String> {
    let (session, run_id) = validate_managed_session_run_control_request(
        RunControlAction::Repair,
        &request.session_id,
        request.confirm_repair,
    )?;
    if !trusted_run_control_confirmed(
        app,
        RunControlAction::Repair,
        RunControlTarget::ManagedSession {
            session: &session,
            run_id: &run_id,
        },
    )
    .await?
    {
        return Err(RunControlAction::Repair.canceled_error().to_string());
    }
    let result = repair_run_checked(&run_id)?;
    Ok(RepairManagedSessionResponse { session, result })
}

#[cfg(test)]
fn repair_managed_session_with_approval(
    request: RepairManagedSessionRequest,
    approve: impl FnOnce(&ManagedSessionSummaryData, &str) -> Result<bool, String>,
    repair: impl FnOnce(&str) -> Result<RepairRunResponse, String>,
) -> Result<RepairManagedSessionResponse, String> {
    let (session, run_id) = validate_managed_session_run_control_request(
        RunControlAction::Repair,
        &request.session_id,
        request.confirm_repair,
    )?;
    if !approve(&session, &run_id)? {
        return Err(RunControlAction::Repair.canceled_error().to_string());
    }
    let result = repair(&run_id)?;
    Ok(RepairManagedSessionResponse { session, result })
}

#[cfg(test)]
pub(crate) fn repair_managed_session_for_test(
    request: RepairManagedSessionRequest,
    approve: impl FnOnce(&ManagedSessionSummaryData, &str) -> Result<bool, String>,
    repair: impl FnOnce(&str) -> Result<RepairRunResponse, String>,
) -> Result<RepairManagedSessionResponse, String> {
    repair_managed_session_with_approval(request, approve, repair)
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

fn validate_direct_run_control_request(
    action: RunControlAction,
    run_id: &str,
    confirmed: bool,
) -> Result<(), String> {
    validate_text_len("run id", run_id, MAX_RUN_ID_LEN)?;
    if !confirmed {
        return Err(action.confirm_error().to_string());
    }
    Ok(())
}

fn validate_managed_session_run_control_request(
    action: RunControlAction,
    session_id: &str,
    confirmed: bool,
) -> Result<(ManagedSessionSummaryData, String), String> {
    if !confirmed {
        return Err(action.confirm_error().to_string());
    }
    crate::commands::sessions::managed_session_for_run_control(session_id)
}

fn stop_run_checked(run_id: &str) -> Result<StopRunResponse, String> {
    let payload = stop_active_run(run_id).map_err(|error| error.to_string())?;
    stop_run_response(&payload)
}

#[cfg(test)]
pub(crate) fn stop_run_checked_for_test(run_id: &str) -> Result<StopRunResponse, String> {
    stop_run_checked(run_id)
}

fn kill_run_checked(run_id: &str) -> Result<KillRunResponse, String> {
    let payload = kill_active_run(run_id).map_err(|error| error.to_string())?;
    kill_run_response(&payload)
}

fn repair_run_checked(run_id: &str) -> Result<RepairRunResponse, String> {
    let payload = repair_active_run(run_id).map_err(|error| error.to_string())?;
    repair_run_response(&payload)
}

fn required_string(payload: &Value, name: &str) -> Result<String, String> {
    payload
        .get(name)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("run-control payload is missing {name}"))
}

#[cfg(test)]
mod tests;
