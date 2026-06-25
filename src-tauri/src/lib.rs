#[cfg(test)]
mod capability_guard;
mod commands;
mod contracts;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
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
            commands::run_control::stop_run,
            commands::run_control::kill_run,
            commands::run_control::repair_run,
            commands::plan_run,
            commands::launch_run,
        ])
        .run(tauri::generate_context!())
        .expect("error while running RunHaven desktop app");
}
