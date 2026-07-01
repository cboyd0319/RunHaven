#[cfg(test)]
mod capability_guard;
mod commands;
mod contracts;
pub mod session_input;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(session_input::SessionInputRegistry::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_setup_status,
            commands::list_agents,
            commands::get_dashboard_status,
            commands::image_status::get_image_status,
            commands::run_status::get_run_status,
            commands::log_snapshot::get_log_snapshot,
            commands::diagnostics::get_egress_log,
            commands::diagnostics::get_auth_log,
            commands::diagnostics::get_auth_status,
            commands::sessions::get_managed_sessions,
            commands::sessions::get_managed_session_log_snapshot,
            commands::run_control::stop_run,
            commands::run_control::kill_run,
            commands::run_control::repair_run,
            commands::run_control::stop_managed_session,
            commands::run_control::kill_managed_session,
            commands::run_control::repair_managed_session,
            commands::plan_run,
            commands::launch_run,
        ])
        .run(tauri::generate_context!())
        .expect("error while running RunHaven desktop app");
}
