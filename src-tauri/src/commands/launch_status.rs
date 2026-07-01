use tauri::State;

use crate::contracts::{LaunchStatusRequest, LaunchStatusResponse};
use crate::launch_status::LaunchStatusRegistry;

#[tauri::command]
pub(crate) fn get_launch_status(
    request: LaunchStatusRequest,
    launch_status: State<'_, LaunchStatusRegistry>,
) -> Result<LaunchStatusResponse, String> {
    launch_status.get(&request.run_id)
}

#[cfg(test)]
pub(crate) fn get_launch_status_for_test(
    request: LaunchStatusRequest,
    launch_status: &LaunchStatusRegistry,
) -> Result<LaunchStatusResponse, String> {
    launch_status.get(&request.run_id)
}
