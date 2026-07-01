use runhaven_core::diagnostics::{read_auth_broker_log, read_egress_policy_log};
use runhaven_core::ui_contracts::ManagedSessionSummaryData;
use serde_json::Value;

use crate::contracts::{
    AuthProfileStatus, ManagedSessionDiagnosticsRequest, ManagedSessionDiagnosticsResponse,
};

#[tauri::command]
pub(crate) fn get_managed_session_diagnostics(
    request: ManagedSessionDiagnosticsRequest,
) -> Result<ManagedSessionDiagnosticsResponse, String> {
    get_managed_session_diagnostics_with_input(request)
}

fn get_managed_session_diagnostics_with_input(
    request: ManagedSessionDiagnosticsRequest,
) -> Result<ManagedSessionDiagnosticsResponse, String> {
    let (session, run_id) = super::sessions::managed_session_for_id(&request.session_id)?;
    let auth_log_entries = read_auth_broker_log(super::diagnostics::MAX_LOG_ENTRIES)
        .map_err(|error| error.to_string())?
        .iter()
        .filter(|record| record_belongs_to_run(record, &run_id))
        .map(super::diagnostics::auth_entry)
        .collect();
    let egress_log_entries = read_egress_policy_log(super::diagnostics::MAX_LOG_ENTRIES)
        .map_err(|error| error.to_string())?
        .iter()
        .filter(|record| record_belongs_to_run(record, &run_id))
        .map(super::diagnostics::egress_entry)
        .collect();
    Ok(ManagedSessionDiagnosticsResponse {
        auth_status: session_auth_status(&session),
        session,
        auth_log_entries,
        egress_log_entries,
    })
}

#[cfg(test)]
pub(crate) fn get_managed_session_diagnostics_for_test(
    request: ManagedSessionDiagnosticsRequest,
) -> Result<ManagedSessionDiagnosticsResponse, String> {
    get_managed_session_diagnostics_with_input(request)
}

fn session_auth_status(session: &ManagedSessionSummaryData) -> AuthProfileStatus {
    AuthProfileStatus {
        name: session.agent.clone(),
        status: session.auth_status.clone(),
    }
}

#[cfg(test)]
pub(crate) fn session_auth_status_for_test(
    session: &ManagedSessionSummaryData,
) -> AuthProfileStatus {
    session_auth_status(session)
}

fn record_belongs_to_run(record: &Value, run_id: &str) -> bool {
    record.get("run_id").and_then(Value::as_str) == Some(run_id)
}
