mod agent_sign_in_runner;
#[cfg(test)]
mod capability_guard;
mod claude_oauth_signin;
mod commands;
mod contracts;
mod image_build_runner;
mod launch_runner;
mod launch_status;
mod log_format;
mod menu;
mod read_guard;
mod registry_support;
pub(crate) mod session_input;
mod session_output;
pub(crate) mod session_windows;
mod tail_status;
pub(crate) mod terminal;

use tauri::Manager;
use tauri_plugin_window_state::StateFlags;
use window_vibrancy::{NSVisualEffectMaterial, apply_vibrancy};

use session_windows::SESSION_WINDOW_LABEL_PREFIX;

/// The main window: the one control surface. Every mutating capability and
/// menu dispatch targets this label. The only other windows are the read-only
/// per-session monitors (`session-*`, see `session_windows.rs`), which carry
/// their own narrow read capability and never gain controls.
pub(crate) const MAIN_WINDOW_LABEL: &str = "main";

pub fn run() {
    install_panic_log_hook();
    tauri::Builder::default()
        // Diagnostic log trail, active in every build (not compiled out of
        // release), registered first so every later plugin, setup
        // step, and command failure is captured. Two local-only sinks: stdout
        // (picked up by the tauri:dev:log tee) and runhaven.log under the app
        // log dir (~/Library/Logs/<identifier>/ on macOS), rotated at 5 MiB
        // with the five most recent rotated files kept, so a test run always
        // leaves a bounded, reviewable trail. The global level is capped below
        // Trace (see the level cap below). Never log secrets, request/plan
        // payloads, env values, or terminal bytes: the live PTY stream flows
        // over Channels and stays out of the `log` pipeline by construction.
        .plugin(
            tauri_plugin_log::Builder::new()
                // Global cap DEBUG in dev, INFO in release -- deliberately NOT
                // Trace. The broker's HTTP client runs in-process, and several
                // http/tls crates hexdump raw request bytes (token + body) only
                // at Trace; capping the global level below Trace means no crate,
                // present or future, can wire-dump a secret into the log, not
                // just the ones named below. RunHaven's own logging uses
                // info/debug, so nothing of ours is lost.
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    log::LevelFilter::Info
                })
                // Dampen the window machinery: tao emits a TRACE line for every
                // window-delegate callback (resize, key focus), which was 74% of
                // a real test session's log and drowned the app's own trail.
                // Real window problems still surface at info and above.
                .level_for("tao", log::LevelFilter::Info)
                .level_for("wry", log::LevelFilter::Info)
                // SECURITY, not just noise: the broker's HTTP client runs IN this
                // process, and at TRACE the http/tls stack hexdumps raw request
                // bytes -- which include the injected `Authorization: Bearer
                // sk-ant-...` token and the request bodies (prompts, code). Cap
                // the whole stack at WARN so no wire content ever reaches the log.
                // The broker's own audit trail (auth-broker.jsonl) records
                // method/path/status without secrets. Do not raise these.
                .level_for("ureq", log::LevelFilter::Warn)
                .level_for("ureq_proto", log::LevelFilter::Warn)
                .level_for("rustls", log::LevelFilter::Warn)
                .level_for("hyper", log::LevelFilter::Warn)
                .level_for("hyper_util", log::LevelFilter::Warn)
                .level_for("h2", log::LevelFilter::Warn)
                .level_for("reqwest", log::LevelFilter::Warn)
                // Reproduce the plugin's default human layout
                // (`[date][time][target][level] message`) but scrub any residual
                // secret shape from the final message text. Belt-and-suspenders
                // on top of the level cap above, NOT a replacement for it: the
                // cap still stops wire dumps at the source. See log_format.rs.
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "{}[{}][{}] {}",
                        log_format::utc_log_timestamp(),
                        record.target(),
                        record.level(),
                        log_format::redact_log_line(&message.to_string()),
                    ));
                })
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("runhaven".into()),
                    }),
                ])
                .max_file_size(5 * 1024 * 1024)
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepSome(5))
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        // Frame persistence only: size, position, and maximized state restore
        // on relaunch. Fullscreen and visibility are excluded on purpose so
        // entering full screen stays a per-launch choice through the standard
        // controls (navigation spec section 7). The plugin exposes commands,
        // but no capability grants them, so the WebView cannot save, restore,
        // or read window state. Per-session monitor windows are filtered out
        // entirely: they are never persisted or restored across relaunch, not
        // even their geometry.
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(StateFlags::SIZE | StateFlags::POSITION | StateFlags::MAXIMIZED)
                .with_filter(|label| !label.starts_with(SESSION_WINDOW_LABEL_PREFIX))
                .build(),
        )
        // The menu bar is dispatch-only: items emit ids to the main window
        // WebView and never call runtime commands. See src/menu.rs.
        .menu(menu::build)
        .on_menu_event(menu::on_menu_event)
        .setup(|app| {
            // Anchor every log trail: version plus where the file sink lives,
            // so a shared log is self-describing.
            match app.path().app_log_dir() {
                Ok(dir) => log::info!(
                    "RunHaven desktop {} started; log file: {}",
                    app.package_info().version,
                    dir.join("runhaven.log").display()
                ),
                Err(error) => log::warn!(
                    "RunHaven desktop {} started; app log dir unresolved: {error}",
                    app.package_info().version
                ),
            }
            commands::attention_badge::clear_badge_on_start(app.handle());
            // Native macOS sidebar vibrancy (NAT-2): the transparent main
            // window's backdrop becomes a frosted NSVisualEffectView. The
            // sidebar paints a translucent tint over it (glass); the main
            // content stays opaque (SidebarInset is bg-background). Non-fatal --
            // if the effect cannot apply, the transparent window falls back to
            // the per-region backgrounds and the app runs normally. Main window
            // only; the read-only session monitors stay opaque.
            if let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL)
                && let Err(error) =
                    apply_vibrancy(&window, NSVisualEffectMaterial::Sidebar, None, None)
            {
                log::warn!("macOS vibrancy unavailable, using solid background: {error}");
            }
            Ok(())
        })
        // Monitor windows exist to watch sessions while the main window is
        // the control surface. When the main window goes away, close them
        // too, so closing the main window keeps meaning "quit" (Tauri exits
        // when the last window closes) and no orphan monitors outlive their
        // controls.
        .on_window_event(|window, event| {
            if window.label() == MAIN_WINDOW_LABEL && matches!(event, tauri::WindowEvent::Destroyed)
            {
                log::info!("main window destroyed; closing monitor windows and terminal sessions");
                for (label, monitor) in window.app_handle().webview_windows() {
                    if label.starts_with(SESSION_WINDOW_LABEL_PREFIX) {
                        log::debug!("closing monitor window {label}");
                        let _ = monitor.close();
                    }
                }
                // Terminal sessions are live PTY children; kill them all so no
                // `container exec` process outlives the app (terminal design
                // section 9). They are never persisted or restored.
                window
                    .app_handle()
                    .state::<terminal::TerminalSessionRegistry>()
                    .close_all();
            }
        })
        .manage(session_input::SessionInputRegistry::default())
        .manage(terminal::TerminalSessionRegistry::default())
        .manage(session_windows::SessionWindowRegistry::default())
        .manage(session_output::SessionOutputRegistry::default())
        .manage(launch_status::LaunchStatusRegistry::default())
        .manage(tail_status::ImageBuildStatusRegistry::default())
        .manage(tail_status::AgentSignInStatusRegistry::default())
        .manage(claude_oauth_signin::ClaudeOAuthSignInRegistry::default())
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
            commands::claude_oauth::start_claude_oauth_signin,
            commands::claude_oauth::get_claude_oauth_status,
            commands::claude_oauth::clear_claude_oauth,
            commands::network::prune_managed_networks,
            commands::launch::plan_run,
            commands::launch::launch_run,
            commands::attention_badge::set_attention_badge,
            commands::menu_sync::sync_menu_state,
            commands::session_window::open_session_window,
            commands::session_window::get_session_window_target,
            terminal::open_terminal_session,
            terminal::open_claude_sign_in_terminal,
            terminal::open_agent_run_terminal,
            terminal::write_terminal_input,
            terminal::resize_terminal,
            terminal::close_terminal_session,
        ])
        .run(tauri::generate_context!())
        .expect("error while running RunHaven desktop app");
}

/// Route Rust panics into the diagnostic log before the process unwinds, then
/// chain to the previous hook so nothing else changes. A panic otherwise reaches
/// stderr only and never lands in runhaven.log. Installed before the Tauri
/// builder so a panic in setup, a command, or a background runner is captured.
/// The payload flows through the log format filter, so a stray secret in a panic
/// message is scrubbed like any other log line. The panic itself is not
/// swallowed: the previous hook still runs and unwinding proceeds.
fn install_panic_log_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("unnamed");
        let location = info
            .location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "unknown location".to_string());
        let payload = if let Some(text) = info.payload().downcast_ref::<&str>() {
            *text
        } else if let Some(text) = info.payload().downcast_ref::<String>() {
            text.as_str()
        } else {
            "non-string panic payload"
        };
        log::error!("panic in thread '{thread_name}' at {location}: {payload}");
        previous(info);
    }));
}
