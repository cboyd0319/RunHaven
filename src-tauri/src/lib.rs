mod commands;
mod contracts;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_setup_status,
            commands::list_agents,
            commands::get_dashboard_status,
            commands::plan_run,
        ])
        .run(tauri::generate_context!())
        .expect("error while running RunHaven desktop app");
}
