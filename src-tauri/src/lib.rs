mod agent_sign_in_runner;
#[cfg(test)]
mod capability_guard;
mod commands;
mod contracts;
mod image_build_runner;
mod launch_runner;
mod launch_status;
mod menu;
mod read_guard;
mod registry_support;
pub(crate) mod session_input;
mod session_output;
mod tail_status;

use tauri_plugin_window_state::StateFlags;

/// The one main window. Every capability and window lookup targets this
/// label; future auxiliary windows must opt in deliberately.
pub(crate) const MAIN_WINDOW_LABEL: &str = "main";

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        // Frame persistence only: size, position, and maximized state restore
        // on relaunch. Fullscreen and visibility are excluded on purpose so
        // entering full screen stays a per-launch choice through the standard
        // controls (navigation spec section 7). The plugin exposes commands,
        // but no capability grants them, so the WebView cannot save, restore,
        // or read window state.
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(StateFlags::SIZE | StateFlags::POSITION | StateFlags::MAXIMIZED)
                .build(),
        )
        // The menu bar is dispatch-only: items emit ids to the main window
        // WebView and never call runtime commands. See src/menu.rs.
        .menu(menu::build)
        .on_menu_event(menu::on_menu_event)
        .setup(|app| {
            commands::attention_badge::clear_badge_on_start(app.handle());
            Ok(())
        })
        .manage(session_input::SessionInputRegistry::default())
        .manage(session_output::SessionOutputRegistry::default())
        .manage(launch_status::LaunchStatusRegistry::default())
        .manage(tail_status::ImageBuildStatusRegistry::default())
        .manage(tail_status::AgentSignInStatusRegistry::default())
        .manage(read_guard::ReadGuards::default())
        .invoke_handler(tauri::generate_handler![
            commands::get_dashboard_status,
            commands::image_status::get_image_status,
            commands::image_build_status::get_image_build_status,
            commands::network::get_network_prune_preview,
            commands::run_status::get_run_status,
            commands::log_snapshot::get_log_snapshot,
            commands::diagnostics::get_egress_log,
            commands::diagnostics::get_auth_log,
            commands::diagnostics::get_auth_status,
            commands::diagnostics::get_auth_explain,
            commands::diagnostics_why::get_why_network,
            commands::diagnostics_why::get_why_state,
            commands::diagnostics_why::get_why_host,
            commands::diagnostics_why::get_why_workspace,
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
            commands::session_worktree::get_managed_session_worktree,
            commands::session_worktree::actions::keep_managed_session_worktree,
            commands::session_worktree::actions::merge_managed_session_worktree,
            commands::session_worktree::actions::recover_managed_session_worktree,
            commands::session_worktree::actions::discard_managed_session_worktree,
            commands::image_build::rebuild_agent_image,
            commands::agent_sign_in::start_agent_sign_in,
            commands::agent_sign_in_status::get_agent_sign_in_status,
            commands::network::prune_managed_networks,
            commands::launch::plan_run,
            commands::launch::launch_run,
            commands::attention_badge::set_attention_badge,
        ])
        .run(tauri::generate_context!())
        .expect("error while running RunHaven desktop app");
}
