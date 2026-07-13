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
use approval::{RunControlAction, trusted_run_control_confirmed};

#[cfg(test)]
pub(crate) use approval::trusted_run_control_messages_for_test;

// Six thin commands map onto two enum-driven paths: one direct path over a run
// id and one managed path over a session id. Both reach the same core functions
// (which require a live RunHaven active record); the managed path additionally
// requires the session to still resolve to an active managed run.

#[tauri::command]
pub(crate) async fn stop_run(
    app: AppHandle,
    request: StopRunRequest,
) -> Result<StopRunResponse, String> {
    confirm_direct_run_control(
        app,
        RunControlAction::Stop,
        &request.run_id,
        request.confirm_stop,
    )
    .await
    .map(stop_response)
}

#[tauri::command]
pub(crate) async fn kill_run(
    app: AppHandle,
    request: KillRunRequest,
) -> Result<KillRunResponse, String> {
    confirm_direct_run_control(
        app,
        RunControlAction::Kill,
        &request.run_id,
        request.confirm_kill,
    )
    .await
    .map(kill_response)
}

#[tauri::command]
pub(crate) async fn repair_run(
    app: AppHandle,
    request: RepairRunRequest,
) -> Result<RepairRunResponse, String> {
    confirm_direct_run_control(
        app,
        RunControlAction::Repair,
        &request.run_id,
        request.confirm_repair,
    )
    .await
    .map(repair_response)
}

#[tauri::command]
pub(crate) async fn stop_managed_session(
    app: AppHandle,
    request: StopManagedSessionRequest,
) -> Result<StopManagedSessionResponse, String> {
    let (session, outcome) = confirm_managed_run_control(
        app,
        RunControlAction::Stop,
        &request.session_id,
        request.confirm_stop,
    )
    .await?;
    Ok(StopManagedSessionResponse {
        session,
        result: stop_response(outcome),
    })
}

#[tauri::command]
pub(crate) async fn kill_managed_session(
    app: AppHandle,
    request: KillManagedSessionRequest,
) -> Result<KillManagedSessionResponse, String> {
    let (session, outcome) = confirm_managed_run_control(
        app,
        RunControlAction::Kill,
        &request.session_id,
        request.confirm_kill,
    )
    .await?;
    Ok(KillManagedSessionResponse {
        session,
        result: kill_response(outcome),
    })
}

#[tauri::command]
pub(crate) async fn repair_managed_session(
    app: AppHandle,
    request: RepairManagedSessionRequest,
) -> Result<RepairManagedSessionResponse, String> {
    let (session, outcome) = confirm_managed_run_control(
        app,
        RunControlAction::Repair,
        &request.session_id,
        request.confirm_repair,
    )
    .await?;
    Ok(RepairManagedSessionResponse {
        session,
        result: repair_response(outcome),
    })
}

/// A run-control result before it is mapped onto a per-action response type.
#[derive(Debug)]
struct RunControlOutcome {
    run_id: String,
    container_name: String,
    status: String,
    marker_removed: bool,
}

/// Validate a direct run-control request, require a native confirmation, then run
/// the action against core. Nothing mutates before the user confirms.
async fn confirm_direct_run_control(
    app: AppHandle,
    action: RunControlAction,
    run_id: &str,
    confirmed: bool,
) -> Result<RunControlOutcome, String> {
    validate_direct_run_control_request(action, run_id, confirmed)?;
    if !trusted_run_control_confirmed(app, action).await? {
        return Err(action.canceled_error().to_string());
    }
    run_control_checked(action, run_id)
}

/// Resolve and RunHaven-verify the managed session, require a native
/// confirmation, then run the action against core. Nothing mutates before the
/// user confirms.
async fn confirm_managed_run_control(
    app: AppHandle,
    action: RunControlAction,
    session_id: &str,
    confirmed: bool,
) -> Result<(ManagedSessionSummaryData, RunControlOutcome), String> {
    let (session, run_id) =
        validate_managed_session_run_control_request(action, session_id, confirmed)?;
    if !trusted_run_control_confirmed(app, action).await? {
        return Err(action.canceled_error().to_string());
    }
    let outcome = run_control_checked(action, &run_id)?;
    Ok((session, outcome))
}

fn validate_direct_run_control_request(
    action: RunControlAction,
    run_id: &str,
    confirmed: bool,
) -> Result<(), String> {
    validate_text_len("run id", run_id, MAX_RUN_ID_LEN)
        .map_err(|_| action.failure_message().to_string())?;
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
        .map_err(|_| action.failure_message().to_string())
}

/// Run the action against the tested core function and map its payload.
fn run_control_checked(
    action: RunControlAction,
    run_id: &str,
) -> Result<RunControlOutcome, String> {
    let payload = match action {
        RunControlAction::Stop => stop_active_run(run_id),
        RunControlAction::Kill => kill_active_run(run_id),
        RunControlAction::Repair => repair_active_run(run_id),
    }
    .map_err(|_| action.failure_message().to_string())?;
    map_run_control_payload(action, &payload).map_err(|_| action.failure_message().to_string())
}

