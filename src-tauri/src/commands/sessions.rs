use runhaven_core::diagnostics::auth_status_payload;
use runhaven_core::runtime::active::read_active_run_records;
use runhaven_core::ui_contracts::ManagedSessionListData;

#[tauri::command]
pub(crate) fn get_managed_sessions() -> Result<ManagedSessionListData, String> {
    let active_runs = read_active_run_records();
    Ok(ManagedSessionListData::from_active_runs_and_auth_status(
        active_runs,
        auth_status_payload(),
    ))
}
