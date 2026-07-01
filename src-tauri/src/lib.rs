#[cfg(test)]
mod capability_guard;
mod commands;
mod contracts;
mod launch_runner;
mod launch_status;
pub mod session_input;
mod session_output;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(session_input::SessionInputRegistry::default())
        .manage(session_output::SessionOutputRegistry::default())
        .manage(launch_status::LaunchStatusRegistry::default())
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
            commands::diagnostics::get_auth_explain,
            commands::sessions::get_managed_sessions,
            commands::managed_session_status::get_managed_session_status,
            commands::session_diagnostics::get_managed_session_diagnostics,
            commands::managed_session_output::get_managed_session_log_snapshot,
            commands::managed_session_input::write_session_input,
            commands::launch_status::get_launch_status,
            commands::run_control::stop_run,
            commands::run_control::kill_run,
            commands::run_control::repair_run,
            commands::run_control::stop_managed_session,
            commands::run_control::kill_managed_session,
            commands::run_control::repair_managed_session,
            commands::session_cleanup::cleanup_managed_session,
            commands::launch::plan_run,
            commands::launch::launch_run,
        ])
        .run(tauri::generate_context!())
        .expect("error while running RunHaven desktop app");
}