/// The single payload mapper. Stop and kill share the return-code check and
/// differ only in their fixed status and error strings; repair reads its status
/// and removed-marker flag directly from the core payload.
fn map_run_control_payload(
    action: RunControlAction,
    payload: &Value,
) -> Result<RunControlOutcome, String> {
    let run_id = required_string(payload, "run_id")?;
    let container_name = required_string(payload, "container_name")?;
    match action {
        RunControlAction::Repair => Ok(RunControlOutcome {
            run_id,
            container_name,
            status: required_string(payload, "status")?,
            marker_removed: payload
                .get("marker_removed")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        }),
        RunControlAction::Stop | RunControlAction::Kill => {
            let return_code = payload
                .get("return_code")
                .and_then(Value::as_i64)
                .ok_or_else(|| action.missing_return_code_error())?;
            if return_code != 0 {
                return Err(action.nonzero_exit_error(&run_id, &container_name, return_code));
            }
            Ok(RunControlOutcome {
                run_id,
                container_name,
                status: action.requested_status().to_string(),
                marker_removed: false,
            })
        }
    }
}

fn stop_response(outcome: RunControlOutcome) -> StopRunResponse {
    StopRunResponse {
        run_id: outcome.run_id,
        container_name: outcome.container_name,
        status: outcome.status,
    }
}

fn kill_response(outcome: RunControlOutcome) -> KillRunResponse {
    KillRunResponse {
        run_id: outcome.run_id,
        container_name: outcome.container_name,
        status: outcome.status,
    }
}

fn repair_response(outcome: RunControlOutcome) -> RepairRunResponse {
    RepairRunResponse {
        run_id: outcome.run_id,
        container_name: outcome.container_name,
        status: outcome.status,
        marker_removed: outcome.marker_removed,
    }
}

fn required_string(payload: &Value, name: &str) -> Result<String, String> {
    payload
        .get(name)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("run-control payload is missing {name}"))
}

#[cfg(test)]
pub(crate) fn stop_run_checked_for_test(run_id: &str) -> Result<StopRunResponse, String> {
    run_control_checked(RunControlAction::Stop, run_id).map(stop_response)
}

/// The single managed run-control test seam: validate the session, run an
/// injected approval, then an injected mutation, and wrap the typed result. The
/// three typed adapters below preserve the signatures the tests already call.
#[cfg(test)]
fn managed_run_control_for_test<Out, Res>(
    action: RunControlAction,
    session_id: &str,
    confirmed: bool,
    approve: impl FnOnce(&ManagedSessionSummaryData, &str) -> Result<bool, String>,
    mutate: impl FnOnce(&str) -> Result<Out, String>,
    wrap: impl FnOnce(ManagedSessionSummaryData, Out) -> Res,
) -> Result<Res, String> {
    let (session, run_id) =
        validate_managed_session_run_control_request(action, session_id, confirmed)?;
    if !approve(&session, &run_id)? {
        return Err(action.canceled_error().to_string());
    }
    let result = mutate(&run_id)?;
    Ok(wrap(session, result))
}

#[cfg(test)]
pub(crate) fn stop_managed_session_for_test(
    request: StopManagedSessionRequest,
    approve: impl FnOnce(&ManagedSessionSummaryData, &str) -> Result<bool, String>,
    stop: impl FnOnce(&str) -> Result<StopRunResponse, String>,
) -> Result<StopManagedSessionResponse, String> {
    managed_run_control_for_test(
        RunControlAction::Stop,
        &request.session_id,
        request.confirm_stop,
        approve,
        stop,
        |session, result| StopManagedSessionResponse { session, result },
    )
}

#[cfg(test)]
pub(crate) fn kill_managed_session_for_test(
    request: KillManagedSessionRequest,
    approve: impl FnOnce(&ManagedSessionSummaryData, &str) -> Result<bool, String>,
    kill: impl FnOnce(&str) -> Result<KillRunResponse, String>,
) -> Result<KillManagedSessionResponse, String> {
    managed_run_control_for_test(
        RunControlAction::Kill,
        &request.session_id,
        request.confirm_kill,
        approve,
        kill,
        |session, result| KillManagedSessionResponse { session, result },
    )
}

#[cfg(test)]
pub(crate) fn repair_managed_session_for_test(
    request: RepairManagedSessionRequest,
    approve: impl FnOnce(&ManagedSessionSummaryData, &str) -> Result<bool, String>,
    repair: impl FnOnce(&str) -> Result<RepairRunResponse, String>,
) -> Result<RepairManagedSessionResponse, String> {
    managed_run_control_for_test(
        RunControlAction::Repair,
        &request.session_id,
        request.confirm_repair,
        approve,
        repair,
        |session, result| RepairManagedSessionResponse { session, result },
    )
}

#[cfg(test)]
mod tests;
